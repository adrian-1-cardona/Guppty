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

#[test]
fn lex_errors_are_readable_and_located() {
    let (stderr, exit_code) = run_temp_source("lex-error", "out(@)");

    assert_ne!(exit_code, 0);
    assert!(stderr.contains("lex error"));
    assert!(stderr.contains("span: line 1, column 5, length 1"));
    assert!(stderr.contains("out(@)"));
    assert!(stderr.contains("^"));
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn parse_errors_are_readable_and_located() {
    let (stderr, exit_code) = run_temp_source("parse-error", "out(1 + )");

    assert_ne!(exit_code, 0);
    assert!(stderr.contains("parse error"));
    assert!(stderr.contains("span: line 1, column 9, length 1"));
    assert!(stderr.contains("I expected an expression"));
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn runtime_errors_are_readable_and_located() {
    let (stderr, exit_code) = run_temp_source("runtime-error", "out(missing)");

    assert_ne!(exit_code, 0);
    assert!(stderr.contains("runtime error"));
    assert!(stderr.contains("span: line 1, column 5, length 7"));
    assert!(stderr.contains("Variable 'missing' is not defined yet"));
    assert!(!stderr.contains("panicked at"));
}
