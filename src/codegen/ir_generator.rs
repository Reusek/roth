use crate::types::AstNode;
use crate::codegen::CodeGenerator;
use crate::ir_lowering::{IRLowering, StackEffectAnalyzer};
use crate::ir_optimizer::IROptimizer;
use crate::ir_codegen::{IRRustGenerator, IRCGenerator};
use crate::ir_lowering::IRPrettyPrinter;

/// IR-based Rust code generator
pub struct IRBasedRustGenerator {
    show_ir: bool,
    show_optimization_stats: bool,
}

impl IRBasedRustGenerator {
    pub fn new() -> Self {
        Self {
            show_ir: false,
            show_optimization_stats: true,
        }
    }

    pub fn with_ir_debug(mut self, show_ir: bool) -> Self {
        self.show_ir = show_ir;
        self
    }

    pub fn with_optimization_stats(mut self, show_stats: bool) -> Self {
        self.show_optimization_stats = show_stats;
        self
    }
}

impl CodeGenerator for IRBasedRustGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        let mut output = String::new();
        
        // Step 1: Lower AST to IR
        let mut lowering = IRLowering::new();
        let mut ir_program = lowering.lower(ast);
        
        // Step 2: Analyze stack effects
        StackEffectAnalyzer::analyze_program(&mut ir_program);
        
        if self.show_ir {
            output.push_str("=== UNOPTIMIZED IR ===\n");
            output.push_str(&IRPrettyPrinter::print_with_stack_analysis(&ir_program));
            output.push_str("\n");
        }
        
        // Step 3: Optimize IR
        let mut optimizer = IROptimizer::new();
        let optimization_stats = optimizer.optimize(&mut ir_program);
        
        if self.show_optimization_stats && !optimization_stats.is_empty() {
            output.push_str("=== OPTIMIZATION STATS ===\n");
            for stat in &optimization_stats {
                output.push_str(&format!("// {}\n", stat));
            }
            output.push_str("\n");
        }
        
        if self.show_ir {
            output.push_str("=== OPTIMIZED IR ===\n");
            output.push_str(&IRPrettyPrinter::print_with_stack_analysis(&ir_program));
            output.push_str("\n");
        }
        
        // Step 4: Generate target code
        let mut rust_generator = IRRustGenerator::new();
        let generated_code = rust_generator.generate_program(&ir_program);
        
        output.push_str(&generated_code);
        output
    }

    fn get_file_extension(&self) -> &str {
        "rs"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("rustc -O {}", filename)
    }
}

/// IR-based C code generator
pub struct IRBasedCGenerator {
    show_ir: bool,
    show_optimization_stats: bool,
}

impl IRBasedCGenerator {
    pub fn new() -> Self {
        Self {
            show_ir: false,
            show_optimization_stats: true,
        }
    }

    pub fn with_ir_debug(mut self, show_ir: bool) -> Self {
        self.show_ir = show_ir;
        self
    }

    pub fn with_optimization_stats(mut self, show_stats: bool) -> Self {
        self.show_optimization_stats = show_stats;
        self
    }
}

impl CodeGenerator for IRBasedCGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        let mut output = String::new();
        
        // Step 1: Lower AST to IR
        let mut lowering = IRLowering::new();
        let mut ir_program = lowering.lower(ast);
        
        // Step 2: Analyze stack effects
        StackEffectAnalyzer::analyze_program(&mut ir_program);
        
        if self.show_ir {
            output.push_str("/*\n=== UNOPTIMIZED IR ===\n");
            output.push_str(&IRPrettyPrinter::print_with_stack_analysis(&ir_program));
            output.push_str("*/\n\n");
        }
        
        // Step 3: Optimize IR
        let mut optimizer = IROptimizer::new();
        let optimization_stats = optimizer.optimize(&mut ir_program);
        
        if self.show_optimization_stats && !optimization_stats.is_empty() {
            output.push_str("/*\n=== OPTIMIZATION STATS ===\n");
            for stat in &optimization_stats {
                output.push_str(&format!("{}\n", stat));
            }
            output.push_str("*/\n\n");
        }
        
        if self.show_ir {
            output.push_str("/*\n=== OPTIMIZED IR ===\n");
            output.push_str(&IRPrettyPrinter::print_with_stack_analysis(&ir_program));
            output.push_str("*/\n\n");
        }
        
        // Step 4: Generate target code
        let mut c_generator = IRCGenerator::new();
        let generated_code = c_generator.generate_program(&ir_program);
        
        output.push_str(&generated_code);
        output
    }

    fn get_file_extension(&self) -> &str {
        "c"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("gcc -O2 -o {} {}", filename.trim_end_matches(".c"), filename)
    }
}

/// Debug generator that shows the complete IR pipeline
pub struct IRDebugGenerator {
    target: String,
}

impl IRDebugGenerator {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
        }
    }
}

impl CodeGenerator for IRDebugGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        let mut output = String::new();
        
        output.push_str("=== IR COMPILATION PIPELINE DEBUG ===\n\n");
        
        // Step 1: Show original AST
        output.push_str("=== ORIGINAL AST ===\n");
        output.push_str(&format!("{:#?}\n\n", ast));
        
        // Step 2: Lower AST to IR
        let mut lowering = IRLowering::new();
        let mut ir_program = lowering.lower(ast);
        
        output.push_str("=== UNOPTIMIZED IR ===\n");
        output.push_str(&format!("{}\n", ir_program));
        
        // Step 3: Analyze stack effects
        StackEffectAnalyzer::analyze_program(&mut ir_program);
        
        output.push_str("=== IR WITH STACK ANALYSIS ===\n");
        output.push_str(&IRPrettyPrinter::print_with_stack_analysis(&ir_program));
        output.push_str("\n");
        
        // Step 4: Optimize IR
        let mut optimizer = IROptimizer::new();
        let optimization_stats = optimizer.optimize(&mut ir_program);
        
        output.push_str("=== OPTIMIZATION STATS ===\n");
        for stat in &optimization_stats {
            output.push_str(&format!("{}\n", stat));
        }
        output.push_str("\n");
        
        output.push_str("=== OPTIMIZED IR ===\n");
        output.push_str(&IRPrettyPrinter::print_with_stack_analysis(&ir_program));
        output.push_str("\n");
        
        // Step 5: Generate target code
        output.push_str(&format!("=== GENERATED {} CODE ===\n", self.target.to_uppercase()));
        
        match self.target.as_str() {
            "rust" => {
                let mut rust_generator = IRRustGenerator::new();
                output.push_str(&rust_generator.generate_program(&ir_program));
            }
            "c" => {
                let mut c_generator = IRCGenerator::new();
                output.push_str(&c_generator.generate_program(&ir_program));
            }
            _ => {
                output.push_str(&format!("Unknown target: {}\n", self.target));
            }
        }
        
        output
    }

    fn get_file_extension(&self) -> &str {
        match self.target.as_str() {
            "rust" => "rs",
            "c" => "c",
            _ => "txt",
        }
    }

    fn get_compile_command(&self, filename: &str) -> String {
        match self.target.as_str() {
            "rust" => format!("rustc -O {}", filename),
            "c" => format!("gcc -O2 -o {} {}", filename.trim_end_matches(".c"), filename),
            _ => format!("# Unknown target: {}", self.target),
        }
    }
}