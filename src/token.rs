// === token.rs ===
// Think of tokens like LEGO pieces.
// Before we can build anything, we need to break our code
// into small labeled pieces. Each piece is a "token."

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Identifier(String),
    StringLiteral(String),
    CharLiteral(char),
    NumberLiteral(i64),
    FloatLiteral(f64),
    True,
    False,

    // Keywords
    For,
    In,
    Range,
    Through,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equal,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,

    // Structure
    Newline,
    Indent,
    Dedent,

    EOF,
}
