//! REPL (Read-Eval-Print-Loop) module for the Roth Forth compiler.
//!
//! This module provides a JIT-style REPL that compiles each input to native
//! code via Rust, dynamically loads it, and executes while preserving state.

pub mod codegen;
pub mod loader;
pub mod state;

use crate::analyzer::SemanticAnalyzer;
use crate::ir_lowering::IRLowering;
use crate::ir_optimizer::IROptimizer;
use crate::lexer::Lexer;
use crate::parser::Parser;
use colored::Colorize;
use roth_runtime::RuntimeContext;
use std::io::{self, BufRead, Write};

use self::codegen::ReplCodegen;
use self::loader::LibraryLoader;
use self::state::{CompilerContext, REPLState};

/// Configuration for the REPL.
#[derive(Debug, Clone)]
pub struct ReplConfig {
    /// Debug level (0=off, 1=timing, 2=verbose, 3=show code)
    pub debug: u8,
    /// Whether to show the welcome message
    pub show_welcome: bool,
    /// Prompt string
    pub prompt: String,
    /// Continue prompt (for multi-line input)
    pub continue_prompt: String,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            debug: 0,
            show_welcome: true,
            prompt: "roth> ".to_string(),
            continue_prompt: "  ... ".to_string(),
        }
    }
}

/// Main REPL structure.
pub struct Repl {
    state: REPLState,
    codegen: ReplCodegen,
    loader: LibraryLoader,
    config: ReplConfig,
    analyzer: SemanticAnalyzer,
}

impl Repl {
    /// Create a new REPL instance.
    pub fn new(config: ReplConfig) -> io::Result<Self> {
        let loader = LibraryLoader::new()?;

        Ok(Self {
            state: REPLState::new(),
            codegen: ReplCodegen::new(),
            loader,
            config,
            analyzer: SemanticAnalyzer::new(),
        })
    }

    /// Run the REPL loop.
    pub fn run(&mut self) -> io::Result<()> {
        if self.config.show_welcome {
            self.print_welcome();
        }

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            // Print prompt
            print!("{}", self.config.prompt);
            stdout.flush()?;

            // Read input
            let mut input = String::new();
            let bytes_read = stdin.lock().read_line(&mut input)?;

            // Check for EOF
            if bytes_read == 0 {
                println!();
                break;
            }

            let input = input.trim();

            // Handle special commands
            if input.is_empty() {
                continue;
            }

            // Handle special commands (: followed by a non-space letter)
            // Forth definitions like ": SQUARE" have a space after the colon
            if input.starts_with(':') && !input.starts_with(": ") && input.len() > 1 {
                if self.handle_command(input) {
                    break;
                }
                continue;
            }

            // Process Forth input
            match self.process_input(input) {
                Ok(()) => {
                    println!("{}", " ok".green());
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                }
            }
        }

        Ok(())
    }

    /// Process a single line of Forth input.
    pub fn process_input(&mut self, input: &str) -> Result<(), String> {
        // Step 1: Lexing
        if self.config.debug >= 2 {
            println!("{}  {}", "Input:".cyan(), input);
        }

        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer
            .tokenize()
            .map_err(|e| format!("Lexer error: {}", e))?;

        if self.config.debug >= 2 {
            println!("{}  {:?}", "Tokens:".cyan(), tokens);
        }

        // Step 2: Parsing
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| format!("Parser error: {}", e))?;

        if self.config.debug >= 2 {
            println!("{}  {:?}", "AST:".cyan(), ast);
        }

        // Step 3: Semantic Analysis with REPL context
        // Create a temporary analyzer that includes our existing definitions
        let mut analyzer = SemanticAnalyzer::new();

        // Add all known user-defined words from compiler context
        for name in self.state.compiler_ctx.definitions.keys() {
            analyzer.add_user_word(name.clone());
        }

        // Add all known variables
        for name in &self.state.compiler_ctx.variables {
            analyzer.add_variable(name.clone());
        }

        analyzer
            .analyze(&ast)
            .map_err(|e| format!("Semantic error: {}", e))?;

        // Step 4: IR Lowering
        let mut ir_lowering = IRLowering::new();

        // Add all known user-defined words from previous REPL entries
        for name in self.state.compiler_ctx.definitions.keys() {
            ir_lowering.add_known_word(name.clone());
        }

        // Add all known variables from previous REPL entries
        for name in &self.state.compiler_ctx.variables {
            ir_lowering.add_known_variable(name.clone());
        }

        let mut ir = ir_lowering.lower(&ast);

        if self.config.debug >= 2 {
            println!("{}  {}", "IR:".cyan(), ir);
        }

        // Step 5: Optimization
        let mut optimizer = IROptimizer::new();
        let _stats = optimizer.optimize(&mut ir);

        if self.config.debug >= 2 {
            println!("{}  {}", "Optimized IR:".cyan(), ir);
        }

        // Step 6: Code Generation for REPL
        let (rust_code, defined_words) = self.codegen.generate(&ir, &self.state.compiler_ctx);

        if self.config.debug >= 3 {
            println!("{}:\n{}", "Generated Rust".cyan(), rust_code);
        }

        // Step 7: Compile and Load
        let entry_fn = self
            .loader
            .compile_and_load(&rust_code, self.config.debug)?;

        // Step 8: Execute
        let result = entry_fn(&mut self.state.runtime_ctx);

        // Step 9: Handle result and update state
        // Note: Words are now registered by __repl_entry itself during execution
        match result {
            Ok(()) => {
                // Store IR in compiler context for future optimization
                for word_name in &defined_words {
                    if let Some(ir_func) = ir.functions.get(word_name) {
                        self.state
                            .compiler_ctx
                            .definitions
                            .insert(word_name.clone(), ir_func.clone());
                    }
                }

                // Track new variables
                if let crate::types::AstNode::Program(nodes) = &ast {
                    for node in nodes {
                        if let crate::types::AstNode::VariableDeclaration { name, .. } = node {
                            self.state.compiler_ctx.variables.insert(name.clone());
                            self.state.runtime_ctx.declare_variable(name.clone());
                        }
                    }
                }

                Ok(())
            }
            Err(e) => Err(format!("Runtime error: {}", e)),
        }
    }

    /// Handle REPL commands (starting with ':').
    fn handle_command(&mut self, input: &str) -> bool {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.first().map(|s| *s).unwrap_or("");

        match cmd {
            ":quit" | ":q" | ":exit" => {
                println!("Goodbye!");
                return true;
            }
            ":help" | ":h" | ":?" => {
                self.print_help();
            }
            ":stack" | ":s" => {
                self.print_stack();
            }
            ":words" | ":w" => {
                self.print_words();
            }
            ":vars" | ":v" => {
                self.print_variables();
            }
            ":clear" | ":c" => {
                self.state.runtime_ctx.stack.clear();
                println!("Stack cleared.");
            }
            ":reset" | ":r" => {
                self.state = REPLState::new();
                println!("State reset.");
            }
            ":debug" => {
                if let Some(level) = parts.get(1) {
                    if let Ok(n) = level.parse::<u8>() {
                        self.config.debug = n;
                        println!("Debug level set to {}", n);
                    } else {
                        println!("Invalid debug level. Use 0-3.");
                    }
                } else {
                    println!("Current debug level: {}", self.config.debug);
                }
            }
            _ => {
                println!(
                    "Unknown command: {}. Type :help for available commands.",
                    cmd
                );
            }
        }

        false
    }

    fn print_welcome(&self) {
        println!(
            "{}",
            "╔═══════════════════════════════════════╗".bright_blue()
        );
        println!(
            "{}",
            "║     Roth Forth REPL v0.1.0            ║".bright_blue()
        );
        println!(
            "{}",
            "║     Type :help for commands           ║".bright_blue()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════╝".bright_blue()
        );
        println!();
    }

    fn print_help(&self) {
        println!("{}", "REPL Commands:".bold());
        println!("  {:12} - Show this help", ":help, :h, :?");
        println!("  {:12} - Exit the REPL", ":quit, :q");
        println!("  {:12} - Show the stack", ":stack, :s");
        println!("  {:12} - List defined words", ":words, :w");
        println!("  {:12} - List variables", ":vars, :v");
        println!("  {:12} - Clear the stack", ":clear, :c");
        println!("  {:12} - Reset all state", ":reset, :r");
        println!("  {:12} - Set/show debug level (0-3)", ":debug [N]");
        println!();
        println!("{}", "Forth Examples:".bold());
        println!("  5 3 + .           - Add 5 and 3, print result");
        println!("  : SQUARE DUP * ;  - Define a word");
        println!("  VARIABLE X        - Create a variable");
        println!("  10 X !            - Store 10 in X");
        println!("  X @               - Fetch value from X");
    }

    fn print_stack(&self) {
        let stack = &self.state.runtime_ctx.stack;
        if stack.is_empty() {
            println!("Stack is empty");
        } else {
            print!("<{}> ", stack.len());
            for val in stack {
                print!("{} ", val);
            }
            println!();
        }
    }

    fn print_words(&self) {
        let words: Vec<_> = self.state.compiler_ctx.definitions.keys().collect();
        if words.is_empty() {
            println!("No user-defined words");
        } else {
            println!("{}", "User-defined words:".bold());
            for word in words {
                println!("  {}", word);
            }
        }
    }

    fn print_variables(&self) {
        if self.state.compiler_ctx.variables.is_empty() {
            println!("No variables defined");
        } else {
            println!("{}", "Variables:".bold());
            for var in &self.state.compiler_ctx.variables {
                let value = self.state.runtime_ctx.memory.get(var).copied().unwrap_or(0);
                println!("  {} = {}", var, value);
            }
        }
    }
}
