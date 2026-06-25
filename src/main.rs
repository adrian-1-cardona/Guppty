// === main.rs ===
// This is the STARTING POINT of the whole Guppty language!
// When you type "guppty hello.gup" in your terminal, THIS is what runs first.
//
// Think of it like the front door of a house:
//   1. You knock (run the command)
//   2. It opens the door (reads your .gup file)
//   3. It sends your code through the pipeline:
//        Source Code → Lexer → Parser → Interpreter
//        (text)       (tokens) (tree)   (actually does stuff!)
//
// That's it! The main.rs just connects everything together.

// --- These lines tell Rust "hey, I have code in these other files, use them!" ---
mod token;        // our LEGO piece types (what kinds of tokens exist)
mod syntax;       // the menu of special words — change syntax in ONE place!
mod lexer;        // the cookie cutter (breaks code into tokens)
mod ast;          // the family tree structure (how code pieces relate)
mod parser;       // the detective (figures out what tokens mean together)
mod environment;  // the boxes that hold variables (scopes!)
mod value;        // the runtime values (actual data when program runs)
mod interpreter;  // the actor (reads the script and performs it)

// --- We need these tools from Rust's standard library ---
use std::env;    // Lets us read command-line arguments (like the filename)
use std::fs;     // Lets us read files from disk

fn main() {
    // Step 1: Grab the command-line arguments
    // When you type "guppty hello.gup", args will be:
    //   args[0] = "guppty"      (the program name itself)
    //   args[1] = "hello.gup"   (the file you want to run)
    let args: Vec<String> = env::args().collect();

    // Step 2: Make sure they actually gave us a file to run!
    // If they just typed "guppty" with no file, tell them how to use it.
    if args.len() < 2 {
        eprintln!("Usage: guppty <file.gup>");
        eprintln!("Example: guppty hello.gup");
        std::process::exit(1);
    }

    // Step 3: Get the filename from the arguments
    let filename = &args[1];

    // Step 4: Read the file's contents into a string
    // If the file doesn't exist or can't be read, show a helpful error
    let source = fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Oops! I couldn't read the file '{}': {}", filename, e);
        std::process::exit(1);
    });

    // Step 5: THE PIPELINE — this is where the magic happens!

    // 5a: LEXER — chop the source code into tokens (LEGO pieces)
    let tokens = lexer::lex(&source);

    // 5b: PARSER — arrange the tokens into a tree (figure out the structure)
    let program = parser::parse(tokens);

    // 5c: INTERPRETER — walk the tree and actually DO what the code says!
    interpreter::interpret(program);
}
