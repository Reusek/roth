use roth::lexer::Lexer;
use roth::types::TokenType;

#[test]
fn test_tokenize_numbers() {
    let mut lexer = Lexer::new("42 -17 0".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].token_type, TokenType::Number(42));
    assert_eq!(tokens[1].token_type, TokenType::Number(-17));
    assert_eq!(tokens[2].token_type, TokenType::Number(0));
}

#[test]
fn test_tokenize_words() {
    let mut lexer = Lexer::new("DUP SWAP add".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].token_type, TokenType::Word("DUP".to_string()));
    assert_eq!(tokens[1].token_type, TokenType::Word("SWAP".to_string()));
    assert_eq!(tokens[2].token_type, TokenType::Word("ADD".to_string()));
}

#[test]
fn test_tokenize_definition_markers() {
    let mut lexer = Lexer::new(": test ;".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].token_type, TokenType::StartDefinition);
    assert_eq!(tokens[1].token_type, TokenType::Word("TEST".to_string()));
    assert_eq!(tokens[2].token_type, TokenType::EndDefinition);
}

#[test]
fn test_tokenize_comments() {
    let mut lexer = Lexer::new("( this is a comment ) 42".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 2);
    assert_eq!(
        tokens[0].token_type,
        TokenType::Comment(" this is a comment ".to_string())
    );
    assert_eq!(tokens[1].token_type, TokenType::Number(42));
}

#[test]
fn test_tokenize_string_literals() {
    let mut lexer = Lexer::new("\"hello world\" \"test\\nstring\"".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 2);
    assert_eq!(
        tokens[0].token_type,
        TokenType::StringLiteral("hello world".to_string())
    );
    assert_eq!(
        tokens[1].token_type,
        TokenType::StringLiteral("test\nstring".to_string())
    );
}

#[test]
fn test_tokenize_mixed_content() {
    let mut lexer = Lexer::new(": square ( n -- n*n ) DUP * ;".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens[0].token_type, TokenType::StartDefinition);
    assert_eq!(tokens[1].token_type, TokenType::Word("SQUARE".to_string()));
    assert_eq!(
        tokens[2].token_type,
        TokenType::Comment(" n -- n*n ".to_string())
    );
    assert_eq!(tokens[3].token_type, TokenType::Word("DUP".to_string()));
    assert_eq!(tokens[4].token_type, TokenType::Word("*".to_string()));
    assert_eq!(tokens[5].token_type, TokenType::EndDefinition);
}

#[test]
fn test_position_tracking() {
    let mut lexer = Lexer::new("42\n  DUP".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].position.line, 1);
    assert_eq!(tokens[0].position.column, 1);
    assert_eq!(tokens[1].position.line, 2);
    assert_eq!(tokens[1].position.column, 3);
}

#[test]
fn test_empty_input() {
    let mut lexer = Lexer::new("".to_string());
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_whitespace_only() {
    let mut lexer = Lexer::new("   \n\t  ".to_string());
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_invalid_number() {
    let mut lexer = Lexer::new("999999999999999999999".to_string());
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_unclosed_comment() {
    let mut lexer = Lexer::new("( unclosed comment".to_string());
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].token_type,
        TokenType::Comment(" unclosed comment".to_string())
    );
}

#[test]
fn test_unclosed_string() {
    let mut lexer = Lexer::new("\"unclosed string".to_string());
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].token_type,
        TokenType::StringLiteral("unclosed string".to_string())
    );
}

#[test]
fn test_escape_sequences() {
    let mut lexer = Lexer::new("\"line1\\nline2\\ttab\\\\backslash\\\"quote\"".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].token_type,
        TokenType::StringLiteral("line1\nline2\ttab\\backslash\"quote".to_string())
    );
}

#[test]
fn test_negative_numbers() {
    let mut lexer = Lexer::new("-42 - -0".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].token_type, TokenType::Number(-42));
    assert_eq!(tokens[1].token_type, TokenType::Word("-".to_string()));
    assert_eq!(tokens[2].token_type, TokenType::Number(0));
}

#[test]
fn test_special_characters_in_words() {
    let mut lexer = Lexer::new("+ - * / = < > ! @ # $ % ^ & | ~ ?".to_string());
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 17);
    for token in tokens {
        match token.token_type {
            TokenType::Word(_) => {}
            _ => panic!("Expected word token"),
        }
    }
}
