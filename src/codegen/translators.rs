use crate::codegen::framework::{IRTranslator, CodegenContext, CodegenResult, CodegenError};
use crate::ir::{IRProgram, IRFunction, IRInstruction, IRValue, BinaryOpKind, UnaryOpKind, IRLabel};

pub struct RustTranslator {
    loop_counter: usize,
}

impl RustTranslator {
    pub fn new() -> Self {
        Self {
            loop_counter: 0,
        }
    }

    fn translate_value(&self, value: &IRValue) -> String {
        match value {
            IRValue::Constant(n) => n.to_string(),
            IRValue::StackTop => "self.stack.last().copied().unwrap_or(0)".to_string(),
            IRValue::StackPos(pos) => {
                format!("self.stack.get(self.stack.len().saturating_sub({} + 1)).copied().unwrap_or(0)", pos)
            }
            IRValue::Variable(name) => format!("/* variable {} */", name),
            IRValue::Temporary(id) => format!("temp_{}", id),
        }
    }

    fn translate_binary_op(&self, op: &BinaryOpKind) -> &'static str {
        match op {
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
        }
    }

    fn translate_unary_op(&self, op: &UnaryOpKind) -> &'static str {
        match op {
            UnaryOpKind::Neg => "-",
            UnaryOpKind::Not => "!",
        }
    }
}

impl IRTranslator for RustTranslator {
    fn translate_instruction(&mut self, instr: &IRInstruction, ctx: &mut CodegenContext) -> CodegenResult {
        let code = match instr {
            IRInstruction::Push(value) => {
                format!("self.stack.push({});", self.translate_value(value))
            }
            IRInstruction::Pop => {
                "self.stack.pop();".to_string()
            }
            IRInstruction::Dup => {
                "if let Some(&top) = self.stack.last() { self.stack.push(top); }".to_string()
            }
            IRInstruction::Drop => {
                "self.stack.pop();".to_string()
            }
            IRInstruction::Swap => {
                let mut code = String::new();
                code.push_str("if self.stack.len() >= 2 {\n");
                code.push_str("    let len = self.stack.len();\n");
                code.push_str("    self.stack.swap(len - 1, len - 2);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Over => {
                let mut code = String::new();
                code.push_str("if self.stack.len() >= 2 {\n");
                code.push_str("    let len = self.stack.len();\n");
                code.push_str("    let val = self.stack[len - 2];\n");
                code.push_str("    self.stack.push(val);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Rot => {
                let mut code = String::new();
                code.push_str("if self.stack.len() >= 3 {\n");
                code.push_str("    let len = self.stack.len();\n");
                code.push_str("    let a = self.stack.remove(len - 3);\n");
                code.push_str("    self.stack.push(a);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Add => {
                let mut code = String::new();
                code.push_str("if self.stack.len() >= 2 {\n");
                code.push_str("    let b = self.stack.pop().unwrap();\n");
                code.push_str("    let a = self.stack.pop().unwrap();\n");
                code.push_str("    self.stack.push(a + b);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Sub => {
                let mut code = String::new();
                code.push_str("if self.stack.len() >= 2 {\n");
                code.push_str("    let b = self.stack.pop().unwrap();\n");
                code.push_str("    let a = self.stack.pop().unwrap();\n");
                code.push_str("    self.stack.push(a - b);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Mul => {
                let mut code = String::new();
                code.push_str("if self.stack.len() >= 2 {\n");
                code.push_str("    let b = self.stack.pop().unwrap();\n");
                code.push_str("    let a = self.stack.pop().unwrap();\n");
                code.push_str("    self.stack.push(a * b);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Div => {
                let mut code = String::new();
                code.push_str("if self.stack.len() >= 2 {\n");
                code.push_str("    let b = self.stack.pop().unwrap();\n");
                code.push_str("    let a = self.stack.pop().unwrap();\n");
                code.push_str("    if b != 0 { self.stack.push(a / b); }\n");
                code.push_str("}");
                code
            }
            IRInstruction::Print => {
                "if let Some(val) = self.stack.pop() { println!(\"{}\", val); }".to_string()
            }
            IRInstruction::PrintStack => {
                "println!(\"{:?}\", self.stack);".to_string()
            }
            IRInstruction::Jump(label) => {
                format!("// goto {}", label)
            }
            IRInstruction::JumpIf(label) => {
                format!("if self.stack.pop().unwrap_or(0) != 0 {{ /* goto {} */ }}", label)
            }
            IRInstruction::JumpIfNot(label) => {
                format!("if self.stack.pop().unwrap_or(0) == 0 {{ /* goto {} */ }}", label)
            }
            IRInstruction::Label(label) => {
                format!("// {}: ", label)
            }
            IRInstruction::Comment(text) => {
                format!("// {}", text)
            }
            IRInstruction::LoadConst(value) => {
                format!("self.stack.push({});", value)
            }
            IRInstruction::BinaryOp(op, left, right) => {
                let left_val = self.translate_value(left);
                let right_val = self.translate_value(right);
                let op_str = self.translate_binary_op(op);
                format!("self.stack.push(({}) {} ({}));", left_val, op_str, right_val)
            }
            IRInstruction::UnaryOp(op, operand) => {
                let operand_val = self.translate_value(operand);
                let op_str = self.translate_unary_op(op);
                format!("self.stack.push({}({}));", op_str, operand_val)
            }
            IRInstruction::DoLoop(loop_label, end_label) => {
                let loop_id = self.loop_counter;
                self.loop_counter += 1;
                let mut code = String::new();
                code.push_str("if self.stack.len() >= 2 {\n");
                code.push_str("    let limit = self.stack.pop().unwrap();\n");
                code.push_str("    let start = self.stack.pop().unwrap();\n");
                code.push_str("    if start < limit {\n");
                code.push_str(&format!("        for loop_idx_{} in start..limit {{\n", loop_id));
                code.push_str(&format!("            self.loop_stack.push((loop_idx_{}, limit));\n", loop_id));
                code.push_str(&format!("            // loop body for {} to {}\n", loop_label, end_label));
                code
            }
            IRInstruction::Loop(label) => {
                "        }\n        self.loop_stack.pop();\n    }\n}".to_string()
            }
            IRInstruction::PushLoopIndex => {
                "if let Some(&(idx, _)) = self.loop_stack.last() { self.stack.push(idx); }".to_string()
            }
            _ => {
                format!("// TODO: implement {:?}", instr)
            }
        };

        Ok(code)
    }

    fn translate_function(&mut self, func: &IRFunction, ctx: &mut CodegenContext) -> CodegenResult {
        let mut code = String::new();
        
        if func.name == "main" {
            code.push_str("    pub fn execute(&mut self) {\n");
        } else {
            code.push_str(&format!("    pub fn {}(&mut self) {{\n", func.name));
        }

        for instr in &func.instructions {
            let instr_code = self.translate_instruction(instr, ctx)?;
            for line in instr_code.lines() {
                if !line.trim().is_empty() {
                    code.push_str(&format!("        {}\n", line));
                }
            }
        }

        code.push_str("    }\n");
        Ok(code)
    }

    fn translate_program(&mut self, program: &IRProgram, ctx: &mut CodegenContext) -> CodegenResult {
        let mut code = String::new();

        // Translate all functions
        for (name, func) in &program.functions {
            let func_code = self.translate_function(func, ctx)?;
            code.push_str(&func_code);
            code.push('\n');
        }

        // Translate main function
        let main_code = self.translate_function(&program.main, ctx)?;
        code.push_str(&main_code);

        Ok(code)
    }
}

pub struct CTranslator {
    loop_counter: usize,
}

impl CTranslator {
    pub fn new() -> Self {
        Self {
            loop_counter: 0,
        }
    }

    fn translate_value(&self, value: &IRValue) -> String {
        match value {
            IRValue::Constant(n) => n.to_string(),
            IRValue::StackTop => "vm->stack.data[vm->stack.top - 1]".to_string(),
            IRValue::StackPos(pos) => {
                format!("vm->stack.data[vm->stack.top - {} - 1]", pos)
            }
            IRValue::Variable(name) => format!("/* variable {} */", name),
            IRValue::Temporary(id) => format!("temp_{}", id),
        }
    }

    fn translate_binary_op(&self, op: &BinaryOpKind) -> &'static str {
        match op {
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
        }
    }
}

impl IRTranslator for CTranslator {
    fn translate_instruction(&mut self, instr: &IRInstruction, ctx: &mut CodegenContext) -> CodegenResult {
        let code = match instr {
            IRInstruction::Push(value) => {
                format!("push(vm, {});", self.translate_value(value))
            }
            IRInstruction::Pop => {
                "pop(vm);".to_string()
            }
            IRInstruction::Add => {
                let mut code = String::new();
                code.push_str("if (vm->stack.top >= 2) {\n");
                code.push_str("    int b = pop(vm);\n");
                code.push_str("    int a = pop(vm);\n");
                code.push_str("    push(vm, a + b);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Sub => {
                let mut code = String::new();
                code.push_str("if (vm->stack.top >= 2) {\n");
                code.push_str("    int b = pop(vm);\n");
                code.push_str("    int a = pop(vm);\n");
                code.push_str("    push(vm, a - b);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Mul => {
                let mut code = String::new();
                code.push_str("if (vm->stack.top >= 2) {\n");
                code.push_str("    int b = pop(vm);\n");
                code.push_str("    int a = pop(vm);\n");
                code.push_str("    push(vm, a * b);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Div => {
                let mut code = String::new();
                code.push_str("if (vm->stack.top >= 2) {\n");
                code.push_str("    int b = pop(vm);\n");
                code.push_str("    int a = pop(vm);\n");
                code.push_str("    if (b != 0) push(vm, a / b);\n");
                code.push_str("}");
                code
            }
            IRInstruction::Print => {
                "if (vm->stack.top > 0) printf(\"%d\\n\", pop(vm));".to_string()
            }
            IRInstruction::PrintStack => {
                let mut code = String::new();
                code.push_str("printf(\"Stack: \");\n");
                code.push_str("for (int i = 0; i < vm->stack.top; i++) {\n");
                code.push_str("    printf(\"%d \", vm->stack.data[i]);\n");
                code.push_str("}\n");
                code.push_str("printf(\"\\n\");");
                code
            }
            IRInstruction::Label(label) => {
                format!("{}:", label)
            }
            IRInstruction::Comment(text) => {
                format!("/* {} */", text)
            }
            IRInstruction::LoadConst(value) => {
                format!("push(vm, {});", value)
            }
            _ => {
                format!("/* TODO: implement {:?} */", instr)
            }
        };

        Ok(code)
    }

    fn translate_function(&mut self, func: &IRFunction, ctx: &mut CodegenContext) -> CodegenResult {
        let mut code = String::new();
        
        if func.name == "main" {
            code.push_str("void forth_main(ForthVM* vm) {\n");
        } else {
            code.push_str(&format!("void {}(ForthVM* vm) {{\n", func.name));
        }

        for instr in &func.instructions {
            let instr_code = self.translate_instruction(instr, ctx)?;
            for line in instr_code.lines() {
                if !line.trim().is_empty() {
                    code.push_str(&format!("    {}\n", line));
                }
            }
        }

        code.push_str("}\n");
        Ok(code)
    }

    fn translate_program(&mut self, program: &IRProgram, ctx: &mut CodegenContext) -> CodegenResult {
        let mut code = String::new();

        // Translate all functions
        for (name, func) in &program.functions {
            let func_code = self.translate_function(func, ctx)?;
            code.push_str(&func_code);
            code.push('\n');
        }

        // Translate main function
        let main_code = self.translate_function(&program.main, ctx)?;
        code.push_str(&main_code);

        Ok(code)
    }
}