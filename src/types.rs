#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Number(i32),
    Word(String),
    StartDefinition,
    EndDefinition,
    Comment(String),
    StringLiteral(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Position,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Number(i32, Position),
    Word(String, Position),
    Definition {
        name: String,
        body: Vec<AstNode>,
        position: Position,
    },
    Program(Vec<AstNode>),
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: Position,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at line {}, column {}: {}", 
               self.position.line, self.position.column, self.message)
    }
}