use std::collections::HashMap;
use std::io::{self, Write};
use crate::types::{AstNode};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::analyzer::SemanticAnalyzer;
use crate::codegen::{Backend, create_generator};

#[derive(Debug, Clone)]
pub enum ForthWord {
    Native(fn(&mut ForthInterpreter) -> Result<(), String>),
    UserDefined(Vec<String>),
}

pub struct ForthInterpreter {
    stack: Vec<i32>,
    dictionary: HashMap<String, ForthWord>,
    is_compiling: bool,
    current_definition: Vec<String>,
    current_word: Option<String>,
}

impl ForthInterpreter {
    pub fn new() -> Self {
        let mut interpreter = ForthInterpreter {
            stack: Vec::new(),
            dictionary: HashMap::new(),
            is_compiling: false,
            current_definition: Vec::new(),
            current_word: None,
        };

        // Define native words
        interpreter.define_native_words();
        interpreter
    }

    fn define_native_words(&mut self) {
        // Arithmetic operations
        self.dictionary.insert(
            "+".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.len() < 2 {
                    return Err("Stack underflow".to_string());
                }
                let b = interp.stack.pop().unwrap();
                let a = interp.stack.pop().unwrap();
                interp.stack.push(a + b);
                Ok(())
            }),
        );

        self.dictionary.insert(
            "-".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.len() < 2 {
                    return Err("Stack underflow".to_string());
                }
                let b = interp.stack.pop().unwrap();
                let a = interp.stack.pop().unwrap();
                interp.stack.push(a - b);
                Ok(())
            }),
        );

        self.dictionary.insert(
            "*".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.len() < 2 {
                    return Err("Stack underflow".to_string());
                }
                let b = interp.stack.pop().unwrap();
                let a = interp.stack.pop().unwrap();
                interp.stack.push(a * b);
                Ok(())
            }),
        );

        self.dictionary.insert(
            "/".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.len() < 2 {
                    return Err("Stack underflow".to_string());
                }
                let b = interp.stack.pop().unwrap();
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                let a = interp.stack.pop().unwrap();
                interp.stack.push(a / b);
                Ok(())
            }),
        );

        // Stack manipulation
        self.dictionary.insert(
            "DUP".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.is_empty() {
                    return Err("Stack underflow".to_string());
                }
                let a = *interp.stack.last().unwrap();
                interp.stack.push(a);
                Ok(())
            }),
        );

        self.dictionary.insert(
            "DROP".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.is_empty() {
                    return Err("Stack underflow".to_string());
                }
                interp.stack.pop();
                Ok(())
            }),
        );

        self.dictionary.insert(
            "SWAP".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.len() < 2 {
                    return Err("Stack underflow".to_string());
                }
                let len = interp.stack.len();
                interp.stack.swap(len - 1, len - 2);
                Ok(())
            }),
        );

        self.dictionary.insert(
            "OVER".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.len() < 2 {
                    return Err("Stack underflow".to_string());
                }
                let len = interp.stack.len();
                let value = interp.stack[len - 2];
                interp.stack.push(value);
                Ok(())
            }),
        );

        // Word definition
        self.dictionary.insert(
            ":".to_string(),
            ForthWord::Native(|interp| {
                interp.is_compiling = true;
                interp.current_definition.clear();
                Ok(())
            }),
        );

        self.dictionary.insert(
            ";".to_string(),
            ForthWord::Native(|interp| {
                if !interp.is_compiling {
                    return Err("Not in compilation mode".to_string());
                }

                if let Some(word_name) = &interp.current_word {
                    let definition = interp.current_definition.clone();
                    interp
                        .dictionary
                        .insert(word_name.clone(), ForthWord::UserDefined(definition));
                    interp.is_compiling = false;
                    interp.current_word = None;
                } else {
                    return Err("No word name provided".to_string());
                }

                Ok(())
            }),
        );

        // Output
        self.dictionary.insert(
            ".".to_string(),
            ForthWord::Native(|interp| {
                if interp.stack.is_empty() {
                    return Err("Stack underflow".to_string());
                }
                let value = interp.stack.pop().unwrap();
                print!("{} ", value);
                io::stdout().flush().unwrap();
                Ok(())
            }),
        );

        self.dictionary.insert(
            ".S".to_string(),
            ForthWord::Native(|interp| {
                print!("<{}> ", interp.stack.len());
                for value in &interp.stack {
                    print!("{} ", value);
                }
                println!();
                Ok(())
            }),
        );

        // Control flow
        self.dictionary.insert(
            "CR".to_string(),
            ForthWord::Native(|_| {
                println!();
                Ok(())
            }),
        );
    }

    pub fn interpret(&mut self, input: &str) -> Result<(), String> {
        // Use new parsing system
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| e.to_string())?;
        
        // Optional: Run semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        if let Err(e) = analyzer.analyze(&ast) {
            println!("Warning: {}", e);
        }
        
        // Execute the AST
        self.execute_ast(&ast)
    }

    fn execute_ast(&mut self, node: &AstNode) -> Result<(), String> {
        match node {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.execute_ast(node)?;
                }
            },
            AstNode::Number(n, _) => {
                self.stack.push(*n);
            },
            AstNode::Word(name, _) => {
                self.execute_word(name)?;
            },
            AstNode::Definition { name, body, .. } => {
                let mut definition = Vec::new();
                for node in body {
                    match node {
                        AstNode::Word(w, _) => definition.push(w.clone()),
                        AstNode::Number(n, _) => definition.push(n.to_string()),
                        _ => return Err("Invalid definition body".to_string()),
                    }
                }
                self.dictionary.insert(name.clone(), ForthWord::UserDefined(definition));
            },
        }
        Ok(())
    }

    pub fn generate_code(&self, input: &str, backend: Backend) -> Result<String, String> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| e.to_string())?;
        
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast).map_err(|e| e.to_string())?;
        
        let mut generator = create_generator(backend);
        Ok(generator.generate(&ast))
    }

    pub fn compile_code(&self, input: &str, backend: Backend, output_file: Option<&str>) -> Result<String, String> {
        let code = self.generate_code(input, backend.clone())?;
        
        let generator = create_generator(backend);
        let extension = generator.get_file_extension();
        let default_filename = format!("generated.{}", extension);
        let filename = output_file.unwrap_or(&default_filename);
        
        // Write code to file
        std::fs::write(filename, &code).map_err(|e| format!("Failed to write file: {}", e))?;
        
        let compile_cmd = generator.get_compile_command(filename);
        
        Ok(format!("Generated: {}\nCompile with: {}", filename, compile_cmd))
    }

    fn execute_word(&mut self, word: &str) -> Result<(), String> {
        if let Some(forth_word) = self.dictionary.get(word).cloned() {
            match forth_word {
                ForthWord::Native(func) => func(self),
                ForthWord::UserDefined(definition) => {
                    for word in definition {
                        self.execute_word(&word)?;
                    }
                    Ok(())
                }
            }
        } else {
            Err(format!("Unknown word: {}", word))
        }
    }

    pub fn get_word_completions(&self, prefix: &str) -> Vec<String> {
        let mut completions = Vec::new();
        let prefix_lower = prefix.to_lowercase();
        
        for word in self.dictionary.keys() {
            if word.to_lowercase().starts_with(&prefix_lower) {
                completions.push(word.clone());
            }
        }
        
        // Add REPL commands
        let commands = ["gen", "compile", "parse", "backends", "bye"];
        for cmd in &commands {
            if cmd.to_lowercase().starts_with(&prefix_lower) {
                completions.push(cmd.to_string());
            }
        }
        
        completions.sort();
        completions
    }
}