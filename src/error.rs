// === error.rs ===
// Friendly errors live here so every stage can point at the same kind of place.
//
// The goal: instead of a scary stack trace like Java, a Guppty error tells you
// three things, plainly:
//   1. WHERE it happened  -> file:line:column, plus the exact source line.
//   2. WHAT kind it is     -> a short type name (NameError, SyntaxError, ...).
//   3. HOW to fix it       -> a single-line "help:" suggestion.

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

/// Which stage of the pipeline produced the error. Used as a fallback when we
/// cannot pin down a more specific [`ErrorKind`] from the message text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPhase {
    Lex,
    Parse,
    Runtime,
}

/// A short, human-friendly "type of error", inspired by Python's exception
/// names. This is the "what kind is it" part of a Guppty error message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    SyntaxError,
    IndentationError,
    NameError,
    TypeError,
    ValueError,
    MathError,
    ArgumentError,
    IndexError,
    RuntimeError,
    InternalError,
}

impl ErrorKind {
    pub fn title(self) -> &'static str {
        match self {
            ErrorKind::SyntaxError => "SyntaxError",
            ErrorKind::IndentationError => "IndentationError",
            ErrorKind::NameError => "NameError",
            ErrorKind::TypeError => "TypeError",
            ErrorKind::ValueError => "ValueError",
            ErrorKind::MathError => "MathError",
            ErrorKind::ArgumentError => "ArgumentError",
            ErrorKind::IndexError => "IndexError",
            ErrorKind::RuntimeError => "RuntimeError",
            ErrorKind::InternalError => "InternalError",
        }
    }

    /// A generic suggestion used when the message does not match a more
    /// specific, tailored hint.
    fn default_hint(self) -> &'static str {
        match self {
            ErrorKind::SyntaxError => "Check the syntax around here for a typo or a missing symbol.",
            ErrorKind::IndentationError => {
                "Line up this block with consistent indentation (use the same spaces as its siblings)."
            }
            ErrorKind::NameError => {
                "Declare the name first (e.g. `name = value`) or fix the spelling before using it."
            }
            ErrorKind::TypeError => "Use a value of the type this operation expects here.",
            ErrorKind::ValueError => "Give this a valid value in the form Guppty expects.",
            ErrorKind::MathError => "Adjust the numbers so the calculation is valid.",
            ErrorKind::ArgumentError => "Call the function with the number of arguments it defines.",
            ErrorKind::IndexError => "Use an index between 0 and the array's length minus one.",
            ErrorKind::RuntimeError => "Re-check the values flowing into this line.",
            ErrorKind::InternalError => {
                "This looks like a bug inside Guppty itself; please report it with your program."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GupError {
    pub phase: ErrorPhase,
    pub kind: ErrorKind,
    pub span: Span,
    pub message: String,
    pub hint: Option<String>,
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
        let message = message.into();
        let (kind, hint) = classify(phase, &message);
        GupError {
            phase,
            kind,
            span,
            message,
            hint: Some(hint),
        }
    }

    /// Override the auto-detected error type (used when a call site knows better
    /// than the message-text classifier).
    pub fn with_kind(mut self, kind: ErrorKind) -> Self {
        self.kind = kind;
        self
    }

    /// Override the auto-detected one-line fix suggestion.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn render(&self, filename: &str, source: &str) -> String {
        // Line 1 leads with the exact location so editors and terminals can jump
        // straight to it, then the error type, then the message.
        let mut output = format!(
            "{}:{}:{}: {}: {}",
            filename,
            self.span.line,
            self.span.column,
            self.kind.title(),
            self.message
        );

        let line_label = self.span.line.to_string();
        let gutter = " ".repeat(line_label.len());

        if let Some(line_text) = source.lines().nth(self.span.line.saturating_sub(1)) {
            output.push_str(&format!("\n{} |", gutter));
            output.push_str(&format!("\n{} | {}", line_label, line_text));

            let pad = " ".repeat(self.span.column.saturating_sub(1));
            let carets = "^".repeat(self.span.length.min(80).max(1));
            output.push_str(&format!(
                "\n{} | {}{} {} here",
                gutter,
                pad,
                carets,
                self.kind.title()
            ));
            output.push_str(&format!("\n{} |", gutter));
        }

        let hint = self
            .hint
            .clone()
            .unwrap_or_else(|| self.kind.default_hint().to_string());
        output.push_str(&format!("\n{} = help: {}", gutter, hint));

        output
    }
}

/// Turn a raw message + stage into a specific error type and a tailored,
/// single-line fix suggestion. Keeping this in one place means every stage of
/// the language gets typed, actionable errors without changing each call site.
fn classify(phase: ErrorPhase, message: &str) -> (ErrorKind, String) {
    let lower = message.to_lowercase();

    let contains = |needle: &str| lower.contains(needle);

    // --- Lexing: characters and literals ---
    if contains("lonely '!'") {
        return (
            ErrorKind::SyntaxError,
            "Guppty uses '!=' for 'not equal'; there is no standalone '!'.".to_string(),
        );
    }
    if contains("started a string") {
        return (
            ErrorKind::SyntaxError,
            "Add a closing double quote (\") to finish the string.".to_string(),
        );
    }
    if contains("started a char") {
        return (
            ErrorKind::SyntaxError,
            "Close the char with a matching single quote, like 'a'.".to_string(),
        );
    }
    if contains("char must be exactly one") {
        return (
            ErrorKind::SyntaxError,
            "Put exactly one character in single quotes ('h'); use double quotes for text."
                .to_string(),
        );
    }
    if contains("too many decimal points") {
        return (
            ErrorKind::SyntaxError,
            "Use just one '.' in a number, like 3.14.".to_string(),
        );
    }
    if contains("valid decimal number") {
        return (
            ErrorKind::ValueError,
            "Write decimals as digits with a single dot, like 3.14.".to_string(),
        );
    }
    if contains("valid whole number") {
        return (
            ErrorKind::ValueError,
            "Write whole numbers as digits only, like 42.".to_string(),
        );
    }
    if contains("do not know what to do with") {
        return (
            ErrorKind::SyntaxError,
            "Remove this unexpected character or replace it with valid Guppty syntax.".to_string(),
        );
    }

    // --- Parsing: shape of the program ---
    if contains("indented block") || contains("indent") {
        return (
            ErrorKind::IndentationError,
            "Indent the block under this line so its statements line up.".to_string(),
        );
    }
    if contains("expected an expression") {
        return (
            ErrorKind::SyntaxError,
            "Put a value here: a number, string, name, or a (parenthesized) expression."
                .to_string(),
        );
    }
    if contains("only empty arrays") {
        return (
            ErrorKind::SyntaxError,
            "Guppty only supports empty arrays [] for now; remove the items inside.".to_string(),
        );
    }
    if contains("expected") && contains("but found") {
        return (
            ErrorKind::SyntaxError,
            "A token is missing or misplaced here; add or fix what was expected.".to_string(),
        );
    }

    // --- Runtime: values and behavior ---
    if contains("is not defined yet") {
        return (
            ErrorKind::NameError,
            "Declare it first (e.g. `name = value`) or check the spelling.".to_string(),
        );
    }
    if contains("is not a function") {
        return (
            ErrorKind::TypeError,
            "Only call functions; check the name before '(' or define it with a function."
                .to_string(),
        );
    }
    if contains("range() can only be used") {
        return (
            ErrorKind::ValueError,
            "Use range() only in a for-loop header: for i in range(1 through 6).".to_string(),
        );
    }
    if contains("wrong number of arguments") {
        return (
            ErrorKind::ArgumentError,
            "Pass exactly the number of arguments the function was defined with.".to_string(),
        );
    }
    if contains("divide by zero") {
        return (
            ErrorKind::MathError,
            "Make sure the divisor is not zero before dividing.".to_string(),
        );
    }
    if contains("cannot be used as a math operator") {
        return (
            ErrorKind::TypeError,
            "Use +, -, *, or / with numbers here.".to_string(),
        );
    }
    if contains("expected a number but got") {
        return (
            ErrorKind::TypeError,
            "Use a number here, or convert the value to a number first.".to_string(),
        );
    }
    if contains("expected an array but got") {
        return (
            ErrorKind::TypeError,
            "Use an array here, like [] or a variable that holds an array.".to_string(),
        );
    }
    if contains("index out of bounds") {
        return (
            ErrorKind::IndexError,
            "Use an index from 0 to the array's length minus one.".to_string(),
        );
    }
    if contains("opcode")
        || contains("constant")
        || contains("empty stack")
        || contains("call frame")
    {
        return (
            ErrorKind::InternalError,
            "This looks like a bug inside Guppty itself; please report it with your program."
                .to_string(),
        );
    }

    // --- Fallbacks keyed off the pipeline stage ---
    let kind = match phase {
        ErrorPhase::Lex => ErrorKind::SyntaxError,
        ErrorPhase::Parse => ErrorKind::SyntaxError,
        ErrorPhase::Runtime => ErrorKind::RuntimeError,
    };
    (kind, kind.default_hint().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_shows_location_type_and_help() {
        let source = "out(missing)";
        let error = GupError::runtime(Span::new(1, 5, 7), "Variable 'missing' is not defined yet!");
        let rendered = error.render("program.gup", source);

        // WHERE: exact file:line:column, and the source line with a caret.
        assert!(rendered.contains("program.gup:1:5:"));
        assert!(rendered.contains("out(missing)"));
        assert!(rendered.contains('^'));
        // WHAT: a specific error type, not a generic stack trace.
        assert!(rendered.contains("NameError"));
        assert!(!rendered.to_lowercase().contains("stack trace"));
        // HOW: a single-line fix suggestion.
        assert!(rendered.contains("help:"));
    }

    #[test]
    fn classifier_picks_specific_types() {
        assert_eq!(
            GupError::runtime(Span::new(1, 1, 1), "Cannot divide by zero.").kind,
            ErrorKind::MathError
        );
        assert_eq!(
            GupError::parse(Span::new(1, 1, 1), "Expected an indented block but found end of file.")
                .kind,
            ErrorKind::IndentationError
        );
        assert_eq!(
            GupError::runtime(Span::new(1, 1, 1), "Array index out of bounds.").kind,
            ErrorKind::IndexError
        );
        assert_eq!(
            GupError::lex(Span::new(1, 1, 1), "I do not know what to do with '@'.").kind,
            ErrorKind::SyntaxError
        );
    }

    #[test]
    fn builders_override_kind_and_hint() {
        let error = GupError::runtime(Span::new(1, 1, 1), "something went sideways")
            .with_kind(ErrorKind::InternalError)
            .with_hint("please report this with your program");

        assert_eq!(error.kind, ErrorKind::InternalError);
        let rendered = error.render("bug.gup", "boom");
        assert!(rendered.contains("InternalError"));
        assert!(rendered.contains("help: please report this with your program"));
    }
}
