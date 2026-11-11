// === lib.rs ===
//! # Guppty Language Pipeline & Research API
//!
//! Guppty provides a staged compilation and execution pipeline that can be
//! invoked end-to-end or phase-by-phase without launching a separate process:
//!
//! 1. **Lexing**: `guppty::lex(source)` -> `Vec<Token>`
//! 2. **Parsing**: `guppty::parse(tokens)` -> `Program` (AST)
//! 3. **Compilation**: `guppty::compile(&program)` -> `RcCompiledFunction` (Bytecode)
//! 4. **Execution**:
//!    - Bytecode VM: `guppty::run_vm(compiled)`
//!    - Tree-walking Interpreter: `guppty::interpret(program)`
//!
//! Research tools and benchmarks can also record structured phase timings and
//! metrics using `guppty::execute_instrumented(source, engine)`.

pub mod ast;
pub mod bytecode;
pub mod compiler;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod value;
pub mod vm;

pub(crate) mod environment;
pub(crate) mod syntax;

use std::time::{Duration, Instant};

pub use ast::Program;
pub use bytecode::RcCompiledFunction;
pub use error::{ErrorKind, ErrorPhase, GupError, Span};
pub use token::Token;
pub use value::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Execution backend engine choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionEngine {
    #[default]
    Vm,
    Interpreter,
}

/// Structured execution and compilation telemetry recorded across pipeline phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrumentation {
    /// The Guppty compiler/runtime package version.
    pub compiler_version: String,
    /// Backend engine used for execution.
    pub engine: ExecutionEngine,
    /// Wall-clock duration spent lexing tokens.
    pub lex_duration: Duration,
    /// Number of lexical tokens produced.
    pub token_count: usize,
    /// Wall-clock duration spent parsing tokens into an AST.
    pub parse_duration: Duration,
    /// Number of top-level AST statements parsed.
    pub ast_statement_count: usize,
    /// Wall-clock duration spent compiling the AST to bytecode (VM backend only).
    pub compile_duration: Option<Duration>,
    /// Total bytecode payload size in bytes (VM backend only).
    pub bytecode_bytes: Option<usize>,
    /// Wall-clock duration spent executing the program.
    pub execute_duration: Duration,
    /// Total wall-clock duration from start of lexing to completion of execution.
    pub total_duration: Duration,
}

/// Tokenize Guppty source text into a stream of lexical tokens.
pub fn lex(source: &str) -> Result<Vec<Token>, GupError> {
    lexer::lex(source)
}

/// Parse a stream of tokens into a Guppty AST program.
pub fn parse(tokens: Vec<Token>) -> Result<Program, GupError> {
    parser::parse(tokens)
}

/// Compile a Guppty AST program into runnable bytecode for the VM.
pub fn compile(program: &Program) -> Result<RcCompiledFunction, GupError> {
    compiler::compile(program)
}

/// Run compiled bytecode on the Guppty virtual machine.
pub fn run_vm(script: RcCompiledFunction) -> Result<(), GupError> {
    vm::run(script)
}

/// Run an AST program directly using the tree-walking interpreter.
pub fn interpret(program: Program) -> Result<(), GupError> {
    interpreter::interpret(program)
}

/// Statically check a source string through lexing, parsing, and bytecode compilation without executing it.
pub fn check(source: &str) -> Result<(), GupError> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    let _ = compile(&program)?;
    Ok(())
}

/// Execute a Guppty source string end-to-end with the specified backend engine.
pub fn execute(source: &str, engine: ExecutionEngine) -> Result<(), GupError> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    match engine {
        ExecutionEngine::Vm => {
            let script = compile(&program)?;
            run_vm(script)
        }
        ExecutionEngine::Interpreter => interpret(program),
    }
}

/// Execute a Guppty source string and record precise phase telemetry for research experiments.
pub fn execute_instrumented(
    source: &str,
    engine: ExecutionEngine,
) -> Result<Instrumentation, GupError> {
    let total_start = Instant::now();

    let lex_start = Instant::now();
    let tokens = lex(source)?;
    let lex_duration = lex_start.elapsed();
    let token_count = tokens.len();

    let parse_start = Instant::now();
    let program = parse(tokens)?;
    let parse_duration = parse_start.elapsed();
    let ast_statement_count = program.len();

    match engine {
        ExecutionEngine::Vm => {
            let compile_start = Instant::now();
            let script = compile(&program)?;
            let compile_duration = Some(compile_start.elapsed());
            let bytecode_bytes = Some(script.borrow().chunk.code.len());

            let exec_start = Instant::now();
            run_vm(script)?;
            let execute_duration = exec_start.elapsed();

            Ok(Instrumentation {
                compiler_version: VERSION.to_string(),
                engine,
                lex_duration,
                token_count,
                parse_duration,
                ast_statement_count,
                compile_duration,
                bytecode_bytes,
                execute_duration,
                total_duration: total_start.elapsed(),
            })
        }
        ExecutionEngine::Interpreter => {
            let exec_start = Instant::now();
            interpret(program)?;
            let execute_duration = exec_start.elapsed();

            Ok(Instrumentation {
                compiler_version: VERSION.to_string(),
                engine,
                lex_duration,
                token_count,
                parse_duration,
                ast_statement_count,
                compile_duration: None,
                bytecode_bytes: None,
                execute_duration,
                total_duration: total_start.elapsed(),
            })
        }
    }
}
