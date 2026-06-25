// === syntax.rs ===
// this file is like a menu at a restaurant!
// if you want to change how guppy words look later,
// you change them HERE and not in 50 other places.
// that matters SO much because one change fixes everything!

/// all the special words and names guppy uses.
/// change a word here and the whole language listens!
pub struct SyntaxConfig {
    pub print_fn: &'static str,
    pub for_kw: &'static str,
    pub in_kw: &'static str,
    pub range_fn: &'static str,
    pub through_kw: &'static str,
    pub if_kw: &'static str,
    pub else_kw: &'static str,
    pub while_kw: &'static str,
    pub return_kw: &'static str,
    pub and_kw: &'static str,
    pub or_kw: &'static str,
    pub not_kw: &'static str,
    pub true_kw: &'static str,
    pub false_kw: &'static str,
}

/// the default menu — matches design/syntax.md right now.
pub const SYNTAX: SyntaxConfig = SyntaxConfig {
    print_fn: "out",
    for_kw: "for",
    in_kw: "in",
    range_fn: "range",
    through_kw: "through",
    if_kw: "if",
    else_kw: "else",
    while_kw: "while",
    return_kw: "return",
    and_kw: "and",
    or_kw: "or",
    not_kw: "not",
    true_kw: "true",
    false_kw: "false",
};

/// turn a word into a token kind if it is a keyword.
/// if it is not special we just call it a name (identifier).
pub fn keyword_token(word: &str) -> Option<crate::token::TokenKind> {
    use crate::token::TokenKind;

    if word == SYNTAX.for_kw {
        return Some(TokenKind::For);
    }
    if word == SYNTAX.in_kw {
        return Some(TokenKind::In);
    }
    if word == SYNTAX.range_fn {
        return Some(TokenKind::Range);
    }
    if word == SYNTAX.through_kw {
        return Some(TokenKind::Through);
    }
    if word == SYNTAX.if_kw {
        return Some(TokenKind::If);
    }
    if word == SYNTAX.else_kw {
        return Some(TokenKind::Else);
    }
    if word == SYNTAX.while_kw {
        return Some(TokenKind::While);
    }
    if word == SYNTAX.return_kw {
        return Some(TokenKind::Return);
    }
    if word == SYNTAX.and_kw {
        return Some(TokenKind::And);
    }
    if word == SYNTAX.or_kw {
        return Some(TokenKind::Or);
    }
    if word == SYNTAX.not_kw {
        return Some(TokenKind::Not);
    }
    if word == SYNTAX.true_kw {
        return Some(TokenKind::True);
    }
    if word == SYNTAX.false_kw {
        return Some(TokenKind::False);
    }

    None
}
