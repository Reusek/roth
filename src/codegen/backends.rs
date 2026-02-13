use crate::codegen::emitters::{CEmitter, RustEmitter};
use crate::codegen::framework::{
    Backend, BackendCapabilities, CodeEmitter, CodegenContext, CodegenResult, IRTranslator,
    NativeType, TargetInfo, TargetLanguage,
};
use crate::codegen::languages::{CLanguage, RustLanguage};
use crate::codegen::templates::TemplateEngine;
use crate::codegen::translators::{CTranslator, RustTranslator};
use crate::ir::IRProgram;

pub struct ModularRustBackend {
    emitter: RustEmitter,
    language: RustLanguage,
    translator: RustTranslator,
    templates: TemplateEngine,
}

impl ModularRustBackend {
    pub fn new() -> Self {
        Self {
            emitter: RustEmitter::new(),
            language: RustLanguage,
            translator: RustTranslator::new(),
            templates: TemplateEngine::new().with_rust_templates(),
        }
    }
}

impl Backend for ModularRustBackend {
    fn name(&self) -> &str {
        "rust-modular"
    }

    fn generate_program(&mut self, ir: &IRProgram, ctx: &mut CodegenContext) -> CodegenResult {
        self.emitter.clear();

        // Add required dependencies
        ctx.add_dependency("std::collections::HashMap".to_string());

        // Generate header
        let header = self.language.emit_header(ctx);
        for line in header.lines() {
            self.emitter.emit_line(line);
        }

        // Translate and emit the program
        let program_code = self.translator.translate_program(ir, ctx)?;
        for line in program_code.lines() {
            self.emitter.emit_line(line);
        }

        // Generate footer
        let footer = self.language.emit_footer(ctx);
        for line in footer.lines() {
            self.emitter.emit_line(line);
        }

        Ok(self.emitter.get_output())
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_inline_assembly: false,
            supports_tail_calls: true,
            supports_computed_goto: false,
            max_stack_size: None,
            native_types: vec![
                NativeType::Integer(32),
                NativeType::Integer(64),
                NativeType::String,
                NativeType::Array,
                NativeType::Struct,
            ],
            optimization_passes: vec![
                "dead_code_elimination".to_string(),
                "constant_folding".to_string(),
                "tail_call_optimization".to_string(),
            ],
        }
    }
}

pub struct ModularCBackend {
    emitter: CEmitter,
    language: CLanguage,
    translator: CTranslator,
    templates: TemplateEngine,
}

impl ModularCBackend {
    pub fn new() -> Self {
        Self {
            emitter: CEmitter::new(),
            language: CLanguage,
            translator: CTranslator::new(),
            templates: TemplateEngine::new().with_c_templates(),
        }
    }
}

impl Backend for ModularCBackend {
    fn name(&self) -> &str {
        "c-modular"
    }

    fn generate_program(&mut self, ir: &IRProgram, ctx: &mut CodegenContext) -> CodegenResult {
        self.emitter.clear();

        // Generate header
        let header = self.language.emit_header(ctx);
        for line in header.lines() {
            self.emitter.emit_line(line);
        }

        // Translate and emit the program
        let program_code = self.translator.translate_program(ir, ctx)?;
        for line in program_code.lines() {
            self.emitter.emit_line(line);
        }

        // Generate footer
        let footer = self.language.emit_footer(ctx);
        for line in footer.lines() {
            self.emitter.emit_line(line);
        }

        Ok(self.emitter.get_output())
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_inline_assembly: true,
            supports_tail_calls: false,
            supports_computed_goto: true,
            max_stack_size: Some(1000),
            native_types: vec![
                NativeType::Integer(8),
                NativeType::Integer(16),
                NativeType::Integer(32),
                NativeType::Integer(64),
                NativeType::Float(32),
                NativeType::Float(64),
                NativeType::Array,
                NativeType::Struct,
            ],
            optimization_passes: vec![
                "dead_code_elimination".to_string(),
                "constant_folding".to_string(),
                "loop_unrolling".to_string(),
            ],
        }
    }
}

pub struct DebugBackend {
    inner: Box<dyn Backend>,
    show_ir: bool,
    show_capabilities: bool,
}

impl DebugBackend {
    pub fn new(inner: Box<dyn Backend>) -> Self {
        Self {
            inner,
            show_ir: true,
            show_capabilities: true,
        }
    }

    pub fn with_ir_debug(mut self, show_ir: bool) -> Self {
        self.show_ir = show_ir;
        self
    }

    pub fn with_capabilities_debug(mut self, show_capabilities: bool) -> Self {
        self.show_capabilities = show_capabilities;
        self
    }
}

impl Backend for DebugBackend {
    fn name(&self) -> &str {
        // We need to return a static string, so we'll use a match instead
        match self.inner.name() {
            "rust-modular" => "rust-modular-debug",
            "c-modular" => "c-modular-debug",
            _ => "unknown-debug",
        }
    }

    fn generate_program(&mut self, ir: &IRProgram, ctx: &mut CodegenContext) -> CodegenResult {
        let mut output = String::new();

        if self.show_capabilities {
            output.push_str("=== BACKEND CAPABILITIES ===\n");
            let caps = self.inner.capabilities();
            output.push_str(&format!("Backend: {}\n", self.inner.name()));
            output.push_str(&format!(
                "Inline Assembly: {}\n",
                caps.supports_inline_assembly
            ));
            output.push_str(&format!("Tail Calls: {}\n", caps.supports_tail_calls));
            output.push_str(&format!("Computed Goto: {}\n", caps.supports_computed_goto));
            output.push_str(&format!("Max Stack Size: {:?}\n", caps.max_stack_size));
            output.push_str(&format!("Native Types: {:?}\n", caps.native_types));
            output.push_str(&format!(
                "Optimization Passes: {:?}\n",
                caps.optimization_passes
            ));
            output.push_str("\n");
        }

        if self.show_ir {
            output.push_str("=== IR PROGRAM ===\n");
            output.push_str(&format!("{:#?}\n\n", ir));
        }

        output.push_str("=== GENERATED CODE ===\n");
        let generated = self.inner.generate_program(ir, ctx)?;
        output.push_str(&generated);

        Ok(output)
    }

    fn capabilities(&self) -> BackendCapabilities {
        self.inner.capabilities()
    }
}

pub fn create_target_info(name: &str) -> TargetInfo {
    match name {
        "rust" | "rust-modular" => TargetInfo {
            name: "rust".to_string(),
            architecture: "x86_64".to_string(),
            pointer_size: 8,
            endianness: "little".to_string(),
        },
        "c" | "c-modular" => TargetInfo {
            name: "c".to_string(),
            architecture: "x86_64".to_string(),
            pointer_size: 8,
            endianness: "little".to_string(),
        },
        _ => TargetInfo {
            name: name.to_string(),
            architecture: "unknown".to_string(),
            pointer_size: 8,
            endianness: "little".to_string(),
        },
    }
}
