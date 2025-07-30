pub mod base;
pub mod rust;
pub mod c;
pub mod pattern;
pub mod generator;
pub mod rust_pattern;
pub mod c_pattern;
pub mod examples;
pub mod optimizer;
pub mod optimized_rust;
pub mod ir_generator;

use crate::types::AstNode;

pub trait CodeGenerator {
    fn generate(&mut self, ast: &AstNode) -> String;
    fn get_file_extension(&self) -> &str;
    fn get_compile_command(&self, filename: &str) -> String;
}

#[derive(Debug, Clone)]
pub enum Backend {
    Rust,
    C,
    RustPattern,
    CPattern,
    RustOptimized,
    RustIR,
    CIR,
    IRDebugRust,
    IRDebugC,
}

impl Backend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Some(Backend::Rust),
            "c" | "gcc" => Some(Backend::C),
            "rust-pattern" | "rs-pattern" => Some(Backend::RustPattern),
            "c-pattern" | "gcc-pattern" => Some(Backend::CPattern),
            "rust-optimized" | "rs-opt" => Some(Backend::RustOptimized),
            "rust-ir" | "rs-ir" => Some(Backend::RustIR),
            "c-ir" | "gcc-ir" => Some(Backend::CIR),
            "ir-debug-rust" | "ir-rust-debug" => Some(Backend::IRDebugRust),
            "ir-debug-c" | "ir-c-debug" => Some(Backend::IRDebugC),
            _ => None,
        }
    }
}

pub fn create_generator(backend: Backend) -> Box<dyn CodeGenerator> {
    match backend {
        Backend::Rust => Box::new(rust::RustGenerator::new()),
        Backend::C => Box::new(c::CGenerator::new()),
        Backend::RustPattern => Box::new(rust_pattern::RustPatternGenerator::new()),
        Backend::CPattern => Box::new(c_pattern::CPatternGenerator::new()),
        Backend::RustOptimized => Box::new(optimized_rust::OptimizedRustGenerator::new()),
        Backend::RustIR => Box::new(ir_generator::IRBasedRustGenerator::new()),
        Backend::CIR => Box::new(ir_generator::IRBasedCGenerator::new()),
        Backend::IRDebugRust => Box::new(ir_generator::IRDebugGenerator::new("rust")),
        Backend::IRDebugC => Box::new(ir_generator::IRDebugGenerator::new("c")),
    }
}