// =============================================================================
// GUPPTY REUSABLE LIBRARY API TESTS
// =============================================================================
// Tests that tests and external tools can call each pipeline phase in memory
// without launching a separate process or reaching into private internals.

use guppty::{
    check, compile, execute, execute_instrumented, interpret, lex, parse, run_vm, ExecutionEngine,
    VERSION,
};

#[test]
fn lex_phase_tokenizes_in_memory() {
    let source = "x = 42\nout(x)\n";
    let tokens = lex(source).expect("lexing must succeed");
    assert!(!tokens.is_empty());
    assert_eq!(tokens[0].span.line, 1);
}

#[test]
fn lex_phase_returns_structured_error() {
    let source = "x = !\n";
    let error = lex(source).expect_err("lonely exclamation should fail lexing");
    assert!(error.message.contains("lonely '!'"));
    assert_eq!(error.span.line, 1);
}

#[test]
fn parse_phase_constructs_ast() {
    let source = "add(a, b)\n    return a + b\n\nout(add(1, 2))\n";
    let tokens = lex(source).expect("lexing must succeed");
    let program = parse(tokens).expect("parsing must succeed");
    assert_eq!(program.len(), 2);
}

#[test]
fn compile_phase_produces_bytecode_chunk() {
    let source = "x = 10 + 20\nout(x)\n";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let compiled = compile(&program).expect("compilation must succeed");

    let function = compiled.borrow();
    assert!(!function.chunk.code.is_empty());
}

#[test]
fn run_vm_executes_compiled_code() {
    let source = "math()\n    x = 15\n    y = 25\n    out(x + y)\n\nmath()\n";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let compiled = compile(&program).unwrap();

    let result = run_vm(compiled);
    assert!(result.is_ok());
}

#[test]
fn interpret_phase_executes_ast_directly() {
    let source = "calc()\n    a = 100\n    b = 20\n    out(a / b)\n\ncalc()\n";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();

    let result = interpret(program);
    assert!(result.is_ok());
}

#[test]
fn check_validates_without_execution() {
    let source = "valid()\n    return 42\nvalid()\n";
    assert!(check(source).is_ok());

    let invalid = "valid(\n";
    assert!(check(invalid).is_err());
}

#[test]
fn execute_runs_end_to_end_on_both_engines() {
    let source = "say()\n    out(\"library execution works!\")\nsay()\n";

    assert!(execute(source, ExecutionEngine::Vm).is_ok());
    assert!(execute(source, ExecutionEngine::Interpreter).is_ok());
}

#[test]
fn instrumentation_reports_precise_telemetry_for_vm() {
    let source = "total = 0\nfor i in range(1 through 10)\n    total = total + i\nout(total)\n";

    let stats = execute_instrumented(source, ExecutionEngine::Vm).expect("telemetry run succeeded");
    assert_eq!(stats.compiler_version, VERSION);
    assert_eq!(stats.engine, ExecutionEngine::Vm);
    assert!(stats.token_count > 0);
    assert!(stats.ast_statement_count > 0);
    assert!(stats.compile_duration.is_some());
    assert!(stats.bytecode_bytes.is_some_and(|bytes| bytes > 0));
    assert!(stats.total_duration >= stats.execute_duration);
}

#[test]
fn instrumentation_reports_precise_telemetry_for_interpreter() {
    let source = "compute()\n    out(\"interpreter telemetry\")\ncompute()\n";

    let stats = execute_instrumented(source, ExecutionEngine::Interpreter)
        .expect("telemetry run succeeded");
    assert_eq!(stats.compiler_version, VERSION);
    assert_eq!(stats.engine, ExecutionEngine::Interpreter);
    assert!(stats.token_count > 0);
    assert!(stats.ast_statement_count > 0);
    assert!(stats.compile_duration.is_none());
    assert!(stats.bytecode_bytes.is_none());
}
