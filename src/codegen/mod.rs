pub mod ir_generator;

// New modular framework
pub mod backends;
pub mod emitters;
pub mod framework;
pub mod languages;
pub mod registry;
pub mod templates;
pub mod translators;

use crate::types::AstNode;
pub use framework::{Backend as NewBackend, CodegenContext, CodegenResult};
pub use registry::{BackendRegistry, CodegenPipeline};

// Legacy trait for backward compatibility
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
    // New framework backends
    ModularRust,
    ModularC,
    ModularRustDebug,
    ModularCDebug,
}

impl Backend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust-ir" | "rs-ir" | "rust" | "rs" => Some(Backend::RustIR),
            "c-ir" | "gcc-ir" | "c" | "gcc" => Some(Backend::CIR),
            "ir-debug-rust" | "ir-rust-debug" => Some(Backend::IRDebugRust),
            "ir-debug-c" | "ir-c-debug" => Some(Backend::IRDebugC),
            // New framework backends
            "rust-modular" => Some(Backend::ModularRust),
            "c-modular" => Some(Backend::ModularC),
            "rust-debug" => Some(Backend::ModularRustDebug),
            "c-debug" => Some(Backend::ModularCDebug),
            _ => None,
        }
    }

    pub fn to_registry_name(&self) -> &str {
        match self {
            Backend::RustIR => "rust-ir",
            Backend::CIR => "c-ir",
            Backend::IRDebugRust => "ir-debug-rust",
            Backend::IRDebugC => "ir-debug-c",
            Backend::ModularRust => "rust",
            Backend::ModularC => "c",
            Backend::ModularRustDebug => "rust-debug",
            Backend::ModularCDebug => "c-debug",
        }
    }
}

pub fn create_generator(backend: Backend) -> Box<dyn CodeGenerator> {
    match backend {
        Backend::RustIR => Box::new(ir_generator::IRBasedRustGenerator::new()),
        Backend::CIR => Box::new(ir_generator::IRBasedCGenerator::new()),
        Backend::IRDebugRust => Box::new(ir_generator::IRDebugGenerator::new("rust")),
        Backend::IRDebugC => Box::new(ir_generator::IRDebugGenerator::new("c")),
        // For new framework backends, we'll create a wrapper
        Backend::ModularRust
        | Backend::ModularC
        | Backend::ModularRustDebug
        | Backend::ModularCDebug => Box::new(ModularCodeGeneratorWrapper::new(backend)),
    }
}

// Wrapper to make new framework compatible with legacy interface
struct ModularCodeGeneratorWrapper {
    backend_name: String,
    pipeline: CodegenPipeline,
}

impl ModularCodeGeneratorWrapper {
    fn new(backend: Backend) -> Self {
        Self {
            backend_name: backend.to_registry_name().to_string(),
            pipeline: CodegenPipeline::new(),
        }
    }
}

impl CodeGenerator for ModularCodeGeneratorWrapper {
    fn generate(&mut self, ast: &AstNode) -> String {
        // Convert AST to IR first (this would need to be implemented)
        // For now, return a placeholder
        format!(
            "// Generated using new modular framework\n// Backend: {}\n// AST: {:#?}",
            self.backend_name, ast
        )
    }

    fn get_file_extension(&self) -> &str {
        match self.backend_name.as_str() {
            name if name.contains("rust") => "rs",
            name if name.contains("c") => "c",
            _ => "txt",
        }
    }

    fn get_compile_command(&self, filename: &str) -> String {
        let base_name = std::path::Path::new(filename)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();

        match self.backend_name.as_str() {
            name if name.contains("rust") => {
                format!("rustc -O {} -o .build/{}", filename, base_name)
            }
            name if name.contains("c") => format!("gcc -O2 -o .build/{} {}", base_name, filename),
            _ => format!("# Unknown backend: {}", self.backend_name),
        }
    }
}
