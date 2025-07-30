use crate::types::AstNode;
use crate::codegen::{CodeGenerator, optimizer::{OptimizationPipeline, Optimizer}};

pub struct OptimizedRustGenerator {
    optimization_pipeline: OptimizationPipeline,
}

impl OptimizedRustGenerator {
    pub fn new() -> Self {
        Self {
            optimization_pipeline: OptimizationPipeline::new(),
        }
    }

    fn generate_optimized_node(&mut self, node: &AstNode) -> String {
        match node {
            AstNode::Program(nodes) => {
                // Apply optimizations to the entire program
                let (optimized_nodes, stats) = self.optimization_pipeline.optimize(nodes.clone());
                
                let mut output = String::new();
                
                // Generate header
                output.push_str("// Optimized Generated Forth code\n");
                if !stats.is_empty() {
                    output.push_str("// Optimizations applied:\n");
                    for stat in &stats {
                        output.push_str(&format!("// - {}\n", stat));
                    }
                }
                output.push_str("use std::collections::HashMap;\n\n");
                output.push_str("pub struct OptimizedForth {\n");
                output.push_str("    stack: Vec<i32>,\n");
                output.push_str("    words: HashMap<String, Vec<String>>,\n");
                output.push_str("}\n\n");
                
                output.push_str("impl OptimizedForth {\n");
                output.push_str("    pub fn new() -> Self {\n");
                output.push_str("        Self {\n");
                output.push_str("            stack: Vec::new(),\n");
                output.push_str("            words: HashMap::new(),\n");
                output.push_str("        }\n");
                output.push_str("    }\n\n");
                
                // Generate definitions first
                for node in &optimized_nodes {
                    if let AstNode::Definition { .. } = node {
                        output.push_str(&self.generate_optimized_node(node));
                    }
                }
                
                // Generate main execution function with optimized code
                output.push_str("    pub fn execute(&mut self) -> Result<(), String> {\n");
                
                // Generate optimized sequence
                let execution_nodes: Vec<&AstNode> = optimized_nodes.iter()
                    .filter(|n| !matches!(n, AstNode::Definition { .. }))
                    .collect();
                
                let optimized_code = self.generate_optimized_sequence(&execution_nodes);
                for line in optimized_code.lines() {
                    if !line.trim().is_empty() {
                        output.push_str("        ");
                        output.push_str(line);
                        output.push('\n');
                    }
                }
                
                output.push_str("        Ok(())\n");
                output.push_str("    }\n");
                output.push_str("}\n");
                
                output
            },
            AstNode::Definition { name, body, .. } => {
                let mut output = String::new();
                output.push_str(&format!("    // Definition: {}\n", name));
                output.push_str(&format!("    fn {}(&mut self) -> Result<(), String> {{\n", name.to_lowercase()));
                
                // Optimize the definition body
                let (optimized_body, _) = self.optimization_pipeline.optimize(body.clone());
                let body_refs: Vec<&AstNode> = optimized_body.iter().collect();
                let optimized_code = self.generate_optimized_sequence(&body_refs);
                
                for line in optimized_code.lines() {
                    if !line.trim().is_empty() {
                        output.push_str("        ");
                        output.push_str(line);
                        output.push('\n');
                    }
                }
                
                output.push_str("        Ok(())\n");
                output.push_str("    }\n\n");
                output
            },
            _ => self.generate_single_node(node),
        }
    }

    fn generate_optimized_sequence(&self, nodes: &[&AstNode]) -> String {
        let mut output = String::new();
        let mut stack_simulation = Vec::new();
        
        for node in nodes {
            match node {
                AstNode::Number(n, _) => {
                    stack_simulation.push(*n);
                    output.push_str(&format!("self.stack.push({});\n", n));
                }
                AstNode::Word(op, _) => {
                    match op.as_str() {
                        "+" => {
                            if stack_simulation.len() >= 2 {
                                let b = stack_simulation.pop().unwrap();
                                let a = stack_simulation.pop().unwrap();
                                let result = a + b;
                                stack_simulation.push(result);
                                output.push_str(&format!("// Optimized: {} + {} = {}\n", a, b, result));
                                output.push_str(&format!("self.stack.push({});\n", result));
                            } else {
                                output.push_str("{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }\n");
                                stack_simulation.clear(); // Can't track anymore
                            }
                        }
                        "-" => {
                            if stack_simulation.len() >= 2 {
                                let b = stack_simulation.pop().unwrap();
                                let a = stack_simulation.pop().unwrap();
                                let result = a - b;
                                stack_simulation.push(result);
                                output.push_str(&format!("// Optimized: {} - {} = {}\n", a, b, result));
                                output.push_str(&format!("self.stack.push({});\n", result));
                            } else {
                                output.push_str("{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a - b); }\n");
                                stack_simulation.clear();
                            }
                        }
                        "*" => {
                            if stack_simulation.len() >= 2 {
                                let b = stack_simulation.pop().unwrap();
                                let a = stack_simulation.pop().unwrap();
                                let result = a * b;
                                stack_simulation.push(result);
                                output.push_str(&format!("// Optimized: {} * {} = {}\n", a, b, result));
                                output.push_str(&format!("self.stack.push({});\n", result));
                            } else {
                                output.push_str("{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }\n");
                                stack_simulation.clear();
                            }
                        }
                        "/" => {
                            if stack_simulation.len() >= 2 {
                                let b = stack_simulation.pop().unwrap();
                                let a = stack_simulation.pop().unwrap();
                                if b != 0 {
                                    let result = a / b;
                                    stack_simulation.push(result);
                                    output.push_str(&format!("// Optimized: {} / {} = {}\n", a, b, result));
                                    output.push_str(&format!("self.stack.push({});\n", result));
                                } else {
                                    output.push_str("{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a / b); }\n");
                                    stack_simulation.clear();
                                }
                            } else {
                                output.push_str("{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a / b); }\n");
                                stack_simulation.clear();
                            }
                        }
                        "DUP" => {
                            if let Some(&top) = stack_simulation.last() {
                                stack_simulation.push(top);
                                output.push_str(&format!("// Optimized DUP: {}\n", top));
                                output.push_str(&format!("self.stack.push({});\n", top));
                            } else {
                                output.push_str("{ let top = *self.stack.last().unwrap(); self.stack.push(top); }\n");
                                stack_simulation.clear();
                            }
                        }
                        "DROP" => {
                            if !stack_simulation.is_empty() {
                                let dropped = stack_simulation.pop().unwrap();
                                output.push_str(&format!("// Optimized DROP: {} (eliminated)\n", dropped));
                                // No code generated - value is simply not used
                            } else {
                                output.push_str("self.stack.pop();\n");
                            }
                        }
                        "SWAP" => {
                            if stack_simulation.len() >= 2 {
                                let top = stack_simulation.pop().unwrap();
                                let second = stack_simulation.pop().unwrap();
                                stack_simulation.push(top);
                                stack_simulation.push(second);
                                output.push_str(&format!("// Optimized SWAP: {} {} -> {} {}\n", second, top, top, second));
                                output.push_str(&format!("self.stack.push({});\n", top));
                                output.push_str(&format!("self.stack.push({});\n", second));
                            } else {
                                output.push_str("{ let len = self.stack.len(); self.stack.swap(len-1, len-2); }\n");
                                stack_simulation.clear();
                            }
                        }
                        "." => {
                            if let Some(value) = stack_simulation.pop() {
                                output.push_str(&format!("// Optimized print: {}\n", value));
                                output.push_str(&format!("print!(\"{} \");\n", value));
                            } else {
                                output.push_str("print!(\"{} \", self.stack.pop().unwrap());\n");
                            }
                        }
                        ".S" => {
                            output.push_str("println!(\"<{}> {:?}\", self.stack.len(), self.stack);\n");
                            stack_simulation.clear(); // Stack state is printed, can't optimize further
                        }
                        "CR" => {
                            output.push_str("println!();\n");
                        }
                        _ => {
                            output.push_str(&format!("self.execute_word(\"{}\")?;\n", op));
                            stack_simulation.clear(); // Unknown operation
                        }
                    }
                }
                _ => {
                    output.push_str(&self.generate_single_node(node));
                    stack_simulation.clear();
                }
            }
        }
        
        output
    }

    fn generate_single_node(&self, node: &AstNode) -> String {
        match node {
            AstNode::Number(n, _) => format!("self.stack.push({});\n", n),
            AstNode::Word(name, _) => format!("self.execute_word(\"{}\")?;\n", name),
            _ => String::new(),
        }
    }
}

impl CodeGenerator for OptimizedRustGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        self.generate_optimized_node(ast)
    }

    fn get_file_extension(&self) -> &str {
        "rs"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("rustc -O {}", filename)
    }
}