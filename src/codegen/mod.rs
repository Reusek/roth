pub mod base;
pub mod rust;
pub mod c;

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
}

impl Backend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Some(Backend::Rust),
            "c" | "gcc" => Some(Backend::C),
            _ => None,
        }
    }
}

pub fn create_generator(backend: Backend) -> Box<dyn CodeGenerator> {
    match backend {
        Backend::Rust => Box::new(rust::RustGenerator::new()),
        Backend::C => Box::new(c::CGenerator::new()),
    }
}