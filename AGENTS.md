# AGENTS.md - Development Guidelines for Roth Forth Interpreter

## Build/Test Commands
- `cargo build` - Build the project
- `cargo run` - Run the interactive Forth interpreter (type 'bye' to exit)
- `cargo check` - Fast syntax/type checking
- `cargo clippy` - Lint with Clippy
- `cargo fmt` - Format code
- No tests found in codebase - use manual testing via `cargo run`

## Code Style Guidelines
- **Language**: Rust 2024 edition
- **Imports**: Group std imports first, then external crates, then local modules
- **Naming**: snake_case for functions/variables, PascalCase for types/enums
- **Types**: Use explicit types for public APIs, prefer `Result<T, E>` for error handling
- **Error Handling**: Custom error types (e.g., `ParseError`), implement `Display` trait
- **Structs**: Use `pub` fields for data structures, derive `Debug, Clone` as needed
- **Enums**: Use descriptive variants with associated data where appropriate
- **Comments**: Minimal comments, prefer self-documenting code
- **Formatting**: Use `cargo fmt` for consistent formatting

## Project Structure
- `src/main.rs` - CLI entry point with interactive REPL
- `src/types.rs` - Core data structures (Token, AST, Position, errors)
- `src/lexer.rs` - Tokenization logic
- `src/parser.rs` - AST generation
- `src/codegen/` - Code generation backends (Rust, C)
- `src/analyzer.rs` - Semantic analysis
- `src/interpreter.rs` - Forth execution engine