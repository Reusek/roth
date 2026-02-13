//! Runtime context for Forth execution.

use crate::error::{ForthError, ForthResult, SourceLocation};
use std::collections::HashMap;

/// Function pointer type for user-defined words.
/// Note: We use regular Rust ABI here since both the runtime and dynamically
/// loaded libraries are compiled with the same Rust compiler.
pub type WordFn = fn(&mut RuntimeContext) -> ForthResult<()>;

/// Maximum stack size to prevent runaway programs.
pub const DEFAULT_MAX_STACK_SIZE: usize = 10_000;

/// Runtime context containing all execution state.
///
/// This struct is passed to all compiled words and contains:
/// - The main data stack
/// - The return stack (for control flow)
/// - Variable storage
/// - Registered user-defined words
#[derive(Default)]
pub struct RuntimeContext {
    /// Main data stack.
    pub stack: Vec<i64>,

    /// Return stack (for control flow, loop indices, etc.).
    pub rstack: Vec<i64>,

    /// Variable storage (name -> value).
    pub memory: HashMap<String, i64>,

    /// Registered user-defined words (name -> function pointer).
    pub words: HashMap<String, WordFn>,

    /// Maximum stack size (0 = unlimited).
    pub max_stack_size: usize,

    /// Current execution location (for error reporting).
    pub current_location: SourceLocation,
}

impl RuntimeContext {
    /// Create a new runtime context with default settings.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            rstack: Vec::new(),
            memory: HashMap::new(),
            words: HashMap::new(),
            max_stack_size: DEFAULT_MAX_STACK_SIZE,
            current_location: SourceLocation::default(),
        }
    }

    /// Create a new runtime context with a custom stack size limit.
    pub fn with_max_stack_size(max_size: usize) -> Self {
        Self {
            max_stack_size: max_size,
            ..Self::new()
        }
    }

    /// Set the current execution location (for error reporting).
    pub fn set_location(&mut self, location: SourceLocation) {
        self.current_location = location;
    }

    /// Set the current word being executed.
    pub fn set_current_word(&mut self, word: impl Into<String>) {
        self.current_location = SourceLocation::with_word(word);
    }

    /// Clear the current location.
    pub fn clear_location(&mut self) {
        self.current_location = SourceLocation::default();
    }

    /// Register a user-defined word.
    pub fn register_word(&mut self, name: impl Into<String>, func: WordFn) {
        self.words.insert(name.into(), func);
    }

    /// Check if a word is defined.
    pub fn has_word(&self, name: &str) -> bool {
        self.words.contains_key(name)
    }

    /// Declare a variable (initialize to 0 if not present).
    pub fn declare_variable(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.memory.entry(name).or_insert(0);
    }

    /// Get stack depth.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Check if stack has at least n elements.
    pub fn has_n(&self, n: usize) -> bool {
        self.stack.len() >= n
    }

    /// Helper to create stack underflow error with current location.
    fn underflow_error(&self) -> ForthError {
        ForthError::StackUnderflow {
            location: self.current_location.clone(),
        }
    }

    /// Helper to check stack size and return overflow error if exceeded.
    fn check_overflow(&self) -> ForthResult<()> {
        if self.max_stack_size > 0 && self.stack.len() >= self.max_stack_size {
            return Err(ForthError::StackOverflow {
                location: self.current_location.clone(),
                max_size: self.max_stack_size,
            });
        }
        Ok(())
    }

    /// Push a value onto the stack.
    pub fn push(&mut self, value: i64) -> ForthResult<()> {
        self.check_overflow()?;
        self.stack.push(value);
        Ok(())
    }

    /// Pop a value from the stack.
    pub fn pop(&mut self) -> ForthResult<i64> {
        self.stack.pop().ok_or_else(|| self.underflow_error())
    }

    /// Peek at the top of the stack without removing it.
    pub fn peek(&self) -> ForthResult<i64> {
        self.stack
            .last()
            .copied()
            .ok_or_else(|| ForthError::StackUnderflow {
                location: self.current_location.clone(),
            })
    }

    /// Peek at the nth element from the top (0 = top).
    pub fn peek_n(&self, n: usize) -> ForthResult<i64> {
        let len = self.stack.len();
        if n >= len {
            return Err(self.underflow_error());
        }
        Ok(self.stack[len - 1 - n])
    }
}
