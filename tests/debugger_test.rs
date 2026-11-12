use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, process};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn run_debug_commands(source: &str, commands: &str) -> (String, String, bool) {
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut path = std::env::temp_dir();
    path.push(format!("guppty-debug-test-{}-{}.gup", process::id(), count));
    fs::write(&path, source).expect("failed to write temp gup file");

    let bin = env!("CARGO_BIN_EXE_guppty");
    let output = Command::new(bin)
        .args(["debug", path.to_str().unwrap(), "--commands", commands])
        .output()
        .expect("failed to run guppty debug");

    let _ = fs::remove_file(&path);
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

#[test]
fn debug_line_breakpoint_and_variable_inspection() {
    let source = "run_me()\n    x = 10\n    y = 20\n    out(x + y)\n\nrun_me()\n";
    let (stdout, _stderr, success) = run_debug_commands(source, "b 3; c; p x; locals; c");

    assert!(success);
    assert!(stdout.contains("Breakpoint 1 set at"));
    assert!(stdout.contains("[Hit breakpoint 1 at"));
    assert!(stdout.contains("x = 10"));
    assert!(stdout.contains("Local variables:"));
    assert!(stdout.contains("30"));
}

#[test]
fn debug_stepping_into_and_over_functions() {
    let source = "add(a, b)\n    return a + b\n\nout(add(3, 4))\n";

    // Step into add()
    let (stdout_step, _, success_step) = run_debug_commands(source, "s; s; bt; locals; c");
    assert!(success_step);
    assert!(stdout_step.contains("in add"));
    assert!(stdout_step.contains("a = 3"));
    assert!(stdout_step.contains("b = 4"));
    assert!(stdout_step.contains("#1 add(a=3, b=4)"));

    // Next (step over) add()
    let (stdout_next, _, success_next) = run_debug_commands(source, "s; n; c");
    assert!(success_next);
    assert!(stdout_next.contains("7"));
}

#[test]
fn debug_finish_step_out() {
    let source = "compute(x)\n    val = x * 2\n    return val\n\nout(compute(5))\n";
    let (stdout, _, success) = run_debug_commands(source, "s; s; f; bt; c");

    assert!(success);
    assert!(stdout.contains("[Finish]"));
    assert!(stdout.contains("#0 <main>"));
    assert!(stdout.contains("10"));
}

#[test]
fn debug_function_breakpoint() {
    let source = "greet(name)\n    out(name)\n\ngreet(\"hello\")\n";
    let (stdout, _, success) = run_debug_commands(source, "b greet; c; bt; c");

    assert!(success);
    assert!(stdout.contains("Breakpoint 1 set for function 'greet'"));
    assert!(stdout.contains("[Hit breakpoint 1 entering function 'greet']"));
    assert!(stdout.contains("#1 greet(name=hello)"));
    assert!(stdout.contains("hello"));
}

#[test]
fn debug_instruction_stepping_and_stack_inspection() {
    let source = "x = 42\nout(x)\n";
    let (stdout, _, success) = run_debug_commands(source, "s; si; stack; c");

    assert!(success);
    assert!(stdout.contains("[Single instruction step]"));
    assert!(stdout.contains("Operand stack"));
    assert!(stdout.contains("42"));
}

#[test]
fn debug_disassembly_command() {
    let source = "x = 10\nout(x)\n";
    let (stdout, _, success) = run_debug_commands(source, "disasm; c");

    assert!(success);
    assert!(stdout.contains("== Bytecode for <main> =="));
    assert!(stdout.contains("Constant"));
    assert!(stdout.contains("Print"));
}

#[test]
fn debug_backtrace_recursive_calls() {
    let source = "fib(n)\n    if n <= 1\n        return n\n    return fib(n - 1) + fib(n - 2)\n\nout(fib(3))\n";
    let (stdout, _, success) = run_debug_commands(source, "b 2; c; c; bt; c");

    assert!(success);
    assert!(stdout.contains("Call stack:"));
    assert!(stdout.contains("fib(n="));
    assert!(stdout.contains("<main>"));
}

#[test]
fn debug_post_mortem_on_runtime_error() {
    let source = "x = 10\nout(x / 0)\n";
    let (stdout, stderr, success) = run_debug_commands(source, "c; bt; locals; q");

    assert!(!success);
    assert!(stdout.contains("*** Unhandled Runtime Error ***") || stderr.contains("MathError"));
    assert!(stdout.contains("Call stack:"));
    assert!(stdout.contains("x = 10"));
}

#[test]
fn debug_cli_rejects_unknown_options() {
    let bin = env!("CARGO_BIN_EXE_guppty");
    let output = Command::new(bin)
        .args(["debug", "examples/hello.gup", "--invalid-flag"])
        .output()
        .expect("run debug with invalid flag");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown option: --invalid-flag"));
}
