use crate::ir::{IRBuilder, IRFunction, IRInstruction, IRLabel, IRProgram, IRValue, StackEffect};
use crate::types::AstNode;
use std::collections::HashMap;

/// Lowers AST to IR
pub struct IRLowering {
    builder: IRBuilder,
    word_definitions: HashMap<String, Vec<AstNode>>,
    loop_stack: Vec<(IRLabel, IRLabel)>, // (loop_start_label, loop_end_label)
    conditional_stack: Vec<(Option<IRLabel>, IRLabel)>, // (else_label, endif_label)
}

impl IRLowering {
    pub fn new() -> Self {
        Self {
            builder: IRBuilder::new("main"),
            word_definitions: HashMap::new(),
            loop_stack: Vec::new(),
            conditional_stack: Vec::new(),
        }
    }

    pub fn lower(&mut self, ast: &AstNode) -> IRProgram {
        self.lower_node(ast);
        let builder = std::mem::replace(&mut self.builder, IRBuilder::new("temp"));
        builder.build()
    }

    fn lower_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Program(nodes) => {
                self.builder.emit_comment("Generated from Forth AST");

                // First pass: collect definitions
                for node in nodes {
                    if let AstNode::Definition { name, body, .. } = node {
                        self.word_definitions.insert(name.clone(), body.clone());
                    }
                }

                // Second pass: generate IR for definitions
                for (name, body) in &self.word_definitions.clone() {
                    self.lower_definition(name, body);
                }

                // Third pass: generate main program (non-definitions)
                for node in nodes {
                    if !matches!(node, AstNode::Definition { .. }) {
                        self.lower_node(node);
                    }
                }
            }
            AstNode::Number(n, _) => {
                self.builder.emit_comment(&format!("Push constant {}", n));
                self.builder
                    .emit(IRInstruction::Push(IRValue::Constant(*n)));
            }
            AstNode::Word(name, _) => {
                self.lower_word(name);
            }
            AstNode::StringLiteral(s, _) => {
                self.builder.emit_comment(&format!("String literal: \"{}\"", s));
                // Push each character of the string onto the stack
                for ch in s.chars() {
                    self.builder.emit(IRInstruction::Push(IRValue::Constant(ch as i32)));
                }
                // Push string length
                self.builder.emit(IRInstruction::Push(IRValue::Constant(s.len() as i32)));
            }
            AstNode::Definition { .. } => {
                // Definitions are handled in the Program case
            }
        }
    }

    fn lower_definition(&mut self, name: &str, body: &[AstNode]) {
        // Start a new function for this definition
        self.builder.start_function(name);
        self.builder.emit_comment(&format!("Definition: {}", name));

        for node in body {
            self.lower_node(node);
        }

        self.builder.emit(IRInstruction::Return);

        // Switch back to main function
        self.builder.start_function("main");
    }

    fn lower_word(&mut self, name: &str) {
        match name {
            // Arithmetic operations
            "+" => {
                self.builder.emit_comment("Addition");
                self.builder.emit(IRInstruction::Add);
            }
            "-" => {
                self.builder.emit_comment("Subtraction");
                self.builder.emit(IRInstruction::Sub);
            }
            "*" => {
                self.builder.emit_comment("Multiplication");
                self.builder.emit(IRInstruction::Mul);
            }
            "/" => {
                self.builder.emit_comment("Division");
                self.builder.emit(IRInstruction::Div);
            }
            "MOD" => {
                self.builder.emit_comment("Modulo");
                self.builder.emit(IRInstruction::Mod);
            }
            "NEGATE" => {
                self.builder.emit_comment("Negate");
                self.builder.emit(IRInstruction::Neg);
            }

            // Stack operations
            "DUP" => {
                self.builder.emit_comment("Duplicate top of stack");
                self.builder.emit(IRInstruction::Dup);
            }
            "DROP" => {
                self.builder.emit_comment("Drop top of stack");
                self.builder.emit(IRInstruction::Drop);
            }
            "SWAP" => {
                self.builder.emit_comment("Swap top two stack items");
                self.builder.emit(IRInstruction::Swap);
            }
            "OVER" => {
                self.builder.emit_comment("Copy second stack item to top");
                self.builder.emit(IRInstruction::Over);
            }
            "ROT" => {
                self.builder.emit_comment("Rotate top three stack items");
                self.builder.emit(IRInstruction::Rot);
            }

            // Comparison operations
            "=" => {
                self.builder.emit_comment("Equal comparison");
                self.builder.emit(IRInstruction::Equal);
            }
            "<>" => {
                self.builder.emit_comment("Not equal comparison");
                self.builder.emit(IRInstruction::NotEqual);
            }
            "<" => {
                self.builder.emit_comment("Less than comparison");
                self.builder.emit(IRInstruction::Less);
            }
            ">" => {
                self.builder.emit_comment("Greater than comparison");
                self.builder.emit(IRInstruction::Greater);
            }
            "<=" => {
                self.builder.emit_comment("Less than or equal comparison");
                self.builder.emit(IRInstruction::LessEqual);
            }
            ">=" => {
                self.builder
                    .emit_comment("Greater than or equal comparison");
                self.builder.emit(IRInstruction::GreaterEqual);
            }

            // Logical operations
            "AND" => {
                self.builder.emit_comment("Logical AND");
                self.builder.emit(IRInstruction::And);
            }
            "OR" => {
                self.builder.emit_comment("Logical OR");
                self.builder.emit(IRInstruction::Or);
            }
            "NOT" => {
                self.builder.emit_comment("Logical NOT");
                self.builder.emit(IRInstruction::Not);
            }

            // I/O operations
            "." => {
                self.builder.emit_comment("Print top of stack");
                self.builder.emit(IRInstruction::Print);
            }
            ".S" => {
                self.builder.emit_comment("Print entire stack");
                self.builder.emit(IRInstruction::PrintStack);
            }
            "EMIT" => {
                self.builder.emit_comment("Print character");
                self.builder.emit(IRInstruction::PrintChar);
            }
            "KEY" => {
                self.builder.emit_comment("Read character");
                self.builder.emit(IRInstruction::ReadChar);
            }
            "CR" => {
                self.builder.emit_comment("Print newline");
                self.builder
                    .emit(IRInstruction::Push(IRValue::Constant(10))); // ASCII newline
                self.builder.emit(IRInstruction::PrintChar);
            }

            // Control flow operations
            "?DO" => {
                self.builder.emit_comment("Conditional DO loop");
                let loop_start = self.builder.create_label("loop_start");
                let loop_end = self.builder.create_label("loop_end");

                // DoLoop instruction: (limit start -- )
                // If start >= limit, jump to end, otherwise continue
                self.builder
                    .emit(IRInstruction::DoLoop(loop_start.clone(), loop_end.clone()));
                self.builder.emit_label(loop_start.clone());

                // Push this loop onto the stack for LOOP to reference
                self.loop_stack.push((loop_start, loop_end));
            }
            "DO" => {
                self.builder.emit_comment("DO loop");
                let loop_start = self.builder.create_label("loop_start");
                let loop_end = self.builder.create_label("loop_end");

                // Unconditional DO: always enter the loop
                // Still consume limit and start from stack
                self.builder
                    .emit(IRInstruction::DoLoop(loop_start.clone(), loop_end.clone()));
                self.builder.emit_label(loop_start.clone());

                self.loop_stack.push((loop_start, loop_end));
            }
            "LOOP" => {
                self.builder.emit_comment("LOOP");
                if let Some((loop_start, loop_end)) = self.loop_stack.pop() {
                    // Increment loop index and jump back if index < limit
                    self.builder.emit(IRInstruction::Loop(loop_start));
                    self.builder.emit_label(loop_end);
                } else {
                    // Error: LOOP without matching DO
                    self.builder.emit_comment("ERROR: LOOP without matching DO");
                }
            }
            "I" => {
                self.builder.emit_comment("Loop index I");
                self.builder.emit(IRInstruction::PushLoopIndex);
            }
            "J" => {
                self.builder.emit_comment("Loop index J (outer loop)");
                // For now, just push the current loop index
                // In a full implementation, this would be the outer loop index
                self.builder.emit(IRInstruction::PushLoopIndex);
            }

            // Conditional control flow
            "IF" => {
                self.builder.emit_comment("IF conditional");
                let else_label = self.builder.create_label("else");
                let endif_label = self.builder.create_label("endif");
                
                // Jump to else/endif if top of stack is false (0)
                self.builder.emit(IRInstruction::JumpIfNot(else_label.clone()));
                
                // Push conditional info onto stack
                self.conditional_stack.push((Some(else_label), endif_label));
            }
            "ELSE" => {
                self.builder.emit_comment("ELSE");
                if let Some((Some(else_label), endif_label)) = self.conditional_stack.pop() {
                    // Jump to endif (skip else part)
                    self.builder.emit(IRInstruction::Jump(endif_label.clone()));
                    // Place else label here
                    self.builder.emit_label(else_label);
                    // Update stack with no else label (already used)
                    self.conditional_stack.push((None, endif_label));
                } else {
                    self.builder.emit_comment("ERROR: ELSE without matching IF");
                }
            }
            "THEN" => {
                self.builder.emit_comment("THEN (endif)");
                if let Some((else_label, endif_label)) = self.conditional_stack.pop() {
                    // If there was an unused else label, place it here
                    if let Some(else_lbl) = else_label {
                        self.builder.emit_label(else_lbl);
                    }
                    // Place endif label
                    self.builder.emit_label(endif_label);
                } else {
                    self.builder.emit_comment("ERROR: THEN without matching IF");
                }
            }

            // Additional useful words
            "TYPE" => {
                self.builder.emit_comment("TYPE - print string");
                // Expects: addr count -- 
                // For now, just print characters from stack
                self.builder.emit(IRInstruction::PrintString);
            }
            "SPACE" => {
                self.builder.emit_comment("SPACE - print a space");
                self.builder.emit(IRInstruction::Push(IRValue::Constant(32))); // ASCII space
                self.builder.emit(IRInstruction::PrintChar);
            }
            "BL" => {
                self.builder.emit_comment("BL - push space character");
                self.builder.emit(IRInstruction::Push(IRValue::Constant(32))); // ASCII space
            }

            // User-defined words
            _ => {
                if self.word_definitions.contains_key(name) {
                    self.builder
                        .emit_comment(&format!("Call user-defined word: {}", name));
                    self.builder.emit(IRInstruction::Call(name.to_string()));
                } else {
                    self.builder
                        .emit_comment(&format!("Unknown word: {}", name));
                    // For now, treat as no-op, but in a real implementation
                    // this should be an error
                    self.builder.emit(IRInstruction::Nop);
                }
            }
        }
    }
}

/// Analyzes IR to compute stack effects for functions
pub struct StackEffectAnalyzer;

impl StackEffectAnalyzer {
    pub fn analyze_program(program: &mut IRProgram) {
        Self::analyze_function(&mut program.main);

        for (_, function) in program.functions.iter_mut() {
            Self::analyze_function(function);
        }
    }

    fn analyze_function(function: &mut IRFunction) {
        let mut stack_depth = 0i32;
        let mut max_depth = 0i32;
        let mut min_depth = 0i32;

        for instruction in &function.instructions {
            let effect = instruction.stack_effect();
            stack_depth = stack_depth - effect.consumes as i32 + effect.produces as i32;

            max_depth = max_depth.max(stack_depth);
            min_depth = min_depth.min(stack_depth);
        }

        // The function's stack effect is the net change
        if stack_depth >= 0 {
            function.stack_effect = StackEffect {
                consumes: (-min_depth) as usize,
                produces: (stack_depth + (-min_depth)) as usize,
            };
        } else {
            function.stack_effect = StackEffect {
                consumes: (-min_depth) as usize,
                produces: 0,
            };
        }
    }
}

/// Pretty printer for IR with stack depth annotations
pub struct IRPrettyPrinter;

impl IRPrettyPrinter {
    pub fn print_with_stack_analysis(program: &IRProgram) -> String {
        let mut output = String::new();

        output.push_str("=== IR Program with Stack Analysis ===\n\n");
        output.push_str(&Self::print_function_with_analysis(&program.main));

        for (name, function) in &program.functions {
            if name != "main" {
                output.push('\n');
                output.push_str(&Self::print_function_with_analysis(function));
            }
        }

        output
    }

    fn print_function_with_analysis(function: &IRFunction) -> String {
        let mut output = String::new();
        let mut stack_depth = 0i32;

        output.push_str(&format!(
            "Function: {} (consumes: {}, produces: {})\n",
            function.name, function.stack_effect.consumes, function.stack_effect.produces
        ));
        output.push_str(&format!("{:>3} | {:>5} | Instruction\n", "PC", "Stack"));
        output.push_str("----+-------+------------\n");

        for (pc, instruction) in function.instructions.iter().enumerate() {
            let effect = instruction.stack_effect();

            output.push_str(&format!("{:3} | {:5} | {}\n", pc, stack_depth, instruction));

            stack_depth = stack_depth - effect.consumes as i32 + effect.produces as i32;
        }

        output.push_str(&format!("    | {:5} | (final stack depth)\n", stack_depth));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AstNode, Position};

    #[test]
    fn test_simple_lowering() {
        let mut lowering = IRLowering::new();
        let pos = Position {
            line: 1,
            column: 1,
            offset: 0,
        };

        let ast = AstNode::Program(vec![
            AstNode::Number(5, pos.clone()),
            AstNode::Number(10, pos.clone()),
            AstNode::Word("+".to_string(), pos.clone()),
            AstNode::Word(".".to_string(), pos.clone()),
        ]);

        let mut program = lowering.lower(&ast);
        StackEffectAnalyzer::analyze_program(&mut program);

        assert_eq!(program.main.instructions.len(), 9); // Including comments

        // Check that we have the right instructions (ignoring comments)
        let non_comment_instructions: Vec<_> = program
            .main
            .instructions
            .iter()
            .filter(|instr| !matches!(instr, IRInstruction::Comment(_)))
            .collect();

        assert_eq!(non_comment_instructions.len(), 4);
        assert!(matches!(
            non_comment_instructions[0],
            IRInstruction::Push(IRValue::Constant(5))
        ));
        assert!(matches!(
            non_comment_instructions[1],
            IRInstruction::Push(IRValue::Constant(10))
        ));
        assert!(matches!(non_comment_instructions[2], IRInstruction::Add));
        assert!(matches!(non_comment_instructions[3], IRInstruction::Print));
    }

    #[test]
    fn test_definition_lowering() {
        let mut lowering = IRLowering::new();
        let pos = Position {
            line: 1,
            column: 1,
            offset: 0,
        };

        let ast = AstNode::Program(vec![
            AstNode::Definition {
                name: "DOUBLE".to_string(),
                body: vec![
                    AstNode::Word("DUP".to_string(), pos.clone()),
                    AstNode::Word("+".to_string(), pos.clone()),
                ],
                position: pos.clone(),
            },
            AstNode::Number(5, pos.clone()),
            AstNode::Word("DOUBLE".to_string(), pos.clone()),
            AstNode::Word(".".to_string(), pos.clone()),
        ]);

        let mut program = lowering.lower(&ast);
        StackEffectAnalyzer::analyze_program(&mut program);

        // Should have a DOUBLE function
        assert!(program.functions.contains_key("DOUBLE"));

        // Main should call DOUBLE
        let main_instructions: Vec<_> = program
            .main
            .instructions
            .iter()
            .filter(|instr| !matches!(instr, IRInstruction::Comment(_)))
            .collect();

        assert!(
            main_instructions
                .iter()
                .any(|instr| matches!(instr, IRInstruction::Call(name) if name == "DOUBLE"))
        );
    }

    #[test]
    fn test_stack_effect_analysis() {
        let mut lowering = IRLowering::new();
        let pos = Position {
            line: 1,
            column: 1,
            offset: 0,
        };

        // Test: 5 DUP + (should consume 0, produce 1)
        let ast = AstNode::Program(vec![
            AstNode::Number(5, pos.clone()),
            AstNode::Word("DUP".to_string(), pos.clone()),
            AstNode::Word("+".to_string(), pos.clone()),
        ]);

        let mut program = lowering.lower(&ast);
        StackEffectAnalyzer::analyze_program(&mut program);

        // The net effect should be: push 1 value, dup (1->2), add (2->1) = net +1
        assert_eq!(program.main.stack_effect.consumes, 0);
        assert_eq!(program.main.stack_effect.produces, 1);
    }
}
