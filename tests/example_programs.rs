// integration tests — run every .gup example and check the output!
// if output changes, the test breaks so we know something regressed.
//
// Guppty has TWO backends that must agree:
//   - the bytecode compiler + VM (the default), and
//   - the older tree-walking interpreter (the `--interp` flag).
// We run every example through BOTH and compare each to the same expected
// output, so neither backend is allowed to drift from the other.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn guppty_binary() -> String {
    if Path::new("target/debug/guppty").exists() {
        "target/debug/guppty".to_string()
    } else {
        "target/release/guppty".to_string()
    }
}

/// The two ways to execute a program, plus the extra CLI args each one needs.
/// "vm" is the default (no flag); "interp" adds `--interp`.
const BACKENDS: &[(&str, &[&str])] = &[("vm", &[]), ("interp", &["--interp"])];

/// Run one example file through one backend and assert its stdout matches the
/// expected output file. `extra_args` is empty for the VM, `--interp` otherwise.
fn check_example(
    binary: &str,
    backend: &str,
    extra_args: &[&str],
    example_path: &Path,
    expected_path: &Path,
    stem: &str,
) {
    let mut command = Command::new(binary);
    command.arg(example_path);
    command.args(extra_args);

    let output = command
        .output()
        .unwrap_or_else(|e| panic!("Failed to run {} ({}): {}", binary, backend, e));

    assert!(
        output.status.success(),
        "Program {} failed on the {} backend: {}",
        stem,
        backend,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let expected = fs::read_to_string(expected_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", expected_path.display(), e))
        .replace("\r\n", "\n");

    assert_eq!(
        actual, expected,
        "Output mismatch for {} on the {} backend\nExpected:\n{}\nActual:\n{}",
        stem, backend, expected, actual
    );
}

fn example_paths() -> Vec<PathBuf> {
    let examples_dir = Path::new("examples");
    let mut paths: Vec<_> = fs::read_dir(examples_dir)
        .expect("examples folder should exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gup"))
        .collect();
    paths.sort();
    paths
}

#[test]
fn all_example_programs_match_expected_output() {
    let expected_dir = Path::new("examples").join("expected");
    let binary = guppty_binary();
    let example_paths = example_paths();

    assert!(
        example_paths.len() >= 15,
        "Need at least 15 example programs, found {}",
        example_paths.len()
    );

    for example_path in example_paths {
        let stem = example_path
            .file_stem()
            .expect("example should have a name")
            .to_string_lossy()
            .into_owned();

        let expected_path = expected_dir.join(format!("{}.txt", stem));

        assert!(
            expected_path.exists(),
            "Missing expected output file for {}: {}",
            stem,
            expected_path.display()
        );

        // Both backends must produce exactly the expected output.
        for (backend, extra_args) in BACKENDS {
            check_example(
                &binary,
                backend,
                extra_args,
                &example_path,
                &expected_path,
                &stem,
            );
        }
    }
}
