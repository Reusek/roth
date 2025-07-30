use crate::ir::{IRProgram, IRFunction, IRInstruction, IRValue, BinaryOpKind, UnaryOpKind};
use std::collections::HashMap;

/// Trait for IR optimization passes
pub trait IROptimizationPass {
    fn name(&self) -> &str;
    fn optimize_program(&mut self, program: &mut IRProgram) -> bool;
    fn optimize_function(&mut self, function: &mut IRFunction) -> bool;
}

/// Constant folding optimization pass
pub struct ConstantFoldingPass {
    optimizations_applied: usize,
}

impl ConstantFoldingPass {
    pub fn new() -> Self {
        Self { optimizations_applied: 0 }
    }

    fn try_fold_binary_op(&self, op: &IRInstruction, a: i32, b: i32) -> Option<IRInstruction> {
        match op {
            IRInstruction::Add => Some(IRInstruction::LoadConst(a + b)),
            IRInstruction::Sub => Some(IRInstruction::LoadConst(a - b)),
            IRInstruction::Mul => Some(IRInstruction::LoadConst(a * b)),
            IRInstruction::Div if b != 0 => Some(IRInstruction::LoadConst(a / b)),
            IRInstruction::Mod if b != 0 => Some(IRInstruction::LoadConst(a % b)),
            IRInstruction::Equal => Some(IRInstruction::LoadConst(if a == b { -1 } else { 0 })),
            IRInstruction::NotEqual => Some(IRInstruction::LoadConst(if a != b { -1 } else { 0 })),
            IRInstruction::Less => Some(IRInstruction::LoadConst(if a < b { -1 } else { 0 })),
            IRInstruction::Greater => Some(IRInstruction::LoadConst(if a > b { -1 } else { 0 })),
            IRInstruction::LessEqual => Some(IRInstruction::LoadConst(if a <= b { -1 } else { 0 })),
            IRInstruction::GreaterEqual => Some(IRInstruction::LoadConst(if a >= b { -1 } else { 0 })),
            IRInstruction::And => Some(IRInstruction::LoadConst(if a != 0 && b != 0 { -1 } else { 0 })),
            IRInstruction::Or => Some(IRInstruction::LoadConst(if a != 0 || b != 0 { -1 } else { 0 })),
            _ => None,
        }
    }

    fn try_fold_unary_op(&self, op: &IRInstruction, a: i32) -> Option<IRInstruction> {
        match op {
            IRInstruction::Neg => Some(IRInstruction::LoadConst(-a)),
            IRInstruction::Not => Some(IRInstruction::LoadConst(if a == 0 { -1 } else { 0 })),
            _ => None,
        }
    }
}

impl IROptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &str {
        "Constant Folding"
    }

    fn optimize_program(&mut self, program: &mut IRProgram) -> bool {
        let mut changed = false;
        changed |= self.optimize_function(&mut program.main);
        
        for (_, function) in program.functions.iter_mut() {
            changed |= self.optimize_function(function);
        }
        
        changed
    }

    fn optimize_function(&mut self, function: &mut IRFunction) -> bool {
        let mut changed = false;
        let mut i = 0;
        
        while i < function.instructions.len() {
            // Skip comments when looking for patterns
            if matches!(function.instructions[i], IRInstruction::Comment(_)) {
                i += 1;
                continue;
            }

            // Look for patterns: Push(a) Push(b) BinaryOp -> LoadConst(result)
            // We need to find the next two non-comment instructions
            let mut next_indices = Vec::new();
            let mut search_idx = i + 1;
            
            // Find next two non-comment instructions
            while search_idx < function.instructions.len() && next_indices.len() < 2 {
                if !matches!(function.instructions[search_idx], IRInstruction::Comment(_)) {
                    next_indices.push(search_idx);
                }
                search_idx += 1;
            }
            
            if next_indices.len() >= 2 {
                let idx1 = next_indices[0];
                let idx2 = next_indices[1];
                
                if let (
                    IRInstruction::Push(IRValue::Constant(a)) | IRInstruction::LoadConst(a),
                    IRInstruction::Push(IRValue::Constant(b)) | IRInstruction::LoadConst(b),
                    binary_op
                ) = (&function.instructions[i], &function.instructions[idx1], &function.instructions[idx2]) {
                    if let Some(folded) = self.try_fold_binary_op(binary_op, *a, *b) {
                        // Replace the three non-comment instructions with one
                        function.instructions[i] = folded;
                        
                        // Remove the other two instructions (in reverse order to maintain indices)
                        if idx2 > idx1 {
                            function.instructions.remove(idx2);
                            function.instructions.remove(idx1);
                        } else {
                            function.instructions.remove(idx1);
                            function.instructions.remove(idx2);
                        }
                        
                        changed = true;
                        self.optimizations_applied += 1;
                        continue; // Don't increment i, check this position again
                    }
                }
            }

            // Look for patterns: Push(a) UnaryOp -> LoadConst(result)
            if next_indices.len() >= 1 {
                let idx1 = next_indices[0];
                
                if let (
                    IRInstruction::Push(IRValue::Constant(a)) | IRInstruction::LoadConst(a),
                    unary_op
                ) = (&function.instructions[i], &function.instructions[idx1]) {
                    if let Some(folded) = self.try_fold_unary_op(unary_op, *a) {
                        // Replace two instructions with one
                        function.instructions[i] = folded;
                        function.instructions.remove(idx1);
                        changed = true;
                        self.optimizations_applied += 1;
                        continue; // Don't increment i, check this position again
                    }
                }
            }

            i += 1;
        }
        
        changed
    }
}

/// Dead code elimination pass
pub struct DeadCodeEliminationPass {
    optimizations_applied: usize,
}

impl DeadCodeEliminationPass {
    pub fn new() -> Self {
        Self { optimizations_applied: 0 }
    }
}

impl IROptimizationPass for DeadCodeEliminationPass {
    fn name(&self) -> &str {
        "Dead Code Elimination"
    }

    fn optimize_program(&mut self, program: &mut IRProgram) -> bool {
        let mut changed = false;
        changed |= self.optimize_function(&mut program.main);
        
        for (_, function) in program.functions.iter_mut() {
            changed |= self.optimize_function(function);
        }
        
        changed
    }

    fn optimize_function(&mut self, function: &mut IRFunction) -> bool {
        let mut changed = false;
        let mut i = 0;
        
        while i < function.instructions.len() {
            let should_remove = match &function.instructions[i] {
                // Remove no-ops
                IRInstruction::Nop => true,
                
                // Remove push followed immediately by drop
                IRInstruction::Push(_) | IRInstruction::LoadConst(_) => {
                    if i + 1 < function.instructions.len() {
                        matches!(function.instructions[i + 1], IRInstruction::Drop)
                    } else {
                        false
                    }
                }
                
                // Remove dup followed immediately by drop (becomes no-op)
                IRInstruction::Dup => {
                    if i + 1 < function.instructions.len() {
                        matches!(function.instructions[i + 1], IRInstruction::Drop)
                    } else {
                        false
                    }
                }
                
                _ => false,
            };

            if should_remove {
                if matches!(function.instructions[i], IRInstruction::Push(_) | IRInstruction::LoadConst(_) | IRInstruction::Dup) {
                    // Remove both the push/dup and the following drop
                    function.instructions.remove(i);
                    if i < function.instructions.len() {
                        function.instructions.remove(i);
                    }
                    self.optimizations_applied += 2;
                } else {
                    // Remove just the no-op
                    function.instructions.remove(i);
                    self.optimizations_applied += 1;
                }
                changed = true;
                // Don't increment i since we removed instructions
            } else {
                i += 1;
            }
        }
        
        changed
    }
}

/// Peephole optimization pass for stack operations
pub struct PeepholeOptimizationPass {
    optimizations_applied: usize,
}

impl PeepholeOptimizationPass {
    pub fn new() -> Self {
        Self { optimizations_applied: 0 }
    }
}

impl IROptimizationPass for PeepholeOptimizationPass {
    fn name(&self) -> &str {
        "Peephole Optimization"
    }

    fn optimize_program(&mut self, program: &mut IRProgram) -> bool {
        let mut changed = false;
        changed |= self.optimize_function(&mut program.main);
        
        for (_, function) in program.functions.iter_mut() {
            changed |= self.optimize_function(function);
        }
        
        changed
    }

    fn optimize_function(&mut self, function: &mut IRFunction) -> bool {
        let mut changed = false;
        let mut i = 0;
        
        while i < function.instructions.len() {
            let mut pattern_matched = false;

            // Pattern: Push(a) Dup Add -> LoadConst(a * 2)
            if i + 2 < function.instructions.len() {
                if let (
                    IRInstruction::Push(IRValue::Constant(a)) | IRInstruction::LoadConst(a),
                    IRInstruction::Dup,
                    IRInstruction::Add
                ) = (&function.instructions[i], &function.instructions[i + 1], &function.instructions[i + 2]) {
                    function.instructions[i] = IRInstruction::LoadConst(a * 2);
                    function.instructions.remove(i + 1);
                    function.instructions.remove(i + 1);
                    changed = true;
                    pattern_matched = true;
                    self.optimizations_applied += 1;
                }
            }

            // Pattern: Push(a) Push(b) Swap -> Push(b) Push(a)
            if !pattern_matched && i + 2 < function.instructions.len() {
                let should_optimize = matches!(
                    (&function.instructions[i], &function.instructions[i + 1], &function.instructions[i + 2]),
                    (
                        IRInstruction::Push(IRValue::Constant(_)) | IRInstruction::LoadConst(_),
                        IRInstruction::Push(IRValue::Constant(_)) | IRInstruction::LoadConst(_),
                        IRInstruction::Swap
                    )
                );
                
                if should_optimize {
                    // Extract values safely
                    let a = match &function.instructions[i] {
                        IRInstruction::Push(IRValue::Constant(n)) | IRInstruction::LoadConst(n) => *n,
                        _ => unreachable!(),
                    };
                    let b = match &function.instructions[i + 1] {
                        IRInstruction::Push(IRValue::Constant(n)) | IRInstruction::LoadConst(n) => *n,
                        _ => unreachable!(),
                    };
                    
                    function.instructions[i] = IRInstruction::LoadConst(b);
                    function.instructions[i + 1] = IRInstruction::LoadConst(a);
                    function.instructions.remove(i + 2);
                    changed = true;
                    pattern_matched = true;
                    self.optimizations_applied += 1;
                }
            }

            // Pattern: Dup Drop -> Nop (will be removed by dead code elimination)
            if !pattern_matched && i + 1 < function.instructions.len() {
                if let (IRInstruction::Dup, IRInstruction::Drop) = 
                    (&function.instructions[i], &function.instructions[i + 1]) {
                    function.instructions[i] = IRInstruction::Nop;
                    function.instructions[i + 1] = IRInstruction::Nop;
                    changed = true;
                    pattern_matched = true;
                    self.optimizations_applied += 1;
                }
            }

            // Pattern: Swap Swap -> Nop
            if !pattern_matched && i + 1 < function.instructions.len() {
                if let (IRInstruction::Swap, IRInstruction::Swap) = 
                    (&function.instructions[i], &function.instructions[i + 1]) {
                    function.instructions[i] = IRInstruction::Nop;
                    function.instructions[i + 1] = IRInstruction::Nop;
                    changed = true;
                    pattern_matched = true;
                    self.optimizations_applied += 1;
                }
            }

            if !pattern_matched {
                i += 1;
            }
            // If pattern matched, don't increment i to check for more patterns at the same position
        }
        
        changed
    }
}

/// Strength reduction pass (replace expensive operations with cheaper ones)
pub struct StrengthReductionPass {
    optimizations_applied: usize,
}

impl StrengthReductionPass {
    pub fn new() -> Self {
        Self { optimizations_applied: 0 }
    }
}

impl IROptimizationPass for StrengthReductionPass {
    fn name(&self) -> &str {
        "Strength Reduction"
    }

    fn optimize_program(&mut self, program: &mut IRProgram) -> bool {
        let mut changed = false;
        changed |= self.optimize_function(&mut program.main);
        
        for (_, function) in program.functions.iter_mut() {
            changed |= self.optimize_function(function);
        }
        
        changed
    }

    fn optimize_function(&mut self, function: &mut IRFunction) -> bool {
        let mut changed = false;
        let mut i = 0;
        
        while i < function.instructions.len() {
            // Pattern: Push(0) Add -> Drop (adding 0 is identity, just remove the 0)
            if i + 1 < function.instructions.len() {
                if let (
                    IRInstruction::Push(IRValue::Constant(0)) | IRInstruction::LoadConst(0),
                    IRInstruction::Add
                ) = (&function.instructions[i], &function.instructions[i + 1]) {
                    function.instructions.remove(i);
                    function.instructions.remove(i); // Remove Add too
                    changed = true;
                    self.optimizations_applied += 1;
                    continue;
                }
            }

            // Pattern: Push(1) Mul -> Drop (multiplying by 1 is identity)
            if i + 1 < function.instructions.len() {
                if let (
                    IRInstruction::Push(IRValue::Constant(1)) | IRInstruction::LoadConst(1),
                    IRInstruction::Mul
                ) = (&function.instructions[i], &function.instructions[i + 1]) {
                    function.instructions.remove(i);
                    function.instructions.remove(i);
                    changed = true;
                    self.optimizations_applied += 1;
                    continue;
                }
            }

            // Pattern: Push(0) Mul -> Drop LoadConst(0) (multiplying by 0 gives 0)
            if i + 1 < function.instructions.len() {
                if let (
                    IRInstruction::Push(IRValue::Constant(0)) | IRInstruction::LoadConst(0),
                    IRInstruction::Mul
                ) = (&function.instructions[i], &function.instructions[i + 1]) {
                    function.instructions[i] = IRInstruction::Drop; // Drop the other operand
                    function.instructions[i + 1] = IRInstruction::LoadConst(0);
                    changed = true;
                    self.optimizations_applied += 1;
                }
            }

            // Pattern: Push(power_of_2) Mul -> Dup Add (for small powers of 2)
            if i + 1 < function.instructions.len() {
                if let (
                    IRInstruction::Push(IRValue::Constant(n)) | IRInstruction::LoadConst(n),
                    IRInstruction::Mul
                ) = (&function.instructions[i], &function.instructions[i + 1]) {
                    if *n == 2 {
                        // x * 2 = x + x = dup add
                        function.instructions[i] = IRInstruction::Dup;
                        function.instructions[i + 1] = IRInstruction::Add;
                        changed = true;
                        self.optimizations_applied += 1;
                    }
                }
            }

            i += 1;
        }
        
        changed
    }
}

/// Function inlining optimization pass
pub struct FunctionInliningPass {
    optimizations_applied: usize,
    max_inline_size: usize, // Maximum function size to inline
}

impl FunctionInliningPass {
    pub fn new() -> Self {
        Self { 
            optimizations_applied: 0,
            max_inline_size: 20, // Don't inline functions with more than 20 instructions
        }
    }

    /// Check if a function is safe to inline
    fn is_inlinable(&self, function: &IRFunction) -> bool {
        // Don't inline if function is too large
        if function.instructions.len() > self.max_inline_size {
            return false;
        }

        // Check for problematic instructions that make inlining unsafe
        for instr in &function.instructions {
            match instr {
                // Don't inline recursive functions
                IRInstruction::Call(name) if name == &function.name => return false,
                // Don't inline functions with control flow (for now)
                IRInstruction::Jump(_) | 
                IRInstruction::JumpIf(_) | 
                IRInstruction::JumpIfNot(_) |
                IRInstruction::Label(_) => return false,
                _ => {}
            }
        }

        true
    }

    /// Get the inlinable body of a function (excluding Return instruction)
    fn get_inline_body(&self, function: &IRFunction) -> Vec<IRInstruction> {
        function.instructions.iter()
            .filter(|instr| !matches!(instr, IRInstruction::Return))
            .cloned()
            .collect()
    }
}

impl IROptimizationPass for FunctionInliningPass {
    fn name(&self) -> &str {
        "Function Inlining"
    }

    fn optimize_program(&mut self, program: &mut IRProgram) -> bool {
        let mut changed = false;
        
        // First, identify which functions are inlinable
        let mut inlinable_functions = HashMap::new();
        for (name, function) in &program.functions {
            if self.is_inlinable(function) {
                inlinable_functions.insert(name.clone(), self.get_inline_body(function));
            }
        }

        // Inline functions in main
        changed |= self.inline_in_function(&mut program.main, &inlinable_functions);
        
        // Inline functions in other functions
        for (_, function) in program.functions.iter_mut() {
            changed |= self.inline_in_function(function, &inlinable_functions);
        }
        
        changed
    }

    fn optimize_function(&mut self, _function: &mut IRFunction) -> bool {
        // This pass needs access to all functions, so we implement optimize_program instead
        false
    }
}

impl FunctionInliningPass {
    /// Inline function calls within a single function
    fn inline_in_function(&mut self, function: &mut IRFunction, inlinable_functions: &HashMap<String, Vec<IRInstruction>>) -> bool {
        let mut changed = false;
        let mut i = 0;
        
        while i < function.instructions.len() {
            if let IRInstruction::Call(function_name) = &function.instructions[i] {
                if let Some(inline_body) = inlinable_functions.get(function_name) {
                    // Replace the Call instruction with the function body
                    function.instructions.remove(i);
                    
                    // Insert the function body at the current position
                    for (offset, instr) in inline_body.iter().enumerate() {
                        function.instructions.insert(i + offset, instr.clone());
                    }
                    
                    changed = true;
                    self.optimizations_applied += 1;
                    
                    // Continue from after the inlined code
                    i += inline_body.len();
                    continue;
                }
            }
            i += 1;
        }
        
        changed
    }
}

/// IR optimization pipeline that runs multiple passes
pub struct IROptimizer {
    passes: Vec<Box<dyn IROptimizationPass>>,
    max_iterations: usize,
}

impl IROptimizer {
    pub fn new() -> Self {
        Self {
            passes: vec![
                Box::new(FunctionInliningPass::new()),  // Run inlining first
                Box::new(ConstantFoldingPass::new()),
                Box::new(PeepholeOptimizationPass::new()),
                Box::new(StrengthReductionPass::new()),
                Box::new(DeadCodeEliminationPass::new()),
            ],
            max_iterations: 10,
        }
    }

    pub fn add_pass(&mut self, pass: Box<dyn IROptimizationPass>) {
        self.passes.push(pass);
    }

    pub fn optimize(&mut self, program: &mut IRProgram) -> Vec<String> {
        let mut stats = Vec::new();
        let mut iteration = 0;
        
        loop {
            let mut any_changed = false;
            iteration += 1;
            
            if iteration > self.max_iterations {
                stats.push(format!("Optimization stopped after {} iterations", self.max_iterations));
                break;
            }

            for pass in &mut self.passes {
                let changed = pass.optimize_program(program);
                if changed {
                    any_changed = true;
                    stats.push(format!("Applied {} (iteration {})", pass.name(), iteration));
                }
            }

            if !any_changed {
                stats.push(format!("Optimization converged after {} iterations", iteration));
                break;
            }
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IRBuilder, IRValue};

    #[test]
    fn test_constant_folding() {
        let mut builder = IRBuilder::new("test");
        builder.emit(IRInstruction::Push(IRValue::Constant(5)));
        builder.emit(IRInstruction::Push(IRValue::Constant(3)));
        builder.emit(IRInstruction::Add);
        
        let mut program = builder.build();
        let mut pass = ConstantFoldingPass::new();
        
        let changed = pass.optimize_program(&mut program);
        assert!(changed);
        
        // Should be optimized to LoadConst(8)
        let non_comment_instructions: Vec<_> = program.main.instructions.iter()
            .filter(|instr| !matches!(instr, IRInstruction::Comment(_)))
            .collect();
        
        assert_eq!(non_comment_instructions.len(), 1);
        assert!(matches!(non_comment_instructions[0], IRInstruction::LoadConst(8)));
    }

    #[test]
    fn test_peephole_optimization() {
        let mut builder = IRBuilder::new("test");
        builder.emit(IRInstruction::Push(IRValue::Constant(5)));
        builder.emit(IRInstruction::Dup);
        builder.emit(IRInstruction::Add);
        
        let mut program = builder.build();
        let mut pass = PeepholeOptimizationPass::new();
        
        let changed = pass.optimize_program(&mut program);
        assert!(changed);
        
        // Should be optimized to LoadConst(10)
        let non_comment_instructions: Vec<_> = program.main.instructions.iter()
            .filter(|instr| !matches!(instr, IRInstruction::Comment(_)))
            .collect();
        
        assert_eq!(non_comment_instructions.len(), 1);
        assert!(matches!(non_comment_instructions[0], IRInstruction::LoadConst(10)));
    }

    #[test]
    fn test_dead_code_elimination() {
        let mut builder = IRBuilder::new("test");
        builder.emit(IRInstruction::Push(IRValue::Constant(42)));
        builder.emit(IRInstruction::Drop);
        builder.emit(IRInstruction::Nop);
        
        let mut program = builder.build();
        let mut pass = DeadCodeEliminationPass::new();
        
        let changed = pass.optimize_program(&mut program);
        assert!(changed);
        
        // Should remove all instructions
        let non_comment_instructions: Vec<_> = program.main.instructions.iter()
            .filter(|instr| !matches!(instr, IRInstruction::Comment(_)))
            .collect();
        
        assert_eq!(non_comment_instructions.len(), 0);
    }

    #[test]
    fn test_strength_reduction() {
        let mut builder = IRBuilder::new("test");
        builder.emit(IRInstruction::Push(IRValue::Constant(42)));
        builder.emit(IRInstruction::Push(IRValue::Constant(2)));
        builder.emit(IRInstruction::Mul);
        
        let mut program = builder.build();
        let mut pass = StrengthReductionPass::new();
        
        let changed = pass.optimize_program(&mut program);
        assert!(changed);
        
        // Should be optimized to: Push(42) Dup Add
        let non_comment_instructions: Vec<_> = program.main.instructions.iter()
            .filter(|instr| !matches!(instr, IRInstruction::Comment(_)))
            .collect();
        
        assert_eq!(non_comment_instructions.len(), 3);
        assert!(matches!(non_comment_instructions[0], IRInstruction::Push(IRValue::Constant(42))));
        assert!(matches!(non_comment_instructions[1], IRInstruction::Dup));
        assert!(matches!(non_comment_instructions[2], IRInstruction::Add));
    }

    #[test]
    fn test_full_optimization_pipeline() {
        let mut builder = IRBuilder::new("test");
        // 5 DUP + (should become 10)
        builder.emit(IRInstruction::Push(IRValue::Constant(5)));
        builder.emit(IRInstruction::Dup);
        builder.emit(IRInstruction::Add);
        
        let mut program = builder.build();
        let mut optimizer = IROptimizer::new();
        
        let stats = optimizer.optimize(&mut program);
        assert!(!stats.is_empty());
        
        // Should be fully optimized to LoadConst(10)
        let non_comment_instructions: Vec<_> = program.main.instructions.iter()
            .filter(|instr| !matches!(instr, IRInstruction::Comment(_)))
            .collect();
        
        assert_eq!(non_comment_instructions.len(), 1);
        assert!(matches!(non_comment_instructions[0], IRInstruction::LoadConst(10)));
    }
}