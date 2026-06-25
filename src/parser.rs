// === parser.rs ===
// The parser is like a detective that looks at our tokens
// and figures out what they MEAN together.
//
// The lexer gave us: [Identifier("out"), LeftParen, StringLiteral("Hello World!"), RightParen]
// The parser says: "Aha! That's a function call named 'out' with one argument!"
//
// It builds the AST (our family tree of code) from the flat list of tokens.
// Think of it like putting together a puzzle — the pieces (tokens) go in,
// and a nice picture (the AST) comes out.

use crate::token::Token;
use crate::ast::{Expr, Stmt, Program};

/// The Parser struct keeps track of where we are in the token list.
/// It's like a bookmark — it remembers which token we're looking at right now.
struct Parser {
    tokens: Vec<Token>,  // All our tokens (the flat list of pieces)
    pos: usize,          // Our current position (which token are we on?)
}

impl Parser {
    /// Create a brand new parser with a list of tokens
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Peek at the current token WITHOUT moving forward.
    /// Like looking at the next card in a deck without picking it up.
    fn current(&self) -> &Token {
        // If we're past the end, just return EOF (the "end" marker)
        if self.pos >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.pos]
        }
    }

    /// Move forward to the next token and give back the one we just passed.
    /// Like picking up a card from the deck.
    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    /// Check if the current token matches what we expect, and if so, move past it.
    /// If it DOESN'T match, panic! (something went wrong in the code).
    /// This is like saying "I expect a ')' here — if it's not there, the code is broken."
    fn expect(&mut self, expected: &Token) {
        let current = self.current().clone();
        if &current == expected {
            self.advance();
        } else {
            panic!(
                "Parser error! Expected {:?} but found {:?}",
                expected, current
            );
        }
    }

    /// Parse the entire program — this is the main entry point.
    /// It reads statements one by one until we hit the end of the file.
    fn parse_program(&mut self) -> Program {
        let mut statements: Vec<Stmt> = Vec::new();

        // Keep reading statements until we reach the end of the file
        while *self.current() != Token::EOF {
            let stmt = self.parse_statement();
            statements.push(stmt);
        }

        statements
    }

    /// Parse one single statement.
    /// Right now, Guppty only has "expression statements" —
    /// that means you write an expression (like a function call) and that's your statement.
    fn parse_statement(&mut self) -> Stmt {
        // Parse the expression part (like `out("Hello World!")`)
        let expr = self.parse_expression();

        // If there's a semicolon after it, eat it up (it's optional in Guppty!)
        if *self.current() == Token::Semicolon {
            self.advance(); // Skip the semicolon
        }

        // Wrap the expression in a statement and return it
        Stmt::ExpressionStatement(expr)
    }

    /// Parse an expression.
    /// An expression is something that produces a value.
    /// Right now we handle:
    ///   - String literals: "Hello World!"
    ///   - Function calls: out("Hello World!")
    ///   - Just a plain name: someVariable
    fn parse_expression(&mut self) -> Expr {
        match self.current().clone() {
            // If we see a string in quotes, it's a string literal
            Token::StringLiteral(s) => {
                self.advance(); // Move past the string token
                Expr::StringLiteral(s)
            }

            // If we see a name (identifier), it might be a function call
            Token::Identifier(name) => {
                self.advance(); // Move past the name

                // Check if there's a "(" right after — that means it's a function call!
                if *self.current() == Token::LeftParen {
                    self.advance(); // Eat the "("

                    // Collect all the arguments (stuff inside the parentheses)
                    let mut args: Vec<Expr> = Vec::new();

                    // Keep reading arguments until we see ")"
                    while *self.current() != Token::RightParen {
                        let arg = self.parse_expression();
                        args.push(arg);

                        // If there's a comma, skip it (for multiple arguments later)
                        // For now, out("Hello") only has one argument, so this is future-proofing
                    }

                    // We expect a ")" to close the function call
                    self.expect(&Token::RightParen);

                    // Build and return the function call expression
                    Expr::FunctionCall { name, args }
                } else {
                    // If there's no "(", it's just a plain name for now
                    // We'll treat it as a function call with no args for simplicity
                    // (This won't happen with our current hello.gup but it's safe)
                    panic!("I found the name '{}' but I don't know what to do with it yet! Try calling it like: {}(\"something\")", name, name);
                }
            }

            // If we see anything else, we don't know how to handle it
            other => {
                panic!("I don't understand this token: {:?}", other);
            }
        }
    }
}

/// The main function that the rest of the program calls.
/// Give it tokens, get back a program (list of statements).
pub fn parse(tokens: Vec<Token>) -> Program {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}
