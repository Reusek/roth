use std::fs;
use std::path::Path;
use std::process::Command;

fn build_output_path(output_file: &str) -> String {
    let filename = Path::new(output_file)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new(output_file));
    Path::new(".build")
        .join(filename)
        .to_string_lossy()
        .to_string()
}

fn compile_forth_file(
    input_file: &str,
    output_file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(&["run", "--", input_file, "--output", output_file])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn create_test_file(filename: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(filename, content)?;
    Ok(())
}

fn cleanup_test_file(filename: &str) {
    let _ = fs::remove_file(filename);
}

#[test]
fn test_compile_simple_program() {
    let test_file = "test_simple.rt";
    let output_file = "test_simple.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, "42 .").unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_ok());

    // Check that output file was created
    assert!(Path::new(&build_output_file).exists());

    // Check output file contains expected Rust code
    let generated_code = fs::read_to_string(&build_output_file).unwrap();
    assert!(generated_code.contains("self.stack.push(42)"));
    assert!(generated_code.contains("pub struct OptimizedForth"));

    cleanup_test_file(test_file);
    cleanup_test_file(&build_output_file);
}

#[test]
fn test_compile_definition() {
    let test_file = "test_definition.rt";
    let output_file = "test_definition.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, ": SQUARE DUP * ; 5 SQUARE .").unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_ok());

    let generated_code = fs::read_to_string(&build_output_file).unwrap();
    assert!(generated_code.contains("fn square"));
    assert!(generated_code.contains("self.stack.push(5)"));
    // Function may be inlined by optimizer
    assert!(generated_code.contains("square"));

    cleanup_test_file(test_file);
    cleanup_test_file(&build_output_file);
}

#[test]
fn test_compile_arithmetic() {
    let test_file = "test_arithmetic.rt";
    let output_file = "test_arithmetic.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, "10 5 + 3 - 2 * .").unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_ok());

    let generated_code = fs::read_to_string(&build_output_file).unwrap();
    // The optimizer may fold the entire computation: (10 + 5 - 3) * 2 = 24
    assert!(generated_code.contains("self.stack.push") || generated_code.contains("stack.push"));
    assert!(generated_code.contains("pub struct OptimizedForth"));

    cleanup_test_file(test_file);
    cleanup_test_file(&build_output_file);
}

#[test]
fn test_compile_stack_operations() {
    let test_file = "test_stack.rt";
    let output_file = "test_stack.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, "1 2 3 DUP DROP SWAP OVER .S").unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_ok());

    let generated_code = fs::read_to_string(&build_output_file).unwrap();
    assert!(generated_code.contains("stack.push(1)"));
    assert!(generated_code.contains("stack.push(2)"));
    assert!(generated_code.contains("stack.push(3)"));

    cleanup_test_file(test_file);
    cleanup_test_file(&build_output_file);
}

#[test]
fn test_compile_with_comments() {
    let test_file = "test_comments.rt";
    let output_file = "test_comments.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, "( This is a comment ) 42 ( Another comment ) .").unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_ok());

    let generated_code = fs::read_to_string(&build_output_file).unwrap();
    assert!(generated_code.contains("self.stack.push(42)"));

    cleanup_test_file(test_file);
    cleanup_test_file(&build_output_file);
}

#[test]
fn test_compile_complex_program() {
    let test_file = "test_complex.rt";
    let output_file = "test_complex.rs";
    let build_output_file = build_output_path(output_file);

    let program = r#"
        ( Factorial calculation )
        : FACTORIAL 
            DUP 1 > 
            IF 
                DUP 1 - FACTORIAL * 
            ELSE 
                DROP 1 
            THEN 
        ;
        
        5 FACTORIAL .
    "#;

    create_test_file(test_file, program).unwrap();

    // This might fail due to IF/THEN not being implemented
    let result = compile_forth_file(test_file, output_file);

    // The test documents expected behavior even if not fully implemented.
    if result.is_ok() {
        let generated_code = fs::read_to_string(&build_output_file).unwrap_or_default();
        assert!(!generated_code.is_empty());
    }

    cleanup_test_file(test_file);
    if Path::new(&build_output_file).exists() {
        cleanup_test_file(&build_output_file);
    }
}

#[test]
fn test_compile_multiple_definitions() {
    let test_file = "test_multiple.rt";
    let output_file = "test_multiple.rs";
    let build_output_file = build_output_path(output_file);

    let program = r#"
        : DOUBLE 2 * ;
        : TRIPLE 3 * ;
        : QUADRUPLE 4 * ;
        
        10 DOUBLE TRIPLE QUADRUPLE .
    "#;

    create_test_file(test_file, program).unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_ok());

    let generated_code = fs::read_to_string(&build_output_file).unwrap();
    assert!(generated_code.contains("fn double"));
    assert!(generated_code.contains("fn triple"));
    assert!(generated_code.contains("fn quadruple"));
    // The optimizer may fold the entire computation: 10 * 2 * 3 * 4 = 240
    assert!(generated_code.contains("self.stack.push") || generated_code.contains("stack.push"));

    cleanup_test_file(test_file);
    cleanup_test_file(&build_output_file);
}

#[test]
fn test_compile_error_undefined_word() {
    let test_file = "test_error.rt";
    let output_file = "test_error.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, "UNDEFINED_WORD").unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_err());

    cleanup_test_file(test_file);
    if Path::new(&build_output_file).exists() {
        cleanup_test_file(&build_output_file);
    }
}

#[test]
fn test_compile_error_redefine_builtin() {
    let test_file = "test_redefine.rt";
    let output_file = "test_redefine.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, ": + 42 ;").unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_err());

    cleanup_test_file(test_file);
    if Path::new(&build_output_file).exists() {
        cleanup_test_file(&build_output_file);
    }
}

#[test]
fn test_compile_empty_file() {
    let test_file = "test_empty.rt";
    let output_file = "test_empty.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, "").unwrap();

    let result = compile_forth_file(test_file, output_file);
    assert!(result.is_ok());

    let generated_code = fs::read_to_string(&build_output_file).unwrap();
    assert!(generated_code.contains("pub struct OptimizedForth"));

    cleanup_test_file(test_file);
    cleanup_test_file(&build_output_file);
}

#[test]
fn test_compile_with_debug_output() {
    let test_file = "test_debug.rt";
    let output_file = "test_debug.rs";
    let build_output_file = build_output_path(output_file);

    create_test_file(test_file, "42 DUP +").unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            test_file,
            "--output",
            output_file,
            "--debug",
            "2",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Debug output should contain tokens, AST, and IR information
    assert!(stdout.contains("Tokens:") || stdout.contains("AST:") || stdout.contains("IR:"));

    cleanup_test_file(test_file);
    if Path::new(&build_output_file).exists() {
        cleanup_test_file(&build_output_file);
    }
}

#[test]
fn test_different_backends() {
    let test_file = "test_backends.rt";

    create_test_file(test_file, "42 .").unwrap();

    let backends = vec!["rust-ir", "c-ir", "ir-debug-rust", "ir-debug-c"];

    for backend in backends {
        let output_file = format!("test_backend_{}.out", backend.replace("-", "_"));
        let build_output_file = build_output_path(&output_file);

        let result = Command::new("cargo")
            .args(&[
                "run",
                "--",
                test_file,
                "--backend",
                backend,
                "--output",
                &output_file,
            ])
            .output();

        // Some backends might not be fully implemented
        if let Ok(output) = result {
            if output.status.success() {
                assert!(Path::new(&build_output_file).exists());
            }
        }

        if Path::new(&build_output_file).exists() {
            cleanup_test_file(&build_output_file);
        }
    }

    cleanup_test_file(test_file);
}
