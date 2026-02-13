//! REPL state management.
//!
//! Contains the runtime and compiler contexts that persist between REPL inputs.

use crate::ir::IRFunction;
use roth_runtime::RuntimeContext;
use std::collections::{HashMap, HashSet};

/// Compiler context containing compilation state for optimization across inputs.
#[derive(Debug, Default)]
pub struct CompilerContext {
    /// Accumulated word definitions (IR form for optimization).
    pub definitions: HashMap<String, IRFunction>,

    /// Declared variables.
    pub variables: HashSet<String>,

    /// Counter for generating unique library names.
    pub lib_counter: usize,
}

impl CompilerContext {
    /// Create a new compiler context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the next unique library ID and increment the counter.
    pub fn next_lib_id(&mut self) -> usize {
        let id = self.lib_counter;
        self.lib_counter += 1;
        id
    }

    /// Check if a word is defined.
    pub fn has_word(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }

    /// Check if a variable is declared.
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains(name)
    }
}

/// Complete REPL state containing both runtime and compiler contexts.
pub struct REPLState {
    /// Runtime context (execution state).
    pub runtime_ctx: RuntimeContext,

    /// Compiler context (compilation state).
    pub compiler_ctx: CompilerContext,
}

impl REPLState {
    /// Create a new REPL state.
    pub fn new() -> Self {
        Self {
            runtime_ctx: RuntimeContext::new(),
            compiler_ctx: CompilerContext::new(),
        }
    }
}

impl Default for REPLState {
    fn default() -> Self {
        Self::new()
    }
}
