use crate::types::AstNode;
use std::collections::HashMap;

pub enum Pattern {
    Exact(AstNode),
    AnyNumber,
    AnyWord,
    AnyDefinition,
    Sequence(Vec<Pattern>),
    Optional(Box<Pattern>),
    Repeat(Box<Pattern>),
    Named(String, Box<Pattern>),
    Guard(Box<Pattern>, Box<dyn Fn(&AstNode) -> bool>),
}

impl Clone for Pattern {
    fn clone(&self) -> Self {
        match self {
            Pattern::Exact(node) => Pattern::Exact(node.clone()),
            Pattern::AnyNumber => Pattern::AnyNumber,
            Pattern::AnyWord => Pattern::AnyWord,
            Pattern::AnyDefinition => Pattern::AnyDefinition,
            Pattern::Sequence(patterns) => Pattern::Sequence(patterns.clone()),
            Pattern::Optional(pattern) => Pattern::Optional(pattern.clone()),
            Pattern::Repeat(pattern) => Pattern::Repeat(pattern.clone()),
            Pattern::Named(name, pattern) => Pattern::Named(name.clone(), pattern.clone()),
            Pattern::Guard(_, _) => {
                // For simplicity, we'll create a new guard that always returns false
                // In a real implementation, you might want to handle this differently
                Pattern::Guard(
                    Box::new(Pattern::AnyWord),
                    Box::new(|_| false)
                )
            },
        }
    }
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Exact(node) => write!(f, "Exact({:?})", node),
            Pattern::AnyNumber => write!(f, "AnyNumber"),
            Pattern::AnyWord => write!(f, "AnyWord"),
            Pattern::AnyDefinition => write!(f, "AnyDefinition"),
            Pattern::Sequence(patterns) => write!(f, "Sequence({:?})", patterns),
            Pattern::Optional(pattern) => write!(f, "Optional({:?})", pattern),
            Pattern::Repeat(pattern) => write!(f, "Repeat({:?})", pattern),
            Pattern::Named(name, pattern) => write!(f, "Named({}, {:?})", name, pattern),
            Pattern::Guard(pattern, _) => write!(f, "Guard({:?}, <function>)", pattern),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    pub parts: Vec<TemplatePart>,
}

#[derive(Debug, Clone)]
pub enum TemplatePart {
    Literal(String),
    Variable(String),
    Block(String, Vec<TemplatePart>),
    Conditional(String, Vec<TemplatePart>, Option<Vec<TemplatePart>>),
    Loop(String, Vec<TemplatePart>),
    Indent,
    Dedent,
    NewLine,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub pattern: Pattern,
    pub template: Template,
    pub priority: i32,
}

pub struct PatternMatcher {
    rules: Vec<Rule>,
    variables: HashMap<String, MatchValue>,
}

#[derive(Debug, Clone)]
pub enum MatchValue {
    Node(AstNode),
    Nodes(Vec<AstNode>),
    String(String),
    Number(i32),
}

#[derive(Debug)]
pub struct MatchResult {
    pub matched: bool,
    pub variables: HashMap<String, MatchValue>,
    pub consumed: usize,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            variables: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn match_pattern(&self, pattern: &Pattern, nodes: &[AstNode], start: usize) -> MatchResult {
        let mut variables = HashMap::new();
        let consumed = self.match_pattern_impl(pattern, nodes, start, &mut variables);
        
        MatchResult {
            matched: consumed > 0,
            variables,
            consumed,
        }
    }

    fn match_pattern_impl(&self, pattern: &Pattern, nodes: &[AstNode], start: usize, variables: &mut HashMap<String, MatchValue>) -> usize {
        if start >= nodes.len() {
            return match pattern {
                Pattern::Optional(_) | Pattern::Repeat(_) => 0,
                _ => 0,
            };
        }

        match pattern {
            Pattern::Exact(expected) => {
                if self.nodes_equal(expected, &nodes[start]) {
                    1
                } else {
                    0
                }
            },
            Pattern::AnyNumber => {
                if let AstNode::Number(_, _) = &nodes[start] {
                    1
                } else {
                    0
                }
            },
            Pattern::AnyWord => {
                if let AstNode::Word(_, _) = &nodes[start] {
                    1
                } else {
                    0
                }
            },
            Pattern::AnyDefinition => {
                if let AstNode::Definition { .. } = &nodes[start] {
                    1
                } else {
                    0
                }
            },
            Pattern::Sequence(patterns) => {
                let mut total_consumed = 0;
                let mut current_pos = start;
                
                for pattern in patterns {
                    let consumed = self.match_pattern_impl(pattern, nodes, current_pos, variables);
                    if consumed == 0 {
                        return 0;
                    }
                    total_consumed += consumed;
                    current_pos += consumed;
                }
                total_consumed
            },
            Pattern::Optional(inner) => {
                let consumed = self.match_pattern_impl(inner, nodes, start, variables);
                if consumed > 0 { consumed } else { 0 }
            },
            Pattern::Repeat(inner) => {
                let mut total_consumed = 0;
                let mut current_pos = start;
                
                while current_pos < nodes.len() {
                    let consumed = self.match_pattern_impl(inner, nodes, current_pos, variables);
                    if consumed == 0 {
                        break;
                    }
                    total_consumed += consumed;
                    current_pos += consumed;
                }
                total_consumed
            },
            Pattern::Named(name, inner) => {
                let consumed = self.match_pattern_impl(inner, nodes, start, variables);
                if consumed > 0 {
                    if consumed == 1 {
                        variables.insert(name.clone(), MatchValue::Node(nodes[start].clone()));
                    } else {
                        variables.insert(name.clone(), MatchValue::Nodes(nodes[start..start + consumed].to_vec()));
                    }
                }
                consumed
            },
            Pattern::Guard(inner, guard_fn) => {
                let consumed = self.match_pattern_impl(inner, nodes, start, variables);
                if consumed > 0 && guard_fn(&nodes[start]) {
                    consumed
                } else {
                    0
                }
            },
        }
    }

    fn nodes_equal(&self, a: &AstNode, b: &AstNode) -> bool {
        match (a, b) {
            (AstNode::Number(n1, _), AstNode::Number(n2, _)) => n1 == n2,
            (AstNode::Word(w1, _), AstNode::Word(w2, _)) => w1 == w2,
            (AstNode::Definition { name: n1, .. }, AstNode::Definition { name: n2, .. }) => n1 == n2,
            _ => false,
        }
    }

    pub fn find_matching_rule(&self, nodes: &[AstNode], start: usize) -> Option<(&Rule, MatchResult)> {
        for rule in &self.rules {
            let result = self.match_pattern(&rule.pattern, nodes, start);
            if result.matched {
                return Some((rule, result));
            }
        }
        None
    }

    pub fn render_template(&self, template: &Template, variables: &HashMap<String, MatchValue>) -> String {
        let mut output = String::new();
        let mut indent_level = 0;
        
        for part in &template.parts {
            self.render_template_part(part, variables, &mut output, &mut indent_level);
        }
        
        output
    }

    fn render_template_part(&self, part: &TemplatePart, variables: &HashMap<String, MatchValue>, output: &mut String, indent_level: &mut usize) {
        match part {
            TemplatePart::Literal(text) => {
                output.push_str(text);
            },
            TemplatePart::Variable(name) => {
                if let Some(value) = variables.get(name) {
                    match value {
                        MatchValue::String(s) => output.push_str(s),
                        MatchValue::Number(n) => output.push_str(&n.to_string()),
                        MatchValue::Node(node) => {
                            output.push_str(&self.node_to_string(node));
                        },
                        MatchValue::Nodes(nodes) => {
                            for (i, node) in nodes.iter().enumerate() {
                                if i > 0 { output.push(' '); }
                                output.push_str(&self.node_to_string(node));
                            }
                        },
                    }
                }
            },
            TemplatePart::Block(var_name, parts) => {
                if variables.contains_key(var_name) {
                    for part in parts {
                        self.render_template_part(part, variables, output, indent_level);
                    }
                }
            },
            TemplatePart::Conditional(var_name, then_parts, else_parts) => {
                if variables.contains_key(var_name) {
                    for part in then_parts {
                        self.render_template_part(part, variables, output, indent_level);
                    }
                } else if let Some(else_parts) = else_parts {
                    for part in else_parts {
                        self.render_template_part(part, variables, output, indent_level);
                    }
                }
            },
            TemplatePart::Loop(var_name, parts) => {
                if let Some(MatchValue::Nodes(nodes)) = variables.get(var_name) {
                    for node in nodes {
                        let mut loop_vars = variables.clone();
                        loop_vars.insert("item".to_string(), MatchValue::Node(node.clone()));
                        for part in parts {
                            self.render_template_part(part, &loop_vars, output, indent_level);
                        }
                    }
                }
            },
            TemplatePart::Indent => {
                *indent_level += 1;
            },
            TemplatePart::Dedent => {
                if *indent_level > 0 {
                    *indent_level -= 1;
                }
            },
            TemplatePart::NewLine => {
                output.push('\n');
                for _ in 0..*indent_level {
                    output.push_str("    ");
                }
            },
        }
    }

    fn node_to_string(&self, node: &AstNode) -> String {
        match node {
            AstNode::Number(n, _) => n.to_string(),
            AstNode::Word(w, _) => w.clone(),
            AstNode::Definition { name, .. } => name.clone(),
            AstNode::Program(_) => "program".to_string(),
        }
    }
}

pub fn create_pattern(pattern_str: &str) -> Pattern {
    match pattern_str {
        "number" => Pattern::AnyNumber,
        "word" => Pattern::AnyWord,
        "definition" => Pattern::AnyDefinition,
        _ => Pattern::AnyWord,
    }
}

pub fn create_template(template_str: &str) -> Template {
    Template {
        parts: vec![TemplatePart::Literal(template_str.to_string())],
    }
}