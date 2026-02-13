//! REPL-specific code generation.
//!
//! Generates Rust code that can be compiled to a shared library for the REPL.

use crate::ir::{
    BinaryOpKind, IRFunction, IRInstruction, IRLabel, IRProgram, IRValue, UnaryOpKind,
};
use crate::repl::state::CompilerContext;

/// REPL code generator.
pub struct ReplCodegen {
    /// Indentation level.
    indent: usize,

    /// Output buffer.
    output: String,

    /// String literals collected during code generation.
    string_literals: Vec<String>,
}

impl ReplCodegen {
    /// Create a new REPL code generator.
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: String::new(),
            string_literals: Vec::new(),
        }
    }

    /// Generate Rust code for a REPL input.
    ///
    /// Returns the generated code and a list of defined word names.
    pub fn generate(&mut self, ir: &IRProgram, _ctx: &CompilerContext) -> (String, Vec<String>) {
        self.output.clear();
        self.indent = 0;
        self.string_literals.clear();

        // Collect defined words
        let defined_words: Vec<String> = ir.functions.keys().cloned().collect();

        // Header
        self.emit_line("// Auto-generated REPL module");
        self.emit_line(
            "use roth_runtime::{RuntimeContext, ForthResult, ForthError, SourceLocation};",
        );
        self.emit_line("");

        // Generate string literal constants
        self.collect_string_literals(ir);
        if !self.string_literals.is_empty() {
            let literals: Vec<_> = self.string_literals.clone();
            for (i, s) in literals.iter().enumerate() {
                self.emit_line(&format!("const STRING_{}: &str = {:?};", i, s));
            }
            self.emit_line("");
        }

        // Generate user-defined words
        for (name, func) in &ir.functions {
            self.generate_word_function(name, func);
            self.emit_line("");
        }

        // Generate entry point that registers words and executes main code
        self.generate_entry_point_with_registration(&ir.main, &defined_words);
        self.emit_line("");

        // Generate defined words metadata (still useful for tracking)
        self.emit_line("// Metadata: words defined in this library");
        self.emit_line("#[unsafe(no_mangle)]");
        if defined_words.is_empty() {
            self.emit_line("pub static __defined_words: &[&str] = &[];");
        } else {
            let words_str = defined_words
                .iter()
                .map(|w| format!("{:?}", w))
                .collect::<Vec<_>>()
                .join(", ");
            self.emit_line(&format!(
                "pub static __defined_words: &[&str] = &[{}];",
                words_str
            ));
        }

        (self.output.clone(), defined_words)
    }

    /// Collect all string literals from the IR.
    fn collect_string_literals(&mut self, ir: &IRProgram) {
        self.collect_strings_from_function(&ir.main);
        for func in ir.functions.values() {
            self.collect_strings_from_function(func);
        }
    }

    fn collect_strings_from_function(&mut self, func: &IRFunction) {
        for instr in &func.instructions {
            if let IRInstruction::Push(IRValue::Variable(s)) = instr {
                // Check if this looks like a string literal marker
                if s.starts_with("__str_") {
                    // Extract the actual string
                    if let Some(idx) = s.strip_prefix("__str_") {
                        if let Ok(n) = idx.parse::<usize>() {
                            while self.string_literals.len() <= n {
                                self.string_literals.push(String::new());
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generate a user-defined word function.
    fn generate_word_function(&mut self, name: &str, func: &IRFunction) {
        let fn_name = format!("word_{}", name.to_lowercase().replace("-", "_"));

        // Word functions are internal to the library - they'll be registered via __repl_entry
        self.emit_line(&format!(
            "fn {}(ctx: &mut RuntimeContext) -> ForthResult<()> {{",
            fn_name
        ));
        self.indent += 1;

        // Set current word for error reporting
        self.emit_line(&format!("ctx.set_current_word({:?});", name));

        // Generate function body
        self.generate_function_body(&func.instructions);

        self.emit_line("Ok(())");
        self.indent -= 1;
        self.emit_line("}");
    }

    /// Generate the REPL entry point with word registration.
    fn generate_entry_point_with_registration(
        &mut self,
        main: &IRFunction,
        defined_words: &[String],
    ) {
        self.emit_line("#[unsafe(no_mangle)]");
        self.emit_line(
            "pub extern \"C\" fn __repl_entry(ctx: &mut RuntimeContext) -> ForthResult<()> {",
        );
        self.indent += 1;

        // Register all defined words first (except "main" which is special)
        for word in defined_words {
            if word != "main" {
                let fn_name = format!("word_{}", word.to_lowercase().replace("-", "_"));
                self.emit_line(&format!("ctx.register_word({:?}, {});", word, fn_name));
            }
        }

        // Then execute the main code
        self.generate_function_body(&main.instructions);

        self.emit_line("Ok(())");
        self.indent -= 1;
        self.emit_line("}");
    }

    /// Generate code for a function body.
    fn generate_function_body(&mut self, instructions: &[IRInstruction]) {
        // Generate loop control variables if needed
        let has_loops = instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::DoLoop(_, _)));
        if has_loops {
            self.emit_line("let mut loop_stack: Vec<(i64, i64)> = Vec::new();");
        }

        // Track if we need a state machine for control flow
        let has_jumps = instructions.iter().any(|i| {
            matches!(
                i,
                IRInstruction::Jump(_) | IRInstruction::JumpIf(_) | IRInstruction::JumpIfNot(_)
            )
        });

        if has_jumps {
            self.generate_state_machine(instructions);
        } else {
            // Simple linear code generation
            for instr in instructions {
                self.generate_instruction(instr);
            }
        }
    }

    /// Generate a state machine for control flow.
    fn generate_state_machine(&mut self, instructions: &[IRInstruction]) {
        // Collect all labels
        let mut labels: Vec<&IRLabel> = Vec::new();
        for instr in instructions {
            if let IRInstruction::Label(label) = instr {
                labels.push(label);
            }
        }

        // Generate state enum
        self.emit_line("let mut loop_stack: Vec<(i64, i64)> = Vec::new();");
        self.emit_line("let mut state = 0usize;");
        self.emit_line("loop {");
        self.indent += 1;
        self.emit_line("match state {");
        self.indent += 1;

        // State 0 is the entry point
        let mut current_state = 0usize;
        let mut label_to_state: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        // First pass: assign state numbers to labels
        for instr in instructions {
            if let IRInstruction::Label(label) = instr {
                current_state += 1;
                label_to_state.insert(label.to_string(), current_state);
            }
        }

        // Second pass: generate code
        current_state = 0;
        self.emit_line(&format!("{} => {{", current_state));
        self.indent += 1;

        for instr in instructions {
            match instr {
                IRInstruction::Label(label) => {
                    // Close previous state and start new one
                    let next_state = label_to_state
                        .get(&label.to_string())
                        .copied()
                        .unwrap_or(current_state + 1);
                    self.emit_line(&format!("state = {};", next_state));
                    self.indent -= 1;
                    self.emit_line("}");
                    current_state = next_state;
                    self.emit_line(&format!("{} => {{ // {}", current_state, label));
                    self.indent += 1;
                }
                IRInstruction::Jump(label) => {
                    let target_state = label_to_state.get(&label.to_string()).copied().unwrap_or(0);
                    self.emit_line(&format!("state = {};", target_state));
                    self.emit_line("continue;");
                }
                IRInstruction::JumpIf(label) => {
                    let target_state = label_to_state.get(&label.to_string()).copied().unwrap_or(0);
                    self.emit_line("let cond = ctx.pop()?;");
                    self.emit_line(&format!(
                        "if cond != 0 {{ state = {}; continue; }}",
                        target_state
                    ));
                }
                IRInstruction::JumpIfNot(label) => {
                    let target_state = label_to_state.get(&label.to_string()).copied().unwrap_or(0);
                    self.emit_line("let cond = ctx.pop()?;");
                    self.emit_line(&format!(
                        "if cond == 0 {{ state = {}; continue; }}",
                        target_state
                    ));
                }
                IRInstruction::Return => {
                    self.emit_line("return Ok(());");
                }
                _ => {
                    self.generate_instruction(instr);
                }
            }
        }

        // Close last state and break
        self.emit_line("break;");
        self.indent -= 1;
        self.emit_line("}");

        // Default case
        self.emit_line("_ => break,");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
    }

    /// Generate code for a single instruction.
    fn generate_instruction(&mut self, instr: &IRInstruction) {
        match instr {
            IRInstruction::Push(value) => {
                let val_code = self.generate_value(value);
                self.emit_line(&format!("ctx.push({})?;", val_code));
            }
            IRInstruction::Pop => {
                self.emit_line("ctx.pop()?;");
            }
            IRInstruction::Dup => {
                self.emit_line("ctx.dup()?;");
            }
            IRInstruction::Drop => {
                self.emit_line("ctx.drop_top()?;");
            }
            IRInstruction::Swap => {
                self.emit_line("ctx.swap()?;");
            }
            IRInstruction::Over => {
                self.emit_line("ctx.over()?;");
            }
            IRInstruction::Rot => {
                self.emit_line("ctx.rot()?;");
            }
            IRInstruction::Add => {
                self.emit_line("ctx.add()?;");
            }
            IRInstruction::Sub => {
                self.emit_line("ctx.sub()?;");
            }
            IRInstruction::Mul => {
                self.emit_line("ctx.mul()?;");
            }
            IRInstruction::Div => {
                self.emit_line("ctx.div()?;");
            }
            IRInstruction::Mod => {
                self.emit_line("ctx.modulo()?;");
            }
            IRInstruction::Neg => {
                self.emit_line("ctx.negate()?;");
            }
            IRInstruction::Equal => {
                self.emit_line("ctx.eq()?;");
            }
            IRInstruction::NotEqual => {
                self.emit_line("ctx.ne()?;");
            }
            IRInstruction::Less => {
                self.emit_line("ctx.lt()?;");
            }
            IRInstruction::Greater => {
                self.emit_line("ctx.gt()?;");
            }
            IRInstruction::LessEqual => {
                self.emit_line("ctx.le()?;");
            }
            IRInstruction::GreaterEqual => {
                self.emit_line("ctx.ge()?;");
            }
            IRInstruction::And => {
                self.emit_line("ctx.and()?;");
            }
            IRInstruction::Or => {
                self.emit_line("ctx.or()?;");
            }
            IRInstruction::Not => {
                self.emit_line("ctx.invert()?;");
            }
            IRInstruction::Load(addr) => {
                if let IRValue::Variable(name) = addr {
                    self.emit_line(&format!("ctx.fetch({:?})?;", name));
                } else {
                    self.emit_line("// TODO: Load from computed address");
                }
            }
            IRInstruction::Store(addr) => {
                if let IRValue::Variable(name) = addr {
                    self.emit_line(&format!("ctx.store({:?})?;", name));
                } else {
                    self.emit_line("// TODO: Store to computed address");
                }
            }
            IRInstruction::Call(name) => {
                // Check if this is a call to a word defined in the same library
                let fn_name = format!("__word_{}", name.to_lowercase().replace("-", "_"));
                // Try local function first, fallback to runtime lookup
                self.emit_line(&format!("// Call word: {}", name));
                self.emit_line(&format!("ctx.call_word({:?})?;", name));
            }
            IRInstruction::Return => {
                self.emit_line("return Ok(());");
            }
            IRInstruction::DoLoop(loop_label, end_label) => {
                // ?DO implementation: (limit start -- )
                self.emit_line("{ // DO/?DO");
                self.indent += 1;
                self.emit_line("let start = ctx.pop()?;");
                self.emit_line("let limit = ctx.pop()?;");
                self.emit_line("loop_stack.push((start, limit));");
                self.emit_line("if start >= limit {");
                self.indent += 1;
                self.emit_line("loop_stack.pop();");
                // Jump to end will be handled by state machine
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("}");
            }
            IRInstruction::Loop(loop_label) => {
                // LOOP: increment and check
                self.emit_line("{ // LOOP");
                self.indent += 1;
                self.emit_line("if let Some((idx, limit)) = loop_stack.last_mut() {");
                self.indent += 1;
                self.emit_line("*idx += 1;");
                self.emit_line("if *idx >= *limit {");
                self.indent += 1;
                self.emit_line("loop_stack.pop();");
                // Continue to after loop (handled by state machine)
                self.indent -= 1;
                self.emit_line("} else {");
                self.indent += 1;
                // Jump back to loop start (handled by state machine)
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("}");
            }
            IRInstruction::PushLoopIndex => {
                // I: push current loop index
                self.emit_line("if let Some((idx, _)) = loop_stack.last() {");
                self.indent += 1;
                self.emit_line("ctx.push(*idx)?;");
                self.indent -= 1;
                self.emit_line("} else {");
                self.indent += 1;
                self.emit_line("return Err(ForthError::RuntimeError {");
                self.indent += 1;
                self.emit_line("message: \"I used outside loop\".to_string(),");
                self.emit_line("location: SourceLocation::default(),");
                self.indent -= 1;
                self.emit_line("});");
                self.indent -= 1;
                self.emit_line("}");
            }
            IRInstruction::PushLoopLimit => {
                self.emit_line("if let Some((_, limit)) = loop_stack.last() {");
                self.indent += 1;
                self.emit_line("ctx.push(*limit)?;");
                self.indent -= 1;
                self.emit_line("} else {");
                self.indent += 1;
                self.emit_line("return Err(ForthError::RuntimeError {");
                self.indent += 1;
                self.emit_line("message: \"Loop limit used outside loop\".to_string(),");
                self.emit_line("location: SourceLocation::default(),");
                self.indent -= 1;
                self.emit_line("});");
                self.indent -= 1;
                self.emit_line("}");
            }
            IRInstruction::Print => {
                self.emit_line("ctx.print_top()?;");
            }
            IRInstruction::PrintStack => {
                self.emit_line("ctx.print_stack()?;");
            }
            IRInstruction::PrintChar => {
                self.emit_line("ctx.emit()?;");
            }
            IRInstruction::PrintString => {
                // String printing: (addr len -- )
                self.emit_line("{ // Print string");
                self.indent += 1;
                self.emit_line("let len = ctx.pop()?;");
                self.emit_line("let addr = ctx.pop()?;");
                self.emit_line("// String printing handled via literals");
                self.indent -= 1;
                self.emit_line("}");
            }
            IRInstruction::ReadChar => {
                self.emit_line("ctx.key()?;");
            }
            IRInstruction::Label(label) => {
                // Labels are handled by state machine
                self.emit_line(&format!("// Label: {}", label));
            }
            IRInstruction::Comment(text) => {
                self.emit_line(&format!("// {}", text));
            }
            IRInstruction::LoadConst(val) => {
                self.emit_line(&format!("ctx.push({})?;", val));
            }
            IRInstruction::BinaryOp(op, a, b) => {
                let a_code = self.generate_value(a);
                let b_code = self.generate_value(b);
                let op_code = match op {
                    BinaryOpKind::Add => format!("{}.wrapping_add({})", a_code, b_code),
                    BinaryOpKind::Sub => format!("{}.wrapping_sub({})", a_code, b_code),
                    BinaryOpKind::Mul => format!("{}.wrapping_mul({})", a_code, b_code),
                    BinaryOpKind::Div => format!("{} / {}", a_code, b_code),
                    BinaryOpKind::Mod => format!("{} % {}", a_code, b_code),
                    BinaryOpKind::Equal => {
                        format!("if {} == {} {{ -1 }} else {{ 0 }}", a_code, b_code)
                    }
                    BinaryOpKind::NotEqual => {
                        format!("if {} != {} {{ -1 }} else {{ 0 }}", a_code, b_code)
                    }
                    BinaryOpKind::Less => {
                        format!("if {} < {} {{ -1 }} else {{ 0 }}", a_code, b_code)
                    }
                    BinaryOpKind::Greater => {
                        format!("if {} > {} {{ -1 }} else {{ 0 }}", a_code, b_code)
                    }
                    BinaryOpKind::LessEqual => {
                        format!("if {} <= {} {{ -1 }} else {{ 0 }}", a_code, b_code)
                    }
                    BinaryOpKind::GreaterEqual => {
                        format!("if {} >= {} {{ -1 }} else {{ 0 }}", a_code, b_code)
                    }
                    BinaryOpKind::And => format!("{} & {}", a_code, b_code),
                    BinaryOpKind::Or => format!("{} | {}", a_code, b_code),
                };
                self.emit_line(&format!("ctx.push({})?;", op_code));
            }
            IRInstruction::UnaryOp(op, a) => {
                let a_code = self.generate_value(a);
                let op_code = match op {
                    UnaryOpKind::Neg => format!("-{}", a_code),
                    UnaryOpKind::Not => format!("!{}", a_code),
                };
                self.emit_line(&format!("ctx.push({})?;", op_code));
            }
            IRInstruction::StackGet(pos) => {
                self.emit_line(&format!("let val = ctx.peek_n({})?;", pos));
                self.emit_line("ctx.push(val)?;");
            }
            IRInstruction::StackSet(pos, val) => {
                // Not commonly used, implement if needed
                self.emit_line(&format!("// StackSet {} to {:?}", pos, val));
            }
            IRInstruction::StackAlloc(size) => {
                for _ in 0..*size {
                    self.emit_line("ctx.push(0)?;");
                }
            }
            IRInstruction::StackFree(size) => {
                for _ in 0..*size {
                    self.emit_line("ctx.pop()?;");
                }
            }
            IRInstruction::Nop => {
                // No operation
            }
            IRInstruction::Jump(_) | IRInstruction::JumpIf(_) | IRInstruction::JumpIfNot(_) => {
                // Handled by state machine
            }
        }
    }

    /// Generate code for a value.
    fn generate_value(&self, value: &IRValue) -> String {
        match value {
            IRValue::Constant(n) => format!("{}_i64", n),
            IRValue::StackTop => "ctx.peek()?".to_string(),
            IRValue::StackPos(pos) => format!("ctx.peek_n({})?", pos),
            IRValue::Variable(name) => format!("*ctx.memory.get({:?}).unwrap_or(&0)", name),
            IRValue::Temporary(id) => format!("tmp_{}", id),
        }
    }

    /// Emit a line of code with proper indentation.
    fn emit_line(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
    }
}

impl Default for ReplCodegen {
    fn default() -> Self {
        Self::new()
    }
}
