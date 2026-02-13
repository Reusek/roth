//! Roth Forth Runtime Library
//!
//! This crate provides the runtime support for the Roth Forth REPL.
//! It includes:
//!
//! - `RuntimeContext`: The execution context passed to all compiled words
//! - `ForthError`: Runtime error types with source location tracking
//! - Builtin operation implementations
//!
//! # Example
//!
//! ```rust
//! use roth_runtime::{RuntimeContext, ForthResult};
//!
//! fn example(ctx: &mut RuntimeContext) -> ForthResult<()> {
//!     ctx.push(5)?;
//!     ctx.dup()?;
//!     ctx.mul()?;
//!     ctx.print_top()?;
//!     Ok(())
//! }
//! ```

pub mod builtins;
pub mod context;
pub mod error;

// Re-export main types at crate root
pub use context::{RuntimeContext, WordFn, DEFAULT_MAX_STACK_SIZE};
pub use error::{ForthError, ForthResult, Position, SourceLocation};
