mod types;
mod lexer;
mod parser;
mod analyzer;
mod codegen;
mod highlighter;
mod ir;
mod ir_lowering;
mod ir_optimizer;
mod ir_codegen;

use std::fs;
use std::process;
use crate::codegen::Backend;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::analyzer::SemanticAnalyzer;
use crate::ir_lowering::IRLowering;
use crate::ir_optimizer::IROptimizer;
use crate::ir_codegen::IRRustGenerator;
use crate::highlighter::SyntaxHighlighter;
use clap::Parser as ClapParser;

#[derive(ClapParser, Debug)]
#[command(name = "roth")]
#[command(about = "Enhanced Forth Compiler with IR Backend")]
struct Args {
    #[arg(help = "Forth file to compile")]
    file: Option<String>,
    
    #[arg(long, help = "Disable syntax highlighting")]
    no_color: bool,
    
    #[arg(long, short, default_value = "0", help = "Debug level (0=off, 1=timing, 2=verbose, 3=show highlighted code)")]
    debug: u8,
    
    #[arg(long, short, default_value = "rust-ir", help = "Backend to use (rust-ir, c-ir, ir-debug-rust, ir-debug-c)")]
    backend: String,
    
    #[arg(long, short, help = "Output file name")]
    output: Option<String>,
}

fn compile_file(filename: &str, backend: Backend, output: Option<String>, debug: u8, no_color: bool) -> Result<(), String> {
    let content = fs::read_to_string(filename)
        .map_err(|e| format!("Error reading file '{}': {}", filename, e))?;

    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize()
        .map_err(|e| format!("Lexer error: {}", e))?;

    if debug >= 2 {
        println!("Tokens: {:?}", tokens);
    }

    let mut parser = Parser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| format!("Parser error: {}", e))?;

    if debug >= 2 {
        println!("AST: {:#?}", ast);
    }

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast)
        .map_err(|e| format!("Semantic analysis error: {}", e))?;

    let mut ir_lowering = IRLowering::new();
    let mut ir = ir_lowering.lower(&ast);

    if debug >= 2 {
        println!("IR: {}", ir);
    }

    let mut optimizer = IROptimizer::new();
    let optimization_stats = optimizer.optimize(&mut ir);

    if debug >= 2 {
        println!("Optimization stats: \n - {}", optimization_stats.join("\n - "));
        println!("Optimized IR: {}", ir);
    }

    let mut codegen = IRRustGenerator::new();
    let generated_code = codegen.generate_program(&ir);

    if debug >= 3 && !no_color {
        if let Ok(mut highlighter) = SyntaxHighlighter::new() {
            if let Ok(highlighted) = highlighter.highlight_with_force(&generated_code, true) {
                println!("Generated code:\n{}", highlighted);
            } else {
                println!("Generated code:\n{}", generated_code);
            }
        } else {
            println!("Generated code:\n{}", generated_code);
        }
    } else {
        println!("Generated code:\n{}", generated_code);
    }

    if let Some(output_file) = output {
        fs::write(&output_file, &generated_code)
            .map_err(|e| format!("Error writing output file '{}': {}", output_file, e))?;
        println!("Code written to: {}", output_file);
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    
    let backend = match Backend::from_str(&args.backend) {
        Some(b) => b,
        None => {
            eprintln!("Unknown backend: {}. Available backends: rust-ir, c-ir, ir-debug-rust, ir-debug-c", args.backend);
            process::exit(1);
        }
    };

    match &args.file {
        Some(filename) => {
            if let Err(e) = compile_file(filename, backend, args.output, args.debug, args.no_color) {
                eprintln!("Compilation failed: {}", e);
                process::exit(1);
            }
        }
        None => {
            eprintln!("No input file specified. Use --help for usage information.");
            process::exit(1);
        }
    }
}