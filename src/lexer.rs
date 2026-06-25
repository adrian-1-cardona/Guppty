// === lexer.rs ===
// the lexer is like a cookie cutter — it chops code into token shapes!
// it also watches spaces at the start of lines to know when blocks begin and end.
// blocks matter SO much because that is how scopes work!

use crate::error::{GupError, Span};
use crate::syntax::{keyword_token, SYNTAX};
use crate::token::{Token, TokenKind};

/// cut off // comments so they do not become code
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let chars: Vec<char> = line.chars().collect();

    for i in 0..chars.len() {
        if chars[i] == '"' {
            in_string = !in_string;
        } else if !in_string && chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            return &line[..i];
        }
    }

    line
}

fn push_token(tokens: &mut Vec<Token>, kind: TokenKind, line: usize, column: usize, length: usize) {
    tokens.push(Token::at(kind, line, column, length));
}

fn lex_line(line: &str, line_number: usize, start_column: usize) -> Result<Vec<Token>, GupError> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        let ch = chars[pos];

        // skip spaces inside a line — indentation is handled elsewhere
        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        if ch == '(' {
            push_token(
                &mut tokens,
                TokenKind::LeftParen,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        if ch == ')' {
            push_token(
                &mut tokens,
                TokenKind::RightParen,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        if ch == '[' {
            push_token(
                &mut tokens,
                TokenKind::LeftBracket,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        if ch == ']' {
            push_token(
                &mut tokens,
                TokenKind::RightBracket,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        if ch == ';' {
            push_token(
                &mut tokens,
                TokenKind::Semicolon,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        if ch == ',' {
            push_token(
                &mut tokens,
                TokenKind::Comma,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        if ch == '+' {
            push_token(
                &mut tokens,
                TokenKind::Plus,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        if ch == '*' {
            push_token(
                &mut tokens,
                TokenKind::Star,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        if ch == '/' {
            push_token(
                &mut tokens,
                TokenKind::Slash,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        // two-character operators like == and !=
        if ch == '=' {
            if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                push_token(
                    &mut tokens,
                    TokenKind::EqualEqual,
                    line_number,
                    start_column + pos,
                    2,
                );
                pos += 2;
            } else {
                push_token(
                    &mut tokens,
                    TokenKind::Equal,
                    line_number,
                    start_column + pos,
                    1,
                );
                pos += 1;
            }
            continue;
        }

        if ch == '!' {
            if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                push_token(
                    &mut tokens,
                    TokenKind::BangEqual,
                    line_number,
                    start_column + pos,
                    2,
                );
                pos += 2;
            } else {
                return Err(GupError::lex(
                    Span::new(line_number, start_column + pos, 1),
                    "I only know '!=' as a pair. A lonely '!' is not valid here.",
                ));
            }
            continue;
        }

        if ch == '<' {
            if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                push_token(
                    &mut tokens,
                    TokenKind::LessEqual,
                    line_number,
                    start_column + pos,
                    2,
                );
                pos += 2;
            } else {
                push_token(
                    &mut tokens,
                    TokenKind::Less,
                    line_number,
                    start_column + pos,
                    1,
                );
                pos += 1;
            }
            continue;
        }

        if ch == '>' {
            if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                push_token(
                    &mut tokens,
                    TokenKind::GreaterEqual,
                    line_number,
                    start_column + pos,
                    2,
                );
                pos += 2;
            } else {
                push_token(
                    &mut tokens,
                    TokenKind::Greater,
                    line_number,
                    start_column + pos,
                    1,
                );
                pos += 1;
            }
            continue;
        }

        if ch == '-' {
            push_token(
                &mut tokens,
                TokenKind::Minus,
                line_number,
                start_column + pos,
                1,
            );
            pos += 1;
            continue;
        }

        // strings live inside "quotes"
        if ch == '"' {
            let string_start = pos;
            pos += 1;
            let mut string_content = String::new();

            while pos < chars.len() && chars[pos] != '"' {
                string_content.push(chars[pos]);
                pos += 1;
            }

            if pos >= chars.len() {
                return Err(GupError::lex(
                    Span::new(
                        line_number,
                        start_column + string_start,
                        chars.len() - string_start,
                    ),
                    "You started a string with '\"' but never closed it.",
                ));
            }

            pos += 1;
            push_token(
                &mut tokens,
                TokenKind::StringLiteral(string_content),
                line_number,
                start_column + string_start,
                pos - string_start,
            );
            continue;
        }

        // chars live inside 'quotes' and are exactly one letter
        if ch == '\'' {
            let char_start = pos;
            pos += 1;
            if pos >= chars.len() {
                return Err(GupError::lex(
                    Span::new(line_number, start_column + char_start, 1),
                    "You started a char with '\\'' but never closed it.",
                ));
            }

            let char_value = chars[pos];
            pos += 1;

            if pos >= chars.len() || chars[pos] != '\'' {
                return Err(GupError::lex(
                    Span::new(line_number, start_column + char_start, pos - char_start),
                    "A char must be exactly one character, like 'h'.",
                ));
            }

            pos += 1;
            push_token(
                &mut tokens,
                TokenKind::CharLiteral(char_value),
                line_number,
                start_column + char_start,
                pos - char_start,
            );
            continue;
        }

        // numbers can be whole or have a dot for decimals
        if ch.is_ascii_digit() {
            let number_start = pos;
            let mut number_text = String::new();
            let mut is_float = false;

            while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                if chars[pos] == '.' {
                    if is_float {
                        return Err(GupError::lex(
                            Span::new(line_number, start_column + pos, 1),
                            "This number has too many decimal points.",
                        ));
                    }
                    is_float = true;
                }
                number_text.push(chars[pos]);
                pos += 1;
            }

            if is_float {
                let value: f64 = number_text.parse().map_err(|_| {
                    GupError::lex(
                        Span::new(line_number, start_column + number_start, pos - number_start),
                        format!("'{}' is not a valid decimal number.", number_text),
                    )
                })?;
                push_token(
                    &mut tokens,
                    TokenKind::FloatLiteral(value),
                    line_number,
                    start_column + number_start,
                    pos - number_start,
                );
            } else {
                let value: i64 = number_text.parse().map_err(|_| {
                    GupError::lex(
                        Span::new(line_number, start_column + number_start, pos - number_start),
                        format!("'{}' is not a valid whole number.", number_text),
                    )
                })?;
                push_token(
                    &mut tokens,
                    TokenKind::NumberLiteral(value),
                    line_number,
                    start_column + number_start,
                    pos - number_start,
                );
            }
            continue;
        }

        // words can be keywords or variable names
        if ch.is_alphabetic() || ch == '_' {
            let word_start = pos;
            let mut word = String::new();

            while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                word.push(chars[pos]);
                pos += 1;
            }

            if let Some(keyword) = keyword_token(&word) {
                push_token(
                    &mut tokens,
                    keyword,
                    line_number,
                    start_column + word_start,
                    pos - word_start,
                );
            } else {
                push_token(
                    &mut tokens,
                    TokenKind::Identifier(word),
                    line_number,
                    start_column + word_start,
                    pos - word_start,
                );
            }
            continue;
        }

        return Err(GupError::lex(
            Span::new(line_number, start_column + pos, 1),
            format!("I do not know what to do with '{}'.", ch),
        ));
    }

    Ok(tokens)
}

fn measure_indent(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

/// turn the whole source file into a token list with indent/dedent markers
pub fn lex(source: &str) -> Result<Vec<Token>, GupError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];
    let mut line_number = 0;

    for raw_line in source.lines() {
        line_number += 1;
        let line = strip_comment(raw_line);
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let indent = measure_indent(line);

        // less indent means a block ended — pop dedent tokens
        while indent < *indent_stack.last().unwrap() {
            indent_stack.pop();
            push_token(&mut tokens, TokenKind::Dedent, line_number, indent + 1, 1);
        }

        // more indent means a new block started — push indent token
        if indent > *indent_stack.last().unwrap() {
            indent_stack.push(indent);
            push_token(&mut tokens, TokenKind::Indent, line_number, indent + 1, 1);
        }

        tokens.extend(lex_line(trimmed, line_number, indent + 1)?);
        push_token(
            &mut tokens,
            TokenKind::Newline,
            line_number,
            raw_line.chars().count() + 1,
            1,
        );
    }

    // close any blocks still open at the end of the file
    while indent_stack.len() > 1 {
        indent_stack.pop();
        push_token(&mut tokens, TokenKind::Dedent, line_number.max(1), 1, 1);
    }

    push_token(&mut tokens, TokenKind::EOF, line_number.max(1), 1, 1);
    Ok(tokens)
}

// tiny helper so parser can ask "is this the print function name?"
pub fn is_print_function(name: &str) -> bool {
    name == SYNTAX.print_fn
}

// =============================================================================
// UNIT TESTS — we check that the lexer chops code the right way!
// If these break, words and numbers might get mixed up. That would be bad.
// =============================================================================
#[cfg(test)]
mod tests {
    // we need the stuff from this same file
    use super::*;
    // we need TokenKind to check what kind of slice each token is
    use crate::token::TokenKind;

    // -------------------------------------------------------------------------
    // helper: pull out only the "real" token kinds (not newlines/indents/end)
    // why? because we mostly care about words and symbols, not the glue between
    // -------------------------------------------------------------------------
    fn without_structure(tokens: &[Token]) -> Vec<TokenKind> {
        tokens
            .iter()
            .map(|token| token.kind.clone())
            .filter(|kind| {
                !matches!(
                    kind,
                    TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::EOF
                )
            })
            .collect()
    }

    // -------------------------------------------------------------------------
    // helper: check if the token list has a certain kind somewhere in it
    // why? indent/dedent tests need this because tokens also have line numbers
    // -------------------------------------------------------------------------
    fn has_kind(tokens: &[Token], kind: TokenKind) -> bool {
        tokens.iter().any(|token| token.kind == kind)
    }

    // -------------------------------------------------------------------------
    // TEST: out("hi") should become: out, (, "hi", )
    // this matters because EVERY program uses out() to print stuff!
    // -------------------------------------------------------------------------
    #[test]
    fn lex_simple_out_call() {
        // step 1: give the lexer a tiny program
        let tokens = lex(r#"out("hi")"#).unwrap();

        // step 2: throw away newline/indent junk we don't care about here
        let important = without_structure(&tokens);

        // step 3: make sure we got the right LEGO pieces in the right order
        assert_eq!(
            important,
            vec![
                TokenKind::Identifier("out".into()),
                TokenKind::LeftParen,
                TokenKind::StringLiteral("hi".into()),
                TokenKind::RightParen,
            ]
        );
    }

    // -------------------------------------------------------------------------
    // TEST: // comments should disappear (the lexer eats them)
    // why? so you can write notes in your code and the computer ignores them
    // -------------------------------------------------------------------------
    #[test]
    fn strips_inline_comments() {
        let tokens = lex(r#"out("ok") // this is a comment"#).unwrap();
        let important = without_structure(&tokens);

        // the comment words should NOT show up as tokens!
        assert_eq!(
            important,
            vec![
                TokenKind::Identifier("out".into()),
                TokenKind::LeftParen,
                TokenKind::StringLiteral("ok".into()),
                TokenKind::RightParen,
            ]
        );
    }

    // -------------------------------------------------------------------------
    // TEST: numbers — whole numbers AND numbers with a dot (like 3.14)
    // why? math needs numbers or nothing adds up!
    // -------------------------------------------------------------------------
    #[test]
    fn lex_number_literals() {
        let tokens = lex("x = 42\nf = 3.14").unwrap();
        let important = without_structure(&tokens);

        assert!(important.contains(&TokenKind::NumberLiteral(42)));
        assert!(important.contains(&TokenKind::FloatLiteral(3.14)));
    }

    // -------------------------------------------------------------------------
    // TEST: special words like "for" and "true" are their own token kinds
    // why? the parser needs to know they are magic words, not regular names
    // -------------------------------------------------------------------------
    #[test]
    fn lex_keywords() {
        let tokens = lex("for in range through true false").unwrap();
        let important = without_structure(&tokens);

        assert_eq!(
            important,
            vec![
                TokenKind::For,
                TokenKind::In,
                TokenKind::Range,
                TokenKind::Through,
                TokenKind::True,
                TokenKind::False,
            ]
        );
    }

    // -------------------------------------------------------------------------
    // TEST: when a line is pushed in (more spaces), we get Indent
    // when it pops back out, we get Dedent — like folding paper tabs
    // why? guppy uses spaces to know "this code goes INSIDE that block"
    // -------------------------------------------------------------------------
    #[test]
    fn emits_indent_and_dedent() {
        let source = "outer()\n    inner()";
        let tokens = lex(source).unwrap();

        assert!(has_kind(&tokens, TokenKind::Indent));
        assert!(has_kind(&tokens, TokenKind::Dedent));
    }

    // -------------------------------------------------------------------------
    // TEST: math symbols + - * / = all become their own tokens
    // why? 2+2 only works if the lexer sees Plus, not a random letter
    // -------------------------------------------------------------------------
    #[test]
    fn lex_operators() {
        let tokens = lex("a + b - c * d / e = f").unwrap();
        let important = without_structure(&tokens);

        assert!(important.contains(&TokenKind::Plus));
        assert!(important.contains(&TokenKind::Minus));
        assert!(important.contains(&TokenKind::Star));
        assert!(important.contains(&TokenKind::Slash));
        assert!(important.contains(&TokenKind::Equal));
    }

    #[test]
    fn lex_errors_have_locations_and_plain_words() {
        let error = lex("out(@)").unwrap_err();

        assert_eq!(error.span.line, 1);
        assert_eq!(error.span.column, 5);
        assert!(error.message.contains("@"));
    }
}
