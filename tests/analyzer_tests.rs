use roth::analyzer::SemanticAnalyzer;
use roth::lexer::Lexer;
use roth::parser::Parser;

fn analyze_input(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast)?;
    Ok(())
}

#[test]
fn test_analyze_builtin_words() {
    assert!(analyze_input("42 DUP + DROP").is_ok());
    assert!(analyze_input("1 2 SWAP - .").is_ok());
    assert!(analyze_input("5 DUP * .S CR").is_ok());
}

#[test]
fn test_analyze_numbers_only() {
    assert!(analyze_input("1 2 3 42 -17").is_ok());
}

#[test]
fn test_analyze_simple_definition() {
    assert!(analyze_input(": SQUARE DUP * ;").is_ok());
    assert!(analyze_input(": ADD2 2 + ;").is_ok());
    // NEGATE is a builtin word in Roth and cannot be redefined.
    // Use a different name for a user-defined negation helper.
    assert!(analyze_input(": NEG1 -1 * ;").is_ok());
}

#[test]
fn test_analyze_definition_usage() {
    assert!(analyze_input(": DOUBLE 2 * ; 5 DOUBLE").is_ok());
    assert!(analyze_input(": SQUARE DUP * ; : CUBE DUP SQUARE * ; 3 CUBE").is_ok());
}

#[test]
fn test_analyze_multiple_definitions() {
    let input = r#"
        : ADD2 2 + ;
        : SUB2 2 - ;
        : DOUBLE 2 * ;
        : HALF 2 / ;
        10 ADD2 SUB2 DOUBLE HALF
    "#;
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_definition_with_builtins() {
    let input = r#"
        : SHOW_STACK .S CR ;
        : PRINT_AND_DROP . DROP ;
        42 DUP SHOW_STACK PRINT_AND_DROP
    "#;
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_recursive_definition() {
    let input = r#"
        : COUNTDOWN DUP . 1 - DUP 0 > IF COUNTDOWN THEN DROP ;
    "#;
    // Recursive definitions are allowed: the word name is registered before the body is analyzed.
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_empty_definition() {
    assert!(analyze_input(": EMPTY ;").is_ok());
}

#[test]
fn test_analyze_definition_order_independence() {
    let input = r#"
        : MAIN HELPER 42 + ;
        : HELPER 10 * ;
        5 MAIN
    "#;
    // This should fail because HELPER is used before it's defined
    assert!(analyze_input(input).is_err());
}

#[test]
fn test_analyze_error_undefined_word() {
    assert!(analyze_input("UNDEFINED_WORD").is_err());
    assert!(analyze_input("42 UNKNOWN_OPERATION").is_err());
    assert!(analyze_input(": TEST UNDEFINED_HELPER ;").is_err());
}

#[test]
fn test_analyze_error_redefine_builtin() {
    assert!(analyze_input(": + 42 ;").is_err());
    assert!(analyze_input(": DUP DROP ;").is_err());
    assert!(analyze_input(": . CR ;").is_err());
}

#[test]
fn test_analyze_case_sensitivity() {
    // Words should be case-insensitive (converted to uppercase by lexer)
    assert!(analyze_input("dup swap +").is_ok());
    assert!(analyze_input(": test dup * ; 5 test").is_ok());
}

#[test]
fn test_analyze_with_comments() {
    let input = r#"
        ( This is a comment )
        : SQUARE ( n -- n*n ) DUP * ;
        ( Another comment )
        5 SQUARE ( Calculate 5 squared )
    "#;
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_complex_program() {
    let input = r#"
        : DOUBLE 2 * ;
        : SQUARE DUP * ;
        : SHOW DUP . ;
        
        5 SHOW DOUBLE SHOW SQUARE SHOW DROP
    "#;
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_all_builtins() {
    let input = "+ - * / DUP DROP SWAP OVER . .S CR";
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_definition_with_all_builtins() {
    let input = r#"
        : DEMO 
            DUP .           ( Print top of stack )
            SWAP OVER       ( Rearrange stack )
            + - * /         ( Arithmetic operations )
            .S CR           ( Show stack and newline )
            DROP            ( Clean up )
        ;
    "#;
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_nested_word_usage() {
    let input = r#"
        : HELPER1 2 * ;
        : HELPER2 HELPER1 1 + ;
        : MAIN HELPER2 HELPER1 + ;
        10 MAIN
    "#;
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_word_redefinition() {
    let input = r#"
        : TEST 1 + ;
        : TEST 2 * ;  ( Redefining TEST )
        5 TEST
    "#;
    // This should be allowed (Forth allows redefinition of user words)
    assert!(analyze_input(input).is_ok());
}

#[test]
fn test_analyze_empty_program() {
    assert!(analyze_input("").is_ok());
    assert!(analyze_input("   ").is_ok());
    assert!(analyze_input("( just a comment )").is_ok());
}
