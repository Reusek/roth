# roth

A simple Forth interpreter written in Rust.

## Features
- Supports basic arithmetic operations: `+`, `-`, `*`, `/`
- Stack manipulation: `DUP`, `DROP`, `SWAP`, `OVER`
- User-defined words (functions) using `:` and `;`
- Output commands: `.`, `.S` (print stack)
- Interactive REPL interface
- Error handling for stack underflow, division by zero, and unknown words

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (edition 2024 or later)

### Build and Run

```sh
cargo run
```

You will see:
```
Forth Interpreter in Rust
Type 'bye' to exit
> 
```

Type Forth commands at the prompt. For example:

```
> 2 3 + .
5 > 4 DUP * .
16 > : SQUARE DUP * ;
> 5 SQUARE .
25 > .S
<0>
> bye
```

## Project Structure
- `src/main.rs`: Main interpreter implementation and REPL
- `Cargo.toml`: Project manifest

## License
MIT 