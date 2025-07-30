use crate::types::AstNode;
use crate::codegen::{CodeGenerator, pattern::{PatternMatcher, Rule, Pattern, Template, TemplatePart}};
use crate::codegen::generator::{PatternBasedGenerator, create_word_pattern, create_number_pattern, create_definition_pattern};
use std::collections::HashMap;

pub struct RustPatternGenerator {
    matcher: PatternMatcher,
}

impl RustPatternGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            matcher: PatternMatcher::new(),
        };
        generator.setup_rules();
        generator
    }

    fn setup_rules(&mut self) {
        // Program header rule
        self.matcher.add_rule(Rule {
            pattern: Pattern::Named("program".to_string(), Box::new(Pattern::AnyWord)),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("// Generated Forth code\n".to_string()),
                    TemplatePart::Literal("use std::collections::HashMap;\n\n".to_string()),
                    TemplatePart::Literal("pub struct GeneratedForth {\n".to_string()),
                    TemplatePart::Indent,
                    TemplatePart::NewLine,
                    TemplatePart::Literal("stack: Vec<i32>,".to_string()),
                    TemplatePart::NewLine,
                    TemplatePart::Literal("words: HashMap<String, Vec<String>>,".to_string()),
                    TemplatePart::Dedent,
                    TemplatePart::NewLine,
                    TemplatePart::Literal("}\n\n".to_string()),
                ],
            },
            priority: 100,
        });

        // Number pattern
        self.matcher.add_rule(Rule {
            pattern: create_number_pattern(),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("self.stack.push(".to_string()),
                    TemplatePart::Variable("number".to_string()),
                    TemplatePart::Literal(");\n".to_string()),
                ],
            },
            priority: 10,
        });

        // Arithmetic operations
        self.add_arithmetic_rules();
        
        // Stack operations
        self.add_stack_rules();
        
        // Definition pattern
        self.matcher.add_rule(Rule {
            pattern: create_definition_pattern(),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("// Definition: ".to_string()),
                    TemplatePart::Variable("definition".to_string()),
                    TemplatePart::NewLine,
                    TemplatePart::Literal("fn ".to_string()),
                    TemplatePart::Variable("definition".to_string()),
                    TemplatePart::Literal("(&mut self) -> Result<(), String> {".to_string()),
                    TemplatePart::Indent,
                    TemplatePart::NewLine,
                    TemplatePart::Block("body".to_string(), vec![
                        TemplatePart::Variable("body".to_string()),
                    ]),
                    TemplatePart::Literal("Ok(())".to_string()),
                    TemplatePart::Dedent,
                    TemplatePart::NewLine,
                    TemplatePart::Literal("}\n\n".to_string()),
                ],
            },
            priority: 20,
        });

        // Generic word pattern (lowest priority)
        self.matcher.add_rule(Rule {
            pattern: Pattern::Named("word".to_string(), Box::new(Pattern::AnyWord)),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("self.execute_word(\"".to_string()),
                    TemplatePart::Variable("word".to_string()),
                    TemplatePart::Literal("\")?.;\n".to_string()),
                ],
            },
            priority: 1,
        });
    }

    fn add_arithmetic_rules(&mut self) {
        let arithmetic_ops = vec![
            ("+", "self.stack.push(self.stack.pop().unwrap() + self.stack.pop().unwrap());\n"),
            ("-", "{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a - b); }\n"),
            ("*", "self.stack.push(self.stack.pop().unwrap() * self.stack.pop().unwrap());\n"),
            ("/", "{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a / b); }\n"),
        ];

        for (op, code) in arithmetic_ops {
            self.matcher.add_rule(Rule {
                pattern: create_word_pattern(op),
                template: Template {
                    parts: vec![TemplatePart::Literal(code.to_string())],
                },
                priority: 50,
            });
        }
    }

    fn add_stack_rules(&mut self) {
        let stack_ops = vec![
            ("DUP", "{ let top = *self.stack.last().unwrap(); self.stack.push(top); }\n"),
            ("DROP", "self.stack.pop();\n"),
            ("SWAP", "{ let len = self.stack.len(); self.stack.swap(len-1, len-2); }\n"),
            ("OVER", "{ let val = self.stack[self.stack.len()-2]; self.stack.push(val); }\n"),
            (".", "print!(\"{} \", self.stack.pop().unwrap());\n"),
            (".S", "println!(\"<{}> {:?}\", self.stack.len(), self.stack);\n"),
            ("CR", "println!();\n"),
        ];

        for (op, code) in stack_ops {
            self.matcher.add_rule(Rule {
                pattern: create_word_pattern(op),
                template: Template {
                    parts: vec![TemplatePart::Literal(code.to_string())],
                },
                priority: 50,
            });
        }
    }

    fn generate_node(&mut self, node: &AstNode) -> String {
        match node {
            AstNode::Program(nodes) => {
                let mut output = String::new();
                
                // Generate header
                output.push_str("// Generated Forth code\n");
                output.push_str("use std::collections::HashMap;\n\n");
                output.push_str("pub struct GeneratedForth {\n");
                output.push_str("    stack: Vec<i32>,\n");
                output.push_str("    words: HashMap<String, Vec<String>>,\n");
                output.push_str("}\n\n");
                
                output.push_str("impl GeneratedForth {\n");
                output.push_str("    pub fn new() -> Self {\n");
                output.push_str("        Self {\n");
                output.push_str("            stack: Vec::new(),\n");
                output.push_str("            words: HashMap::new(),\n");
                output.push_str("        }\n");
                output.push_str("    }\n\n");
                
                // Generate definitions first
                for node in nodes {
                    if let AstNode::Definition { .. } = node {
                        output.push_str(&self.generate_node(node));
                    }
                }
                
                // Generate main execution function
                output.push_str("    pub fn execute(&mut self) -> Result<(), String> {\n");
                for node in nodes {
                    if let AstNode::Definition { .. } = node {
                        // Skip definitions in main execution
                    } else {
                        let node_code = self.generate_node(node);
                        for line in node_code.lines() {
                            if !line.trim().is_empty() {
                                output.push_str("        ");
                                output.push_str(line);
                                output.push('\n');
                            }
                        }
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
                
                for node in body {
                    let node_code = self.generate_node(node);
                    for line in node_code.lines() {
                        if !line.trim().is_empty() {
                            output.push_str("        ");
                            output.push_str(line);
                            output.push('\n');
                        }
                    }
                }
                
                output.push_str("        Ok(())\n");
                output.push_str("    }\n\n");
                output
            },
            AstNode::Word(name, _) => {
                match name.as_str() {
                    "+" => "{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }\n".to_string(),
                    "-" => "{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a - b); }\n".to_string(),
                    "*" => "{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }\n".to_string(),
                    "/" => "{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a / b); }\n".to_string(),
                    "DUP" => "{ let top = *self.stack.last().unwrap(); self.stack.push(top); }\n".to_string(),
                    "DROP" => "self.stack.pop();\n".to_string(),
                    "SWAP" => "{ let len = self.stack.len(); self.stack.swap(len-1, len-2); }\n".to_string(),
                    "OVER" => "{ let val = self.stack[self.stack.len()-2]; self.stack.push(val); }\n".to_string(),
                    "." => "print!(\"{} \", self.stack.pop().unwrap());\n".to_string(),
                    ".S" => "println!(\"<{}> {:?}\", self.stack.len(), self.stack);\n".to_string(),
                    "CR" => "println!();\n".to_string(),
                    _ => format!("self.execute_word(\"{}\")?;\n", name),
                }
            },
            AstNode::Number(n, _) => {
                format!("self.stack.push({});\n", n)
            },
        }
    }
}

impl CodeGenerator for RustPatternGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        self.generate_node(ast)
    }

    fn get_file_extension(&self) -> &str {
        "rs"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("rustc {}", filename)
    }
}