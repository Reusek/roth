use crate::types::AstNode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OptimizationContext {
    pub stack_depth: usize,
    pub known_values: HashMap<usize, i32>, // Stack position -> known constant value
    pub optimizations_applied: Vec<String>,
}

impl OptimizationContext {
    pub fn new() -> Self {
        Self {
            stack_depth: 0,
            known_values: HashMap::new(),
            optimizations_applied: Vec::new(),
        }
    }
}

pub trait Optimizer {
    fn optimize(&mut self, nodes: Vec<AstNode>) -> Vec<AstNode>;
    fn get_optimization_stats(&self) -> Vec<String>;
}

pub struct PeepholeOptimizer {
    optimizations_applied: Vec<String>,
}

impl PeepholeOptimizer {
    pub fn new() -> Self {
        Self {
            optimizations_applied: Vec::new(),
        }
    }

    fn optimize_sequence(&mut self, nodes: &[AstNode]) -> Vec<AstNode> {
        if nodes.len() < 2 {
            return nodes.to_vec();
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i < nodes.len() {
            let optimized = self.try_optimize_at(nodes, i);
            match optimized {
                Some((new_nodes, consumed)) => {
                    result.extend(new_nodes);
                    i += consumed;
                }
                None => {
                    result.push(nodes[i].clone());
                    i += 1;
                }
            }
        }

        result
    }

    fn try_optimize_at(&mut self, nodes: &[AstNode], start: usize) -> Option<(Vec<AstNode>, usize)> {
        // Constant folding: number number op -> result
        if start + 2 < nodes.len() {
            if let (AstNode::Number(a, pos1), AstNode::Number(b, _), AstNode::Word(op, _)) = 
                (&nodes[start], &nodes[start + 1], &nodes[start + 2]) {
                
                let result = match op.as_str() {
                    "+" => Some(a + b),
                    "-" => Some(a - b),
                    "*" => Some(a * b),
                    "/" if *b != 0 => Some(a / b),
                    _ => None,
                };

                if let Some(value) = result {
                    self.optimizations_applied.push(format!("Constant folding: {} {} {} -> {}", a, b, op, value));
                    return Some((vec![AstNode::Number(value, pos1.clone())], 3));
                }
            }
        }

        // DUP optimization: number DUP -> number number
        if start + 1 < nodes.len() {
            if let (AstNode::Number(n, pos1), AstNode::Word(op, _)) = (&nodes[start], &nodes[start + 1]) {
                if op == "DUP" {
                    self.optimizations_applied.push(format!("DUP optimization: {} DUP -> {} {}", n, n, n));
                    return Some((vec![
                        AstNode::Number(*n, pos1.clone()),
                        AstNode::Number(*n, pos1.clone())
                    ], 2));
                }
            }
        }

        // DROP optimization: number DROP -> (nothing)
        if start + 1 < nodes.len() {
            if let (AstNode::Number(n, _), AstNode::Word(op, _)) = (&nodes[start], &nodes[start + 1]) {
                if op == "DROP" {
                    self.optimizations_applied.push(format!("DROP optimization: {} DROP -> (removed)", n));
                    return Some((vec![], 2));
                }
            }
        }

        // SWAP optimization: a b SWAP -> b a
        if start + 2 < nodes.len() {
            if let (AstNode::Number(a, pos1), AstNode::Number(b, pos2), AstNode::Word(op, _)) = 
                (&nodes[start], &nodes[start + 1], &nodes[start + 2]) {
                if op == "SWAP" {
                    self.optimizations_applied.push(format!("SWAP optimization: {} {} SWAP -> {} {}", a, b, b, a));
                    return Some((vec![
                        AstNode::Number(*b, pos2.clone()),
                        AstNode::Number(*a, pos1.clone())
                    ], 3));
                }
            }
        }

        // Complex pattern: number DUP + -> number*2
        if start + 2 < nodes.len() {
            if let (AstNode::Number(n, pos), AstNode::Word(dup, _), AstNode::Word(add, _)) = 
                (&nodes[start], &nodes[start + 1], &nodes[start + 2]) {
                if dup == "DUP" && add == "+" {
                    self.optimizations_applied.push(format!("DUP + optimization: {} DUP + -> {}", n, n * 2));
                    return Some((vec![AstNode::Number(n * 2, pos.clone())], 3));
                }
            }
        }

        None
    }
}

impl Optimizer for PeepholeOptimizer {
    fn optimize(&mut self, nodes: Vec<AstNode>) -> Vec<AstNode> {
        let mut current = nodes;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10;

        // Keep optimizing until no more changes or max iterations
        loop {
            let optimized = self.optimize_sequence(&current);
            if optimized.len() == current.len() && 
               optimized.iter().zip(current.iter()).all(|(a, b)| std::ptr::eq(a, b)) {
                break;
            }
            current = optimized;
            iterations += 1;
            if iterations >= MAX_ITERATIONS {
                break;
            }
        }

        current
    }

    fn get_optimization_stats(&self) -> Vec<String> {
        self.optimizations_applied.clone()
    }
}

pub struct StackOptimizer {
    optimizations_applied: Vec<String>,
}

impl StackOptimizer {
    pub fn new() -> Self {
        Self {
            optimizations_applied: Vec::new(),
        }
    }

    fn analyze_stack_usage(&self, nodes: &[AstNode]) -> OptimizationContext {
        let mut ctx = OptimizationContext::new();
        
        for (i, node) in nodes.iter().enumerate() {
            match node {
                AstNode::Number(n, _) => {
                    ctx.known_values.insert(ctx.stack_depth, *n);
                    ctx.stack_depth += 1;
                }
                AstNode::Word(op, _) => {
                    match op.as_str() {
                        "+" | "-" | "*" | "/" => {
                            if ctx.stack_depth >= 2 {
                                ctx.stack_depth -= 1; // Two values consumed, one produced
                                ctx.known_values.remove(&ctx.stack_depth);
                            }
                        }
                        "DUP" => {
                            if ctx.stack_depth > 0 {
                                if let Some(&value) = ctx.known_values.get(&(ctx.stack_depth - 1)) {
                                    ctx.known_values.insert(ctx.stack_depth, value);
                                }
                                ctx.stack_depth += 1;
                            }
                        }
                        "DROP" => {
                            if ctx.stack_depth > 0 {
                                ctx.stack_depth -= 1;
                                ctx.known_values.remove(&ctx.stack_depth);
                            }
                        }
                        "SWAP" => {
                            if ctx.stack_depth >= 2 {
                                let top = ctx.known_values.remove(&(ctx.stack_depth - 1));
                                let second = ctx.known_values.remove(&(ctx.stack_depth - 2));
                                if let Some(val) = top {
                                    ctx.known_values.insert(ctx.stack_depth - 2, val);
                                }
                                if let Some(val) = second {
                                    ctx.known_values.insert(ctx.stack_depth - 1, val);
                                }
                            }
                        }
                        "." => {
                            if ctx.stack_depth > 0 {
                                ctx.stack_depth -= 1;
                                ctx.known_values.remove(&ctx.stack_depth);
                            }
                        }
                        _ => {
                            // Unknown operation, clear known values
                            ctx.known_values.clear();
                        }
                    }
                }
                _ => {}
            }
        }

        ctx
    }
}

impl Optimizer for StackOptimizer {
    fn optimize(&mut self, nodes: Vec<AstNode>) -> Vec<AstNode> {
        let ctx = self.analyze_stack_usage(&nodes);
        
        // For now, just return the nodes as-is
        // In a full implementation, we'd use the stack analysis to optimize
        self.optimizations_applied.push(format!("Stack analysis: max depth {}, {} known values", 
            ctx.stack_depth, ctx.known_values.len()));
        
        nodes
    }

    fn get_optimization_stats(&self) -> Vec<String> {
        self.optimizations_applied.clone()
    }
}

pub struct OptimizationPipeline {
    optimizers: Vec<Box<dyn Optimizer>>,
}

impl OptimizationPipeline {
    pub fn new() -> Self {
        Self {
            optimizers: vec![
                Box::new(PeepholeOptimizer::new()),
                Box::new(StackOptimizer::new()),
            ],
        }
    }

    pub fn add_optimizer(&mut self, optimizer: Box<dyn Optimizer>) {
        self.optimizers.push(optimizer);
    }

    pub fn optimize(&mut self, nodes: Vec<AstNode>) -> (Vec<AstNode>, Vec<String>) {
        let mut current = nodes;
        let mut all_stats = Vec::new();

        for optimizer in &mut self.optimizers {
            current = optimizer.optimize(current);
            all_stats.extend(optimizer.get_optimization_stats());
        }

        (current, all_stats)
    }
}

// Utility functions for pattern-based optimizations
pub fn create_optimized_pattern_rules() -> Vec<(Vec<AstNode>, Vec<AstNode>, String)> {
    use crate::types::Position;
    let pos = Position { line: 0, column: 0, offset: 0 };

    vec![
        // 0 + -> (nothing)
        (
            vec![AstNode::Number(0, pos.clone()), AstNode::Word("+".to_string(), pos.clone())],
            vec![],
            "Zero addition elimination".to_string()
        ),
        // 1 * -> (nothing)
        (
            vec![AstNode::Number(1, pos.clone()), AstNode::Word("*".to_string(), pos.clone())],
            vec![],
            "Identity multiplication elimination".to_string()
        ),
        // DUP DROP -> (nothing)
        (
            vec![AstNode::Word("DUP".to_string(), pos.clone()), AstNode::Word("DROP".to_string(), pos.clone())],
            vec![],
            "DUP DROP elimination".to_string()
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Position;

    #[test]
    fn test_constant_folding() {
        let mut optimizer = PeepholeOptimizer::new();
        let pos = Position { line: 1, column: 1, offset: 0 };
        
        let nodes = vec![
            AstNode::Number(5, pos.clone()),
            AstNode::Number(3, pos.clone()),
            AstNode::Word("+".to_string(), pos.clone()),
        ];

        let optimized = optimizer.optimize(nodes);
        
        assert_eq!(optimized.len(), 1);
        if let AstNode::Number(n, _) = &optimized[0] {
            assert_eq!(*n, 8);
        } else {
            panic!("Expected optimized result to be a number");
        }
    }

    #[test]
    fn test_dup_optimization() {
        let mut optimizer = PeepholeOptimizer::new();
        let pos = Position { line: 1, column: 1, offset: 0 };
        
        let nodes = vec![
            AstNode::Number(5, pos.clone()),
            AstNode::Word("DUP".to_string(), pos.clone()),
        ];

        let optimized = optimizer.optimize(nodes);
        
        assert_eq!(optimized.len(), 2);
        if let (AstNode::Number(n1, _), AstNode::Number(n2, _)) = (&optimized[0], &optimized[1]) {
            assert_eq!(*n1, 5);
            assert_eq!(*n2, 5);
        } else {
            panic!("Expected two numbers after DUP optimization");
        }
    }

    #[test]
    fn test_complex_optimization() {
        let mut optimizer = PeepholeOptimizer::new();
        let pos = Position { line: 1, column: 1, offset: 0 };
        
        // 5 DUP + should become 10
        let nodes = vec![
            AstNode::Number(5, pos.clone()),
            AstNode::Word("DUP".to_string(), pos.clone()),
            AstNode::Word("+".to_string(), pos.clone()),
        ];

        let optimized = optimizer.optimize(nodes);
        
        assert_eq!(optimized.len(), 1);
        if let AstNode::Number(n, _) = &optimized[0] {
            assert_eq!(*n, 10);
        } else {
            panic!("Expected optimized result to be 10");
        }
    }
}