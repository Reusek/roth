use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
// use std::time::Duration;

use colored::Colorize;
use futures::future::join_all;
use similar::{ChangeTag, TextDiff};

mod highlighter;
use clap::Parser;
use highlighter::ForthHighlighter;
use tokio::sync::Mutex;

const FORTH_A: &'static str = r#"\ Simple factorial implementation
: factorial ( n -- n! )
  dup 1 <= if
    drop 1
  else
    dup 1 - factorial *
  then ;

\ Test with small number
5 factorial . cr

\ Stack manipulation example
: test-stack ( a b c -- )
  rot swap over
  . . . ;

variable counter
42 counter !
"#;

const FORTH_B: &'static str = r#"\ Enhanced factorial with better base case
: factorial ( n -- n! )
  dup 2 <= if
    drop 1
  else
    dup 1 - factorial *
  then ;

\ Test with larger number
10 factorial . cr

\ Stack manipulation example with 2dup
: test-stack ( a b c -- )
  rot swap 2dup
  . . . . ;

variable counter
100 counter !

\ Additional lines to test padding
\ Line 19
\ Line 20
\ Line 21
\ Line 22
\ Line 23
\ Line 24
\ Line 25
\ Line 26
\ Line 27
\ Line 28
\ Line 29
\ Line 30
"#;

const FORTH_C: &'static str = r#"\ Simple factorial implementation
: factorial ( n -- n! )
  dup 1 <= if
    drop 1
  else
    dup 1 - factorial *
  then ;
  then ;

\ Test with small number
5 factorial . cr

\ Stack manipulation example
: test-stack ( a b c -- )
  rot swap over
  . . . ;

variable counter
42 counter !
"#;

pub fn get_forth_diff(text_a: &str, text_b: &str) -> String {
    let highlighter = ForthHighlighter::new();
    let diff = TextDiff::from_lines(text_a, text_b);

    let mut result = String::new();
    result.push_str(&format!(
        "{}\n\n",
        "Forth Code Diff with Syntax Highlighting:"
            .bold()
            .underline()
    ));

    // Calculate max line numbers for padding
    let max_old_line = text_a.lines().count();
    let max_new_line = text_b.lines().count();
    let old_width = max_old_line.to_string().len();
    let new_width = max_new_line.to_string().len();

    let mut old_line = 1;
    let mut new_line = 1;

    for change in diff.iter_all_changes() {
        let (sign, old_num, new_num) = match change.tag() {
            ChangeTag::Delete => {
                let old_num = format!("{:<width$}", old_line, width = old_width);
                let new_num = " ".repeat(new_width);
                old_line += 1;
                ("- ", old_num, new_num)
            }
            ChangeTag::Insert => {
                let old_num = " ".repeat(old_width);
                let new_num = format!("{:<width$}", new_line, width = new_width);
                new_line += 1;
                ("+ ", old_num, new_num)
            }
            ChangeTag::Equal => {
                let old_num = format!("{:<width$}", old_line, width = old_width);
                let new_num = format!("{:<width$}", new_line, width = new_width);
                old_line += 1;
                new_line += 1;
                ("  ", old_num, new_num)
            }
        };

        let line = change.value();
        let plain_line = highlighter.highlight(line.trim_end());

        let colored_line = match change.tag() {
            ChangeTag::Delete => format!(
                "{} {}|{}{}",
                old_num,
                new_num,
                sign.red().bold(),
                plain_line
            ),
            ChangeTag::Insert => format!(
                "{} {}|{}{}",
                old_num,
                new_num,
                sign.green().bold(),
                plain_line
            ),
            ChangeTag::Equal => format!("{} {}|{}{}", old_num, new_num, sign.dimmed(), plain_line),
        };

        result.push_str(&format!("{}\n", colored_line));
    }

    result
}

pub fn get_forth_diff_unified(text_a: &str, text_b: &str, context_lines: usize) -> String {
    let highlighter = ForthHighlighter::new();
    let diff = TextDiff::from_lines(text_a, text_b);

    let mut result = String::new();
    // result.push_str(&format!("{}\n\n", "Forth Code Unified Diff:".bold().underline()));

    // Calculate max line numbers for padding
    let max_old_line = text_a.lines().count();
    let max_new_line = text_b.lines().count();
    let old_width = max_old_line.to_string().len();
    let new_width = max_new_line.to_string().len();

    for (idx, group) in diff.grouped_ops(context_lines).iter().enumerate() {
        if idx > 0 {
            result.push_str(&format!("{}\n", "---".dimmed()));
        }

        let old_start = group
            .first()
            .map(|op| op.old_range().start + 1)
            .unwrap_or(1);
        let new_start = group
            .first()
            .map(|op| op.new_range().start + 1)
            .unwrap_or(1);
        // let old_len = group.iter().map(|op| op.old_range().len()).sum::<usize>();
        // let new_len = group.iter().map(|op| op.new_range().len()).sum::<usize>();
        //
        // result.push_str(&format!(
        //     "{}\n",
        //     format!(
        //         "@@ -{},{} +{},{} @@",
        //         old_start, old_len, new_start, new_len
        //     )
        //     .cyan()
        //     .bold()
        // ));

        let mut old_line = old_start;
        let mut new_line = new_start;

        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, old_num, new_num) = match change.tag() {
                    ChangeTag::Delete => {
                        let old_num = format!("{:<width$}", old_line, width = old_width)
                            .red()
                            .dimmed()
                            .to_string();
                        let new_num = " ".repeat(new_width);
                        old_line += 1;
                        ("-", old_num, new_num)
                    }
                    ChangeTag::Insert => {
                        let old_num = " ".repeat(old_width);
                        let new_num = format!("{:<width$}", new_line, width = new_width)
                            .green()
                            .dimmed()
                            .to_string();
                        new_line += 1;
                        ("+", old_num, new_num)
                    }
                    ChangeTag::Equal => {
                        let old_num = format!("{:<width$}", old_line, width = old_width)
                            .dimmed()
                            .to_string();
                        let new_num = format!("{:<width$}", new_line, width = new_width)
                            .dimmed()
                            .to_string();
                        old_line += 1;
                        new_line += 1;
                        (" ", old_num, new_num)
                    }
                };

                let line = change.value();
                let plain_line = highlighter.highlight(line.trim_end());

                let colored_line = match change.tag() {
                    ChangeTag::Delete => {
                        format!(
                            "{} {}|{} {}",
                            old_num,
                            new_num,
                            sign.red().bold(),
                            plain_line
                        )
                    }
                    ChangeTag::Insert => {
                        format!(
                            "{} {}|{} {}",
                            old_num,
                            new_num,
                            sign.green().bold(),
                            plain_line
                        )
                    }
                    ChangeTag::Equal => {
                        format!("{} {}|{} {}", old_num, new_num, sign.dimmed(), plain_line)
                    }
                };

                result.push_str(&format!("{}\n", colored_line));
            }
        }
    }

    result
}

pub fn find_fs_files<P: AsRef<Path>>(root_path: P) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut fs_files = Vec::new();
    find_fs_files_recursive(root_path.as_ref(), &mut fs_files)?;
    Ok(fs_files)
}

fn find_fs_files_recursive(dir: &Path, fs_files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                find_fs_files_recursive(&path, fs_files)?;
            } else if let Some(extension) = path.extension() {
                if extension == "fs" {
                    fs_files.push(path);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    bin: String,

    #[arg(short, long)]
    test: String,
}

struct Test {
    id: usize,

    /// Path to the Forth file.
    file_path: String,

    /// Path to the program that executes the source file.
    executer: String,

    stdout: Option<String>,
    expected_stdout: Option<String>,
    passed: Option<bool>,
}

impl Test {
    pub fn new(id: usize, executer: String, file_path: String) -> Self {
        // Try to find expected stdout file
        let expected_stdout = {
            let stdout_path = file_path.replace(".fs", "_stdout.txt");
            std::fs::read_to_string(&stdout_path).ok()
        };

        Self {
            id,
            executer,
            file_path,
            stdout: None,
            expected_stdout,
            passed: None,
        }
    }
}

enum TestState {
    Idle,
    Running,
    Finished(Test),
}

struct TestRunner {
    status_tx: tokio::sync::mpsc::Sender<Test>,
    test_counter: Mutex<usize>,
    tests_to_run: Vec<Test>,
    running_tests: Arc<Mutex<Vec<TestState>>>,
}

impl TestRunner {
    pub fn new(tests: Vec<Test>) -> (Self, tokio::sync::mpsc::Receiver<Test>) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let runner = Self {
            status_tx: tx,
            test_counter: Mutex::new(0),
            tests_to_run: tests,
            running_tests: Arc::new(Mutex::new(Vec::new())),
        };

        (runner, rx)
    }

    pub async fn start(self, workers: usize) {
        let s = Arc::new(self);

        // Spawn workers
        let workers = (0..workers)
            .into_iter()
            .map(|_| {
                let ss = s.clone();
                tokio::spawn(async move { ss.worker().await })
            })
            .collect::<Vec<_>>();

        join_all(workers).await;
    }

    async fn worker(&self) {
        loop {
            let id = {
                let mut counter_lock = self.test_counter.lock().await;
                let value = *counter_lock;
                *counter_lock += 1;
                value
            };

            if id >= self.tests_to_run.len() {
                return;
            }

            let test = &self.tests_to_run[id];

            let mut command = tokio::process::Command::new(&test.executer);
            command.arg(&test.file_path);

            match command.output().await {
                Ok(output) => {
                    let mut completed_test =
                        Test::new(test.id, test.executer.clone(), test.file_path.clone());
                    let actual_stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    completed_test.stdout = Some(actual_stdout.clone());

                    // Compare with expected output
                    if let Some(expected) = &completed_test.expected_stdout {
                        completed_test.passed = Some(actual_stdout.trim() == expected.trim());
                    }

                    if let Err(_) = self.status_tx.send(completed_test).await {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute test {}: {}", test.id, e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    // Test the find_fs_files function
    match find_fs_files(args.test) {
        Ok(fs_files) => {
            println!("Found {} .fs files:", fs_files.len());
            for file in &fs_files {
                println!("  {}", file.display());
            }

            // Create tests from found .fs files
            let tests: Vec<Test> = fs_files
                .into_iter()
                .enumerate()
                .map(|(id, file_path)| {
                    Test::new(
                        id,
                        args.bin.clone(),
                        file_path.to_string_lossy().to_string(),
                    )
                })
                .collect();

            let total_tests = tests.len();
            let (test_runner, mut status_rx) = TestRunner::new(tests);

            // Start test runner
            let runner_handle = tokio::spawn(async move {
                test_runner.start(4).await;
            });

            // Handle results
            let mut completed = 0;
            let mut passed = 0;
            let mut failed = 0;

            while completed < total_tests {
                if let Some(test) = status_rx.recv().await {
                    completed += 1;

                    match test.passed {
                        Some(true) => {
                            passed += 1;
                            println!("✅ Test {} PASSED: {}", test.id, test.file_path);
                        }
                        Some(false) => {
                            failed += 1;
                            println!("❌ Test {} FAILED: {}", test.id, test.file_path);

                            if let (Some(actual), Some(expected)) =
                                (&test.stdout, &test.expected_stdout)
                            {
                                let diff =
                                    get_forth_diff_unified(expected.trim(), actual.trim(), 3);
                                if !diff.trim().is_empty() {
                                    println!("Diff (expected vs actual):");
                                    println!("{}", diff);
                                }
                            }
                        }
                        None => {
                            println!("⚠️  Test {} NO REFERENCE: {}", test.id, test.file_path);
                            if let Some(stdout) = &test.stdout {
                                if !stdout.trim().is_empty() {
                                    println!("  Output: {}", stdout.trim());
                                }
                            }
                        }
                    }
                }
            }

            println!("\n📊 Test Summary:");
            println!("  Total: {}", total_tests);
            println!("  Passed: {}", passed);
            println!("  Failed: {}", failed);
            println!("  No Reference: {}", total_tests - passed - failed);

            runner_handle.await.unwrap();
        }
        Err(e) => {
            eprintln!("Error finding .fs files: {}", e);
        }
    }

    // let unified_diff_result = get_forth_diff_unified(FORTH_A, FORTH_C, 2);
    // println!("{}", unified_diff_result);
}
