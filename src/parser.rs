use crate::types::{Token, TokenType, AstNode, ParseError};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        let mut nodes = Vec::new();
        
        while self.position < self.tokens.len() {
            let node = self.parse_statement()?;
            nodes.push(node);
        }

        Ok(AstNode::Program(nodes))
    }

    fn parse_statement(&mut self) -> Result<AstNode, ParseError> {
        let token = &self.tokens[self.position];
        
        match &token.token_type {
            TokenType::Number(n) => {
                let pos = token.position.clone();
                self.position += 1;
                Ok(AstNode::Number(*n, pos))
            },
            TokenType::Word(w) => {
                let pos = token.position.clone();
                self.position += 1;
                Ok(AstNode::Word(w.clone(), pos))
            },
            TokenType::StartDefinition => {
                self.parse_definition()
            },
            TokenType::Comment(_) => {
                self.position += 1;
                self.parse_statement()
            },
            _ => Err(ParseError {
                message: format!("Unexpected token: {:?}", token.token_type),
                position: token.position.clone(),
            }),
        }
    }

    fn parse_definition(&mut self) -> Result<AstNode, ParseError> {
        let start_pos = self.tokens[self.position].position.clone();
        self.position += 1; // skip ':'

        if self.position >= self.tokens.len() {
            return Err(ParseError {
                message: "Expected word name after ':'".to_string(),
                position: start_pos,
            });
        }

        let name = match &self.tokens[self.position].token_type {
            TokenType::Word(w) => w.clone(),
            _ => return Err(ParseError {
                message: "Expected word name after ':'".to_string(),
                position: self.tokens[self.position].position.clone(),
            }),
        };
        self.position += 1;

        let mut body = Vec::new();
        while self.position < self.tokens.len() {
            match &self.tokens[self.position].token_type {
                TokenType::EndDefinition => {
                    self.position += 1;
                    break;
                },
                TokenType::StartDefinition => {
                    return Err(ParseError {
                        message: "Cannot nest word definitions".to_string(),
                        position: self.tokens[self.position].position.clone(),
                    });
                },
                TokenType::Comment(_) => {
                    self.position += 1;
                    continue;
                },
                _ => {
                    let node = self.parse_statement()?;
                    body.push(node);
                },
            }
        }

        Ok(AstNode::Definition {
            name,
            body,
            position: start_pos,
        })
    }
}