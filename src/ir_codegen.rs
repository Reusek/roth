use crate::ir::{IRProgram, IRFunction, IRInstruction, IRValue, BinaryOpKind, UnaryOpKind};
use crate::codegen::CodeGenerator;
use std::collections::HashMap;

/// Generates Rust code from IR
pub struct IRRustGenerator {
    indent_level: usize,
}

impl IRRustGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    fn emit_indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    pub fn generate_program(&mut self, program: &IRProgram) -> String {
        let mut output = String::new();
        
        // Generate header
        output.push_str("// Generated from optimized IR\n");
        output.push_str("use std::collections::HashMap;\n\n");
        output.push_str("pub struct OptimizedForth {\n");
        output.push_str("    stack: Vec<i32>,\n");
        output.push_str("    words: HashMap<String, Vec<String>>,\n");
        output.push_str("}\n\n");
        
        output.push_str("impl OptimizedForth {\n");
        self.indent_level += 1;
        
        // Constructor
        output.push_str(&format!("{}pub fn new() -> Self {{\n", self.emit_indent()));
        self.indent_level += 1;
        output.push_str(&format!("{}Self {{\n", self.emit_indent()));
        self.indent_level += 1;
        output.push_str(&format!("{}stack: Vec::new(),\n", self.emit_indent()));
        output.push_str(&format!("{}words: HashMap::new(),\n", self.emit_indent()));
        self.indent_level -= 1;
        output.push_str(&format!("{}}}\n", self.emit_indent()));
        self.indent_level -= 1;
        output.push_str(&format!("{}}}\n\n", self.emit_indent()));
        
        // Generate user-defined functions
        for (name, function) in &program.functions {
            if name != "main" {
                output.push_str(&self.generate_function(function));
                output.push('\n');
            }
        }
        
        // Generate main execution function
        output.push_str(&format!("{}pub fn execute(&mut self) -> Result<(), String> {{\n", self.emit_indent()));
        self.indent_level += 1;
        output.push_str(&self.generate_function_body(&program.main));
        output.push_str(&format!("{}Ok(())\n", self.emit_indent()));
        self.indent_level -= 1;
        output.push_str(&format!("{}}}\n", self.emit_indent()));
        
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn generate_function(&mut self, function: &IRFunction) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}// Function: {} (consumes: {}, produces: {})\n", 
                                self.emit_indent(), function.name, 
                                function.stack_effect.consumes, function.stack_effect.produces));
        output.push_str(&format!("{}fn {}(&mut self) -> Result<(), String> {{\n", 
                                self.emit_indent(), function.name.to_lowercase()));
        self.indent_level += 1;
        
        output.push_str(&self.generate_function_body(function));
        
        output.push_str(&format!("{}Ok(())\n", self.emit_indent()));
        self.indent_level -= 1;
        output.push_str(&format!("{}}}\n", self.emit_indent()));
        
        output
    }

    fn generate_function_body(&mut self, function: &IRFunction) -> String {
        let mut output = String::new();
        
        for instruction in &function.instructions {
            output.push_str(&self.generate_instruction(instruction));
        }
        
        output
    }

    fn generate_instruction(&self, instruction: &IRInstruction) -> String {
        match instruction {
            IRInstruction::Push(value) => {
                format!("{}self.stack.push({});\n", self.emit_indent(), self.generate_value(value))
            }
            IRInstruction::LoadConst(n) => {
                format!("{}self.stack.push({});\n", self.emit_indent(), n)
            }
            IRInstruction::Pop => {
                format!("{}self.stack.pop();\n", self.emit_indent())
            }
            IRInstruction::Drop => {
                format!("{}self.stack.pop();\n", self.emit_indent())
            }
            IRInstruction::Dup => {
                format!("{}{{ let top = *self.stack.last().unwrap(); self.stack.push(top); }}\n", self.emit_indent())
            }
            IRInstruction::Swap => {
                format!("{}{{ let len = self.stack.len(); self.stack.swap(len-1, len-2); }}\n", self.emit_indent())
            }
            IRInstruction::Over => {
                format!("{}{{ let val = self.stack[self.stack.len()-2]; self.stack.push(val); }}\n", self.emit_indent())
            }
            IRInstruction::Rot => {
                format!("{}{{ let c = self.stack.pop().unwrap(); let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(b); self.stack.push(c); self.stack.push(a); }}\n", self.emit_indent())
            }
            IRInstruction::Add => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }}\n", self.emit_indent())
            }
            IRInstruction::Sub => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a - b); }}\n", self.emit_indent())
            }
            IRInstruction::Mul => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }}\n", self.emit_indent())
            }
            IRInstruction::Div => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a / b); }}\n", self.emit_indent())
            }
            IRInstruction::Mod => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a % b); }}\n", self.emit_indent())
            }
            IRInstruction::Neg => {
                format!("{}{{ let a = self.stack.pop().unwrap(); self.stack.push(-a); }}\n", self.emit_indent())
            }
            IRInstruction::Equal => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a == b {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::NotEqual => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a != b {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::Less => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a < b {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::Greater => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a > b {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::LessEqual => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a <= b {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::GreaterEqual => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a >= b {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::And => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a != 0 && b != 0 {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::Or => {
                format!("{}{{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a != 0 || b != 0 {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::Not => {
                format!("{}{{ let a = self.stack.pop().unwrap(); self.stack.push(if a == 0 {{ -1 }} else {{ 0 }}); }}\n", self.emit_indent())
            }
            IRInstruction::Print => {
                format!("{}print!(\"{{}}\", self.stack.pop().unwrap());\n", self.emit_indent())
            }
            IRInstruction::PrintStack => {
                format!("{}println!(\"<{{}}> {{:?}}\", self.stack.len(), self.stack);\n", self.emit_indent())
            }
            IRInstruction::PrintChar => {
                format!("{}print!(\"{{}}\", char::from(self.stack.pop().unwrap() as u8));\n", self.emit_indent())
            }
            IRInstruction::ReadChar => {
                format!("{}// ReadChar not implemented in this generator\n", self.emit_indent())
            }
            IRInstruction::Call(name) => {
                format!("{}self.{}()?;\n", self.emit_indent(), name.to_lowercase())
            }
            IRInstruction::Return => {
                format!("{}return Ok(());\n", self.emit_indent())
            }
            IRInstruction::BinaryOp(op, a, b) => {
                let op_str = match op {
                    BinaryOpKind::Add => "+",
                    BinaryOpKind::Sub => "-",
                    BinaryOpKind::Mul => "*",
                    BinaryOpKind::Div => "/",
                    BinaryOpKind::Mod => "%",
                    BinaryOpKind::Equal => "==",
                    BinaryOpKind::NotEqual => "!=",
                    BinaryOpKind::Less => "<",
                    BinaryOpKind::Greater => ">",
                    BinaryOpKind::LessEqual => "<=",
                    BinaryOpKind::GreaterEqual => ">=",
                    BinaryOpKind::And => "&&",
                    BinaryOpKind::Or => "||",
                };
                format!("{}self.stack.push({} {} {});\n", 
                       self.emit_indent(), 
                       self.generate_value(a), 
                       op_str, 
                       self.generate_value(b))
            }
            IRInstruction::UnaryOp(op, a) => {
                let op_str = match op {
                    UnaryOpKind::Neg => "-",
                    UnaryOpKind::Not => "!",
                };
                format!("{}self.stack.push({}{});\n", 
                       self.emit_indent(), 
                       op_str, 
                       self.generate_value(a))
            }
            IRInstruction::Comment(text) => {
                format!("{}// {}\n", self.emit_indent(), text)
            }
            IRInstruction::Label(_) => {
                // Labels are not needed in Rust code generation
                String::new()
            }
            IRInstruction::Jump(_) | IRInstruction::JumpIf(_) | IRInstruction::JumpIfNot(_) => {
                format!("{}// Jump instructions not implemented in this generator\n", self.emit_indent())
            }
            IRInstruction::Load(_) | IRInstruction::Store(_) => {
                format!("{}// Memory operations not implemented in this generator\n", self.emit_indent())
            }
            IRInstruction::StackGet(pos) => {
                format!("{}self.stack.push(self.stack[self.stack.len() - 1 - {}]);\n", self.emit_indent(), pos)
            }
            IRInstruction::StackSet(pos, value) => {
                format!("{}self.stack[self.stack.len() - 1 - {}] = {};\n", 
                       self.emit_indent(), pos, self.generate_value(value))
            }
            IRInstruction::StackAlloc(size) => {
                format!("{}self.stack.reserve({});\n", self.emit_indent(), size)
            }
            IRInstruction::StackFree(_) => {
                // No explicit stack freeing needed in Rust
                String::new()
            }
            IRInstruction::Nop => {
                format!("{}// nop\n", self.emit_indent())
            }
        }
    }

    fn generate_value(&self, value: &IRValue) -> String {
        match value {
            IRValue::Constant(n) => n.to_string(),
            IRValue::StackTop => "(*self.stack.last().unwrap())".to_string(),
            IRValue::StackPos(pos) => format!("self.stack[self.stack.len() - 1 - {}]", pos),
            IRValue::Variable(name) => format!("/* variable {} */", name),
            IRValue::Temporary(id) => format!("/* temp {} */", id),
        }
    }
}

impl CodeGenerator for IRRustGenerator {
    fn generate(&mut self, _ast: &crate::types::AstNode) -> String {
        // This implementation expects to be called with an IR program, not an AST
        // In practice, you'd convert AST to IR first, then call generate_program
        "// Use generate_program method with IR instead".to_string()
    }

    fn get_file_extension(&self) -> &str {
        "rs"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("rustc -O {}", filename)
    }
}

/// Generates C code from IR
pub struct IRCGenerator {
    indent_level: usize,
}

impl IRCGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    fn emit_indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    pub fn generate_program(&mut self, program: &IRProgram) -> String {
        let mut output = String::new();
        
        // Generate header
        output.push_str("// Generated from optimized IR\n");
        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <stdlib.h>\n");
        output.push_str("#include <string.h>\n\n");
        output.push_str("#define STACK_SIZE 1000\n\n");
        
        // Generate stack structure
        output.push_str("typedef struct {\n");
        output.push_str("    int data[STACK_SIZE];\n");
        output.push_str("    int top;\n");
        output.push_str("} Stack;\n\n");
        output.push_str("Stack stack = {0};\n\n");
        
        // Generate stack functions
        self.generate_stack_functions(&mut output);
        
        // Generate user-defined functions
        for (name, function) in &program.functions {
            if name != "main" {
                output.push_str(&self.generate_function(function));
                output.push('\n');
            }
        }
        
        // Generate main function
        output.push_str("int main() {\n");
        self.indent_level += 1;
        output.push_str(&self.generate_function_body(&program.main));
        output.push_str(&format!("{}return 0;\n", self.emit_indent()));
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn generate_stack_functions(&self, output: &mut String) {
        output.push_str("void push(int value) {\n");
        output.push_str("    if (stack.top < STACK_SIZE) {\n");
        output.push_str("        stack.data[stack.top++] = value;\n");
        output.push_str("    } else {\n");
        output.push_str("        printf(\"Stack overflow\\n\");\n");
        output.push_str("        exit(1);\n");
        output.push_str("    }\n");
        output.push_str("}\n\n");
        
        output.push_str("int pop() {\n");
        output.push_str("    if (stack.top > 0) {\n");
        output.push_str("        return stack.data[--stack.top];\n");
        output.push_str("    } else {\n");
        output.push_str("        printf(\"Stack underflow\\n\");\n");
        output.push_str("        exit(1);\n");
        output.push_str("    }\n");
        output.push_str("}\n\n");
    }

    fn generate_function(&mut self, function: &IRFunction) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("// Function: {} (consumes: {}, produces: {})\n", 
                                function.name, function.stack_effect.consumes, function.stack_effect.produces));
        output.push_str(&format!("void {}() {{\n", function.name.to_lowercase()));
        self.indent_level += 1;
        
        output.push_str(&self.generate_function_body(function));
        
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn generate_function_body(&mut self, function: &IRFunction) -> String {
        let mut output = String::new();
        
        for instruction in &function.instructions {
            output.push_str(&self.generate_instruction(instruction));
        }
        
        output
    }

    fn generate_instruction(&self, instruction: &IRInstruction) -> String {
        match instruction {
            IRInstruction::Push(value) => {
                format!("{}push({});\n", self.emit_indent(), self.generate_value(value))
            }
            IRInstruction::LoadConst(n) => {
                format!("{}push({});\n", self.emit_indent(), n)
            }
            IRInstruction::Pop | IRInstruction::Drop => {
                format!("{}pop();\n", self.emit_indent())
            }
            IRInstruction::Dup => {
                format!("{}if (stack.top > 0) {{ push(stack.data[stack.top - 1]); }} else {{ printf(\"Stack underflow\\n\"); exit(1); }}\n", self.emit_indent())
            }
            IRInstruction::Swap => {
                format!("{}{{ int b = pop(); int a = pop(); push(b); push(a); }}\n", self.emit_indent())
            }
            IRInstruction::Over => {
                format!("{}if (stack.top >= 2) {{ push(stack.data[stack.top - 2]); }} else {{ printf(\"Stack underflow\\n\"); exit(1); }}\n", self.emit_indent())
            }
            IRInstruction::Add => {
                format!("{}{{ int b = pop(); int a = pop(); push(a + b); }}\n", self.emit_indent())
            }
            IRInstruction::Sub => {
                format!("{}{{ int b = pop(); int a = pop(); push(a - b); }}\n", self.emit_indent())
            }
            IRInstruction::Mul => {
                format!("{}{{ int b = pop(); int a = pop(); push(a * b); }}\n", self.emit_indent())
            }
            IRInstruction::Div => {
                format!("{}{{ int b = pop(); int a = pop(); if (b == 0) {{ printf(\"Division by zero\\n\"); exit(1); }} push(a / b); }}\n", self.emit_indent())
            }
            IRInstruction::Print => {
                format!("{}printf(\"%d \", pop());\n", self.emit_indent())
            }
            IRInstruction::PrintStack => {
                format!("{}printf(\"<%d> \", stack.top); for (int i = 0; i < stack.top; i++) {{ printf(\"%d \", stack.data[i]); }} printf(\"\\n\");\n", self.emit_indent())
            }
            IRInstruction::PrintChar => {
                format!("{}printf(\"%c\", (char)pop());\n", self.emit_indent())
            }
            IRInstruction::Call(name) => {
                format!("{}{}();\n", self.emit_indent(), name.to_lowercase())
            }
            IRInstruction::Return => {
                format!("{}return;\n", self.emit_indent())
            }
            IRInstruction::Comment(text) => {
                format!("{}// {}\n", self.emit_indent(), text)
            }
            _ => {
                format!("{}// Instruction not implemented: {:?}\n", self.emit_indent(), instruction)
            }
        }
    }

    fn generate_value(&self, value: &IRValue) -> String {
        match value {
            IRValue::Constant(n) => n.to_string(),
            IRValue::StackTop => "stack.data[stack.top - 1]".to_string(),
            IRValue::StackPos(pos) => format!("stack.data[stack.top - 1 - {}]", pos),
            IRValue::Variable(name) => format!("/* variable {} */", name),
            IRValue::Temporary(id) => format!("/* temp {} */", id),
        }
    }
}

impl CodeGenerator for IRCGenerator {
    fn generate(&mut self, _ast: &crate::types::AstNode) -> String {
        // This implementation expects to be called with an IR program, not an AST
        "// Use generate_program method with IR instead".to_string()
    }

    fn get_file_extension(&self) -> &str {
        "c"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("gcc -O2 -o {} {}", filename.trim_end_matches(".c"), filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IRBuilder, IRValue};

    #[test]
    fn test_rust_generation() {
        let mut builder = IRBuilder::new("main");
        builder.emit(IRInstruction::LoadConst(42));
        builder.emit(IRInstruction::Print);
        
        let program = builder.build();
        let mut generator = IRRustGenerator::new();
        let code = generator.generate_program(&program);
        
        assert!(code.contains("self.stack.push(42);"));
        assert!(code.contains("print!"));
    }

    #[test]
    fn test_c_generation() {
        let mut builder = IRBuilder::new("main");
        builder.emit(IRInstruction::LoadConst(42));
        builder.emit(IRInstruction::Print);
        
        let program = builder.build();
        let mut generator = IRCGenerator::new();
        let code = generator.generate_program(&program);
        
        assert!(code.contains("push(42);"));
        assert!(code.contains("printf"));
    }
}