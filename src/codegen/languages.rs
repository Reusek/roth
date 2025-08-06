use crate::codegen::framework::{TargetLanguage, CommentStyle, CodegenContext};

pub struct RustLanguage;

impl TargetLanguage for RustLanguage {
    fn file_extension(&self) -> &str {
        "rs"
    }

    fn comment_style(&self) -> CommentStyle {
        CommentStyle::DoubleSlash
    }

    fn compile_command(&self, filename: &str) -> String {
        let base_name = std::path::Path::new(filename)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        format!("rustc -O {} -o .build/{}", filename, base_name)
    }

    fn runtime_requirements(&self) -> Vec<String> {
        vec![
            "std::collections::HashMap".to_string(),
            "std::io".to_string(),
        ]
    }

    fn emit_header(&self, ctx: &CodegenContext) -> String {
        let mut header = String::new();
        header.push_str("// Generated from optimized IR\n");
        
        for dep in &ctx.dependencies {
            header.push_str(&format!("use {};\n", dep));
        }
        
        if !ctx.dependencies.is_empty() {
            header.push('\n');
        }

        header.push_str("pub struct OptimizedForth {\n");
        header.push_str("    stack: Vec<i32>,\n");
        header.push_str("    words: std::collections::HashMap<String, Vec<String>>,\n");
        header.push_str("    loop_stack: Vec<(i32, i32)>, // (index, limit) pairs\n");
        header.push_str("}\n\n");

        header.push_str("impl OptimizedForth {\n");
        header.push_str("    pub fn new() -> Self {\n");
        header.push_str("        Self {\n");
        header.push_str("            stack: Vec::new(),\n");
        header.push_str("            words: std::collections::HashMap::new(),\n");
        header.push_str("            loop_stack: Vec::new(),\n");
        header.push_str("        }\n");
        header.push_str("    }\n\n");

        header
    }

    fn emit_footer(&self, _ctx: &CodegenContext) -> String {
        "}\n".to_string()
    }
}

pub struct CLanguage;

impl TargetLanguage for CLanguage {
    fn file_extension(&self) -> &str {
        "c"
    }

    fn comment_style(&self) -> CommentStyle {
        CommentStyle::CStyle
    }

    fn compile_command(&self, filename: &str) -> String {
        let base_name = std::path::Path::new(filename)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        format!("gcc -O2 -o .build/{} {}", base_name, filename)
    }

    fn runtime_requirements(&self) -> Vec<String> {
        vec![
            "stdio.h".to_string(),
            "stdlib.h".to_string(),
            "string.h".to_string(),
        ]
    }

    fn emit_header(&self, ctx: &CodegenContext) -> String {
        let mut header = String::new();
        header.push_str("/* Generated from optimized IR */\n");
        
        for req in self.runtime_requirements() {
            header.push_str(&format!("#include <{}>\n", req));
        }
        header.push('\n');

        header.push_str("#define STACK_SIZE 1000\n");
        header.push_str("#define MAX_WORDS 100\n\n");

        header.push_str("typedef struct {\n");
        header.push_str("    int data[STACK_SIZE];\n");
        header.push_str("    int top;\n");
        header.push_str("} Stack;\n\n");

        header.push_str("typedef struct {\n");
        header.push_str("    int index;\n");
        header.push_str("    int limit;\n");
        header.push_str("} LoopFrame;\n\n");

        header.push_str("typedef struct {\n");
        header.push_str("    Stack stack;\n");
        header.push_str("    LoopFrame loop_stack[10];\n");
        header.push_str("    int loop_top;\n");
        header.push_str("} ForthVM;\n\n");

        if ctx.emit_debug_info {
            header.push_str("#define DEBUG_PRINT(fmt, ...) printf(\"[DEBUG] \" fmt \"\\n\", ##__VA_ARGS__)\n");
        } else {
            header.push_str("#define DEBUG_PRINT(fmt, ...) do {} while(0)\n");
        }
        header.push('\n');

        header.push_str("void init_vm(ForthVM* vm) {\n");
        header.push_str("    vm->stack.top = 0;\n");
        header.push_str("    vm->loop_top = 0;\n");
        header.push_str("}\n\n");

        header.push_str("void push(ForthVM* vm, int value) {\n");
        header.push_str("    if (vm->stack.top < STACK_SIZE) {\n");
        header.push_str("        vm->stack.data[vm->stack.top++] = value;\n");
        header.push_str("        DEBUG_PRINT(\"Push: %d (stack depth: %d)\", value, vm->stack.top);\n");
        header.push_str("    }\n");
        header.push_str("}\n\n");

        header.push_str("int pop(ForthVM* vm) {\n");
        header.push_str("    if (vm->stack.top > 0) {\n");
        header.push_str("        int value = vm->stack.data[--vm->stack.top];\n");
        header.push_str("        DEBUG_PRINT(\"Pop: %d (stack depth: %d)\", value, vm->stack.top);\n");
        header.push_str("        return value;\n");
        header.push_str("    }\n");
        header.push_str("    return 0;\n");
        header.push_str("}\n\n");

        header
    }

    fn emit_footer(&self, _ctx: &CodegenContext) -> String {
        let mut footer = String::new();
        footer.push_str("int main() {\n");
        footer.push_str("    ForthVM vm;\n");
        footer.push_str("    init_vm(&vm);\n");
        footer.push_str("    forth_main(&vm);\n");
        footer.push_str("    return 0;\n");
        footer.push_str("}\n");
        footer
    }
}

pub struct LLVMLanguage;

impl TargetLanguage for LLVMLanguage {
    fn file_extension(&self) -> &str {
        "ll"
    }

    fn comment_style(&self) -> CommentStyle {
        CommentStyle::Hash
    }

    fn compile_command(&self, filename: &str) -> String {
        let base_name = std::path::Path::new(filename)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        format!("llc {} -o {}.s && gcc {}.s -o .build/{}", filename, base_name, base_name, base_name)
    }

    fn runtime_requirements(&self) -> Vec<String> {
        vec![]
    }

    fn emit_header(&self, _ctx: &CodegenContext) -> String {
        let mut header = String::new();
        header.push_str("; Generated from optimized IR\n");
        header.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
        header.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");
        
        header.push_str("@stack = global [1000 x i32] zeroinitializer\n");
        header.push_str("@stack_ptr = global i32 0\n\n");
        
        header
    }

    fn emit_footer(&self, _ctx: &CodegenContext) -> String {
        String::new()
    }
}