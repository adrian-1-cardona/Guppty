// === interpreter.rs ===
// The interpreter is the part that actually DOES the work!
//
// Think of it like an actor reading a script:
//   - The script (AST) says "call out with Hello World!"
//   - The interpreter reads that and actually prints "Hello World!" to the screen
//
// It walks through the AST tree, node by node, and performs
// the action each node describes. This is where the magic happens!

use crate::ast::{Expr, Stmt, Program};
use crate::value::Value;

/// This is the main function — give it a program (list of statements)
/// and it will execute every single one, in order, from top to bottom.
/// Like reading a book page by page.
pub fn interpret(program: Program) {
    // Go through each statement one at a time and run it
    for statement in program {
        execute_statement(&statement);
    }
}

/// Execute ONE statement.
/// Right now, the only kind of statement we have is an ExpressionStatement,
/// which just means "evaluate this expression and move on."
fn execute_statement(stmt: &Stmt) {
    match stmt {
        // An expression statement — just run the expression!
        Stmt::ExpressionStatement(expr) => {
            // We run it but don't need to keep the result
            // (like calling out("hi") — we just want the side effect of printing)
            evaluate_expression(expr);
        }
    }
}

/// Evaluate ONE expression and figure out what value it produces.
/// This is where we actually compute things!
fn evaluate_expression(expr: &Expr) -> Value {
    match expr {
        // --- STRING LITERAL ---
        // If it's a string like "Hello World!", just return it as a value.
        // Easy peasy!
        Expr::StringLiteral(text) => {
            Value::GuppyString(text.clone())
        }

        // --- FUNCTION CALL ---
        // If it's a function call like out("Hello World!"), we need to:
        //   1. Figure out which function is being called
        //   2. Evaluate all the arguments (get their values)
        //   3. Do what that function is supposed to do
        Expr::FunctionCall { name, args } => {
            // First, evaluate all the arguments so we have their actual values
            let evaluated_args: Vec<Value> = args
                .iter()
                .map(|arg| evaluate_expression(arg))
                .collect();

            // Now check which function is being called and do the right thing
            match name.as_str() {
                // "out" is our built-in print function!
                // It takes whatever you give it and prints it to the screen.
                "out" => {
                    // Make sure they gave us at least one thing to print
                    if evaluated_args.is_empty() {
                        panic!("out() needs something to print! Try: out(\"Hello!\")");
                    }

                    // Print each argument
                    for arg_value in &evaluated_args {
                        match arg_value {
                            // If it's a string, print the text
                            Value::GuppyString(text) => {
                                println!("{}", text);
                            }
                            // If it's Nothing... well, print nothing
                            Value::Nothing => {
                                println!();
                            }
                        }
                    }

                    // out() doesn't return anything meaningful
                    Value::Nothing
                }

                // If someone calls a function we don't know about, tell them!
                unknown => {
                    panic!(
                        "I don't know a function called '{}'! Right now I only know 'out'.",
                        unknown
                    );
                }
            }
        }
    }
}
