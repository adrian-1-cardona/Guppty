// =============================================================================
// DUAL-BACKEND TEST — run every example BOTH ways and make sure they agree!
// =============================================================================
// Guppty has two engines that run your code:
//
//   1. the VM          (the default, speedy one)
//   2. the interpreter (the simple one, turned on with --interp)
//
// Big rule: NO MATTER which engine you use, the SAME program must print the
// SAME thing. If the two ever disagree, that is a bug — one engine "changed
// the rules" of the language by accident.
//
// This test is the lock on that rule. For every example program it:
//   * runs it with the VM
//   * runs it with the interpreter
//   * checks BOTH printed exactly what examples/expected/<name>.txt says
//   * checks the VM and the interpreter printed the EXACT same thing
//
// Think of it like baking the same cake recipe in two different ovens and
// making sure both cakes taste identical. Yum!
// =============================================================================

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{project_root, run_gup_file_with_backend, Backend};

// -------------------------------------------------------------------------
// Make line endings boring and the same everywhere.
// Windows likes "\r\n", everyone else likes "\n". We turn them all into "\n"
// so a test never fails just because of an invisible character.
// -------------------------------------------------------------------------
fn tidy(text: &str) -> String {
    text.replace("\r\n", "\n")
}

// -------------------------------------------------------------------------
// Find every examples/*.gup file, sorted so the order is always the same.
// -------------------------------------------------------------------------
fn all_example_files(examples_dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(examples_dir)
        .expect("examples folder should exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gup"))
        .collect();

    paths.sort();
    paths
}

// -------------------------------------------------------------------------
// THE TEST: every example must run the same on BOTH backends.
// -------------------------------------------------------------------------
#[test]
fn examples_match_on_both_backends() {
    let root = project_root();
    let examples_dir = root.join("examples");
    let expected_dir = examples_dir.join("expected");

    let example_files = all_example_files(&examples_dir);

    // Safety net: if someone deletes all the examples by accident, shout.
    assert!(
        example_files.len() >= 15,
        "Expected at least 15 example programs, but only found {}",
        example_files.len()
    );

    for example in &example_files {
        // The short name, e.g. "hello" from "hello.gup".
        let stem = example
            .file_stem()
            .expect("example file should have a name")
            .to_string_lossy()
            .to_string();

        // The matching expected-output file must exist.
        let expected_path = expected_dir.join(format!("{}.txt", stem));
        assert!(
            expected_path.exists(),
            "Missing expected output for '{}': {}",
            stem,
            expected_path.display()
        );

        let expected = tidy(
            &fs::read_to_string(&expected_path)
                .unwrap_or_else(|e| panic!("could not read {}: {}", expected_path.display(), e)),
        );

        // We will remember what each backend printed so we can compare them.
        let mut printed_by_each_backend: Vec<(Backend, String)> = Vec::new();

        // Run the SAME example once per backend (VM, then interpreter).
        for backend in Backend::all() {
            let (stdout, stderr, exit_code) = run_gup_file_with_backend(example, backend);

            // 1) The program should finish happily (exit code 0).
            assert_eq!(
                exit_code,
                0,
                "Example '{}' crashed on the {} backend.\n--- stderr ---\n{}",
                stem,
                backend.name(),
                stderr
            );

            // 2) What it printed should match the expected file.
            let actual = tidy(&stdout);
            assert_eq!(
                actual,
                expected,
                "Example '{}' printed the wrong thing on the {} backend.\n--- expected ---\n{}\n--- actual ---\n{}",
                stem,
                backend.name(),
                expected,
                actual
            );

            printed_by_each_backend.push((backend, actual));
        }

        // 3) THE BIG ONE: both backends must agree with each other.
        // This is what "locks the semantics" — the two engines can never drift.
        let (first_backend, first_output) = &printed_by_each_backend[0];
        for (other_backend, other_output) in &printed_by_each_backend[1..] {
            assert_eq!(
                first_output,
                other_output,
                "Backends disagree on example '{}'! The {} and {} engines printed different things.\n--- {} ---\n{}\n--- {} ---\n{}",
                stem,
                first_backend.name(),
                other_backend.name(),
                first_backend.name(),
                first_output,
                other_backend.name(),
                other_output
            );
        }
    }
}
