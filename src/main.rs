//CLI + runs the pipeline
mod token;
mod lexer;
mod ast;
mod parser;
mod value;
mod interpreter;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: guppty <file.gup>");
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", filename, e);
        std::process::exit(1);
    });

    let tokens = lexer::lex(&source);
    let program = parser::parse(tokens);
    interpreter::interpret(program);
}
