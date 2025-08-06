use roth::codegen::{CodegenPipeline, backends::create_target_info};
use roth::codegen::framework::CodegenContext;
use roth::ir::{IRProgram, IRFunction, IRInstruction, IRValue};
use std::collections::HashMap;

fn main() {
    println!("Testing the new modular codegen framework!");
    
    // Create a simple IR program for testing
    let mut program = IRProgram {
        functions: HashMap::new(),
        main: IRFunction {
            name: "main".to_string(),
            instructions: vec![
                IRInstruction::Push(IRValue::Constant(42)),
                IRInstruction::Push(IRValue::Constant(10)),
                IRInstruction::Add,
                IRInstruction::Print,
            ],
            stack_effect: roth::ir::StackEffect { consumes: 0, produces: 0 },
        },
    };
    
    // Test the new framework with different backends
    let mut pipeline = CodegenPipeline::new();
    
    println!("\n=== Available Backends ===");
    for backend in pipeline.list_available_backends() {
        println!("- {}", backend);
    }
    
    println!("\n=== Testing Rust Backend ===");
    match pipeline.generate_code("rust", &program) {
        Ok(code) => println!("Generated Rust code:\n{}", code),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n=== Testing C Backend ===");
    match pipeline.generate_code("c", &program) {
        Ok(code) => println!("Generated C code:\n{}", code),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n=== Testing Debug Backend ===");
    match pipeline.generate_code("rust-debug", &program) {
        Ok(code) => println!("Generated debug output:\n{}", code),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n=== Framework Features ===");
    println!("✓ Modular architecture with separate concerns");
    println!("✓ Trait-based backend system");
    println!("✓ Template system for code patterns");
    println!("✓ Registry for backend management");
    println!("✓ Configurable code generation context");
    println!("✓ Debug and profiling support");
    println!("✓ Extensible for new target languages");
}