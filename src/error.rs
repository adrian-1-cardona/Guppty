// === error.rs ===
// Friendly errors live here so every stage can point at the same kind of place.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Span {
    pub fn new(line: usize, column: usize, length: usize) -> Self {
        Span {
            line: line.max(1),
            column: column.max(1),
            length: length.max(1),
        }
    }

    pub fn merge(self, other: Span) -> Self {
        if self.line != other.line {
            return self;
        }

        let start = self.column.min(other.column);
        let end = self.end_column().max(other.end_column());
        Span::new(self.line, start, end.saturating_sub(start))
    }

    pub fn end_column(self) -> usize {
        self.column + self.length
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPhase {
    Lex,
    Parse,
    Runtime,
}

impl ErrorPhase {
    fn label(self) -> &'static str {
        match self {
            ErrorPhase::Lex => "lex",
            ErrorPhase::Parse => "parse",
            ErrorPhase::Runtime => "runtime",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GupError {
    pub phase: ErrorPhase,
    pub span: Span,
    pub message: String,
}

impl GupError {
    pub fn lex(span: Span, message: impl Into<String>) -> Self {
        Self::new(ErrorPhase::Lex, span, message)
    }

    pub fn parse(span: Span, message: impl Into<String>) -> Self {
        Self::new(ErrorPhase::Parse, span, message)
    }

    pub fn runtime(span: Span, message: impl Into<String>) -> Self {
        Self::new(ErrorPhase::Runtime, span, message)
    }

    fn new(phase: ErrorPhase, span: Span, message: impl Into<String>) -> Self {
        GupError {
            phase,
            span,
            message: message.into(),
        }
    }

    pub fn render(&self, filename: &str, source: &str) -> String {
        let mut output = format!(
            "{}:{}:{}: {} error: {}\nspan: line {}, column {}, length {}",
            filename,
            self.span.line,
            self.span.column,
            self.phase.label(),
            self.message,
            self.span.line,
            self.span.column,
            self.span.length
        );

        if let Some(line_text) = source.lines().nth(self.span.line.saturating_sub(1)) {
            output.push('\n');
            output.push_str(line_text);
            output.push('\n');
            output.push_str(&" ".repeat(self.span.column.saturating_sub(1)));
            output.push_str(&"^".repeat(self.span.length.min(80)));
        }

        output
    }
}
