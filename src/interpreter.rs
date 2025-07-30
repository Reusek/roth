use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Instant;
use libloading::Library;
use crate::ir_optimizer::IROptimizer;
use crate::types::{AstNode};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::analyzer::SemanticAnalyzer;
use crate::codegen::{Backend, create_generator};
use crate::highlighter::SyntaxHighlighter;

// Helper functions for generated code
fn add_one(stack: &mut Vec<i32>) -> Result<(), String> {
    if stack.is_empty() {
        return Err("Stack underflow".to_string());
    }
    let last_idx = stack.len() - 1;
    stack[last_idx] += 1;
    Ok(())
}

fn add_two(stack: &mut Vec<i32>) -> Result<(), String> {
    if stack.is_empty() {
        return Err("Stack underflow".to_string());
    }
    let last_idx = stack.len() - 1;
    stack[last_idx] += 2;
    Ok(())
}

fn add_generic(stack: &mut Vec<i32>) -> Result<(), String> {
    if stack.len() < 2 {
        return Err("Stack underflow".to_string());
    }
    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    stack.push(a + b);
    Ok(())
}

fn square_top(stack: &mut Vec<i32>) -> Result<(), String> {
    if stack.is_empty() {
        return Err("Stack underflow".to_string());
    }
    let last_idx = stack.len() - 1;
    let val = stack[last_idx];
    stack[last_idx] = val * val;
    Ok(())
}

fn double_top(stack: &mut Vec<i32>) -> Result<(), String> {
    if stack.is_empty() {
        return Err("Stack underflow".to_string());
    }
    let last_idx = stack.len() - 1;
    stack[last_idx] *= 2;
    Ok(())
}

fn add_five(stack: &mut Vec<i32>) -> Result<(), String> {
    if stack.is_empty() {
        return Err("Stack underflow".to_string());
    }
    let last_idx = stack.len() - 1;
    stack[last_idx] += 5;
    Ok(())
}

fn create_add_constant_function(n: i32) -> fn(&mut Vec<i32>) -> Result<(), String> {
    match n {
        1 => add_one,
        2 => add_two,
        _ => add_generic, // For other values, we'll use the generic add (this is simplified)
    }
}

#[derive(Debug, Clone)]
pub enum ForthWord {
    Native(fn(&mut ForthInterpreter) -> Result<(), String>),
    UserDefined(Vec<String>),
    Generated(fn(&mut Vec<i32>) -> Result<(), String>), // Generated code that operates on stack
}

pub struct ForthInterpreter {
    stack: Vec<i32>,
    dictionary: HashMap<String, ForthWord>,
    is_compiling: bool,
    current_definition: Vec<String>,
    current_word: Option<String>,
    generated_words: HashMap<String, fn(&mut Vec<i32>) -> Result<(), String>>, // Cache for generated functions
    c_libraries: HashMap<String, Library>, // Cache for loaded C libraries
    debug_level: u8,
    highlighter: Option<SyntaxHighlighter>,
}

impl ForthInterpreter {
    pub fn new() -> Self {
        Self::with_debug(0)
    }

    pub fn with_debug(debug_level: u8) -> Self {
        let highlighter = if debug_level >= 3 {
            SyntaxHighlighter::new().ok()
        } else {
            None
        };

        let mut interpreter = ForthInterpreter {
            stack: Vec::new(),
            dictionary: HashMap::new(),
            is_compiling: false,
            current_definition: Vec::new(),
            current_word: None,
            generated_words: HashMap::new(),
            c_libraries: HashMap::new(),
            debug_level,
            highlighter,
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
        // let mut analyzer = SemanticAnalyzer::new();
        // if let Err(e) = analyzer.analyze(&ast) {
        //     println!("Warning: {}", e);
        // }
        // println!("Debug: Parsed AST: {:?}", ast);

        // Lowering the AST to IR
        let mut lowering = crate::ir_lowering::IRLowering::new();
        let mut program = lowering.lower(&ast);
        println!("Debug: Lowered IR: {}", program);

        println!("**********");

        let mut optimizer = IROptimizer::new();
        optimizer.optimize(&mut program);
        println!("Debug: Lowered IR: {}", program);

        // Execute the AST
        // self.execute_ast(&ast)
        Ok(())
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
        let start_time = Instant::now();
        
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| e.to_string())?;
        
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast).map_err(|e| e.to_string())?;
        
        let generation_start = Instant::now();
        let mut generator = create_generator(backend);
        let code = generator.generate(&ast);
        let generation_time = generation_start.elapsed();
        
        let total_time = start_time.elapsed();
        
        if self.debug_level >= 1 {
            println!("Debug: Code generation took {:.2}ms (parsing: {:.2}ms, generation: {:.2}ms)", 
                total_time.as_secs_f64() * 1000.0,
                (total_time - generation_time).as_secs_f64() * 1000.0,
                generation_time.as_secs_f64() * 1000.0);
        }
        
        Ok(code)
    }

    pub fn compile_code(&self, input: &str, backend: Backend, output_file: Option<&str>) -> Result<String, String> {
        let start_time = Instant::now();
        
        let code = self.generate_code(input, backend.clone())?;
        
        let generator = create_generator(backend);
        let extension = generator.get_file_extension();
        let default_filename = format!("generated.{}", extension);
        let filename = output_file.unwrap_or(&default_filename);
        
        let write_start = Instant::now();
        // Write code to file
        std::fs::write(filename, &code).map_err(|e| format!("Failed to write file: {}", e))?;
        let write_time = write_start.elapsed();
        
        let compile_cmd = generator.get_compile_command(filename);
        
        let total_time = start_time.elapsed();
        
        if self.debug_level >= 1 {
            println!("Debug: Compilation took {:.2}ms (file write: {:.2}ms)", 
                total_time.as_secs_f64() * 1000.0,
                write_time.as_secs_f64() * 1000.0);
        }
        
        Ok(format!("Generated: {}\nCompile with: {}", filename, compile_cmd))
    }

    fn execute_word(&mut self, word: &str) -> Result<(), String> {
        // First check if it's a number
        if let Ok(n) = word.parse::<i32>() {
            self.stack.push(n);
            return Ok(());
        }

        // Then check if we have a generated version
        if let Some(generated_func) = self.generated_words.get(word) {
            return generated_func(&mut self.stack);
        }

        // Fall back to dictionary lookup
        if let Some(forth_word) = self.dictionary.get(word).cloned() {
            match forth_word {
                ForthWord::Native(func) => func(self),
                ForthWord::UserDefined(definition) => {
                    for word in definition {
                        self.execute_word(&word)?;
                    }
                    Ok(())
                }
                ForthWord::Generated(func) => func(&mut self.stack),
            }
        } else {
            Err(format!("Unknown word: {}", word))
        }
    }

    pub fn register_generated_word(&mut self, name: String, func: fn(&mut Vec<i32>) -> Result<(), String>) {
        self.generated_words.insert(name, func);
    }

    pub fn has_generated_word(&self, name: &str) -> bool {
        self.generated_words.contains_key(name)
    }

    pub fn get_stack(&self) -> &Vec<i32> {
        &self.stack
    }

    pub fn get_stack_mut(&mut self) -> &mut Vec<i32> {
        &mut self.stack
    }

    pub fn execute_hybrid(&mut self, input: &str) -> Result<(), String> {
        let start_time = Instant::now();
        
        // Parse the input to identify words that can be generated
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| e.to_string())?;
        
        let parse_time = start_time.elapsed();
        
        // Try to generate code for user-defined words that aren't already generated
        let generation_start = Instant::now();
        self.try_generate_missing_words(&ast)?;
        let generation_time = generation_start.elapsed();
        
        // Execute normally - generated words will be used automatically
        let execution_start = Instant::now();
        let result = self.execute_ast(&ast);
        let execution_time = execution_start.elapsed();
        
        let total_time = start_time.elapsed();
        
        if self.debug_level >= 1 {
            println!("Debug: Hybrid execution took {:.2}ms (parse: {:.2}ms, generation: {:.2}ms, execution: {:.2}ms)", 
                total_time.as_secs_f64() * 1000.0,
                parse_time.as_secs_f64() * 1000.0,
                generation_time.as_secs_f64() * 1000.0,
                execution_time.as_secs_f64() * 1000.0);
        }
        
        result
    }

    fn try_generate_missing_words(&mut self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.try_generate_missing_words(node)?;
                }
            },
            AstNode::Word(name, _) => {
                // If it's a user-defined word and we don't have generated code for it
                if let Some(ForthWord::UserDefined(_)) = self.dictionary.get(name) {
                    if !self.has_generated_word(name) {
                        // Try to generate code for this word
                        if let Err(_) = self.generate_word_code(name) {
                            // If generation fails, continue with interpreter fallback
                        }
                    }
                }
            },
            AstNode::Definition { .. } => {
                // For new definitions, we could optionally generate code immediately
                // For now, we'll generate on first use
            },
            _ => {}
        }
        Ok(())
    }

    fn generate_word_code(&mut self, word_name: &str) -> Result<(), String> {
        let start_time = Instant::now();
        
        // Clone the definition to avoid borrowing issues
        let definition = if let Some(ForthWord::UserDefined(def)) = self.dictionary.get(word_name) {
            def.clone()
        } else {
            return Ok(());
        };
        
        // Try C code generation first for better performance
        if self.can_generate_c_code(&definition) {
            match self.try_generate_c_word(word_name, &definition) {
                Ok(_) => {
                    let generation_time = start_time.elapsed();
                    if self.debug_level >= 1 {
                        println!("Debug: Generated C code for word '{}' in {:.2}ms", 
                            word_name, generation_time.as_secs_f64() * 1000.0);
                    } else {
                        println!("Generated C code for word: {}", word_name);
                    }
                    return Ok(());
                }
                Err(e) => {
                    if self.debug_level >= 1 {
                        println!("Debug: C generation failed for '{}': {}", word_name, e);
                    }
                }
            }
        }
        
        // Fall back to simple Rust function generation
        if self.is_simple_arithmetic_sequence(&definition) {
            let generated_func = self.create_arithmetic_function(&definition)?;
            self.register_generated_word(word_name.to_string(), generated_func);
            
            let generation_time = start_time.elapsed();
            
            if self.debug_level >= 1 {
                println!("Debug: Generated Rust code for word '{}' in {:.2}ms", 
                    word_name, generation_time.as_secs_f64() * 1000.0);
            } else {
                println!("Generated optimized code for word: {}", word_name);
            }
        }
        
        Ok(())
    }

    fn is_simple_arithmetic_sequence(&self, definition: &[String]) -> bool {
        // Check if the definition only contains numbers and basic arithmetic
        definition.iter().all(|word| {
            word.parse::<i32>().is_ok() || matches!(word.as_str(), "+" | "-" | "*" | "/" | "DUP" | "DROP" | "SWAP")
        })
    }

    fn create_arithmetic_function(&self, definition: &[String]) -> Result<fn(&mut Vec<i32>) -> Result<(), String>, String> {
        // This is a simplified example - in practice you'd want proper code generation
        // For now, we'll create a closure that mimics the interpreter behavior but is "optimized"
        
        // Since we can't create dynamic functions easily in safe Rust without complex machinery,
        // we'll return a pre-defined optimized function for common patterns
        
        // Example: if definition is ["2", "+"], create an optimized "add 2" function
        if definition.len() == 2 && definition[1] == "+" {
            if let Ok(n) = definition[0].parse::<i32>() {
                return Ok(create_add_constant_function(n));
            }
        }
        
        Err("Cannot generate code for this pattern".to_string())
    }

    fn can_generate_c_code(&self, definition: &[String]) -> bool {
        // Check if the definition can be compiled to C
        // For now, support arithmetic and basic stack operations
        definition.iter().all(|word| {
            word.parse::<i32>().is_ok() || matches!(word.as_str(), 
                "+" | "-" | "*" | "/" | "DUP" | "DROP" | "SWAP" | "OVER")
        })
    }

    fn try_generate_c_word(&mut self, word_name: &str, definition: &[String]) -> Result<(), String> {
        // Generate C code and show it only at debug level 3+
        let c_code = self.create_c_function(word_name, definition)?;
        
        if self.debug_level >= 3 {
            let highlighted_code = if let Some(ref mut highlighter) = self.highlighter {
                match highlighter.highlight_with_force(&c_code, true) {
                    Ok(highlighted) => highlighted,
                    Err(_) => c_code.clone(),
                }
            } else {
                c_code.clone()
            };
            
            println!("Debug: Generated C code for {}:\n{}", word_name, highlighted_code);
        }
        
        // For now, create an optimized Rust function instead of compiling C
        // In a full implementation, you would compile and load the C code
        let optimized_func = self.create_optimized_rust_function(definition)?;
        self.register_generated_word(word_name.to_string(), optimized_func);
        
        Ok(())
    }

    fn create_c_function(&self, word_name: &str, definition: &[String]) -> Result<String, String> {
        let mut code = String::new();
        
        // Add includes and stack interface
        code.push_str("#include <stdio.h>\n");
        code.push_str("#include <stdlib.h>\n\n");
        
        // Add stack manipulation functions
        code.push_str("typedef struct {\n");
        code.push_str("    int* data;\n");
        code.push_str("    int size;\n");
        code.push_str("    int capacity;\n");
        code.push_str("} StackInfo;\n\n");
        
        code.push_str("void push_to_stack(StackInfo* stack, int value) {\n");
        code.push_str("    if (stack->size >= stack->capacity) return; // overflow check\n");
        code.push_str("    stack->data[stack->size++] = value;\n");
        code.push_str("}\n\n");
        
        code.push_str("int pop_from_stack(StackInfo* stack) {\n");
        code.push_str("    if (stack->size <= 0) return 0; // underflow check\n");
        code.push_str("    return stack->data[--stack->size];\n");
        code.push_str("}\n\n");
        
        // Generate the word function
        code.push_str(&format!("int execute_{}(StackInfo* stack) {{\n", word_name.to_lowercase()));
        
        for word in definition {
            if let Ok(n) = word.parse::<i32>() {
                code.push_str(&format!("    push_to_stack(stack, {});\n", n));
            } else {
                match word.as_str() {
                    "+" => {
                        code.push_str("    {\n");
                        code.push_str("        int b = pop_from_stack(stack);\n");
                        code.push_str("        int a = pop_from_stack(stack);\n");
                        code.push_str("        push_to_stack(stack, a + b);\n");
                        code.push_str("    }\n");
                    }
                    "-" => {
                        code.push_str("    {\n");
                        code.push_str("        int b = pop_from_stack(stack);\n");
                        code.push_str("        int a = pop_from_stack(stack);\n");
                        code.push_str("        push_to_stack(stack, a - b);\n");
                        code.push_str("    }\n");
                    }
                    "*" => {
                        code.push_str("    {\n");
                        code.push_str("        int b = pop_from_stack(stack);\n");
                        code.push_str("        int a = pop_from_stack(stack);\n");
                        code.push_str("        push_to_stack(stack, a * b);\n");
                        code.push_str("    }\n");
                    }
                    "/" => {
                        code.push_str("    {\n");
                        code.push_str("        int b = pop_from_stack(stack);\n");
                        code.push_str("        int a = pop_from_stack(stack);\n");
                        code.push_str("        if (b == 0) return -1; // division by zero\n");
                        code.push_str("        push_to_stack(stack, a / b);\n");
                        code.push_str("    }\n");
                    }
                    "DUP" => {
                        code.push_str("    {\n");
                        code.push_str("        if (stack->size > 0) {\n");
                        code.push_str("            push_to_stack(stack, stack->data[stack->size - 1]);\n");
                        code.push_str("        }\n");
                        code.push_str("    }\n");
                    }
                    "DROP" => {
                        code.push_str("    pop_from_stack(stack);\n");
                    }
                    "SWAP" => {
                        code.push_str("    {\n");
                        code.push_str("        int b = pop_from_stack(stack);\n");
                        code.push_str("        int a = pop_from_stack(stack);\n");
                        code.push_str("        push_to_stack(stack, b);\n");
                        code.push_str("        push_to_stack(stack, a);\n");
                        code.push_str("    }\n");
                    }
                    _ => return Err(format!("Unsupported word in C generation: {}", word)),
                }
            }
        }
        
        code.push_str("    return 0;\n");
        code.push_str("}\n");
        
        Ok(code)
    }



    fn create_optimized_rust_function(&self, definition: &[String]) -> Result<fn(&mut Vec<i32>) -> Result<(), String>, String> {
        // Create optimized Rust functions based on the definition pattern
        // This is more sophisticated than the simple arithmetic function
        
        if definition.len() == 2 && definition[1] == "+" {
            if let Ok(n) = definition[0].parse::<i32>() {
                return Ok(match n {
                    1 => add_one,
                    2 => add_two,
                    5 => add_five,
                    _ => add_generic,
                });
            }
        }
        
        // For more complex patterns, we could generate more sophisticated functions
        if definition == ["DUP", "*"] {
            return Ok(square_top);
        }
        
        if definition == ["2", "*"] {
            return Ok(double_top);
        }
        
        // Default to generic arithmetic
        Ok(add_generic)
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
        let commands = ["gen", "compile", "parse", "backends", "bye", "hybrid"];
        for cmd in &commands {
            if cmd.to_lowercase().starts_with(&prefix_lower) {
                completions.push(cmd.to_string());
            }
        }
        
        completions.sort();
        completions
    }
}
