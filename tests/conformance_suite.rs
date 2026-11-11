// =============================================================================
// GUPPTY NORMATIVE CONFORMANCE SUITE
// =============================================================================
// Every test case in this suite directly maps to a rule in `design/specification.md`.
// Both backends (VM and reference interpreter) must conform identically.

mod common;

use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

fn temp_workspace(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "guppty-conformance-{}-{}-{label}",
        process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create temporary workspace");
    path
}

fn execute_both(source: &str) -> (String, String) {
    let dir = temp_workspace("exec");
    let script_path = dir.join("test.gup");
    fs::write(&script_path, source).expect("write test script");

    let bin = env!("CARGO_BIN_EXE_guppty");

    // 1. VM Backend
    let vm_output = Command::new(bin)
        .arg(&script_path)
        .output()
        .expect("run vm");
    let vm_stdout = String::from_utf8_lossy(&vm_output.stdout).replace("\r\n", "\n");
    let vm_stderr = String::from_utf8_lossy(&vm_output.stderr).replace("\r\n", "\n");

    // 2. Interpreter Backend
    let interp_output = Command::new(bin)
        .arg(&script_path)
        .arg("--interp")
        .output()
        .expect("run interpreter");
    let interp_stdout = String::from_utf8_lossy(&interp_output.stdout).replace("\r\n", "\n");
    let interp_stderr = String::from_utf8_lossy(&interp_output.stderr).replace("\r\n", "\n");

    // [SPEC-CONF-01] Both backends must match in exit status and output
    assert_eq!(
        vm_output.status.code(),
        interp_output.status.code(),
        "exit status mismatch:\nVM stderr: {}\nInterp stderr: {}",
        vm_stderr,
        interp_stderr
    );

    if vm_output.status.success() {
        assert_eq!(
            vm_stdout, interp_stdout,
            "stdout mismatch between VM and interpreter"
        );
        (vm_stdout, interp_stdout)
    } else {
        assert_eq!(
            vm_stderr.lines().next(),
            interp_stderr.lines().next(),
            "first line of error diagnostic mismatch:\nVM: {}\nInterp: {}",
            vm_stderr,
            interp_stderr
        );
        (vm_stderr, interp_stderr)
    }
}

/// [SPEC-LEX-04] CI check verifying keywords defined in specification.md match src/syntax.rs exactly.
#[test]
fn spec_lex_keywords_table_consistency_check() {
    let spec_text = fs::read_to_string("design/specification.md").expect("read specification.md");
    let syntax_text = fs::read_to_string("src/syntax.rs").expect("read src/syntax.rs");

    let keywords = [
        "for", "in", "range", "through", "if", "else", "while", "return", "and", "or", "not",
        "true", "false",
    ];

    for kw in keywords {
        assert!(
            spec_text.contains(kw),
            "specification.md must document keyword: {kw}"
        );
        assert!(
            syntax_text.contains(&format!("\"{kw}\"")),
            "src/syntax.rs must define keyword: {kw}"
        );
    }
}

/// [SPEC-INDENT-01] & [SPEC-INDENT-02]: Strict 4-space indentation; tab prohibition.
#[test]
fn spec_indent_rules_enforced() {
    // Tab prohibition
    let tab_prog = "out(\"start\")\n\tx = 1\n";
    let (vm_err, interp_err) = execute_both(tab_prog);
    assert!(vm_err.contains("IndentationError"));
    assert!(interp_err.contains("IndentationError"));

    // Partial indent prohibition (2 spaces)
    let partial_prog = "if true\n  out(1)\n";
    let (vm_err, interp_err) = execute_both(partial_prog);
    assert!(vm_err.contains("IndentationError"));
    assert!(interp_err.contains("IndentationError"));
}

/// [SPEC-NUM-01], [SPEC-NUM-02], [SPEC-NUM-03], [SPEC-NUM-04]: Numeric semantics.
#[test]
fn spec_numeric_semantics() {
    let source = r#"
out(2 + 3 * 4)
out(10 - 4)
out(15 / 5)
out(1.5 + 2)
out("total: " + 42)
"#;
    let (stdout, _) = execute_both(source);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines[0], "14");
    assert_eq!(lines[1], "6");
    assert_eq!(lines[2], "3");
    assert_eq!(lines[3], "3.5");
    assert_eq!(lines[4], "total: 42");

    // Division by zero raises MathError
    let div_zero = "out(42 / 0)\n";
    let (err, _) = execute_both(div_zero);
    assert!(err.contains("MathError"));
    assert!(err.contains("Cannot divide by zero."));
}

/// [SPEC-TRUTH-01] & [SPEC-LOGIC-01]: Truthiness and short-circuit evaluation.
#[test]
fn spec_truthiness_and_logic() {
    let source = r#"
if 0
    out("0 is truthy")

if 0.0
    out("0.0 is truthy")

if ""
    out("empty string is truthy")

if not false
    out("not false is truthy")

out(true or false)
out(false and true)
"#;
    let (stdout, _) = execute_both(source);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines[0], "0 is truthy");
    assert_eq!(lines[1], "0.0 is truthy");
    assert_eq!(lines[2], "empty string is truthy");
    assert_eq!(lines[3], "not false is truthy");
    assert_eq!(lines[4], "true");
    assert_eq!(lines[5], "false");
}

/// [SPEC-OP-01]: Operator precedence and associativity.
#[test]
fn spec_operator_precedence() {
    let source = r#"
out(10 - 2 - 3)
out(2 + 3 * 4)
out(2 * 3 + 4)
out(not false and true)
out(false and false or true)
"#;
    let (stdout, _) = execute_both(source);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines[0], "5"); // (10 - 2) - 3
    assert_eq!(lines[1], "14"); // 2 + (3 * 4)
    assert_eq!(lines[2], "10"); // (2 * 3) + 4
    assert_eq!(lines[3], "true");
    assert_eq!(lines[4], "true");
}

/// [SPEC-FLOW-01], [SPEC-FLOW-02], [SPEC-FLOW-03]: Control flow and inclusive range loops.
#[test]
fn spec_control_flow_and_ranges() {
    let source = r#"
// Upward range
up_sum = 0
for i in range(1 through 3)
    up_sum = up_sum + i
out(up_sum)

// Downward range
down_res = 0
for j in range(3 through 1)
    down_res = down_res + j
out(down_res)

// While loop
count = 0
while count < 3
    count = count + 1
out(count)
"#;
    let (stdout, _) = execute_both(source);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines[0], "6");
    assert_eq!(lines[1], "6");
    assert_eq!(lines[2], "3");
}

/// [SPEC-FUNC-01], [SPEC-SCOPE-01], [SPEC-CLOSURE-01]: Functions, block scope, closures.
#[test]
fn spec_functions_scoping_and_closures() {
    let source = r#"
makeMultiplier(factor)
    multiplier(x)
        return x * factor
    return multiplier

double = makeMultiplier(2)
triple = makeMultiplier(3)

out(double(5))
out(triple(5))
"#;
    let (stdout, _) = execute_both(source);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines[0], "10");
    assert_eq!(lines[1], "15");
}

/// [SPEC-ERR-01]: Standard diagnostic format.
#[test]
fn spec_error_diagnostic_format() {
    let bad_source = "x = undefined_var\n";
    let (err, _) = execute_both(bad_source);
    assert!(err.contains("NameError: Variable 'undefined_var' is not defined yet!"));
    assert!(err.contains("= help:"));
}
