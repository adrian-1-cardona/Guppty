// These tests cover CLI behavior and readable errors. Full example-output
// coverage lives in dual_backend.rs, which runs every example on both backends.

use std::process::Command;
use std::{fs, process};

// -------------------------------------------------------------------------
// TEST: running with no file should fail (usage message)
// why? if someone types just "guppty" we should tell them what to do
// -------------------------------------------------------------------------
#[test]
fn guppty_without_file_exits_with_error() {
    let bin = env!("CARGO_BIN_EXE_guppty");
    let output = Command::new(bin).output().expect("failed to run guppty");

    // should NOT be success (they forgot the file!)
    assert!(!output.status.success());
}

fn run_temp_source(name: &str, source: &str) -> (String, i32) {
    let mut path = std::env::temp_dir();
    path.push(format!("guppty-{}-{}.gup", process::id(), name));
    fs::write(&path, source).expect("failed to write temp gup file");

    let bin = env!("CARGO_BIN_EXE_guppty");
    let output = Command::new(bin)
        .arg(&path)
        .output()
        .expect("failed to run guppty");

    let _ = fs::remove_file(&path);
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

// Every readable error should answer three questions, without a stack trace:
//   WHERE it is (file:line:column + the source line and a caret),
//   WHAT type it is (a named error type), and
//   HOW to fix it (a single-line "help:" suggestion).

#[test]
fn lex_errors_are_readable_and_located() {
    let (stderr, exit_code) = run_temp_source("lex-error", "out(@)");

    assert_ne!(exit_code, 0);
    // WHERE
    assert!(stderr.contains(":1:5:"));
    assert!(stderr.contains("out(@)"));
    assert!(stderr.contains("^"));
    // WHAT
    assert!(stderr.contains("SyntaxError"));
    // HOW
    assert!(stderr.contains("help:"));
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn parse_errors_are_readable_and_located() {
    let (stderr, exit_code) = run_temp_source("parse-error", "out(1 + )");

    assert_ne!(exit_code, 0);
    // WHERE
    assert!(stderr.contains(":1:9:"));
    // WHAT + original message
    assert!(stderr.contains("SyntaxError"));
    assert!(stderr.contains("I expected an expression"));
    // HOW
    assert!(stderr.contains("help:"));
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn runtime_errors_are_readable_and_located() {
    let (stderr, exit_code) = run_temp_source("runtime-error", "out(missing)");

    assert_ne!(exit_code, 0);
    // WHERE
    assert!(stderr.contains(":1:5:"));
    // WHAT
    assert!(stderr.contains("NameError"));
    assert!(stderr.contains("Variable 'missing' is not defined yet"));
    // HOW
    assert!(stderr.contains("help:"));
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn errors_point_at_the_right_line_in_a_multiline_program() {
    // The mistake is on line 3; the error must say line 3, not line 1.
    let source = "x = 1\ny = 2\nout(oops)\n";
    let (stderr, exit_code) = run_temp_source("multiline-error", source);

    assert_ne!(exit_code, 0);
    assert!(stderr.contains(":3:5:"), "stderr was:\n{}", stderr);
    assert!(stderr.contains("NameError"));
    assert!(stderr.contains("out(oops)"));
    assert!(stderr.contains("help:"));
}

#[test]
fn divide_by_zero_reports_a_math_error_with_help() {
    let (stderr, exit_code) = run_temp_source("divzero-error", "out(1 / 0)");

    assert_ne!(exit_code, 0);
    assert!(stderr.contains("MathError"), "stderr was:\n{}", stderr);
    assert!(stderr.contains("help:"));
    assert!(!stderr.contains("panicked at"));
}
