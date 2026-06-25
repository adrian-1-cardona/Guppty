// === ast.rs ===
// AST stands for "Abstract Syntax Tree."
// Think of it like a family tree, but for your code!
//
// After we cut the code into tokens (LEGO pieces), we need to
// figure out how they fit together. The AST is like the instruction
// booklet that shows which pieces connect to which.
//
// For example: out("Hello World!")
// The tree looks like:
//
//   Statement: ExpressionStatement
//       └── Expression: FunctionCall
//               ├── name: "out"
//               └── arguments:
//                       └── Expression: StringLiteral("Hello World!")
//
// It's like saying: "Do this thing → call 'out' with 'Hello World!'"

/// An Expression is something that HAS a value.
/// Like "Hello World!" is a value (it's text).
/// And out("Hello") is also a value (it calls a function).
#[derive(Debug, Clone)]
pub enum Expr {
    // A piece of text, like "Hello World!"
    // The String inside holds the actual text content.
    StringLiteral(String),

    // A function call, like out("Hello World!")
    // It has a name (like "out") and a list of things we pass to it (arguments).
    FunctionCall {
        name: String,       // The name of the function (e.g., "out")
        args: Vec<Expr>,    // The stuff inside the parentheses
    },
}

/// A Statement is a complete instruction — like a full sentence.
/// It tells the computer to DO something.
#[derive(Debug, Clone)]
pub enum Stmt {
    // An expression used as a statement.
    // This is when you write something like: out("Hello World!")
    // You're not saving the result anywhere, you just want it to happen.
    ExpressionStatement(Expr),
}

/// A Program is just a list of statements.
/// It's like a to-do list for the computer — do this, then this, then this.
pub type Program = Vec<Stmt>;
