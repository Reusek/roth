use crate::codegen::framework::{Backend, CodegenContext, TargetInfo};
use crate::codegen::backends::{ModularRustBackend, ModularCBackend, DebugBackend, create_target_info};
use std::collections::HashMap;

pub type BackendFactory = Box<dyn Fn() -> Box<dyn Backend>>;

pub struct BackendRegistry {
    backends: HashMap<String, BackendFactory>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            backends: HashMap::new(),
        };
        
        registry.register_default_backends();
        registry
    }

    pub fn register<F>(&mut self, name: &str, factory: F) 
    where F: Fn() -> Box<dyn Backend> + 'static 
    {
        self.backends.insert(name.to_string(), Box::new(factory));
    }

    pub fn create(&self, name: &str) -> Option<Box<dyn Backend>> {
        self.backends.get(name).map(|factory| factory())
    }

    pub fn list_backends(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }

    pub fn has_backend(&self, name: &str) -> bool {
        self.backends.contains_key(name)
    }

    fn register_default_backends(&mut self) {
        // Register modular backends
        self.register("rust", || Box::new(ModularRustBackend::new()));
        self.register("c", || Box::new(ModularCBackend::new()));
        
        // Register debug variants
        self.register("rust-debug", || {
            Box::new(DebugBackend::new(Box::new(ModularRustBackend::new())))
        });
        
        self.register("c-debug", || {
            Box::new(DebugBackend::new(Box::new(ModularCBackend::new())))
        });

        // Register aliases for backward compatibility
        self.register("rust-ir", || Box::new(ModularRustBackend::new()));
        self.register("c-ir", || Box::new(ModularCBackend::new()));
        self.register("ir-debug-rust", || {
            Box::new(DebugBackend::new(Box::new(ModularRustBackend::new())))
        });
        self.register("ir-debug-c", || {
            Box::new(DebugBackend::new(Box::new(ModularCBackend::new())))
        });
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CodegenPipeline {
    registry: BackendRegistry,
}

impl CodegenPipeline {
    pub fn new() -> Self {
        Self {
            registry: BackendRegistry::new(),
        }
    }

    pub fn with_custom_registry(registry: BackendRegistry) -> Self {
        Self { registry }
    }

    pub fn register_backend<F>(&mut self, name: &str, factory: F)
    where F: Fn() -> Box<dyn Backend> + 'static
    {
        self.registry.register(name, factory);
    }

    pub fn generate_code(&mut self, backend_name: &str, ir: &crate::ir::IRProgram) -> Result<String, String> {
        let mut backend = self.registry.create(backend_name)
            .ok_or_else(|| format!("Unknown backend: {}", backend_name))?;

        let target_info = create_target_info(backend_name);
        let mut ctx = CodegenContext::new(target_info);

        backend.generate_program(ir, &mut ctx)
            .map_err(|e| e.to_string())
    }

    pub fn list_available_backends(&self) -> Vec<String> {
        self.registry.list_backends()
    }

    pub fn get_backend_info(&self, name: &str) -> Option<BackendInfo> {
        if let Some(backend) = self.registry.create(name) {
            let capabilities = backend.capabilities();
            Some(BackendInfo {
                name: backend.name().to_string(),
                capabilities,
            })
        } else {
            None
        }
    }
}

impl Default for CodegenPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub name: String,
    pub capabilities: crate::codegen::framework::BackendCapabilities,
}

pub fn create_codegen_context(backend_name: &str) -> CodegenContext {
    let target_info = create_target_info(backend_name);
    CodegenContext::new(target_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = BackendRegistry::new();
        assert!(registry.has_backend("rust"));
        assert!(registry.has_backend("c"));
        assert!(registry.has_backend("rust-debug"));
        assert!(registry.has_backend("c-debug"));
    }

    #[test]
    fn test_backend_creation() {
        let registry = BackendRegistry::new();
        let backend = registry.create("rust");
        assert!(backend.is_some());
        assert_eq!(backend.unwrap().name(), "rust-modular");
    }

    #[test]
    fn test_pipeline() {
        let pipeline = CodegenPipeline::new();
        let backends = pipeline.list_available_backends();
        assert!(backends.contains(&"rust".to_string()));
        assert!(backends.contains(&"c".to_string()));
    }

    #[test]
    fn test_custom_backend_registration() {
        let mut pipeline = CodegenPipeline::new();
        
        pipeline.register_backend("custom", || {
            Box::new(ModularRustBackend::new())
        });
        
        assert!(pipeline.list_available_backends().contains(&"custom".to_string()));
    }
}