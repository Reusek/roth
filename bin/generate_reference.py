import os
import subprocess
import sys
from pathlib import Path

def run_gforth(file_path):
    """Run a Forth file through gforth and capture output."""
    try:
        cmd = ['gforth', '-e', f'include {file_path} bye']
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        return result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired:
        return "", "Timeout expired", 1
    except FileNotFoundError:
        return "", "gforth not found", 1

def generate_reference_outputs():
    """Generate reference outputs for all test source files."""
    project_root = Path(__file__).parent.parent
    test_source_dir = project_root / "test_source"
    
    if not test_source_dir.exists():
        print(f"Error: {test_source_dir} does not exist")
        return 1
    
    # Find all .fs and .rt files
    test_files = list(test_source_dir.glob("**/*.fs"))
    
    if not test_files:
        print(f"No test files found in {test_source_dir}")
        return 1
    
    print(f"Found {len(test_files)} test files")
    
    for test_file in sorted(test_files):
        print(f"\nProcessing {test_file.name}...")
        
        stdout, stderr, returncode = run_gforth(test_file)
        
        # Save stdout to separate file
        stdout_file = test_file.with_name(f"{test_file.stem}_stdout.txt")
        with open(stdout_file, 'w') as f:
            f.write(stdout)
        
        # Save stderr to separate file
        stderr_file = test_file.with_name(f"{test_file.stem}_stderr.txt")
        with open(stderr_file, 'w') as f:
            f.write(stderr)
        print(f"  -> {stdout_file.name}, {stderr_file.name} (return code: {returncode})")
        
        if returncode != 0:
            print(f"  Warning: gforth returned non-zero exit code")
    
    return 0

if __name__ == "__main__":
    sys.exit(generate_reference_outputs())