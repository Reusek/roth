use roth::lexer::Lexer;
use roth::parser::Parser;
use roth::types::AstNode;

fn parse_input(input: &str) -> Result<AstNode, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    Ok(parser.parse()?)
}

#[test]
fn test_parse_numbers() {
    let ast = parse_input("42 -17 0").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 3);
            match &nodes[0] {
                AstNode::Number(42, _) => {}
                _ => panic!("Expected number 42"),
            }
            match &nodes[1] {
                AstNode::Number(-17, _) => {}
                _ => panic!("Expected number -17"),
            }
            match &nodes[2] {
                AstNode::Number(0, _) => {}
                _ => panic!("Expected number 0"),
            }
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_words() {
    let ast = parse_input("DUP SWAP +").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 3);
            match &nodes[0] {
                AstNode::Word(word, _) => assert_eq!(word, "DUP"),
                _ => panic!("Expected word DUP"),
            }
            match &nodes[1] {
                AstNode::Word(word, _) => assert_eq!(word, "SWAP"),
                _ => panic!("Expected word SWAP"),
            }
            match &nodes[2] {
                AstNode::Word(word, _) => assert_eq!(word, "+"),
                _ => panic!("Expected word +"),
            }
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_simple_definition() {
    let ast = parse_input(": SQUARE DUP * ;").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                AstNode::Definition { name, body, .. } => {
                    assert_eq!(name, "SQUARE");
                    assert_eq!(body.len(), 2);
                    match &body[0] {
                        AstNode::Word(word, _) => assert_eq!(word, "DUP"),
                        _ => panic!("Expected word DUP"),
                    }
                    match &body[1] {
                        AstNode::Word(word, _) => assert_eq!(word, "*"),
                        _ => panic!("Expected word *"),
                    }
                }
                _ => panic!("Expected definition node"),
            }
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_definition_with_numbers() {
    let ast = parse_input(": ADD2 2 + ;").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                AstNode::Definition { name, body, .. } => {
                    assert_eq!(name, "ADD2");
                    assert_eq!(body.len(), 2);
                    match &body[0] {
                        AstNode::Number(2, _) => {}
                        _ => panic!("Expected number 2"),
                    }
                    match &body[1] {
                        AstNode::Word(word, _) => assert_eq!(word, "+"),
                        _ => panic!("Expected word +"),
                    }
                }
                _ => panic!("Expected definition node"),
            }
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_mixed_program() {
    let ast = parse_input("42 DUP : DOUBLE 2 * ; 5 DOUBLE").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 4);

            match &nodes[0] {
                AstNode::Number(42, _) => {}
                _ => panic!("Expected number 42"),
            }

            match &nodes[1] {
                AstNode::Word(word, _) => assert_eq!(word, "DUP"),
                _ => panic!("Expected word DUP"),
            }

            match &nodes[2] {
                AstNode::Definition { name, body, .. } => {
                    assert_eq!(name, "DOUBLE");
                    assert_eq!(body.len(), 2);
                }
                _ => panic!("Expected definition node"),
            }

            match &nodes[3] {
                AstNode::Number(5, _) => {}
                _ => panic!("Expected number 5"),
            }
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_with_comments() {
    let ast = parse_input("( comment ) 42 : TEST ( another comment ) DUP ;").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 2);

            match &nodes[0] {
                AstNode::Number(42, _) => {}
                _ => panic!("Expected number 42"),
            }

            match &nodes[1] {
                AstNode::Definition { name, body, .. } => {
                    assert_eq!(name, "TEST");
                    assert_eq!(body.len(), 1);
                    match &body[0] {
                        AstNode::Word(word, _) => assert_eq!(word, "DUP"),
                        _ => panic!("Expected word DUP"),
                    }
                }
                _ => panic!("Expected definition node"),
            }
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_empty_definition() {
    let ast = parse_input(": EMPTY ;").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                AstNode::Definition { name, body, .. } => {
                    assert_eq!(name, "EMPTY");
                    assert_eq!(body.len(), 0);
                }
                _ => panic!("Expected definition node"),
            }
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_empty_program() {
    let ast = parse_input("").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 0);
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_comments_only() {
    let ast = parse_input("( comment 1 ) ( comment 2 )").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 0);
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_error_missing_definition_name() {
    let result = parse_input(":");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_unclosed_definition() {
    let result = parse_input(": TEST DUP");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_nested_definitions() {
    let result = parse_input(": OUTER : INNER ; ;");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_invalid_definition_name() {
    let result = parse_input(": 42 DUP ;");
    assert!(result.is_err());
}

#[test]
fn test_parse_complex_definition() {
    let ast = parse_input(": FACTORIAL DUP 1 - DUP 0 > IF FACTORIAL * THEN ;").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                AstNode::Definition { name, body, .. } => {
                    assert_eq!(name, "FACTORIAL");
                    assert!(body.len() > 5);
                }
                _ => panic!("Expected definition node"),
            }
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_multiple_definitions() {
    let ast = parse_input(": DOUBLE 2 * ; : TRIPLE 3 * ; : QUADRUPLE 4 * ;").unwrap();

    match ast {
        AstNode::Program(nodes) => {
            assert_eq!(nodes.len(), 3);
            for (i, expected_name) in ["DOUBLE", "TRIPLE", "QUADRUPLE"].iter().enumerate() {
                match &nodes[i] {
                    AstNode::Definition { name, .. } => {
                        assert_eq!(name, expected_name);
                    }
                    _ => panic!("Expected definition node"),
                }
            }
        }
        _ => panic!("Expected program node"),
    }
}
