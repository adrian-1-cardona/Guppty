// === lexer.rs ===
// The lexer cuts source code into neat little token pieces.

use crate::token::Token;

/// Strip inline comments starting with //
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

fn keyword_or_identifier(word: &str) -> Token {
    match word {
        "for" => Token::For,
        "in" => Token::In,
        "range" => Token::Range,
        "through" => Token::Through,
        "true" => Token::True,
        "false" => Token::False,
        _ => Token::Identifier(word.to_string()),
    }
}

fn lex_line(line: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        let ch = chars[pos];

        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        if ch == '(' {
            tokens.push(Token::LeftParen);
            pos += 1;
            continue;
        }

        if ch == ')' {
            tokens.push(Token::RightParen);
            pos += 1;
            continue;
        }

        if ch == '[' {
            tokens.push(Token::LeftBracket);
            pos += 1;
            continue;
        }

        if ch == ']' {
            tokens.push(Token::RightBracket);
            pos += 1;
            continue;
        }

        if ch == ';' {
            tokens.push(Token::Semicolon);
            pos += 1;
            continue;
        }

        if ch == ',' {
            tokens.push(Token::Comma);
            pos += 1;
            continue;
        }

        if ch == '+' {
            tokens.push(Token::Plus);
            pos += 1;
            continue;
        }

        if ch == '-' {
            tokens.push(Token::Minus);
            pos += 1;
            continue;
        }

        if ch == '*' {
            tokens.push(Token::Star);
            pos += 1;
            continue;
        }

        if ch == '/' {
            tokens.push(Token::Slash);
            pos += 1;
            continue;
        }

        if ch == '=' {
            tokens.push(Token::Equal);
            pos += 1;
            continue;
        }

        if ch == '"' {
            pos += 1;
            let mut string_content = String::new();

            while pos < chars.len() && chars[pos] != '"' {
                string_content.push(chars[pos]);
                pos += 1;
            }

            if pos >= chars.len() {
                panic!("Oops! You started a string with \" but never closed it!");
            }

            pos += 1;
            tokens.push(Token::StringLiteral(string_content));
            continue;
        }

        if ch == '\'' {
            pos += 1;
            if pos >= chars.len() {
                panic!("Oops! You started a char with ' but never closed it!");
            }

            let char_value = chars[pos];
            pos += 1;

            if pos >= chars.len() || chars[pos] != '\'' {
                panic!("Oops! A char literal must be exactly one character like 'h'");
            }

            pos += 1;
            tokens.push(Token::CharLiteral(char_value));
            continue;
        }

        if ch.is_ascii_digit() {
            let mut number_text = String::new();
            let mut is_float = false;

            while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                if chars[pos] == '.' {
                    if is_float {
                        panic!("Invalid number: too many decimal points");
                    }
                    is_float = true;
                }
                number_text.push(chars[pos]);
                pos += 1;
            }

            if is_float {
                let value: f64 = number_text
                    .parse()
                    .unwrap_or_else(|_| panic!("Invalid float number: {}", number_text));
                tokens.push(Token::FloatLiteral(value));
            } else {
                let value: i64 = number_text
                    .parse()
                    .unwrap_or_else(|_| panic!("Invalid number: {}", number_text));
                tokens.push(Token::NumberLiteral(value));
            }
            continue;
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut word = String::new();

            while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                word.push(chars[pos]);
                pos += 1;
            }

            tokens.push(keyword_or_identifier(&word));
            continue;
        }

        panic!(
            "Yikes! I don't know what this character is: '{}' (found at position {})",
            ch, pos
        );
    }

    tokens
}

fn measure_indent(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

/// Lex the entire source and emit indentation-aware tokens.
pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];

    for raw_line in source.lines() {
        let line = strip_comment(raw_line);
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let indent = measure_indent(line);

        while indent < *indent_stack.last().unwrap() {
            indent_stack.pop();
            tokens.push(Token::Dedent);
        }

        if indent > *indent_stack.last().unwrap() {
            indent_stack.push(indent);
            tokens.push(Token::Indent);
        }

        tokens.extend(lex_line(trimmed));
        tokens.push(Token::Newline);
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token::Dedent);
    }

    tokens.push(Token::EOF);
    tokens
}
