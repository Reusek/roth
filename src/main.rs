mod types;
mod lexer;
mod parser;
mod analyzer;
mod codegen;
mod interpreter;
mod highlighter;


use std::fs;
use std::process;
use crate::interpreter::ForthInterpreter;
use crate::codegen::Backend;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::highlighter::SyntaxHighlighter;
use clap::Parser as ClapParser;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result as RustylineResult};
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context};
use rustyline::Helper;

#[derive(ClapParser, Debug)]
#[command(name = "roth")]
#[command(about = "Enhanced Forth Interpreter in Rust")]
struct Args {
    #[arg(help = "Forth file to execute")]
    file: Option<String>,
    
    #[arg(long, help = "Disable syntax highlighting")]
    no_color: bool,
}

struct ForthCompleter {
    interpreter: *const ForthInterpreter,
}

impl ForthCompleter {
    fn new(interpreter: *const ForthInterpreter) -> Self {
        ForthCompleter { interpreter }
    }
}

impl Helper for ForthCompleter {}

impl Completer for ForthCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        // Find the word being completed
        let words: Vec<&str> = line[..pos].split_whitespace().collect();
        let current_word = if line[..pos].ends_with(' ') {
            ""
        } else {
            words.last().map_or("", |v| v)
        };

        let start = pos - current_word.len();
        
        // Get completions from the interpreter
        let completions = unsafe {
            (*self.interpreter).get_word_completions(current_word)
        };

        let candidates: Vec<Pair> = completions
            .into_iter()
            .map(|completion| Pair {
                display: completion.clone(),
                replacement: completion,
            })
            .collect();

        Ok((start, candidates))
    }
}

impl Hinter for ForthCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for ForthCompleter {}

impl Validator for ForthCompleter {}

fn execute_file(filename: &str, interpreter: &mut ForthInterpreter) {
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match interpreter.interpret(line) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error on line {}: {}", line_num + 1, e);
                process::exit(1);
            }
        }
    }
}

fn run_repl(args: &Args, mut interpreter: ForthInterpreter) {
    let mut highlighter = if args.no_color {
        None
    } else {
        match SyntaxHighlighter::new() {
            Ok(h) => Some(h),
            Err(e) => {
                eprintln!("Failed to create highlighter: {}", e);
                None
            }
        }
    };

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .build();
    
    let mut rl = Editor::with_config(config).unwrap();
    let helper = ForthCompleter::new(&interpreter as *const ForthInterpreter);
    rl.set_helper(Some(helper));

    let history_file = "roth_history.txt";
    let _ = rl.load_history(history_file);

    println!("Enhanced Forth Interpreter in Rust");
    println!("Commands:");
    println!("  'bye' - exit");
    println!("  'gen <backend> <code>' - generate code (backends: rust, c)");
    println!("  'compile <backend> <code>' - generate and save code with compile instructions");
    println!("  'parse <code>' - show parse tree");
    println!("  'backends' - list available backends");
    println!("Use Tab for completion, Up/Down arrows for history");

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line).unwrap();

                if line.eq_ignore_ascii_case("bye") {
                    break;
                }

                if line == "backends" {
                    println!("Available backends:");
                    println!("  rust, rs - Rust code generation");
                    println!("  c, gcc   - C code generation for GCC");
                    continue;
                }

                if line.starts_with("gen ") {
                    let parts: Vec<&str> = line[4..].splitn(2, ' ').collect();
                    if parts.len() != 2 {
                        println!("Usage: gen <backend> <code>");
                        continue;
                    }

                    let backend_str = parts[0];
                    let code = parts[1];

                    match Backend::from_str(backend_str) {
                        Some(backend) => {
                            let is_c_backend = matches!(backend, Backend::C);
                            match interpreter.generate_code(code, backend) {
                                Ok(generated) => {
                                     let output = if is_c_backend && highlighter.is_some() {
                                         match highlighter.as_mut().unwrap().highlight_with_force(&generated, true) {
                                             Ok(highlighted) => highlighted,
                                             Err(_) => generated,
                                         }
                                     } else {
                                         generated
                                     };
                                    println!("Generated code:\n{}", output);
                                },
                                Err(e) => println!("Generation error: {}", e),
                            }
                        },
                        None => println!("Unknown backend: {}. Use 'backends' to see available options.", backend_str),
                    }
                    continue;
                }

                if line.starts_with("compile ") {
                    let parts: Vec<&str> = line[8..].splitn(2, ' ').collect();
                    if parts.len() != 2 {
                        println!("Usage: compile <backend> <code>");
                        continue;
                    }

                    let backend_str = parts[0];
                    let code = parts[1];

                    match Backend::from_str(backend_str) {
                        Some(backend) => {
                            let is_c_backend = matches!(backend, Backend::C);
                            match interpreter.compile_code(code, backend, None) {
                                Ok(result) => {
                                    let output = if is_c_backend && highlighter.is_some() {
                                        if let Some(code_start) = result.find("Generated code:\n") {
                                            let prefix = &result[..code_start + 16];
                                            let code_part = &result[code_start + 16..];
                                             match highlighter.as_mut().unwrap().highlight_with_force(code_part, true) {
                                                 Ok(highlighted) => format!("{}{}", prefix, highlighted),
                                                 Err(_) => result,
                                             }
                                        } else {
                                            result
                                        }
                                    } else {
                                        result
                                    };
                                    println!("{}", output);
                                },
                                Err(e) => println!("Compilation error: {}", e),
                            }
                        },
                        None => println!("Unknown backend: {}. Use 'backends' to see available options.", backend_str),
                    }
                    continue;
                }

                if line.starts_with("parse ") {
                    let code = &line[6..];
                    let mut lexer = Lexer::new(code.to_string());
                    match lexer.tokenize() {
                        Ok(tokens) => {
                            println!("Tokens: {:?}", tokens);
                            let mut parser = Parser::new(tokens);
                            match parser.parse() {
                                Ok(ast) => println!("AST: {:#?}", ast),
                                Err(e) => println!("Parse error: {}", e),
                            }
                        },
                        Err(e) => println!("Lexer error: {}", e),
                    }
                    continue;
                }

                match interpreter.interpret(line) {
                    Ok(_) => {}
                    Err(e) => println!("Error: {}", e),
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(history_file);
}

fn main() {
    let args = Args::parse();
    let mut interpreter = ForthInterpreter::new();

    match &args.file {
        Some(filename) => {
            execute_file(filename, &mut interpreter);
        }
        None => {
            run_repl(&args, interpreter);
        }
    }
}
