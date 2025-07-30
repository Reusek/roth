use crate::types::AstNode;
use crate::codegen::{CodeGenerator, pattern::{PatternMatcher, Rule, Pattern, Template, TemplatePart}};
use crate::codegen::generator::{PatternBasedGenerator, create_word_pattern, create_number_pattern, create_definition_pattern};

pub struct CPatternGenerator {
    matcher: PatternMatcher,
}

impl CPatternGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            matcher: PatternMatcher::new(),
        };
        generator.setup_rules();
        generator
    }

    fn setup_rules(&mut self) {
        // Number pattern
        self.matcher.add_rule(Rule {
            pattern: create_number_pattern(),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("push(".to_string()),
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
                    TemplatePart::Literal("void ".to_string()),
                    TemplatePart::Variable("definition".to_string()),
                    TemplatePart::Literal("() {".to_string()),
                    TemplatePart::Indent,
                    TemplatePart::NewLine,
                    TemplatePart::Block("body".to_string(), vec![
                        TemplatePart::Variable("body".to_string()),
                    ]),
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
                    TemplatePart::Variable("word".to_string()),
                    TemplatePart::Literal("();\n".to_string()),
                ],
            },
            priority: 1,
        });
    }

    fn add_arithmetic_rules(&mut self) {
        let arithmetic_ops = vec![
            ("+", "forth_add();\n"),
            ("-", "forth_sub();\n"),
            ("*", "forth_mul();\n"),
            ("/", "forth_div();\n"),
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
            ("DUP", "forth_dup();\n"),
            ("DROP", "forth_drop();\n"),
            ("SWAP", "forth_swap();\n"),
            ("OVER", "forth_over();\n"),
            (".", "forth_dot();\n"),
            (".S", "forth_dots();\n"),
            ("CR", "printf(\"\\n\");\n"),
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
                output.push_str("// Generated Forth code in C\n");
                output.push_str("#include <stdio.h>\n");
                output.push_str("#include <stdlib.h>\n");
                output.push_str("#include <string.h>\n\n");
                output.push_str("#define STACK_SIZE 1000\n");
                output.push_str("#define MAX_WORDS 100\n\n");
                
                // Generate stack structure
                output.push_str("typedef struct {\n");
                output.push_str("    int data[STACK_SIZE];\n");
                output.push_str("    int top;\n");
                output.push_str("} Stack;\n\n");
                output.push_str("Stack stack = {0};\n\n");
                
                // Generate stack functions
                self.generate_stack_functions(&mut output);
                
                // Generate builtin operations
                self.generate_builtins(&mut output);
                
                // Generate user-defined functions first
                for node in nodes {
                    if let AstNode::Definition { .. } = node {
                        output.push_str(&self.generate_node(node));
                    }
                }
                
                // Generate main function
                output.push_str("int main() {\n");
                for node in nodes {
                    if let AstNode::Definition { .. } = node {
                        // Skip definitions in main - they're already generated above
                    } else {
                        let node_code = self.generate_node(node);
                        for line in node_code.lines() {
                            if !line.trim().is_empty() {
                                output.push_str("    ");
                                output.push_str(line);
                                output.push('\n');
                            }
                        }
                    }
                }
                output.push_str("    return 0;\n");
                output.push_str("}\n");
                
                output
            },
            AstNode::Definition { name, body, .. } => {
                let mut output = String::new();
                output.push_str(&format!("// Definition: {}\n", name));
                output.push_str(&format!("void {}() {{\n", name.to_lowercase()));
                
                for node in body {
                    let node_code = self.generate_node(node);
                    for line in node_code.lines() {
                        if !line.trim().is_empty() {
                            output.push_str("    ");
                            output.push_str(line);
                            output.push('\n');
                        }
                    }
                }
                
                output.push_str("}\n\n");
                output
            },
            AstNode::Word(name, _) => {
                let nodes = vec![node.clone()];
                if let Some((rule, result)) = self.matcher.find_matching_rule(&nodes, 0) {
                    self.matcher.render_template(&rule.template, &result.variables)
                } else {
                    format!("{}();\n", name.to_lowercase())
                }
            },
            AstNode::Number(n, _) => {
                format!("push({});\n", n)
            },
        }
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

    fn generate_builtins(&self, output: &mut String) {
        // Arithmetic operations
        let builtins = vec![
            ("forth_add", "int b = pop(); int a = pop(); push(a + b);"),
            ("forth_sub", "int b = pop(); int a = pop(); push(a - b);"),
            ("forth_mul", "int b = pop(); int a = pop(); push(a * b);"),
            ("forth_div", "int b = pop(); int a = pop(); if (b == 0) { printf(\"Division by zero\\n\"); exit(1); } push(a / b);"),
            ("forth_dup", "if (stack.top > 0) { push(stack.data[stack.top - 1]); } else { printf(\"Stack underflow\\n\"); exit(1); }"),
            ("forth_drop", "pop();"),
            ("forth_swap", "int b = pop(); int a = pop(); push(b); push(a);"),
            ("forth_over", "if (stack.top >= 2) { push(stack.data[stack.top - 2]); } else { printf(\"Stack underflow\\n\"); exit(1); }"),
            ("forth_dot", "printf(\"%d \", pop());"),
            ("forth_dots", "printf(\"<%d> \", stack.top); for (int i = 0; i < stack.top; i++) { printf(\"%d \", stack.data[i]); } printf(\"\\n\");"),
        ];

        for (name, body) in builtins {
            output.push_str(&format!("void {}() {{\n", name));
            output.push_str(&format!("    {}\n", body));
            output.push_str("}\n\n");
        }
    }
}

impl CodeGenerator for CPatternGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        self.generate_node(ast)
    }

    fn get_file_extension(&self) -> &str {
        "c"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("gcc -o {} {}", filename.trim_end_matches(".c"), filename)
    }
}