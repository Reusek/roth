use std::collections::HashMap;
use std::fmt;

/// Intermediate Representation for Forth operations
/// This IR is stack-based but more explicit about operations and data flow

#[derive(Debug, Clone, PartialEq)]
pub struct IRProgram {
    pub functions: HashMap<String, IRFunction>,
    pub main: IRFunction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IRFunction {
    pub name: String,
    pub instructions: Vec<IRInstruction>,
    pub stack_effect: StackEffect, // How many items consumed/produced
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackEffect {
    pub consumes: usize,
    pub produces: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRInstruction {
    // Stack operations
    Push(IRValue),
    Pop,
    Dup,
    Drop,
    Swap,
    Over,
    Rot,   // ( a b c -- b c a )
    
    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    
    // Comparison operations
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Logical operations
    And,
    Or,
    Not,
    
    // Memory operations
    Load(IRValue),   // Load from address
    Store(IRValue),  // Store to address
    
    // Control flow
    Jump(IRLabel),
    JumpIf(IRLabel),     // Jump if top of stack is true
    JumpIfNot(IRLabel),  // Jump if top of stack is false
    Call(String),        // Call function
    Return,
    
    // I/O operations
    Print,
    PrintStack,
    PrintChar,
    ReadChar,
    
    // Labels and metadata
    Label(IRLabel),
    Comment(String),
    
    // Advanced operations for optimization
    LoadConst(i32),      // Optimized constant loading
    BinaryOp(BinaryOpKind, IRValue, IRValue), // Optimized binary operations
    UnaryOp(UnaryOpKind, IRValue),            // Optimized unary operations
    
    // Stack manipulation with known depths
    StackGet(usize),     // Get item at stack position (0 = top)
    StackSet(usize, IRValue), // Set item at stack position
    StackAlloc(usize),   // Allocate stack space
    StackFree(usize),    // Free stack space
    
    // No-op for optimization passes
    Nop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRValue {
    Constant(i32),
    StackTop,           // Top of stack
    StackPos(usize),    // Position on stack (0 = top)
    Variable(String),   // Named variable
    Temporary(usize),   // Temporary value with ID
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOpKind {
    Add, Sub, Mul, Div, Mod,
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind {
    Neg, Not,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct IRLabel {
    pub name: String,
    pub id: usize,
}

impl IRLabel {
    pub fn new(name: &str, id: usize) -> Self {
        Self {
            name: name.to_string(),
            id,
        }
    }
}

impl fmt::Display for IRLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.name, self.id)
    }
}

impl fmt::Display for IRInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRInstruction::Push(val) => write!(f, "push {}", format_value(val)),
            IRInstruction::Pop => write!(f, "pop"),
            IRInstruction::Dup => write!(f, "dup"),
            IRInstruction::Drop => write!(f, "drop"),
            IRInstruction::Swap => write!(f, "swap"),
            IRInstruction::Over => write!(f, "over"),
            IRInstruction::Rot => write!(f, "rot"),
            IRInstruction::Add => write!(f, "add"),
            IRInstruction::Sub => write!(f, "sub"),
            IRInstruction::Mul => write!(f, "mul"),
            IRInstruction::Div => write!(f, "div"),
            IRInstruction::Mod => write!(f, "mod"),
            IRInstruction::Neg => write!(f, "neg"),
            IRInstruction::Equal => write!(f, "eq"),
            IRInstruction::NotEqual => write!(f, "ne"),
            IRInstruction::Less => write!(f, "lt"),
            IRInstruction::Greater => write!(f, "gt"),
            IRInstruction::LessEqual => write!(f, "le"),
            IRInstruction::GreaterEqual => write!(f, "ge"),
            IRInstruction::And => write!(f, "and"),
            IRInstruction::Or => write!(f, "or"),
            IRInstruction::Not => write!(f, "not"),
            IRInstruction::Load(addr) => write!(f, "load {}", format_value(addr)),
            IRInstruction::Store(addr) => write!(f, "store {}", format_value(addr)),
            IRInstruction::Jump(label) => write!(f, "jump {}", label),
            IRInstruction::JumpIf(label) => write!(f, "jump_if {}", label),
            IRInstruction::JumpIfNot(label) => write!(f, "jump_if_not {}", label),
            IRInstruction::Call(name) => write!(f, "call {}", name),
            IRInstruction::Return => write!(f, "return"),
            IRInstruction::Print => write!(f, "print"),
            IRInstruction::PrintStack => write!(f, "print_stack"),
            IRInstruction::PrintChar => write!(f, "print_char"),
            IRInstruction::ReadChar => write!(f, "read_char"),
            IRInstruction::Label(label) => write!(f, "{}:", label),
            IRInstruction::Comment(text) => write!(f, "; {}", text),
            IRInstruction::LoadConst(val) => write!(f, "load_const {}", val),
            IRInstruction::BinaryOp(op, a, b) => write!(f, "{:?} {}, {}", op, format_value(a), format_value(b)),
            IRInstruction::UnaryOp(op, a) => write!(f, "{:?} {}", op, format_value(a)),
            IRInstruction::StackGet(pos) => write!(f, "stack_get {}", pos),
            IRInstruction::StackSet(pos, val) => write!(f, "stack_set {}, {}", pos, format_value(val)),
            IRInstruction::StackAlloc(size) => write!(f, "stack_alloc {}", size),
            IRInstruction::StackFree(size) => write!(f, "stack_free {}", size),
            IRInstruction::Nop => write!(f, "nop"),
        }
    }
}

fn format_value(val: &IRValue) -> String {
    match val {
        IRValue::Constant(n) => n.to_string(),
        IRValue::StackTop => "ST".to_string(),
        IRValue::StackPos(pos) => format!("S{}", pos),
        IRValue::Variable(name) => format!("${}", name),
        IRValue::Temporary(id) => format!("T{}", id),
    }
}

impl fmt::Display for IRFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "function {} (consumes: {}, produces: {}):", 
                 self.name, self.stack_effect.consumes, self.stack_effect.produces)?;
        for (i, instr) in self.instructions.iter().enumerate() {
            writeln!(f, "  {:3}: {}", i, instr)?;
        }
        Ok(())
    }
}

impl fmt::Display for IRProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "IR Program:")?;
        writeln!(f, "{}", self.main)?;
        for (name, func) in &self.functions {
            if name != "main" {
                writeln!(f, "{}", func)?;
            }
        }
        Ok(())
    }
}

/// Builder for constructing IR programs
pub struct IRBuilder {
    current_function: IRFunction,
    functions: HashMap<String, IRFunction>,
    label_counter: usize,
    temp_counter: usize,
}

impl IRBuilder {
    pub fn new(function_name: &str) -> Self {
        Self {
            current_function: IRFunction {
                name: function_name.to_string(),
                instructions: Vec::new(),
                stack_effect: StackEffect { consumes: 0, produces: 0 },
            },
            functions: HashMap::new(),
            label_counter: 0,
            temp_counter: 0,
        }
    }

    pub fn emit(&mut self, instruction: IRInstruction) {
        self.current_function.instructions.push(instruction);
    }

    pub fn emit_comment(&mut self, text: &str) {
        self.emit(IRInstruction::Comment(text.to_string()));
    }

    pub fn create_label(&mut self, name: &str) -> IRLabel {
        let label = IRLabel::new(name, self.label_counter);
        self.label_counter += 1;
        label
    }

    pub fn emit_label(&mut self, label: IRLabel) {
        self.emit(IRInstruction::Label(label));
    }

    pub fn create_temp(&mut self) -> IRValue {
        let temp = IRValue::Temporary(self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    pub fn finish_function(&mut self) -> IRFunction {
        let mut func = IRFunction {
            name: "temp".to_string(),
            instructions: Vec::new(),
            stack_effect: StackEffect { consumes: 0, produces: 0 },
        };
        std::mem::swap(&mut func, &mut self.current_function);
        func
    }

    pub fn start_function(&mut self, name: &str) {
        let finished = self.finish_function();
        if !finished.instructions.is_empty() {
            self.functions.insert(finished.name.clone(), finished);
        }
        
        self.current_function = IRFunction {
            name: name.to_string(),
            instructions: Vec::new(),
            stack_effect: StackEffect { consumes: 0, produces: 0 },
        };
    }

    pub fn build(mut self) -> IRProgram {
        let main = self.finish_function();
        IRProgram {
            main,
            functions: self.functions,
        }
    }
}

/// Stack effect analysis for IR instructions
impl IRInstruction {
    pub fn stack_effect(&self) -> StackEffect {
        match self {
            IRInstruction::Push(_) | IRInstruction::LoadConst(_) => StackEffect { consumes: 0, produces: 1 },
            IRInstruction::Pop | IRInstruction::Drop => StackEffect { consumes: 1, produces: 0 },
            IRInstruction::Dup => StackEffect { consumes: 1, produces: 2 },
            IRInstruction::Swap | IRInstruction::Over => StackEffect { consumes: 2, produces: 2 },
            IRInstruction::Rot => StackEffect { consumes: 3, produces: 3 },
            IRInstruction::Add | IRInstruction::Sub | IRInstruction::Mul | IRInstruction::Div | IRInstruction::Mod => 
                StackEffect { consumes: 2, produces: 1 },
            IRInstruction::Neg | IRInstruction::Not => StackEffect { consumes: 1, produces: 1 },
            IRInstruction::Equal | IRInstruction::NotEqual | IRInstruction::Less | IRInstruction::Greater |
            IRInstruction::LessEqual | IRInstruction::GreaterEqual | IRInstruction::And | IRInstruction::Or => 
                StackEffect { consumes: 2, produces: 1 },
            IRInstruction::Print | IRInstruction::PrintChar => StackEffect { consumes: 1, produces: 0 },
            IRInstruction::PrintStack => StackEffect { consumes: 0, produces: 0 },
            IRInstruction::ReadChar => StackEffect { consumes: 0, produces: 1 },
            IRInstruction::Load(_) => StackEffect { consumes: 0, produces: 1 },
            IRInstruction::Store(_) => StackEffect { consumes: 1, produces: 0 },
            IRInstruction::JumpIf(_) | IRInstruction::JumpIfNot(_) => StackEffect { consumes: 1, produces: 0 },
            IRInstruction::BinaryOp(_, _, _) => StackEffect { consumes: 0, produces: 1 }, // Optimized, no stack effect
            IRInstruction::UnaryOp(_, _) => StackEffect { consumes: 0, produces: 1 },     // Optimized, no stack effect
            IRInstruction::StackGet(_) => StackEffect { consumes: 0, produces: 1 },
            IRInstruction::StackSet(_, _) => StackEffect { consumes: 0, produces: 0 },
            _ => StackEffect { consumes: 0, produces: 0 }, // No stack effect
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_builder() {
        let mut builder = IRBuilder::new("main");
        
        builder.emit_comment("Test program: 5 10 +");
        builder.emit(IRInstruction::Push(IRValue::Constant(5)));
        builder.emit(IRInstruction::Push(IRValue::Constant(10)));
        builder.emit(IRInstruction::Add);
        builder.emit(IRInstruction::Print);
        
        let program = builder.build();
        
        assert_eq!(program.main.instructions.len(), 5);
        assert_eq!(program.main.instructions[1], IRInstruction::Push(IRValue::Constant(5)));
    }

    #[test]
    fn test_stack_effects() {
        assert_eq!(IRInstruction::Push(IRValue::Constant(42)).stack_effect(), 
                   StackEffect { consumes: 0, produces: 1 });
        assert_eq!(IRInstruction::Add.stack_effect(), 
                   StackEffect { consumes: 2, produces: 1 });
        assert_eq!(IRInstruction::Dup.stack_effect(), 
                   StackEffect { consumes: 1, produces: 2 });
    }

    #[test]
    fn test_ir_display() {
        let mut builder = IRBuilder::new("test");
        builder.emit(IRInstruction::Push(IRValue::Constant(42)));
        builder.emit(IRInstruction::Dup);
        builder.emit(IRInstruction::Add);
        
        let program = builder.build();
        let output = format!("{}", program);
        
        assert!(output.contains("push 42"));
        assert!(output.contains("dup"));
        assert!(output.contains("add"));
    }
}