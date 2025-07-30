pub mod ir_generator;

use crate::types::AstNode;

pub trait CodeGenerator {
    fn generate(&mut self, ast: &AstNode) -> String;
    fn get_file_extension(&self) -> &str;
    fn get_compile_command(&self, filename: &str) -> String;
}

#[derive(Debug, Clone)]
pub enum Backend {
    RustIR,
    CIR,
    IRDebugRust,
    IRDebugC,
}

impl Backend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust-ir" | "rs-ir" | "rust" | "rs" => Some(Backend::RustIR),
            "c-ir" | "gcc-ir" | "c" | "gcc" => Some(Backend::CIR),
            "ir-debug-rust" | "ir-rust-debug" => Some(Backend::IRDebugRust),
            "ir-debug-c" | "ir-c-debug" => Some(Backend::IRDebugC),
            _ => None,
        }
    }
}

pub fn create_generator(backend: Backend) -> Box<dyn CodeGenerator> {
    match backend {
        Backend::RustIR => Box::new(ir_generator::IRBasedRustGenerator::new()),
        Backend::CIR => Box::new(ir_generator::IRBasedCGenerator::new()),
        Backend::IRDebugRust => Box::new(ir_generator::IRDebugGenerator::new("rust")),
        Backend::IRDebugC => Box::new(ir_generator::IRDebugGenerator::new("c")),
    }
}