#!/usr/bin/env python3
"""
Pytest test suite for the Roth Forth compiler.
Tests all *.fs files in test_source/ against their reference outputs.
"""

import os
import subprocess
import glob
import pytest
from pathlib import Path





class TestRothCompiler:
    """Test suite for the Roth Forth compiler."""
    
    @pytest.fixture(scope="class")
    def test_source_dir(self):
        """Get the test_source directory path."""
        return Path(__file__).parent / "test_source"
    
    @pytest.fixture(scope="class") 
    def compiler_cmd(self):
        """Get the compiler command."""
        return ["./target/debug/roth"]
    
    def get_test_files(self, test_source_dir):
        """Get all *.fs files in test_source directory."""
        fs_files = list(test_source_dir.glob("*.fs"))
        return sorted(fs_files)
    
    def get_reference_files(self, fs_file):
        """Get the reference stdout and stderr files for a given .fs file."""
        base_name = fs_file.stem
        test_dir = fs_file.parent
        
        stdout_file = test_dir / f"{base_name}_stdout.txt"
        stderr_file = test_dir / f"{base_name}_stderr.txt"
        
        return stdout_file, stderr_file
    
    def read_file_safe(self, file_path):
        """Read file content safely, return empty string if file doesn't exist."""
        try:
            return file_path.read_text().strip()
        except FileNotFoundError:
            return ""
    
    def run_compiler(self, compiler_cmd, fs_file):
        """Run the compiler on a .fs file and capture output."""
        cmd = compiler_cmd + [str(fs_file), "--run"]
        
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30  # 30 second timeout
            )
            return result.stdout.strip(), result.stderr.strip(), result.returncode
        except subprocess.TimeoutExpired:
            return "", "Compiler timed out after 30 seconds", 1
        except Exception as e:
            return "", f"Error running compiler: {e}", 1
    
    @pytest.mark.parametrize("fs_file", [
        pytest.param(f, id=f.name) for f in sorted(Path("test_source").glob("**/*.fs"))
    ])
    def test_compiler_output(self, fs_file, compiler_cmd, test_source_dir):
        """Test compiler output against reference files."""
        fs_file_path = fs_file
        
        # Get reference files
        stdout_ref_file, stderr_ref_file = self.get_reference_files(fs_file_path)
        
        # Read reference outputs
        expected_stdout = self.read_file_safe(stdout_ref_file)
        expected_stderr = self.read_file_safe(stderr_ref_file)
        
        # Run compiler
        actual_stdout, actual_stderr, return_code = self.run_compiler(compiler_cmd, fs_file_path)
        
        # Compare outputs - pytest will show nice diffs automatically
        assert actual_stdout == expected_stdout, f"STDOUT mismatch for {fs_file.name}"
        assert actual_stderr == expected_stderr, f"STDERR mismatch for {fs_file.name}"
        
        # If we have reference files, we expect the compiler to succeed
        # unless the stderr reference indicates an error
        if stdout_ref_file.exists() or stderr_ref_file.exists():
            if expected_stderr and "error" in expected_stderr.lower():
                assert return_code != 0, f"Expected compiler to fail for {fs_file.name}"
            else:
                assert return_code == 0, f"Compiler failed unexpectedly for {fs_file.name}"


if __name__ == "__main__":
    # Run pytest when script is executed directly
    pytest.main([__file__, "-v"])