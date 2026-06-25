// === parser.rs ===
// The parser reads tokens and builds an AST tree for the interpreter.

use crate::ast::{BinaryOp, Expr, ExprKind, Program, Stmt, StmtKind, UnaryOp};
use crate::error::{GupError, Span};
use crate::token::{Token, TokenKind};

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    fallback: Token,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            fallback: Token::at(TokenKind::EOF, 1, 1, 1),
        }
    }

    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .unwrap_or(&self.fallback)
    }

    fn current_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    fn current_span(&self) -> Span {
        self.current().span
    }

    fn peek_kind(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + offset).map(|token| &token.kind)
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect_kind(&mut self, expected: &TokenKind) -> Result<Token, GupError> {
        let current = self.current().clone();
        if &current.kind == expected {
            Ok(self.advance())
        } else {
            Err(GupError::parse(
                current.span,
                format!(
                    "Expected {} but found {}.",
                    describe_kind(expected),
                    describe_kind(&current.kind)
                ),
            ))
        }
    }

    fn skip_newlines(&mut self) {
        while *self.current_kind() == TokenKind::Newline {
            self.advance();
        }
    }

    fn parse_program(&mut self) -> Result<Program, GupError> {
        let mut statements: Vec<Stmt> = Vec::new();

        self.skip_newlines();
        while *self.current_kind() != TokenKind::EOF {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(statements)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, GupError> {
        self.expect_kind(&TokenKind::Indent)?;
        let mut statements: Vec<Stmt> = Vec::new();

        self.skip_newlines();
        while *self.current_kind() != TokenKind::Dedent && *self.current_kind() != TokenKind::EOF {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        if *self.current_kind() == TokenKind::Dedent {
            self.advance();
        }

        Ok(statements)
    }

    fn parse_block_or_single(&mut self) -> Result<Vec<Stmt>, GupError> {
        if *self.current_kind() == TokenKind::Indent {
            self.parse_block()
        } else {
            Ok(vec![self.parse_statement()?])
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt, GupError> {
        if *self.current_kind() == TokenKind::Return {
            return self.parse_return_statement();
        }

        if *self.current_kind() == TokenKind::If {
            return self.parse_if_statement();
        }

        if *self.current_kind() == TokenKind::While {
            return self.parse_while_loop();
        }

        if *self.current_kind() == TokenKind::For {
            return self.parse_for_loop();
        }

        if let TokenKind::Identifier(name) = self.current_kind().clone() {
            if self.peek_kind(1) == Some(&TokenKind::Equal) {
                return self.parse_variable_declaration(name);
            }

            if self.peek_kind(1) == Some(&TokenKind::LeftParen)
                && self.looks_like_function_definition()
            {
                return self.parse_function_def(name);
            }
        }

        if !self.is_expression_start() {
            return Err(GupError::parse(
                self.current_span(),
                format!(
                    "Expected a statement or expression but found {}.",
                    describe_kind(self.current_kind())
                ),
            ));
        }

        let expr = self.parse_expression()?;
        let mut span = expr.span;
        if let Some(end) = self.consume_optional_semicolon() {
            span = span.merge(end);
        }
        Ok(Stmt::new(StmtKind::ExpressionStatement(expr), span))
    }

    fn looks_like_function_definition(&self) -> bool {
        let mut index = self.pos + 2;
        let mut depth = 1;

        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => {
                    depth -= 1;
                    if depth == 0 {
                        let mut next_index = index + 1;
                        while self
                            .tokens
                            .get(next_index)
                            .is_some_and(|next| next.kind == TokenKind::Newline)
                        {
                            next_index += 1;
                        }

                        return self
                            .tokens
                            .get(next_index)
                            .is_some_and(|next| next.kind == TokenKind::Indent);
                    }
                }
                _ => {}
            }
            index += 1;
        }

        false
    }

    fn parse_function_def(&mut self, name: String) -> Result<Stmt, GupError> {
        let start = self.current_span();
        self.advance(); // identifier
        self.expect_kind(&TokenKind::LeftParen)?;

        let mut params: Vec<String> = Vec::new();
        if *self.current_kind() != TokenKind::RightParen {
            loop {
                let token = self.advance();
                match token.kind {
                    TokenKind::Identifier(param) => params.push(param),
                    other => {
                        return Err(GupError::parse(
                            token.span,
                            format!(
                                "Function parameters need names, but I found {}.",
                                describe_kind(&other)
                            ),
                        ));
                    }
                }

                if *self.current_kind() == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let close = self.expect_kind(&TokenKind::RightParen)?;
        self.skip_newlines();

        let body = if *self.current_kind() == TokenKind::Indent {
            self.parse_block()?
        } else {
            Vec::new()
        };

        let end = body.last().map(|stmt| stmt.span).unwrap_or(close.span);
        Ok(Stmt::new(
            StmtKind::FunctionDef { name, params, body },
            start.merge(end),
        ))
    }

    fn parse_for_loop(&mut self) -> Result<Stmt, GupError> {
        let start = self.expect_kind(&TokenKind::For)?.span;

        let variable_token = self.advance();
        let variable = match variable_token.kind {
            TokenKind::Identifier(name) => name,
            other => {
                return Err(GupError::parse(
                    variable_token.span,
                    format!(
                        "Expected a loop variable name, but I found {}.",
                        describe_kind(&other)
                    ),
                ));
            }
        };

        self.expect_kind(&TokenKind::In)?;
        let iterable = self.parse_expression()?;
        self.skip_newlines();

        let body = if *self.current_kind() == TokenKind::Indent {
            self.parse_block()?
        } else {
            Vec::new()
        };

        let end = body.last().map(|stmt| stmt.span).unwrap_or(iterable.span);
        Ok(Stmt::new(
            StmtKind::ForLoop {
                variable,
                iterable,
                body,
            },
            start.merge(end),
        ))
    }

    fn parse_while_loop(&mut self) -> Result<Stmt, GupError> {
        let start = self.expect_kind(&TokenKind::While)?.span;
        let condition = self.parse_expression()?;
        self.skip_newlines();

        let body = self.parse_block_or_single()?;
        let end = body.last().map(|stmt| stmt.span).unwrap_or(condition.span);
        Ok(Stmt::new(
            StmtKind::WhileLoop { condition, body },
            start.merge(end),
        ))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, GupError> {
        let start = self.expect_kind(&TokenKind::If)?.span;
        let condition = self.parse_expression()?;
        self.skip_newlines();

        let then_branch = self.parse_block_or_single()?;
        self.skip_newlines();

        let else_branch = if *self.current_kind() == TokenKind::Else {
            self.advance();
            self.skip_newlines();
            Some(self.parse_block_or_single()?)
        } else {
            None
        };

        let end = else_branch
            .as_ref()
            .and_then(|branch| branch.last())
            .or_else(|| then_branch.last())
            .map(|stmt| stmt.span)
            .unwrap_or(condition.span);

        Ok(Stmt::new(
            StmtKind::IfStatement {
                condition,
                then_branch,
                else_branch,
            },
            start.merge(end),
        ))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, GupError> {
        let start = self.expect_kind(&TokenKind::Return)?.span;

        let value = if self.is_expression_start() {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let mut span = value
            .as_ref()
            .map(|expr| start.merge(expr.span))
            .unwrap_or(start);
        if let Some(end) = self.consume_optional_semicolon() {
            span = span.merge(end);
        }

        Ok(Stmt::new(StmtKind::ReturnStatement { value }, span))
    }

    fn is_expression_start(&self) -> bool {
        matches!(
            self.current_kind(),
            TokenKind::StringLiteral(_)
                | TokenKind::CharLiteral(_)
                | TokenKind::NumberLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::LeftParen
                | TokenKind::LeftBracket
                | TokenKind::Range
                | TokenKind::Minus
                | TokenKind::Not
                | TokenKind::Identifier(_)
        )
    }

    fn parse_variable_declaration(&mut self, name: String) -> Result<Stmt, GupError> {
        let start = self.current_span();
        self.advance(); // identifier
        self.expect_kind(&TokenKind::Equal)?;
        let value = self.parse_expression()?;
        let mut span = start.merge(value.span);
        if let Some(end) = self.consume_optional_semicolon() {
            span = span.merge(end);
        }

        Ok(Stmt::new(
            StmtKind::VariableDeclaration { name, value },
            span,
        ))
    }

    fn consume_optional_semicolon(&mut self) -> Option<Span> {
        let mut end = None;
        if *self.current_kind() == TokenKind::Semicolon {
            end = Some(self.advance().span);
        }
        if *self.current_kind() == TokenKind::Newline {
            end = Some(self.advance().span);
        }
        end
    }

    fn parse_expression(&mut self) -> Result<Expr, GupError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, GupError> {
        let mut expr = self.parse_and()?;

        while *self.current_kind() == TokenKind::Or {
            self.advance();
            let right = self.parse_and()?;
            let span = expr.span.merge(right.span);
            expr = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(expr),
                    op: BinaryOp::Or,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, GupError> {
        let mut expr = self.parse_equality()?;

        while *self.current_kind() == TokenKind::And {
            self.advance();
            let right = self.parse_equality()?;
            let span = expr.span.merge(right.span);
            expr = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(expr),
                    op: BinaryOp::And,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, GupError> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = match self.current_kind() {
                TokenKind::EqualEqual => Some(BinaryOp::Equal),
                TokenKind::BangEqual => Some(BinaryOp::NotEqual),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_comparison()?;
                let span = expr.span.merge(right.span);
                expr = Expr::new(
                    ExprKind::BinaryOp {
                        left: Box::new(expr),
                        op,
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, GupError> {
        let mut expr = self.parse_additive()?;

        loop {
            let op = match self.current_kind() {
                TokenKind::Less => Some(BinaryOp::Less),
                TokenKind::Greater => Some(BinaryOp::Greater),
                TokenKind::LessEqual => Some(BinaryOp::LessEqual),
                TokenKind::GreaterEqual => Some(BinaryOp::GreaterEqual),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_additive()?;
                let span = expr.span.merge(right.span);
                expr = Expr::new(
                    ExprKind::BinaryOp {
                        left: Box::new(expr),
                        op,
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr, GupError> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = match self.current_kind() {
                TokenKind::Plus => Some(BinaryOp::Add),
                TokenKind::Minus => Some(BinaryOp::Sub),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_multiplicative()?;
                let span = expr.span.merge(right.span);
                expr = Expr::new(
                    ExprKind::BinaryOp {
                        left: Box::new(expr),
                        op,
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, GupError> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.current_kind() {
                TokenKind::Star => Some(BinaryOp::Mul),
                TokenKind::Slash => Some(BinaryOp::Div),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_unary()?;
                let span = expr.span.merge(right.span);
                expr = Expr::new(
                    ExprKind::BinaryOp {
                        left: Box::new(expr),
                        op,
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, GupError> {
        if *self.current_kind() == TokenKind::Not {
            let op = self.advance();
            let operand = self.parse_unary()?;
            let span = op.span.merge(operand.span);
            return Ok(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                },
                span,
            ));
        }

        if *self.current_kind() == TokenKind::Minus {
            let op = self.advance();
            let operand = self.parse_unary()?;
            let span = op.span.merge(operand.span);
            return Ok(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                },
                span,
            ));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, GupError> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(Expr::new(ExprKind::StringLiteral(s), token.span))
            }
            TokenKind::CharLiteral(ch) => {
                self.advance();
                Ok(Expr::new(ExprKind::CharLiteral(ch), token.span))
            }
            TokenKind::NumberLiteral(n) => {
                self.advance();
                Ok(Expr::new(ExprKind::NumberLiteral(n), token.span))
            }
            TokenKind::FloatLiteral(f) => {
                self.advance();
                Ok(Expr::new(ExprKind::FloatLiteral(f), token.span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::new(ExprKind::BoolLiteral(true), token.span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::new(ExprKind::BoolLiteral(false), token.span))
            }
            TokenKind::LeftParen => {
                let open = self.advance();
                let mut expr = self.parse_expression()?;
                let close = self.expect_kind(&TokenKind::RightParen)?;
                expr.span = open.span.merge(close.span);
                Ok(expr)
            }
            TokenKind::LeftBracket => {
                let open = self.advance();
                if *self.current_kind() == TokenKind::RightBracket {
                    let close = self.advance();
                    Ok(Expr::new(
                        ExprKind::ArrayLiteral(Vec::new()),
                        open.span.merge(close.span),
                    ))
                } else {
                    Err(GupError::parse(
                        self.current_span(),
                        "Only empty arrays [] are supported right now.",
                    ))
                }
            }
            TokenKind::Range => {
                let range = self.advance();
                self.parse_range_expr(range.span)
            }
            TokenKind::Identifier(name) => {
                let name_token = self.advance();

                if *self.current_kind() == TokenKind::LeftParen {
                    self.advance();
                    let mut args: Vec<Expr> = Vec::new();

                    if *self.current_kind() != TokenKind::RightParen {
                        loop {
                            args.push(self.parse_expression()?);
                            if *self.current_kind() == TokenKind::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }

                    let close = self.expect_kind(&TokenKind::RightParen)?;
                    let span = name_token.span.merge(close.span);
                    Ok(Expr::new(ExprKind::FunctionCall { name, args }, span))
                } else {
                    Ok(Expr::new(ExprKind::Variable(name), name_token.span))
                }
            }
            other => Err(GupError::parse(
                token.span,
                format!(
                    "I expected an expression but found {}.",
                    describe_kind(&other)
                ),
            )),
        }
    }

    fn parse_range_expr(&mut self, range_span: Span) -> Result<Expr, GupError> {
        self.expect_kind(&TokenKind::LeftParen)?;

        let start = self.parse_expression()?;
        self.expect_kind(&TokenKind::Through)?;
        let end = self.parse_expression()?;
        let close = self.expect_kind(&TokenKind::RightParen)?;

        Ok(Expr::new(
            ExprKind::Range {
                start: Box::new(start),
                end: Box::new(end),
            },
            range_span.merge(close.span),
        ))
    }
}

fn describe_kind(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Identifier(name) => format!("name '{}'", name),
        TokenKind::StringLiteral(text) => format!("string \"{}\"", text),
        TokenKind::CharLiteral(ch) => format!("char '{}'", ch),
        TokenKind::NumberLiteral(n) => format!("number {}", n),
        TokenKind::FloatLiteral(f) => format!("number {}", f),
        TokenKind::True => "true".to_string(),
        TokenKind::False => "false".to_string(),
        TokenKind::For => "'for'".to_string(),
        TokenKind::In => "'in'".to_string(),
        TokenKind::Range => "'range'".to_string(),
        TokenKind::Through => "'through'".to_string(),
        TokenKind::If => "'if'".to_string(),
        TokenKind::Else => "'else'".to_string(),
        TokenKind::While => "'while'".to_string(),
        TokenKind::Return => "'return'".to_string(),
        TokenKind::And => "'and'".to_string(),
        TokenKind::Or => "'or'".to_string(),
        TokenKind::Not => "'not'".to_string(),
        TokenKind::Plus => "'+'".to_string(),
        TokenKind::Minus => "'-'".to_string(),
        TokenKind::Star => "'*'".to_string(),
        TokenKind::Slash => "'/'".to_string(),
        TokenKind::Equal => "'='".to_string(),
        TokenKind::EqualEqual => "'=='".to_string(),
        TokenKind::BangEqual => "'!='".to_string(),
        TokenKind::Less => "'<'".to_string(),
        TokenKind::Greater => "'>'".to_string(),
        TokenKind::LessEqual => "'<='".to_string(),
        TokenKind::GreaterEqual => "'>='".to_string(),
        TokenKind::LeftParen => "'('".to_string(),
        TokenKind::RightParen => "')'".to_string(),
        TokenKind::LeftBracket => "'['".to_string(),
        TokenKind::RightBracket => "']'".to_string(),
        TokenKind::Semicolon => "';'".to_string(),
        TokenKind::Comma => "','".to_string(),
        TokenKind::Newline => "end of line".to_string(),
        TokenKind::Indent => "an indented block".to_string(),
        TokenKind::Dedent => "the end of an indented block".to_string(),
        TokenKind::EOF => "end of file".to_string(),
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, GupError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// =============================================================================
// UNIT TESTS — we check that the parser builds the right tree from tokens!
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, ExprKind, StmtKind};
    use crate::lexer;

    fn parse_source(source: &str) -> Program {
        parse(lexer::lex(source).unwrap()).unwrap()
    }

    #[test]
    fn parses_variable_declaration() {
        let program = parse_source("x = 5");

        assert_eq!(program.len(), 1);

        match &program[0].kind {
            StmtKind::VariableDeclaration { name, value } => {
                assert_eq!(name, "x");
                assert!(matches!(value.kind, ExprKind::NumberLiteral(5)));
            }
            other => panic!("expected variable declaration, got {:?}", other),
        }
    }

    #[test]
    fn parses_out_function_call() {
        let program = parse_source(r#"out("hello")"#);

        assert_eq!(program.len(), 1);

        match &program[0].kind {
            StmtKind::ExpressionStatement(expr) => match &expr.kind {
                ExprKind::FunctionCall { name, args } => {
                    assert_eq!(name, "out");
                    assert_eq!(args.len(), 1);
                    assert!(matches!(&args[0].kind, ExprKind::StringLiteral(s) if s == "hello"));
                }
                other => panic!("expected out() call, got {:?}", other),
            },
            other => panic!("expected expression statement, got {:?}", other),
        }
    }

    #[test]
    fn parses_operator_precedence() {
        let program = parse_source("out(2 + 3 * 4)");

        match &program[0].kind {
            StmtKind::ExpressionStatement(expr) => match &expr.kind {
                ExprKind::FunctionCall { args, .. } => match &args[0].kind {
                    ExprKind::BinaryOp { left, op, right } => {
                        assert_eq!(*op, BinaryOp::Add);
                        assert!(matches!(left.kind, ExprKind::NumberLiteral(2)));
                        match &right.kind {
                            ExprKind::BinaryOp {
                                op: BinaryOp::Mul, ..
                            } => {}
                            other => panic!("expected multiply on the right, got {:?}", other),
                        }
                    }
                    other => panic!("expected binary op, got {:?}", other),
                },
                other => panic!("expected function call, got {:?}", other),
            },
            other => panic!("expected expression statement, got {:?}", other),
        }
    }

    #[test]
    fn parses_for_loop() {
        let source = "for i in range(1 through 3)\n    out(i)";
        let program = parse_source(source);

        match &program[0].kind {
            StmtKind::ForLoop {
                variable,
                iterable,
                body,
            } => {
                assert_eq!(variable, "i");
                assert!(matches!(
                    &iterable.kind,
                    ExprKind::Range { start, end }
                    if matches!(start.kind, ExprKind::NumberLiteral(1))
                        && matches!(end.kind, ExprKind::NumberLiteral(3))
                ));
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected for loop, got {:?}", other),
        }
    }

    #[test]
    fn parses_function_definition() {
        let source = "greet()\n    out(\"hi\")";
        let program = parse_source(source);

        match &program[0].kind {
            StmtKind::FunctionDef { name, params, body } => {
                assert_eq!(name, "greet");
                assert!(params.is_empty());
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected function def, got {:?}", other),
        }
    }

    #[test]
    fn parses_literals() {
        let program = parse_source("a = true\nb = false\nc = 'x'\nd = \"hi\"\ne = []");

        assert_eq!(program.len(), 5);

        assert!(matches!(
            &program[0].kind,
            StmtKind::VariableDeclaration {
                value: Expr {
                    kind: ExprKind::BoolLiteral(true),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            &program[1].kind,
            StmtKind::VariableDeclaration {
                value: Expr {
                    kind: ExprKind::BoolLiteral(false),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            &program[2].kind,
            StmtKind::VariableDeclaration {
                value: Expr {
                    kind: ExprKind::CharLiteral('x'),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            &program[3].kind,
            StmtKind::VariableDeclaration {
                value: Expr { kind: ExprKind::StringLiteral(s), .. },
                ..
            } if s == "hi"
        ));
        assert!(matches!(
            &program[4].kind,
            StmtKind::VariableDeclaration {
                value: Expr { kind: ExprKind::ArrayLiteral(items), .. },
                ..
            } if items.is_empty()
        ));
    }

    #[test]
    fn parse_errors_have_locations_and_plain_words() {
        let tokens = lexer::lex("out(1 + )").unwrap();
        let error = parse(tokens).unwrap_err();

        assert_eq!(error.span.line, 1);
        assert_eq!(error.span.column, 9);
        assert!(error.message.contains("expected an expression"));
    }
}
