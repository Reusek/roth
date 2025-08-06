mod analyzer;
mod codegen;
mod highlighter;
mod ir;
mod ir_codegen;
mod ir_lowering;
mod ir_optimizer;
mod lexer;
mod parser;
mod types;

use crate::analyzer::SemanticAnalyzer;
use crate::codegen::{Backend, CodeGenerator};
use crate::highlighter::SyntaxHighlighter;
use crate::ir_codegen::IRRustGenerator;
use crate::ir_lowering::IRLowering;
use crate::ir_optimizer::IROptimizer;
use crate::lexer::Lexer;
use crate::parser::Parser;
use clap::Parser as ClapParser;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

#[derive(ClapParser, Debug)]
#[command(name = "roth")]
#[command(about = "Enhanced Forth Compiler with IR Backend")]
struct Args {
    #[arg(help = "Forth file to compile")]
    file: Option<String>,

    #[arg(long, help = "Disable syntax highlighting")]
    no_color: bool,

    #[arg(
        long,
        short,
        default_value = "0",
        help = "Debug level (0=off, 1=timing, 2=verbose, 3=show highlighted code)"
    )]
    debug: u8,

    #[arg(
        long,
        short,
        default_value = "rust-ir",
        help = "Backend to use (rust-ir, c-ir, ir-debug-rust, ir-debug-c)"
    )]
    backend: String,

    #[arg(long, short, help = "Output file name")]
    output: Option<String>,

    #[arg(long, help = "Compile and run the generated code")]
    run: bool,
}

fn compile_file(
    filename: &str,
    backend: Backend,
    output: Option<String>,
    debug: u8,
    no_color: bool,
    run: bool,
) -> Result<(), String> {
    let content = fs::read_to_string(filename)
        .map_err(|e| format!("Error reading file '{}': {}", filename, e))?;

    let mut lexer = Lexer::new(content);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexer error: {}", e))?;

    if debug >= 2 {
        println!("Tokens: {:?}", tokens);
    }

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parser error: {}", e))?;

    if debug >= 2 {
        println!("AST: {:#?}", ast);
    }

    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze(&ast)
        .map_err(|e| format!("Semantic analysis error: {}", e))?;

    let mut ir_lowering = IRLowering::new();
    let mut ir = ir_lowering.lower(&ast);

    if debug >= 2 {
        println!("IR: {}", ir);
    }

    let mut optimizer = IROptimizer::new();
    let optimization_stats = optimizer.optimize(&mut ir);

    if debug >= 2 {
        println!(
            "Optimization stats: \n - {}",
            optimization_stats.join("\n - ")
        );
        println!("Optimized IR: {}", ir);
    }

    let (generated_code, file_extension) = match backend {
        Backend::RustIR | Backend::IRDebugRust => {
            let mut codegen = IRRustGenerator::new();
            let code = codegen.generate_program(&ir);
            let ext = codegen.get_file_extension().to_string();
            (code, ext)
        }
        Backend::CIR | Backend::IRDebugC => {
            let mut codegen = crate::ir_codegen::IRCGenerator::new();
            let code = codegen.generate_program(&ir);
            let ext = codegen.get_file_extension().to_string();
            (code, ext)
        }
    };

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
    } else if debug >= 1 {
        println!("Generated code:\n{}", generated_code);
    }

    // Create .build directory if it doesn't exist
    let build_dir = Path::new(".build");
    if !build_dir.exists() {
        fs::create_dir_all(build_dir)
            .map_err(|e| format!("Error creating .build directory: {}", e))?;
    }

    // Determine output file name in .build directory
    let output_file = match output {
        Some(ref name) => {
            // If user specifies output, still put it in .build directory
            let output_path = Path::new(name);
            let filename = output_path.file_name()
                .ok_or("Invalid output filename")?
                .to_str()
                .ok_or("Invalid output filename encoding")?;
            build_dir.join(filename).to_string_lossy().to_string()
        }
        None => {
            let base_name = Path::new(filename)
                .file_stem()
                .ok_or("Invalid input filename")?
                .to_str()
                .ok_or("Invalid input filename encoding")?;
            build_dir.join(format!("{}.{}", base_name, &file_extension))
                .to_string_lossy()
                .to_string()
        }
    };

    // Write generated code to file
    fs::write(&output_file, &generated_code)
        .map_err(|e| format!("Error writing output file '{}': {}", output_file, e))?;
    
    if debug >= 1 {
        println!("Code written to: {}", output_file);
    }

    // Compile and run if requested
    if run {
        compile_and_run(&output_file, backend, debug)?;
    }

    Ok(())
}

fn compile_and_run(
    source_file: &str,
    backend: Backend,
    debug: u8,
) -> Result<(), String> {
    let compile_cmd = match backend {
        Backend::RustIR | Backend::IRDebugRust => {
            let crate_name = std::path::Path::new(source_file)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(".", "_");
            let base_name = std::path::Path::new(source_file)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            format!("rustc -O --crate-name {} {} -o .build/{}", crate_name, source_file, base_name)
        }
        Backend::CIR | Backend::IRDebugC => {
            let base_name = std::path::Path::new(source_file)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            format!("gcc -O2 -o .build/{} {}", base_name, source_file)
        }
    };
    let parts: Vec<&str> = compile_cmd.split_whitespace().collect();
    
    if parts.is_empty() {
        return Err("Empty compile command".to_string());
    }

    if debug >= 1 {
        println!("Compiling with: {}", compile_cmd);
    }

    // Execute compile command
    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .map_err(|e| format!("Failed to execute compile command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Compilation failed:\n{}", stderr));
    }

    if debug >= 1 {
        println!("Compilation successful!");
    }

    // Determine executable name - executables are created in .build directory
    let executable = if source_file.ends_with(".rs") {
        let base_name = std::path::Path::new(source_file)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        format!(".build/{}", base_name)
    } else if source_file.ends_with(".c") {
        let base_name = std::path::Path::new(source_file)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        format!(".build/{}", base_name)
    } else {
        let base_name = std::path::Path::new(source_file)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        format!(".build/{}", base_name)
    };

    if debug >= 1 {
        println!("Running: {}", executable);
    }

    // Execute the compiled program
    let run_output = Command::new(&executable)
        .output()
        .map_err(|e| format!("Failed to execute compiled program: {}", e))?;

    // Print the program output
    print!("{}", String::from_utf8_lossy(&run_output.stdout));
    
    if !run_output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&run_output.stderr));
    }

    if !run_output.status.success() {
        return Err(format!("Program execution failed with exit code: {:?}", run_output.status.code()));
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    let backend = match Backend::from_str(&args.backend) {
        Some(b) => b,
        None => {
            eprintln!(
                "Unknown backend: {}. Available backends: rust-ir, c-ir, ir-debug-rust, ir-debug-c",
                args.backend
            );
            process::exit(1);
        }
    };

    match &args.file {
        Some(filename) => {
            if let Err(e) = compile_file(filename, backend, args.output, args.debug, args.no_color, args.run)
            {
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
