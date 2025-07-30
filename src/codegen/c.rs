use crate::types::AstNode;
use crate::codegen::{CodeGenerator, base::BaseGenerator};

pub struct CGenerator {
    base: BaseGenerator,
}

impl CGenerator {
    pub fn new() -> Self {
        Self {
            base: BaseGenerator::new(),
        }
    }

    fn generate_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Program(nodes) => {
                self.base.emit_line("// Generated Forth code in C");
                self.base.emit_line("#include <stdio.h>");
                self.base.emit_line("#include <stdlib.h>");
                self.base.emit_line("#include <string.h>");
                self.base.emit_line("");
                self.base.emit_line("#define STACK_SIZE 1000");
                self.base.emit_line("#define MAX_WORDS 100");
                self.base.emit_line("");
                self.base.emit_line("typedef struct {");
                self.base.indent();
                self.base.emit_line("int data[STACK_SIZE];");
                self.base.emit_line("int top;");
                self.base.dedent();
                self.base.emit_line("} Stack;");
                self.base.emit_line("");
                self.base.emit_line("Stack stack = {0};");
                self.base.emit_line("");
                self.base.emit_line("void push(int value) {");
                self.base.indent();
                self.base.emit_line("if (stack.top < STACK_SIZE) {");
                self.base.indent();
                self.base.emit_line("stack.data[stack.top++] = value;");
                self.base.dedent();
                self.base.emit_line("} else {");
                self.base.indent();
                self.base.emit_line("printf(\"Stack overflow\\n\");");
                self.base.emit_line("exit(1);");
                self.base.dedent();
                self.base.emit_line("}");
                self.base.dedent();
                self.base.emit_line("}");
                self.base.emit_line("");
                self.base.emit_line("int pop() {");
                self.base.indent();
                self.base.emit_line("if (stack.top > 0) {");
                self.base.indent();
                self.base.emit_line("return stack.data[--stack.top];");
                self.base.dedent();
                self.base.emit_line("} else {");
                self.base.indent();
                self.base.emit_line("printf(\"Stack underflow\\n\");");
                self.base.emit_line("exit(1);");
                self.base.dedent();
                self.base.emit_line("}");
                self.base.dedent();
                self.base.emit_line("}");
                self.base.emit_line("");
                
                // Generate builtin operations
                self.generate_builtins();
                
                // Generate user-defined functions first
                for node in nodes {
                    if let AstNode::Definition { .. } = node {
                        self.generate_definition_node(node);
                    }
                }
                
                // Generate main function
                self.base.emit_line("int main() {");
                self.base.indent();
                for node in nodes {
                    if let AstNode::Definition { .. } = node {
                        // Skip definitions in main - they're already generated above
                    } else {
                        self.generate_main_node(node);
                    }
                }
                self.base.emit_line("return 0;");
                self.base.dedent();
                self.base.emit_line("}");
            },
            AstNode::Definition { .. } => {
                // Definitions are handled separately in generate_definition_node
            },
            AstNode::Word(name, _) => {
                match name.as_str() {
                    "+" => self.base.emit_line("forth_add();"),
                    "-" => self.base.emit_line("forth_sub();"),
                    "*" => self.base.emit_line("forth_mul();"),
                    "/" => self.base.emit_line("forth_div();"),
                    "DUP" => self.base.emit_line("forth_dup();"),
                    "DROP" => self.base.emit_line("forth_drop();"),
                    "SWAP" => self.base.emit_line("forth_swap();"),
                    "OVER" => self.base.emit_line("forth_over();"),
                    "." => self.base.emit_line("forth_dot();"),
                    ".S" => self.base.emit_line("forth_dots();"),
                    "CR" => self.base.emit_line("printf(\"\\n\");"),
                    _ => self.base.emit_line(&format!("{}();", name.to_lowercase())),
                }
            },
            AstNode::Number(n, _) => {
                self.base.emit_line(&format!("push({});", n));
            },
        }
    }

    fn generate_definition_node(&mut self, node: &AstNode) {
        if let AstNode::Definition { name, body, .. } = node {
            self.base.emit_line(&format!("// Definition: {}", name));
            self.base.emit_line(&format!("void {}() {{", name.to_lowercase()));
            self.base.indent();
            
            for node in body {
                self.generate_definition_body_node(node);
            }
            
            self.base.dedent();
            self.base.emit_line("}");
            self.base.emit_line("");
        }
    }

    fn generate_definition_body_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Word(name, _) => {
                match name.as_str() {
                    "+" => self.base.emit_line("forth_add();"),
                    "-" => self.base.emit_line("forth_sub();"),
                    "*" => self.base.emit_line("forth_mul();"),
                    "/" => self.base.emit_line("forth_div();"),
                    "DUP" => self.base.emit_line("forth_dup();"),
                    "DROP" => self.base.emit_line("forth_drop();"),
                    "SWAP" => self.base.emit_line("forth_swap();"),
                    "OVER" => self.base.emit_line("forth_over();"),
                    "." => self.base.emit_line("forth_dot();"),
                    ".S" => self.base.emit_line("forth_dots();"),
                    "CR" => self.base.emit_line("printf(\"\\n\");"),
                    _ => self.base.emit_line(&format!("{}();", name.to_lowercase())),
                }
            },
            AstNode::Number(n, _) => {
                self.base.emit_line(&format!("push({});", n));
            },
            _ => {}
        }
    }

    fn generate_main_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Word(name, _) => {
                match name.as_str() {
                    "+" => self.base.emit_line("forth_add();"),
                    "-" => self.base.emit_line("forth_sub();"),
                    "*" => self.base.emit_line("forth_mul();"),
                    "/" => self.base.emit_line("forth_div();"),
                    "DUP" => self.base.emit_line("forth_dup();"),
                    "DROP" => self.base.emit_line("forth_drop();"),
                    "SWAP" => self.base.emit_line("forth_swap();"),
                    "OVER" => self.base.emit_line("forth_over();"),
                    "." => self.base.emit_line("forth_dot();"),
                    ".S" => self.base.emit_line("forth_dots();"),
                    "CR" => self.base.emit_line("printf(\"\\n\");"),
                    _ => self.base.emit_line(&format!("{}();", name.to_lowercase())),
                }
            },
            AstNode::Number(n, _) => {
                self.base.emit_line(&format!("push({});", n));
            },
            _ => {}
        }
    }

    fn generate_builtins(&mut self) {
        // Arithmetic operations
        self.base.emit_line("void forth_add() {");
        self.base.indent();
        self.base.emit_line("int b = pop();");
        self.base.emit_line("int a = pop();");
        self.base.emit_line("push(a + b);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        self.base.emit_line("void forth_sub() {");
        self.base.indent();
        self.base.emit_line("int b = pop();");
        self.base.emit_line("int a = pop();");
        self.base.emit_line("push(a - b);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        self.base.emit_line("void forth_mul() {");
        self.base.indent();
        self.base.emit_line("int b = pop();");
        self.base.emit_line("int a = pop();");
        self.base.emit_line("push(a * b);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        self.base.emit_line("void forth_div() {");
        self.base.indent();
        self.base.emit_line("int b = pop();");
        self.base.emit_line("int a = pop();");
        self.base.emit_line("if (b == 0) {");
        self.base.indent();
        self.base.emit_line("printf(\"Division by zero\\n\");");
        self.base.emit_line("exit(1);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("push(a / b);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        // Stack operations
        self.base.emit_line("void forth_dup() {");
        self.base.indent();
        self.base.emit_line("if (stack.top > 0) {");
        self.base.indent();
        self.base.emit_line("push(stack.data[stack.top - 1]);");
        self.base.dedent();
        self.base.emit_line("} else {");
        self.base.indent();
        self.base.emit_line("printf(\"Stack underflow\\n\");");
        self.base.emit_line("exit(1);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        self.base.emit_line("void forth_drop() {");
        self.base.indent();
        self.base.emit_line("pop();");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        self.base.emit_line("void forth_swap() {");
        self.base.indent();
        self.base.emit_line("int b = pop();");
        self.base.emit_line("int a = pop();");
        self.base.emit_line("push(b);");
        self.base.emit_line("push(a);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        self.base.emit_line("void forth_over() {");
        self.base.indent();
        self.base.emit_line("if (stack.top >= 2) {");
        self.base.indent();
        self.base.emit_line("push(stack.data[stack.top - 2]);");
        self.base.dedent();
        self.base.emit_line("} else {");
        self.base.indent();
        self.base.emit_line("printf(\"Stack underflow\\n\");");
        self.base.emit_line("exit(1);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        // Output operations
        self.base.emit_line("void forth_dot() {");
        self.base.indent();
        self.base.emit_line("printf(\"%d \", pop());");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");

        self.base.emit_line("void forth_dots() {");
        self.base.indent();
        self.base.emit_line("printf(\"<%d> \", stack.top);");
        self.base.emit_line("for (int i = 0; i < stack.top; i++) {");
        self.base.indent();
        self.base.emit_line("printf(\"%d \", stack.data[i]);");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("printf(\"\\n\");");
        self.base.dedent();
        self.base.emit_line("}");
        self.base.emit_line("");
    }
}

impl CodeGenerator for CGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        self.base.clear();
        self.generate_node(ast);
        self.base.get_output().to_string()
    }

    fn get_file_extension(&self) -> &str {
        "c"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("gcc -o {} {}", filename.trim_end_matches(".c"), filename)
    }
}