// === main.rs ===
// This is the STARTING POINT of the whole Guppty language!
// When you type "guppty hello.gup" in your terminal, THIS is what runs first.
//
// Think of it like the front door of a house:
//   1. You knock (run the command)
//   2. It opens the door (reads your .gup file)
//   3. It sends your code through the pipeline:
//        Source Code → Lexer → Parser → Compiler → VM → stdout
//        (text)       (tokens) (tree)   (bytecode) (robot!)
//
// Use --interp if you want the older tree-walking interpreter instead.

mod ast;
mod bytecode;
mod compiler;
mod environment;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod syntax;
mod token;
mod value;
mod vm;

use std::env;
use std::fs;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    eprintln!("Usage: guppty <file.gup> [--interp]");
    eprintln!("Example: guppty hello.gup");
    std::process::exit(1);
  }

  let use_interpreter = args.iter().any(|arg| arg == "--interp");
  let filename = args
    .iter()
    .skip(1)
    .find(|arg| !arg.starts_with("--"))
    .expect("filename argument");

  let source = fs::read_to_string(filename).unwrap_or_else(|e| {
    eprintln!("Oops! I couldn't read the file '{}': {}", filename, e);
    std::process::exit(1);
  });

  let tokens = lexer::lex(&source).unwrap_or_else(|error| {
    eprintln!("{}", error.render(filename, &source));
    std::process::exit(1);
  });

  let program = parser::parse(tokens).unwrap_or_else(|error| {
    eprintln!("{}", error.render(filename, &source));
    std::process::exit(1);
  });

  if use_interpreter {
    interpreter::interpret(program).unwrap_or_else(|error| {
      eprintln!("{}", error.render(filename, &source));
      std::process::exit(1);
    });
  } else {
    let script = compiler::compile(&program).unwrap_or_else(|error| {
      eprintln!("{}", error.render(filename, &source));
      std::process::exit(1);
    });
    vm::run(script).unwrap_or_else(|error| {
      eprintln!("{}", error.render(filename, &source));
      std::process::exit(1);
    });
  }
}
