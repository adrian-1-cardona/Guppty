// =============================================================================
// INTEGRATION TESTS — .gup file in, stdout out!
// =============================================================================
// These tests run REAL .gup programs through the REAL guppty binary.
// They read expected output from tests/gup_cases.txt so you can change
// what "correct" means without rewriting Rust.
//
// Why this matters: unit tests check tiny pieces. These check the WHOLE
// pipeline works end-to-end — like a taste test of the finished soup!
// =============================================================================

mod common;

use std::path::PathBuf;
use std::process::Command;
use std::{fs, process};

use common::{load_cases, project_root, run_gup_file, stdout_lines};

// -------------------------------------------------------------------------
// TEST: every case in gup_cases.txt should run and print the right stuff
// -------------------------------------------------------------------------
#[test]
fn all_gup_files_match_expected_stdout() {
    // step 1: read the list of tests from the txt file
    let cases = load_cases();
    let root = project_root();

    // step 2: run each one!
    for case in cases {
        // build the full path to the .gup file
        let gup_path: PathBuf = root.join(&case.file);

        // make sure the file actually exists (typo in txt = sad test)
        assert!(
            gup_path.is_file(),
            "test case points to missing file: {}",
            gup_path.display()
        );

        // run guppty on it — file in!
        let (stdout, stderr, exit_code) = run_gup_file(&gup_path);

        // the program should not crash (exit 0 = happy)
        assert_eq!(
            exit_code, 0,
            "guppty failed on {} (stderr: {})",
            case.file, stderr
        );

        // compare stdout line by line — stdout out!
        let actual = stdout_lines(&stdout);
        assert_eq!(
            actual,
            case.stdout,
            "wrong output for {}\n--- expected ---\n{}\n--- actual ---\n{}",
            case.file,
            case.stdout.join("\n"),
            actual.join("\n"),
        );
    }
}

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
