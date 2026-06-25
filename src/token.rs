// === token.rs ===
// tokens are tiny pieces of your code, like cutting a pizza into slices!
// the lexer makes slices, the parser eats them in order.
// each slice has a label so we know what it is.

use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // names and literal values (the actual data)
    Identifier(String),
    StringLiteral(String),
    CharLiteral(char),
    NumberLiteral(i64),
    FloatLiteral(f64),
    True,
    False,

    // special words that mean something (keywords)
    For,
    In,
    Range,
    Through,
    If,
    Else,
    While,
    Return,
    And,
    Or,
    Not,

    // math and logic symbols
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // punctuation (the shapes that group things)
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,

    // indentation blocks (python-style — super important for scopes!)
    Newline,
    Indent,
    Dedent,

    EOF,
}

/// a token is a labeled slice sitting at a place in the file.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }

    pub fn at(kind: TokenKind, line: usize, column: usize, length: usize) -> Self {
        Token::new(kind, Span::new(line, column, length))
    }
}
