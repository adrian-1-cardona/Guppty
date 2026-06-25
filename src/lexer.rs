// === lexer.rs ===
// The lexer is like a cookie cutter for code.
// It takes the raw text you wrote (like "out("Hello World!")")
// and cuts it into neat little token pieces.
//
// It reads one character at a time, figures out what kind of
// token it's looking at, and puts it in a list.
//
// Imagine reading a sentence letter by letter and grouping
// the letters into words — that's basically what this does!

use crate::token::Token;

/// This function takes your entire source code as a string
/// and gives back a list of tokens (our LEGO pieces).
pub fn lex(source: &str) -> Vec<Token> {
    // This is our collection bag — we'll put each token in here
    let mut tokens: Vec<Token> = Vec::new();

    // Turn the source code into a list of characters so we can
    // look at them one by one (like laying out letter tiles)
    let chars: Vec<char> = source.chars().collect();

    // This is our finger pointing at the current character.
    // We start at the very beginning (position 0).
    let mut pos = 0;

    // Keep going until we've looked at every single character
    while pos < chars.len() {
        // Grab the character our finger is pointing at right now
        let ch = chars[pos];

        // --- SKIP WHITESPACE ---
        // Spaces, tabs, and newlines don't mean anything to us.
        // Just skip over them like stepping over puddles.
        if ch.is_whitespace() {
            pos += 1;
            continue; // Go back to the top of the loop
        }

        // --- LEFT PARENTHESIS ---
        // If we see "(", that's a LeftParen token
        if ch == '(' {
            tokens.push(Token::LeftParen);
            pos += 1;
            continue;
        }

        // --- RIGHT PARENTHESIS ---
        // If we see ")", that's a RightParen token
        if ch == ')' {
            tokens.push(Token::RightParen);
            pos += 1;
            continue;
        }

        // --- SEMICOLON ---
        // If we see ";", that's a Semicolon token
        if ch == ';' {
            tokens.push(Token::Semicolon);
            pos += 1;
            continue;
        }

        // --- STRING LITERAL ---
        // If we see a double quote, that means a string is starting!
        // We need to collect everything until the closing quote.
        if ch == '"' {
            // Move past the opening quote
            pos += 1;

            // This will hold all the letters inside the quotes
            let mut string_content = String::new();

            // Keep reading characters until we find the closing quote
            // or run out of characters (which would be an error)
            while pos < chars.len() && chars[pos] != '"' {
                string_content.push(chars[pos]);
                pos += 1;
            }

            // If we ran out of characters without finding a closing quote,
            // that means the programmer forgot to close their string!
            if pos >= chars.len() {
                panic!("Oops! You started a string with \" but never closed it!");
            }

            // Move past the closing quote
            pos += 1;

            // Save this string as a token
            tokens.push(Token::StringLiteral(string_content));
            continue;
        }

        // --- IDENTIFIER (a name/word) ---
        // If we see a letter or underscore, it's the start of a word.
        // Words can have letters, numbers, and underscores in them.
        if ch.is_alphabetic() || ch == '_' {
            // Start collecting the letters of this word
            let mut word = String::new();

            // Keep going as long as we see letters, numbers, or underscores
            while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                word.push(chars[pos]);
                pos += 1;
            }

            // Save this word as an Identifier token
            tokens.push(Token::Identifier(word));
            continue;
        }

        // --- UNKNOWN CHARACTER ---
        // If we get here, we found something we don't understand!
        // Let the programmer know something is wrong.
        panic!(
            "Yikes! I don't know what this character is: '{}' (found at position {})",
            ch, pos
        );
    }

    // Add the special "end of file" token so the parser knows we're done
    tokens.push(Token::EOF);

    // Give back our nice list of tokens!
    tokens
}
