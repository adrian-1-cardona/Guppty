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
// Subcommands:
//   guppty new <name>          create a fresh .gup program
//   guppty compile <file.gup>  compile to bytecode (check for errors)
//   guppty run <file.gup>      run a program (same as guppty <file.gup>)
//   guppty help                show usage
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
use std::path::{Path, PathBuf};
use std::process;

fn main() {
  let args: Vec<String> = env::args().skip(1).collect();

  if args.is_empty() || is_help_flag(&args[0]) {
    print_help();
    if args.is_empty() {
      process::exit(1);
    }
    return;
  }

  match args[0].as_str() {
    "help" | "--help" | "-h" => print_help(),
    "new" => cmd_new(&args[1..]),
    "compile" => cmd_compile(&args[1..]),
    "run" => cmd_run(&args[1..]),
    // Backward compatible: `guppty file.gup [--interp]`
    other if looks_like_source_path(other) => {
      let use_interpreter = args.iter().any(|arg| arg == "--interp");
      run_file(other, use_interpreter);
    }
    other if other.starts_with('-') => {
      eprintln!("Unknown flag: {}", other);
      eprintln!();
      print_help();
      process::exit(1);
    }
    other => {
      eprintln!("Unknown command: {}", other);
      eprintln!();
      print_help();
      process::exit(1);
    }
  }
}

fn is_help_flag(arg: &str) -> bool {
  matches!(arg, "help" | "--help" | "-h")
}

fn looks_like_source_path(arg: &str) -> bool {
  arg.ends_with(".gup")
    || arg.contains('/')
    || arg.contains('\\')
    || arg.starts_with('.')
    || Path::new(arg).exists()
}

fn print_help() {
  println!(
    "\
Guppty — a small indentation-based programming language

Usage:
  guppty <file.gup> [--interp]     Run a .gup program (VM by default)
  guppty run <file.gup> [--interp] Same as above
  guppty new <name>                Create a fresh .gup program
  guppty compile <file.gup>        Compile to bytecode (no run)
  guppty help                      Show this help

Examples:
  guppty new hello
  guppty compile hello.gup
  guppty hello.gup

Write programs in files ending with .gup, then compile or run them.
Docs: https://github.com/adrian-1-cardona/guppty"
  );
}

fn cmd_new(args: &[String]) {
  if args.is_empty() {
    eprintln!("Usage: guppty new <name>");
    eprintln!("Example: guppty new hello");
    process::exit(1);
  }

  let raw_name = args[0].trim();
  if raw_name.is_empty() {
    eprintln!("Please give your program a name, like: guppty new hello");
    process::exit(1);
  }

  let path = program_path_from_name(raw_name);
  if path.exists() {
    eprintln!(
      "Oops! '{}' already exists. Pick a new name or delete the old file first.",
      path.display()
    );
    process::exit(1);
  }

  if let Some(parent) = path.parent() {
    if !parent.as_os_str().is_empty() {
      fs::create_dir_all(parent).unwrap_or_else(|e| {
        eprintln!("Couldn't create folder '{}': {}", parent.display(), e);
        process::exit(1);
      });
    }
  }

  let display_name = path
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("program");

  let template = format!(
    r#"// {display_name}.gup — your Guppty program
// Tip: indent with spaces. Run with: guppty {file}
// Compile-check with: guppty compile {file}

out("Hello from {display_name}!")
out("Edit this file, then run: guppty {file}")
"#,
    display_name = display_name,
    file = path.display()
  );

  fs::write(&path, template).unwrap_or_else(|e| {
    eprintln!("Couldn't write '{}': {}", path.display(), e);
    process::exit(1);
  });

  println!("Created {}", path.display());
  println!();
  println!("Next:");
  println!("  guppty compile {}", path.display());
  println!("  guppty {}", path.display());
}

fn program_path_from_name(name: &str) -> PathBuf {
  let path = PathBuf::from(name);
  if name.ends_with(".gup") {
    path
  } else {
    path.with_extension("gup")
  }
}

fn cmd_compile(args: &[String]) {
  let filename = require_file_arg(args, "compile");
  let source = read_source(filename);
  let script = compile_source(filename, &source);

  let code_bytes = script.borrow().chunk.code.len();
  let constants = script.borrow().chunk.constants.len();

  println!("Compiled {} successfully.", filename);
  println!("  bytecode:  {} bytes", code_bytes);
  println!("  constants: {}", constants);
  println!();
  println!("Run it with: guppty {}", filename);
}

fn cmd_run(args: &[String]) {
  let filename = require_file_arg(args, "run");
  let use_interpreter = args.iter().any(|arg| arg == "--interp");
  run_file(filename, use_interpreter);
}

fn require_file_arg<'a>(args: &'a [String], command: &str) -> &'a str {
  let filename = args
    .iter()
    .find(|arg| !arg.starts_with("--"))
    .map(String::as_str);

  match filename {
    Some(name) => name,
    None => {
      eprintln!("Usage: guppty {} <file.gup>", command);
      eprintln!("Example: guppty {} hello.gup", command);
      process::exit(1);
    }
  }
}

fn read_source(filename: &str) -> String {
  fs::read_to_string(filename).unwrap_or_else(|e| {
    eprintln!("Oops! I couldn't read the file '{}': {}", filename, e);
    process::exit(1);
  })
}

fn compile_source(filename: &str, source: &str) -> bytecode::RcCompiledFunction {
  let tokens = lexer::lex(source).unwrap_or_else(|error| {
    eprintln!("{}", error.render(filename, source));
    process::exit(1);
  });

  let program = parser::parse(tokens).unwrap_or_else(|error| {
    eprintln!("{}", error.render(filename, source));
    process::exit(1);
  });

  compiler::compile(&program).unwrap_or_else(|error| {
    eprintln!("{}", error.render(filename, source));
    process::exit(1);
  })
}

fn run_file(filename: &str, use_interpreter: bool) {
  let source = read_source(filename);

  let tokens = lexer::lex(&source).unwrap_or_else(|error| {
    eprintln!("{}", error.render(filename, &source));
    process::exit(1);
  });

  let program = parser::parse(tokens).unwrap_or_else(|error| {
    eprintln!("{}", error.render(filename, &source));
    process::exit(1);
  });

  if use_interpreter {
    interpreter::interpret(program).unwrap_or_else(|error| {
      eprintln!("{}", error.render(filename, &source));
      process::exit(1);
    });
  } else {
    let script = compiler::compile(&program).unwrap_or_else(|error| {
      eprintln!("{}", error.render(filename, &source));
      process::exit(1);
    });
    vm::run(script).unwrap_or_else(|error| {
      eprintln!("{}", error.render(filename, &source));
      process::exit(1);
    });
  }
}
