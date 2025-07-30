use crate::types::{Position, Token, TokenType, ParseError};

pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }

            let start_pos = Position {
                line: self.line,
                column: self.column,
                offset: self.position,
            };

            let token = self.next_token(start_pos)?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn next_token(&mut self, start_pos: Position) -> Result<Token, ParseError> {
        let ch = self.current_char();
        
        match ch {
            '(' => self.read_comment(start_pos),
            '"' => self.read_string_literal(start_pos),
            ':' => {
                self.advance();
                Ok(Token {
                    token_type: TokenType::StartDefinition,
                    position: start_pos,
                    raw: ":".to_string(),
                })
            },
            ';' => {
                self.advance();
                Ok(Token {
                    token_type: TokenType::EndDefinition,
                    position: start_pos,
                    raw: ";".to_string(),
                })
            },
            _ if ch.is_ascii_digit() || (ch == '-' && self.peek_char().map_or(false, |c| c.is_ascii_digit())) => {
                self.read_number(start_pos)
            },
            _ => self.read_word(start_pos),
        }
    }

    fn read_comment(&mut self, start_pos: Position) -> Result<Token, ParseError> {
        let mut comment = String::new();
        self.advance(); // skip '('
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == ')' {
                self.advance();
                break;
            }
            comment.push(ch);
            self.advance();
        }

        Ok(Token {
            token_type: TokenType::Comment(comment.clone()),
            position: start_pos,
            raw: format!("({})", comment),
        })
    }

    fn read_string_literal(&mut self, start_pos: Position) -> Result<Token, ParseError> {
        let mut string = String::new();
        self.advance(); // skip opening '"'
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == '"' {
                self.advance();
                break;
            }
            if ch == '\\' {
                self.advance();
                if self.position < self.input.len() {
                    let escaped = match self.current_char() {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        c => c,
                    };
                    string.push(escaped);
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Ok(Token {
            token_type: TokenType::StringLiteral(string.clone()),
            position: start_pos,
            raw: format!("\"{}\"", string),
        })
    }

    fn read_number(&mut self, start_pos: Position) -> Result<Token, ParseError> {
        let mut number_str = String::new();
        
        if self.current_char() == '-' {
            number_str.push('-');
            self.advance();
        }
        
        while self.position < self.input.len() && self.current_char().is_ascii_digit() {
            number_str.push(self.current_char());
            self.advance();
        }

        match number_str.parse::<i32>() {
            Ok(num) => Ok(Token {
                token_type: TokenType::Number(num),
                position: start_pos,
                raw: number_str,
            }),
            Err(_) => Err(ParseError {
                message: format!("Invalid number: {}", number_str),
                position: start_pos,
            }),
        }
    }

    fn read_word(&mut self, start_pos: Position) -> Result<Token, ParseError> {
        let mut word = String::new();
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '"' {
                break;
            }
            word.push(ch);
            self.advance();
        }

        if word.is_empty() {
            return Err(ParseError {
                message: "Unexpected character".to_string(),
                position: start_pos,
            });
        }

        Ok(Token {
            token_type: TokenType::Word(word.clone().to_uppercase()),
            position: start_pos,
            raw: word,
        })
    }

    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.current_char().is_whitespace() {
            self.advance();
        }
    }
}