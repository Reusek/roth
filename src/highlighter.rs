use colored::{ColoredString, Colorize};
use std::collections::HashMap;
use tree_sitter::{Parser, Query, QueryCursor};

pub struct SyntaxHighlighter {
    parser: Parser,
    query: Query,
    color_map: HashMap<String, Box<dyn Fn(&str) -> ColoredString>>,
}

impl SyntaxHighlighter {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let language = tree_sitter_c::language();
        let mut parser = Parser::new();
        parser.set_language(language)?;

        let query_source = r#"
            (comment) @comment
            (string_literal) @string
            (number_literal) @number
            (primitive_type) @type
            (identifier) @identifier
            (function_declarator (identifier) @function)
            (call_expression (identifier) @function.call)
        "#;

        let query = Query::new(language, query_source)?;

        let mut color_map = HashMap::new();
        color_map.insert(
            "comment".to_string(),
            Box::new(|s: &str| s.bright_black()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "string".to_string(),
            Box::new(|s: &str| s.green()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "number".to_string(),
            Box::new(|s: &str| s.cyan()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "type".to_string(),
            Box::new(|s: &str| s.blue()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "identifier".to_string(),
            Box::new(|s: &str| s.normal()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "function".to_string(),
            Box::new(|s: &str| s.yellow()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "function.call".to_string(),
            Box::new(|s: &str| s.bright_yellow()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "keyword".to_string(),
            Box::new(|s: &str| s.magenta()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "keyword.type".to_string(),
            Box::new(|s: &str| s.bright_blue()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "preprocessor".to_string(),
            Box::new(|s: &str| s.bright_magenta()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "punctuation.bracket".to_string(),
            Box::new(|s: &str| s.bright_white()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "punctuation.delimiter".to_string(),
            Box::new(|s: &str| s.white()) as Box<dyn Fn(&str) -> ColoredString>,
        );
        color_map.insert(
            "operator".to_string(),
            Box::new(|s: &str| s.red()) as Box<dyn Fn(&str) -> ColoredString>,
        );

        Ok(Self {
            parser,
            query,
            color_map,
        })
    }

    pub fn highlight(&mut self, code: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.highlight_with_force(code, false)
    }

    pub fn highlight_with_force(
        &mut self,
        code: &str,
        force_color: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if !force_color && !atty::is(atty::Stream::Stdout) {
            return Ok(code.to_string());
        }

        let tree = self
            .parser
            .parse(code, None)
            .ok_or("Failed to parse code")?;
        let root_node = tree.root_node();

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, root_node, code.as_bytes());

        let mut highlights = Vec::new();
        for m in matches {
            for capture in m.captures {
                let capture_name = &self.query.capture_names()[capture.index as usize];
                let node = capture.node;
                let start = node.start_byte();
                let end = node.end_byte();
                highlights.push((start, end, capture_name.clone()));
            }
        }

        highlights.sort_by_key(|&(start, _, _)| start);

        let mut result = String::new();
        let mut last_end = 0;

        for (start, end, capture_name) in highlights {
            if start < last_end {
                continue;
            }

            if start > last_end {
                result.push_str(&code[last_end..start]);
            }

            let text = &code[start..end];
            if let Some(color_fn) = self.color_map.get(&capture_name) {
                result.push_str(&color_fn(text).to_string());
            } else {
                result.push_str(text);
            }

            last_end = end;
        }

        if last_end < code.len() {
            result.push_str(&code[last_end..]);
        }

        Ok(result)
    }
}
