use crate::types::AstNode;
use crate::codegen::pattern::{PatternMatcher, Rule, Pattern, Template, TemplatePart, MatchValue};
use std::collections::HashMap;

pub trait PatternBasedGenerator {
    fn setup_rules(&mut self);
    fn get_matcher(&mut self) -> &mut PatternMatcher;
    fn generate_with_patterns(&mut self, ast: &AstNode) -> String;
    fn get_builtin_templates(&self) -> HashMap<String, Template>;
}

pub struct PatternCodeGenerator {
    matcher: PatternMatcher,
    builtin_templates: HashMap<String, Template>,
}

impl PatternCodeGenerator {
    pub fn new() -> Self {
        Self {
            matcher: PatternMatcher::new(),
            builtin_templates: HashMap::new(),
        }
    }

    pub fn generate(&mut self, ast: &AstNode) -> String {
        self.setup_rules();
        self.generate_node(ast)
    }

    fn generate_node(&mut self, node: &AstNode) -> String {
        match node {
            AstNode::Program(nodes) => {
                let mut output = String::new();
                let mut i = 0;
                
                while i < nodes.len() {
                    if let Some((rule, result)) = self.matcher.find_matching_rule(nodes, i) {
                        let generated = self.matcher.render_template(&rule.template, &result.variables);
                        output.push_str(&generated);
                        i += result.consumed;
                    } else {
                        output.push_str(&self.generate_node(&nodes[i]));
                        i += 1;
                    }
                }
                output
            },
            AstNode::Definition { name, body, .. } => {
                let nodes = vec![node.clone()];
                if let Some((rule, result)) = self.matcher.find_matching_rule(&nodes, 0) {
                    self.matcher.render_template(&rule.template, &result.variables)
                } else {
                    format!("// Definition: {}\n", name)
                }
            },
            AstNode::Word(name, _) => {
                let nodes = vec![node.clone()];
                if let Some((rule, result)) = self.matcher.find_matching_rule(&nodes, 0) {
                    self.matcher.render_template(&rule.template, &result.variables)
                } else {
                    format!("// Word: {}\n", name)
                }
            },
            AstNode::Number(n, _) => {
                let nodes = vec![node.clone()];
                if let Some((rule, result)) = self.matcher.find_matching_rule(&nodes, 0) {
                    self.matcher.render_template(&rule.template, &result.variables)
                } else {
                    format!("// Number: {}\n", n)
                }
            },
        }
    }

    fn setup_rules(&mut self) {
        // This will be overridden by specific generators
    }
}

impl PatternBasedGenerator for PatternCodeGenerator {
    fn setup_rules(&mut self) {
        // Default empty implementation
    }

    fn get_matcher(&mut self) -> &mut PatternMatcher {
        &mut self.matcher
    }

    fn generate_with_patterns(&mut self, ast: &AstNode) -> String {
        self.generate(ast)
    }

    fn get_builtin_templates(&self) -> HashMap<String, Template> {
        self.builtin_templates.clone()
    }
}

pub fn create_word_pattern(word: &str) -> Pattern {
    let word_owned = word.to_string();
    Pattern::Named(
        "word".to_string(),
        Box::new(Pattern::Guard(
            Box::new(Pattern::AnyWord),
            Box::new(move |node: &AstNode| {
                if let AstNode::Word(w, _) = node {
                    w == &word_owned
                } else {
                    false
                }
            })
        ))
    )
}

pub fn create_number_pattern() -> Pattern {
    Pattern::Named("number".to_string(), Box::new(Pattern::AnyNumber))
}

pub fn create_definition_pattern() -> Pattern {
    Pattern::Named("definition".to_string(), Box::new(Pattern::AnyDefinition))
}

pub fn create_sequence_pattern(patterns: Vec<Pattern>) -> Pattern {
    Pattern::Sequence(patterns)
}

pub fn create_simple_template(template_str: &str) -> Template {
    Template {
        parts: vec![TemplatePart::Literal(template_str.to_string())],
    }
}

pub fn create_variable_template(var_name: &str) -> Template {
    Template {
        parts: vec![TemplatePart::Variable(var_name.to_string())],
    }
}

pub fn create_complex_template(parts: Vec<TemplatePart>) -> Template {
    Template { parts }
}

#[macro_export]
macro_rules! pattern {
    (word($w:expr)) => {
        create_word_pattern($w)
    };
    (number) => {
        create_number_pattern()
    };
    (definition) => {
        create_definition_pattern()
    };
    (seq($($p:expr),*)) => {
        create_sequence_pattern(vec![$($p),*])
    };
}

#[macro_export]
macro_rules! template {
    ($t:expr) => {
        create_simple_template($t)
    };
    (var($v:expr)) => {
        create_variable_template($v)
    };
    (parts($($p:expr),*)) => {
        create_complex_template(vec![$($p),*])
    };
}