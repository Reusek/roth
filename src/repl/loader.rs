//! Dynamic library loading for the REPL.
//!
//! Handles compilation of generated Rust code to shared libraries
//! and loading them at runtime.

use libloading::{Library, Symbol};
use roth_runtime::{ForthResult, RuntimeContext, WordFn};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Entry point function signature for REPL libraries.
pub type EntryFn = fn(&mut RuntimeContext) -> ForthResult<()>;

/// Information about a loaded library.
struct LoadedLibrary {
    /// Handle to keep the library alive.
    _lib: Library,

    /// Entry point function.
    entry: EntryFn,

    /// Words defined in this library.
    defined_words: Vec<String>,

    /// Word function pointers.
    word_fns: HashMap<String, WordFn>,
}

/// Manages compilation and loading of REPL libraries.
pub struct LibraryLoader {
    /// Temporary directory for compiled libraries.
    temp_dir: TempDir,

    /// Loaded libraries (kept alive to prevent unloading).
    libraries: Vec<LoadedLibrary>,

    /// Library counter for unique names.
    lib_counter: usize,

    /// Path to roth-runtime crate (for compilation).
    runtime_path: PathBuf,
}

impl LibraryLoader {
    /// Create a new library loader.
    pub fn new() -> io::Result<Self> {
        let temp_dir = TempDir::new()?;

        // Find the roth-runtime crate path
        let manifest_dir = std::env::current_dir()?;
        let runtime_path = manifest_dir.join("roth-runtime");

        Ok(Self {
            temp_dir,
            libraries: Vec::new(),
            lib_counter: 0,
            runtime_path,
        })
    }

    /// Compile Rust code to a shared library and load it.
    pub fn compile_and_load(&mut self, rust_code: &str, debug: u8) -> Result<EntryFn, String> {
        let lib_id = self.lib_counter;
        self.lib_counter += 1;

        let lib_name = format!("repl_lib_{}", lib_id);
        let source_path = self.temp_dir.path().join(format!("{}.rs", lib_name));
        let lib_path = self.temp_dir.path().join(lib_filename(&lib_name));

        // Write source file
        std::fs::write(&source_path, rust_code)
            .map_err(|e| format!("Failed to write source: {}", e))?;

        if debug >= 2 {
            println!("Source written to: {:?}", source_path);
        }

        // Compile to shared library
        self.compile_library(&source_path, &lib_path, debug)?;

        if debug >= 2 {
            println!("Library compiled to: {:?}", lib_path);
        }

        // Load the library
        let lib = unsafe {
            Library::new(&lib_path).map_err(|e| format!("Failed to load library: {}", e))?
        };

        // Get entry point
        let entry: EntryFn = unsafe {
            let sym: Symbol<EntryFn> = lib
                .get(b"__repl_entry")
                .map_err(|e| format!("Failed to get entry symbol: {}", e))?;
            *sym
        };

        // Get defined words
        let defined_words = self.get_defined_words(&lib)?;

        // Get word function pointers
        let mut word_fns = HashMap::new();
        for word in &defined_words {
            let symbol_name = format!("__word_{}", word.to_lowercase().replace("-", "_"));
            if let Ok(sym) = unsafe { lib.get::<WordFn>(symbol_name.as_bytes()) } {
                word_fns.insert(word.clone(), *sym);
            }
        }

        // Store loaded library
        self.libraries.push(LoadedLibrary {
            _lib: lib,
            entry,
            defined_words,
            word_fns,
        });

        Ok(entry)
    }

    /// Get a word function from the most recently loaded library.
    pub fn get_word_fn(&self, name: &str) -> Result<Option<WordFn>, String> {
        if let Some(lib) = self.libraries.last() {
            Ok(lib.word_fns.get(name).copied())
        } else {
            Ok(None)
        }
    }

    /// Compile a Rust source file to a shared library.
    fn compile_library(
        &self,
        source_path: &PathBuf,
        lib_path: &PathBuf,
        debug: u8,
    ) -> Result<(), String> {
        let mut cmd = Command::new("rustc");

        cmd.arg("--edition=2024")
            .arg("--crate-type=cdylib")
            .arg("-O")
            .arg("--extern")
            .arg(format!(
                "roth_runtime={}",
                self.find_runtime_rlib()
                    .map_err(|e| format!("Failed to find runtime: {}", e))?
                    .display()
            ))
            .arg("-o")
            .arg(lib_path)
            .arg(source_path);

        if debug >= 2 {
            println!("Compile command: {:?}", cmd);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run rustc: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Compilation failed:\n{}", stderr));
        }

        Ok(())
    }

    /// Find the compiled roth-runtime library.
    fn find_runtime_rlib(&self) -> io::Result<PathBuf> {
        // First, try to find a pre-built library in target/release or target/debug
        let manifest_dir = std::env::current_dir()?;

        // Try release first
        let release_lib = manifest_dir
            .join("target")
            .join("release")
            .join("libroth_runtime.rlib");
        if release_lib.exists() {
            return Ok(release_lib);
        }

        // Then try debug
        let debug_lib = manifest_dir
            .join("target")
            .join("debug")
            .join("libroth_runtime.rlib");
        if debug_lib.exists() {
            return Ok(debug_lib);
        }

        // If not found, we need to build it
        self.build_runtime()?;

        // Try again
        if release_lib.exists() {
            return Ok(release_lib);
        }
        if debug_lib.exists() {
            return Ok(debug_lib);
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find or build roth-runtime library",
        ))
    }

    /// Build the roth-runtime library if not present.
    fn build_runtime(&self) -> io::Result<()> {
        print!("Building roth-runtime library... ");
        io::stdout().flush()?;

        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("-p")
            .arg("roth-runtime")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to build runtime: {}", stderr),
            ));
        }

        println!("done.");
        Ok(())
    }

    /// Get the list of words defined in a library.
    fn get_defined_words(&self, lib: &Library) -> Result<Vec<String>, String> {
        // Try to get the __defined_words symbol
        let result: Result<Symbol<*const &[&str]>, _> = unsafe { lib.get(b"__defined_words") };

        match result {
            Ok(sym) => {
                let words_slice: &[&str] = unsafe { **sym };
                Ok(words_slice.iter().map(|s| s.to_string()).collect())
            }
            Err(_) => {
                // No words defined in this library
                Ok(Vec::new())
            }
        }
    }
}

/// Get the platform-specific shared library filename.
fn lib_filename(name: &str) -> String {
    #[cfg(target_os = "linux")]
    {
        format!("lib{}.so", name)
    }
    #[cfg(target_os = "macos")]
    {
        format!("lib{}.dylib", name)
    }
    #[cfg(target_os = "windows")]
    {
        format!("{}.dll", name)
    }
}
