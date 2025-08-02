# Roth Forth Interpreter - Test Suite

This directory contains comprehensive test files for the Roth Forth interpreter.

## Test Structure

### Unit Tests (`tests/` directory)

- **`lexer_tests.rs`** - Tests for tokenization
  - Number parsing (positive, negative, zero)
  - Word tokenization and case conversion
  - Definition markers (`:` and `;`)
  - Comments and string literals
  - Position tracking and error handling

- **`parser_tests.rs`** - Tests for AST generation
  - Simple expressions (numbers, words)
  - Word definitions and nested structures
  - Comment handling and mixed programs
  - Error cases (malformed definitions, etc.)

- **`analyzer_tests.rs`** - Tests for semantic analysis
  - Builtin word validation
  - User-defined word checking
  - Definition order and scope
  - Error detection (undefined words, builtin redefinition)

- **`ir_tests.rs`** - Tests for Intermediate Representation
  - IR program structure
  - Stack operations and arithmetic
  - Function calls and control flow
  - Value types and stack effects

- **`codegen_tests.rs`** - Tests for code generation
  - Rust code generation from IR
  - Function definitions and calls
  - Stack operations and I/O
  - Optimization integration

- **`integration_tests.rs`** - End-to-end compilation tests
  - Full compilation pipeline
  - File I/O and command-line interface
  - Different backend targets
  - Error handling and debug output

### Test Sample Programs (`test_samples/` directory)

- **`basic_arithmetic.rt`** - Simple arithmetic operations
- **`stack_operations.rt`** - Stack manipulation commands
- **`simple_definitions.rt`** - Basic word definitions
- **`nested_definitions.rt`** - Complex word interactions
- **`comments_test.rt`** - Comment handling edge cases
- **`comprehensive_test.rt`** - Full feature demonstration

## Running Tests

### Unit Tests
```bash
cargo test
```

### Specific Test Module
```bash
cargo test lexer_tests
cargo test parser_tests
cargo test analyzer_tests
cargo test ir_tests
cargo test codegen_tests
cargo test integration_tests
```

### Integration Tests with Sample Files
```bash
# Test basic arithmetic
cargo run test_samples/basic_arithmetic.rt --output test_output.rs

# Test with debug output
cargo run test_samples/comprehensive_test.rt --debug 2 --output debug_test.rs

# Test different backends
cargo run test_samples/simple_definitions.rt --backend rust-ir --output rust_test.rs
cargo run test_samples/simple_definitions.rt --backend c-ir --output c_test.rs
```

## Test Coverage

The test suite covers:

1. **Lexical Analysis**
   - All token types (numbers, words, definitions, comments, strings)
   - Edge cases (empty input, whitespace, invalid tokens)
   - Position tracking and error reporting

2. **Parsing**
   - AST construction for all node types
   - Nested structures and definitions
   - Error recovery and reporting

3. **Semantic Analysis**
   - Word resolution and scope checking
   - Builtin word protection
   - Definition validation

4. **IR Generation**
   - Lowering from AST to IR
   - Stack effect calculation
   - Function call resolution

5. **Code Generation**
   - Rust code output
   - Function generation
   - Stack operation implementation

6. **Integration**
   - Full compilation pipeline
   - File handling and CLI interface
   - Error propagation

## Adding New Tests

When adding new features to the Roth compiler:

1. Add unit tests to the appropriate `*_tests.rs` file
2. Create sample programs in `test_samples/` to demonstrate the feature
3. Add integration tests if the feature affects the compilation pipeline
4. Update this README with any new test categories

## Test Framework Integration

The tests are designed to work with any Rust testing framework. The current structure uses:

- Standard Rust `#[test]` attributes
- `assert!` macros for validation
- `Result<T, E>` for error handling tests
- File I/O for integration testing

To integrate with a custom testing framework, modify the test attributes and assertion methods as needed while preserving the test logic and coverage.