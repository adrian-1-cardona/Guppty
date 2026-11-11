// =============================================================================
// RESOURCE LIMITS INTEGRATION TESTS
// =============================================================================
// Tests that execution budgets, recursion depths, range sizes, and output volumes
// are strictly bounded and safely return located runtime errors on both backends.

mod common;

use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

use common::Backend;

fn temp_workspace(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "guppty-limits-{}-{}-{label}",
        process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create temporary workspace");
    path
}

fn run_source_with_args(
    source: &str,
    extra_args: &[&str],
    envs: &[(&str, &str)],
) -> (String, String, i32) {
    let dir = temp_workspace("run");
    let script_path = dir.join("test_program.gup");
    fs::write(&script_path, source).expect("write script");

    let bin = env!("CARGO_BIN_EXE_guppty");
    let mut command = Command::new(bin);
    command.arg(&script_path);
    for arg in extra_args {
        command.arg(arg);
    }
    for (k, v) in envs {
        command.env(k, v);
    }

    let output = command.output().expect("guppty command execution failed");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);

    (stdout, stderr, code)
}

#[test]
fn max_call_depth_halts_infinite_recursion() {
    let script = "recurse()\n    recurse()\n\nrecurse()\n";

    for backend in Backend::all() {
        let mut args = vec!["--max-call-depth", "50"];
        if backend == Backend::Interpreter {
            args.push("--interp");
        }

        let (_stdout, stderr, code) = run_source_with_args(script, &args, &[]);
        assert_eq!(
            code,
            1,
            "backend {} should exit with code 1",
            backend.name()
        );
        assert!(
            stderr.contains("Maximum call stack depth of 50 exceeded."),
            "backend {} should report call depth error, got: {}",
            backend.name(),
            stderr
        );
        assert!(
            stderr.contains("RuntimeError"),
            "backend {} should report RuntimeError, got: {}",
            backend.name(),
            stderr
        );
    }
}

#[test]
fn max_range_size_prevents_unbounded_materialization() {
    let script = "for x in range(1 through 5000000)\n    out(x)\n";

    for backend in Backend::all() {
        let mut args = vec!["--max-range-size", "1000"];
        if backend == Backend::Interpreter {
            args.push("--interp");
        }

        let (_stdout, stderr, code) = run_source_with_args(script, &args, &[]);
        assert_eq!(
            code,
            1,
            "backend {} should exit with code 1",
            backend.name()
        );
        assert!(
            stderr.contains("Range size of 5000000 exceeds maximum allowed limit of 1000."),
            "backend {} should report range size error, got: {}",
            backend.name(),
            stderr
        );
    }
}

#[test]
fn max_steps_halts_infinite_loops() {
    let script = "x = 0\nwhile true\n    x = x + 1\n";

    for backend in Backend::all() {
        let mut args = vec!["--max-steps", "100"];
        if backend == Backend::Interpreter {
            args.push("--interp");
        }

        let (_stdout, stderr, code) = run_source_with_args(script, &args, &[]);
        assert_eq!(
            code,
            1,
            "backend {} should exit with code 1",
            backend.name()
        );
        assert!(
            stderr.contains("Execution step limit of 100 exceeded."),
            "backend {} should report step limit error, got: {}",
            backend.name(),
            stderr
        );
    }
}

#[test]
fn max_output_bytes_halts_runaway_output() {
    let script = "while true\n    out(\"spam spam spam spam\")\n";

    for backend in Backend::all() {
        let mut args = vec!["--max-output-bytes", "50"];
        if backend == Backend::Interpreter {
            args.push("--interp");
        }

        let (_stdout, stderr, code) = run_source_with_args(script, &args, &[]);
        assert_eq!(
            code,
            1,
            "backend {} should exit with code 1",
            backend.name()
        );
        assert!(
            stderr.contains("Output volume limit of 50 bytes exceeded."),
            "backend {} should report output volume error, got: {}",
            backend.name(),
            stderr
        );
    }
}

#[test]
fn conservative_flag_applies_conservative_defaults() {
    let script = "while true\n    x = 1\n";

    let (_stdout, stderr, code) = run_source_with_args(script, &["--conservative"], &[]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("Execution step limit of 100000 exceeded."),
        "conservative should set 100k step limit: {}",
        stderr
    );
}

#[test]
fn env_variables_configure_limits() {
    let script = "recurse()\n    recurse()\n\nrecurse()\n";

    let envs = [("GUPPTY_MAX_CALL_DEPTH", "20")];
    let (_stdout, stderr, code) = run_source_with_args(script, &[], &envs);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("Maximum call stack depth of 20 exceeded."),
        "env var GUPPTY_MAX_CALL_DEPTH should take effect: {}",
        stderr
    );
}
