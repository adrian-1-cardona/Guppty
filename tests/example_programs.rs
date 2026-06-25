// integration tests — run every .gup example and check the output!
// if output changes, the test breaks so we know something regressed.

use std::fs;
use std::path::Path;
use std::process::Command;

fn guppty_binary() -> String {
    if Path::new("target/debug/guppty").exists() {
        "target/debug/guppty".to_string()
    } else {
        "target/release/guppty".to_string()
    }
}

#[test]
fn all_example_programs_match_expected_output() {
    let examples_dir = Path::new("examples");
    let expected_dir = examples_dir.join("expected");
    let binary = guppty_binary();

    let mut example_paths: Vec<_> = fs::read_dir(examples_dir)
        .expect("examples folder should exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gup"))
        .collect();

    example_paths.sort();

    assert!(
        example_paths.len() >= 15,
        "Need at least 15 example programs, found {}",
        example_paths.len()
    );

    for example_path in example_paths {
        let stem = example_path
            .file_stem()
            .expect("example should have a name")
            .to_string_lossy();

        let expected_path = expected_dir.join(format!("{}.txt", stem));

        assert!(
            expected_path.exists(),
            "Missing expected output file for {}: {}",
            stem,
            expected_path.display()
        );

        let output = Command::new(&binary)
            .arg(&example_path)
            .output()
            .unwrap_or_else(|e| panic!("Failed to run {}: {}", binary, e));

        assert!(
            output.status.success(),
            "Program {} failed: {}",
            stem,
            String::from_utf8_lossy(&output.stderr)
        );

        let actual = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", expected_path.display(), e))
            .replace("\r\n", "\n");

        assert_eq!(
            actual, expected,
            "Output mismatch for {}\nExpected:\n{}\nActual:\n{}",
            stem, expected, actual
        );
    }
}
