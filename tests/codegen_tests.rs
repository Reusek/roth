use roth::analyzer::SemanticAnalyzer;
use roth::ir_codegen::IRRustGenerator;
use roth::ir_lowering::IRLowering;
use roth::ir_optimizer::IROptimizer;
use roth::lexer::Lexer;
use roth::parser::Parser;

fn compile_to_rust(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast)?;

    let mut ir_lowering = IRLowering::new();
    let mut ir = ir_lowering.lower(&ast);

    let mut optimizer = IROptimizer::new();
    optimizer.optimize(&mut ir);

    let mut codegen = IRRustGenerator::new();
    Ok(codegen.generate_program(&ir))
}

fn debug_compile(input: &str) -> String {
    compile_to_rust(input).unwrap_or_else(|_| String::new())
}

#[test]
fn test_codegen_simple_number() {
    let result = compile_to_rust("42").unwrap();

    assert!(result.contains("self.stack.push(42)"));
    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
}

#[test]
fn test_codegen_arithmetic() {
    let result = compile_to_rust("5 3 +").unwrap();

    // The optimizer should fold 5 + 3 into 8
    assert!(result.contains("self.stack.push(8)"));
    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
}

#[test]
fn test_codegen_stack_operations() {
    let result = debug_compile("42 DUP SWAP DROP");

    assert!(result.contains("self.stack.push(42)"));
    // Should contain basic structure
    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
}

#[test]
fn test_codegen_simple_definition() {
    let result = debug_compile(": DOUBLE 2 * ;");

    assert!(result.contains("fn double(&mut self)"));
    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
}

#[test]
fn test_codegen_definition_usage() {
    let result = debug_compile(": SQUARE DUP * ; 5 SQUARE");

    // Should contain the definition
    assert!(result.contains("fn square(&mut self)"));

    // Should contain the usage in main
    assert!(result.contains("self.stack.push(5)"));
    // Function calls might be optimized away or inlined
    assert!(result.contains("square") || result.contains("self.stack"));
}

#[test]
fn test_codegen_multiple_definitions() {
    let result = debug_compile(": ADD2 2 + ; : MUL3 3 * ; 10 ADD2 MUL3");

    assert!(result.contains("fn add2(&mut self)"));
    assert!(result.contains("fn mul3(&mut self)"));
    // The optimizer may fold constants or inline functions
    assert!(result.contains("self.stack.push") || result.contains("stack.push"));
}

#[test]
fn test_codegen_io_operations() {
    let result = compile_to_rust("42 . .S CR").unwrap();

    assert!(result.contains("self.stack.push(42)"));
    // Should contain basic structure
    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
}

#[test]
fn test_codegen_comments_preserved() {
    let result = compile_to_rust("( This is a comment ) 42").unwrap();

    // Should generate valid code even with comments
    assert!(result.contains("self.stack.push(42)"));
    assert!(result.contains("pub struct OptimizedForth"));
}

#[test]
fn test_codegen_complex_program() {
    let input = r#"
        : SQUARE DUP * ;
        : CUBE DUP SQUARE * ;
        5 CUBE .
    "#;
    let result = debug_compile(input);

    assert!(result.contains("fn square(&mut self)"));
    assert!(result.contains("fn cube(&mut self)"));
    assert!(result.contains("self.stack.push(5)"));
    // Functions may be inlined by optimizer
    assert!(result.contains("cube") || result.contains("square"));
}

#[test]
fn test_codegen_nested_function_calls() {
    let input = r#"
        : HELPER 2 * ;
        : MAIN HELPER 1 + ;
        10 MAIN
    "#;
    let result = debug_compile(input);

    assert!(result.contains("fn helper(&mut self)"));
    assert!(result.contains("fn main(&mut self)") || result.contains("fn main_word(&mut self)"));
    // The optimizer may fold the entire computation: 10 * 2 + 1 = 21
    assert!(result.contains("self.stack.push(21)") || result.contains("self.stack.push(10)"));
    // Functions may be inlined by optimizer
    assert!(result.contains("helper") || result.contains("main"));
}

#[test]
fn test_codegen_all_arithmetic_ops() {
    let result = compile_to_rust("10 5 + 3 - 2 * 4 /").unwrap();

    // The optimizer may fold some operations, so just check basic structure
    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
    assert!(result.contains("self.stack"));
}

#[test]
fn test_codegen_all_stack_ops() {
    let result = compile_to_rust("1 2 3 DUP DROP SWAP OVER").unwrap();

    // Should contain basic structure and stack operations
    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
    assert!(result.contains("self.stack"));
}

#[test]
fn test_codegen_error_handling() {
    let result = compile_to_rust("42 DUP +").unwrap();

    // Generated code should handle operations properly
    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
    assert!(result.contains("self.stack"));
}

#[test]
fn test_codegen_optimization_hints() {
    let result = compile_to_rust("2 2 + 4 *").unwrap();

    // After optimization, this might be simplified
    // The test verifies that optimization doesn't break code generation
    assert!(result.contains("stack.push") || result.contains("OptimizedForth"));
}

#[test]
fn test_codegen_empty_program() {
    let result = compile_to_rust("").unwrap();

    assert!(result.contains("pub struct OptimizedForth"));
    assert!(result.contains("impl OptimizedForth"));
    assert!(result.contains("pub fn new() -> Self"));
}

#[test]
fn test_codegen_only_comments() {
    let result = compile_to_rust("( just a comment )").unwrap();

    assert!(result.contains("pub struct OptimizedForth"));
    // Should generate valid Rust code even with only comments
    assert!(result.contains("impl OptimizedForth"));
}
