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
            actual, case.stdout,
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
    use std::process::Command;

    let bin = env!("CARGO_BIN_EXE_guppty");
    let output = Command::new(bin).output().expect("failed to run guppty");

    // should NOT be success (they forgot the file!)
    assert!(!output.status.success());
}
