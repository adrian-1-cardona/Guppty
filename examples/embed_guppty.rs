//! Minimal external consumer example embedding Guppty as a library.
//!
//! Demonstrates:
//! 1. Phase-by-phase execution (lex -> parse -> compile -> run_vm)
//! 2. Directly interpreting an AST
//! 3. Collecting research instrumentation metrics across phases

use guppty::{compile, execute_instrumented, interpret, lex, parse, run_vm, ExecutionEngine};

fn main() {
    let source = r#"
math()
    x = 10
    y = 32
    out("Result from embedded Guppty:", x + y)

math()
"#;

    println!("--- 1. Phase-by-phase execution (VM) ---");
    let tokens = lex(source).expect("lexing failed");
    println!("Lexed {} tokens", tokens.len());

    let program = parse(tokens).expect("parsing failed");
    println!("Parsed {} AST statements", program.len());

    let bytecode = compile(&program).expect("compilation failed");
    println!(
        "Compiled bytecode ({} bytes)",
        bytecode.borrow().chunk.code.len()
    );

    print!("Output: ");
    run_vm(bytecode).expect("VM run failed");

    println!("\n--- 2. Direct AST Interpretation ---");
    let tokens = lex(source).expect("lexing failed");
    let program = parse(tokens).expect("parsing failed");
    print!("Output: ");
    interpret(program).expect("interpreter run failed");

    println!("\n--- 3. Research Instrumentation ---");
    let metrics =
        execute_instrumented(source, ExecutionEngine::Vm).expect("instrumented run failed");
    println!("Guppty Version:    {}", metrics.compiler_version);
    println!(
        "Lex Duration:      {:?} ({} tokens)",
        metrics.lex_duration, metrics.token_count
    );
    println!(
        "Parse Duration:    {:?} ({} statements)",
        metrics.parse_duration, metrics.ast_statement_count
    );
    println!(
        "Compile Duration:  {:?} ({} bytes)",
        metrics.compile_duration.unwrap(),
        metrics.bytecode_bytes.unwrap()
    );
    println!("Execute Duration:  {:?}", metrics.execute_duration);
    println!("Total Duration:    {:?}", metrics.total_duration);
}
