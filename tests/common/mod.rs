// =============================================================================
// tests/common/mod.rs
// Helper toolbox for integration tests.
// Reads tests/gup_cases.txt so you can change expected output without
// touching Rust code. Yay!
// =============================================================================

// Each test file gets its own copy of this toolbox. Some files only use
// SOME of the tools. That is fine! This line tells Rust "don't nag me about
// tools I'm not using right now."
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// -------------------------------------------------------------------------
// One test: which .gup file + what lines we expect on stdout
// -------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GupCase {
    pub file: String,
    pub stdout: Vec<String>,
}

// -------------------------------------------------------------------------
// Where the project folder lives (cargo tells us when tests run)
// -------------------------------------------------------------------------
pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// -------------------------------------------------------------------------
// Read tests/gup_cases.txt and turn it into a list of GupCase
//
// Format (see gup_cases.txt for examples):
//   - line starting with # or empty = skip
//   - first non-comment line in a block = file path
//   - following lines = expected stdout until blank line
// -------------------------------------------------------------------------
pub fn load_cases() -> Vec<GupCase> {
    let manifest_path = project_root().join("tests").join("gup_cases.txt");
    let text = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("could not read {}: {}", manifest_path.display(), e));

    let mut cases: Vec<GupCase> = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_stdout: Vec<String> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();

        // skip comments and empty lines BETWEEN blocks
        if trimmed.is_empty() {
            if let Some(file) = current_file.take() {
                cases.push(GupCase {
                    file,
                    stdout: std::mem::take(&mut current_stdout),
                });
            }
            continue;
        }

        if trimmed.starts_with('#') {
            continue;
        }

        // first line of a block = the .gup file path
        if current_file.is_none() {
            current_file = Some(trimmed.to_string());
            continue;
        }

        // every other line = one expected output line
        current_stdout.push(trimmed.to_string());
    }

    // last block might not end with blank line
    if let Some(file) = current_file.take() {
        cases.push(GupCase {
            file,
            stdout: current_stdout,
        });
    }

    assert!(
        !cases.is_empty(),
        "no test cases found in {}",
        manifest_path.display()
    );

    cases
}

// -------------------------------------------------------------------------
// Guppty can run your code in TWO ways. We call each way a "backend":
//
//   * Vm          -> the default, speedy robot. It turns your code into
//                    tiny step-by-step instructions, then races through them.
//   * Interpreter -> the older, simpler reader (turned on with --interp).
//                    It walks over your code tree and does each part directly.
//
// Both ways should ALWAYS print the exact same thing. This little enum lets
// a test say "run it the VM way" or "run it the interpreter way" — like
// picking the red crayon or the blue crayon to draw the same picture.
// -------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Vm,
    Interpreter,
}

impl Backend {
    // A friendly name to show in test messages so a human knows which
    // way was used when something goes wrong.
    pub fn name(self) -> &'static str {
        match self {
            Backend::Vm => "vm (default)",
            Backend::Interpreter => "interpreter (--interp)",
        }
    }

    // Both backends in a little list, handy for "do this for every way" loops.
    pub fn all() -> [Backend; 2] {
        [Backend::Vm, Backend::Interpreter]
    }
}

// -------------------------------------------------------------------------
// Run a .gup file the DEFAULT way (the VM). file in, output out!
// -------------------------------------------------------------------------
pub fn run_gup_file(file: &Path) -> (String, String, i32) {
    run_gup_file_with_backend(file, Backend::Vm)
}

// -------------------------------------------------------------------------
// Run a .gup file the way YOU pick.
// The interpreter needs the extra "--interp" word; the VM needs nothing.
// -------------------------------------------------------------------------
pub fn run_gup_file_with_backend(file: &Path, backend: Backend) -> (String, String, i32) {
    let bin = env!("CARGO_BIN_EXE_guppty");

    let mut command = Command::new(bin);
    command.arg(file);
    if backend == Backend::Interpreter {
        command.arg("--interp");
    }

    let output = command
        .output()
        .unwrap_or_else(|e| panic!("failed to run guppty on {}: {}", file.display(), e));

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);

    (stdout, stderr, code)
}

// -------------------------------------------------------------------------
// Split stdout into lines (for comparing with expected)
// -------------------------------------------------------------------------
pub fn stdout_lines(stdout: &str) -> Vec<String> {
    stdout.lines().map(|line| line.to_string()).collect()
}
