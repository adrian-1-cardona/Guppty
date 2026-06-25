// === token.rs ===
// Think of tokens like LEGO pieces.
// Before we can build anything, we need to break our code
// into small labeled pieces. Each piece is a "token."
//
// For example, the code: out("Hello World!")
// Gets broken into these pieces:
//   1. The word "out"        -> that's a Name (we call it Identifier)
//   2. The "(" symbol        -> that's a LeftParen (left parenthesis)
//   3. "Hello World!"        -> that's a StringLiteral (some text in quotes)
//   4. The ")" symbol        -> that's a RightParen (right parenthesis)
//   5. Maybe a ";" at the end -> that's a Semicolon (optional period at the end)
//   6. The very end of the file -> that's EOF (End Of File, like "The End" in a storybook)

/// This is our list of every possible LEGO piece (token) in Guppty.
/// Each one has a name so we know what kind of piece it is.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // A name/word like "out" — could be a function name or variable name
    Identifier(String),

    // A piece of text inside double quotes, like "Hello World!"
    StringLiteral(String),

    // The "(" character — opens a group
    LeftParen,

    // The ")" character — closes a group
    RightParen,

    // The ";" character — marks the end of a line of code (optional in Guppty!)
    Semicolon,

    // This means "we reached the end of the file, nothing left to read!"
    EOF,
}
