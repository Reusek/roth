use crate::types::{AstNode, ParseError};
use std::collections::HashMap;

pub struct SemanticAnalyzer {
    defined_words: HashMap<String, bool>,
    builtin_words: HashMap<String, bool>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            defined_words: HashMap::new(),
            builtin_words: HashMap::new(),
        };

        // Register builtin words
        let builtins = vec![
            "+", "-", "*", "/", "DUP", "DROP", "SWAP", "OVER", ".", ".S", "CR", "ROT", "?DO", "DO",
            "LOOP", "I", "J", "LEAVE", "UNLOOP", "IF", "ELSE", "THEN", "BEGIN", "WHILE", "REPEAT",
            "UNTIL", "=", "<", ">", "<=", ">=", "<>", "0=", "0<", "0>", "AND", "OR", "XOR", "NOT",
            "INVERT", "MOD", "ABS", "NEGATE", "MIN", "MAX", "EMIT", "KEY", "SPACE", "SPACES",
            "TYPE", "!", "@", "C!", "C@", "ALLOT", "HERE", "VARIABLE", "CONSTANT", "2DUP", "2DROP",
            "2SWAP", "2OVER", "NIP", "TUCK", "PICK", "ROLL",
        ];
        for word in builtins {
            analyzer.builtin_words.insert(word.to_string(), true);
        }

        analyzer
    }

    pub fn analyze(&mut self, ast: &AstNode) -> Result<(), ParseError> {
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.analyze(node)?;
                }
            }
            AstNode::Definition {
                name,
                body,
                position,
            } => {
                if self.builtin_words.contains_key(name) {
                    return Err(ParseError {
                        message: format!("Cannot redefine builtin word: {}", name),
                        position: position.clone(),
                    });
                }

                for node in body {
                    self.analyze(node)?;
                }

                self.defined_words.insert(name.clone(), true);
            }
            AstNode::Word(name, position) => {
                if !self.builtin_words.contains_key(name) && !self.defined_words.contains_key(name)
                {
                    return Err(ParseError {
                        message: format!("Undefined word: {}", name),
                        position: position.clone(),
                    });
                }
            }
            AstNode::Number(_, _) => {}
            AstNode::StringLiteral(_, _) => {}
        }
        Ok(())
    }
}
