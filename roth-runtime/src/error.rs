//! Error types for the Roth Forth runtime.

use std::fmt;

/// Source location information for error reporting.
#[derive(Debug, Clone, Default)]
pub struct SourceLocation {
    /// Word being executed (if applicable)
    pub word: Option<String>,

    /// Original source position (if available)
    pub position: Option<Position>,
}

/// Position in source code.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_word(word: impl Into<String>) -> Self {
        Self {
            word: Some(word.into()),
            position: None,
        }
    }

    pub fn with_position(line: usize, column: usize) -> Self {
        Self {
            word: None,
            position: Some(Position { line, column }),
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.word, &self.position) {
            (Some(word), Some(pos)) => {
                write!(f, "in word '{}' at {}:{}", word, pos.line, pos.column)
            }
            (Some(word), None) => write!(f, "in word '{}'", word),
            (None, Some(pos)) => write!(f, "at {}:{}", pos.line, pos.column),
            (None, None) => write!(f, "at unknown location"),
        }
    }
}

/// Runtime errors that can occur during Forth execution.
#[derive(Debug, Clone)]
pub enum ForthError {
    /// Attempted to pop from an empty stack.
    StackUnderflow { location: SourceLocation },

    /// Stack exceeded maximum allowed size.
    StackOverflow {
        location: SourceLocation,
        max_size: usize,
    },

    /// Attempted to pop from an empty return stack.
    ReturnStackUnderflow { location: SourceLocation },

    /// Division by zero.
    DivisionByZero { location: SourceLocation },

    /// Referenced word is not defined.
    UndefinedWord {
        name: String,
        location: SourceLocation,
    },

    /// Invalid memory access (variable not found).
    InvalidMemoryAccess {
        variable: String,
        location: SourceLocation,
    },

    /// I/O operation failed.
    IOError {
        message: String,
        location: SourceLocation,
    },

    /// Generic runtime error.
    RuntimeError {
        message: String,
        location: SourceLocation,
    },
}

impl fmt::Display for ForthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForthError::StackUnderflow { location } => {
                write!(f, "Stack underflow {}", location)
            }
            ForthError::StackOverflow { location, max_size } => {
                write!(f, "Stack overflow (max {} elements) {}", max_size, location)
            }
            ForthError::ReturnStackUnderflow { location } => {
                write!(f, "Return stack underflow {}", location)
            }
            ForthError::DivisionByZero { location } => {
                write!(f, "Division by zero {}", location)
            }
            ForthError::UndefinedWord { name, location } => {
                write!(f, "Undefined word '{}' {}", name, location)
            }
            ForthError::InvalidMemoryAccess { variable, location } => {
                write!(
                    f,
                    "Invalid memory access: variable '{}' not found {}",
                    variable, location
                )
            }
            ForthError::IOError { message, location } => {
                write!(f, "I/O error: {} {}", message, location)
            }
            ForthError::RuntimeError { message, location } => {
                write!(f, "Runtime error: {} {}", message, location)
            }
        }
    }
}

impl std::error::Error for ForthError {}

/// Result type alias for Forth operations.
pub type ForthResult<T> = Result<T, ForthError>;
