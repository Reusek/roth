use crate::types::AstNode;
use crate::codegen::{CodeGenerator, base::BaseGenerator};

pub struct RustGenerator {
    base: BaseGenerator,
}

impl RustGenerator {
    pub fn new() -> Self {
        Self {
            base: BaseGenerator::new(),
        }
    }

    fn generate_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Program(nodes) => {
                self.base.emit_line("// Generated Forth code");
                self.base.emit_line("use std::collections::HashMap;");
                self.base.emit_line("");
                self.base.emit_line("pub struct GeneratedForth {");
                self.base.indent();
                self.base.emit_line("stack: Vec<i32>,");
                self.base.emit_line("words: HashMap<String, Vec<String>>,");
                self.base.dedent();
                self.base.emit_line("}");
                self.base.emit_line("");
                
                for node in nodes {
                    self.generate_node(node);
                }
            },
            AstNode::Definition { name, body, .. } => {
                self.base.emit_line(&format!("// Definition: {}", name));
                self.base.emit_line(&format!("fn {}(&mut self) -> Result<(), String> {{", name.to_lowercase()));
                self.base.indent();
                
                for node in body {
                    self.generate_node(node);
                }
                
                self.base.emit_line("Ok(())");
                self.base.dedent();
                self.base.emit_line("}");
                self.base.emit_line("");
            },
            AstNode::Word(name, _) => {
                self.base.emit_line(&format!("self.execute_word(\"{}\")?;", name));
            },
            AstNode::Number(n, _) => {
                self.base.emit_line(&format!("self.stack.push({});", n));
            },
        }
    }
}

impl CodeGenerator for RustGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        self.base.clear();
        self.generate_node(ast);
        self.base.get_output().to_string()
    }

    fn get_file_extension(&self) -> &str {
        "rs"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("rustc {}", filename)
    }
}