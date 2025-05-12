use std::collections::HashMap;
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone)]
enum ForthWord {
    Native(fn(&mut ForthInterpreter) -> Result<(), String>),
    UserDefined(Vec<String>),
}

struct ForthInterpreter {
    stack: Vec<i32>,
    dictionary: HashMap<String, ForthWord>,
    is_compiling: bool,
    current_definition: Vec<String>,
    current_word: Option<String>,
}

impl ForthInterpreter {
    fn new() -> Self {
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

    fn interpret(&mut self, input: &str) -> Result<(), String> {
        let tokens: Vec<&str> = input.split_whitespace().collect();

        for token in tokens {
            let token_upper = token.to_uppercase();

            // Handle word definition
            if self.is_compiling {
                if token == ":" {
                    return Err("Cannot nest word definitions".to_string());
                } else if token == ";" {
                    if let Err(e) = self.execute_word(";") {
                        return Err(e);
                    }
                } else if self.current_word.is_none() {
                    self.current_word = Some(token_upper.clone());
                } else {
                    self.current_definition.push(token_upper.clone());
                }
                continue;
            }

            // Try to parse as a number
            if let Ok(number) = token.parse::<i32>() {
                self.stack.push(number);
                continue;
            }

            // Execute word
            if let Err(e) = self.execute_word(&token_upper) {
                return Err(e);
            }
        }

        Ok(())
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
}

fn main() {
    let mut interpreter = ForthInterpreter::new();
    let stdin = io::stdin();

    println!("Forth Interpreter in Rust");
    println!("Type 'bye' to exit");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();

        let line = line.trim();
        if line.eq_ignore_ascii_case("bye") {
            break;
        }

        match interpreter.interpret(line) {
            Ok(_) => {}
            Err(e) => println!("Error: {}", e),
        }
    }
}
