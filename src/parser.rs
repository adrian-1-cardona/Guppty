// === parser.rs ===
// the parser is a detective — it reads tokens and figures out the story!
// it builds the AST tree so the interpreter knows what to run and in what order.

use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::token::{Token, TokenKind};

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> &TokenKind {
        if self.pos >= self.tokens.len() {
            &TokenKind::EOF
        } else {
            &self.tokens[self.pos].kind
        }
    }

    fn current_line(&self) -> usize {
        if self.pos >= self.tokens.len() {
            0
        } else {
            self.tokens[self.pos].line
        }
    }

    fn peek(&self, offset: usize) -> &TokenKind {
        let index = self.pos + offset;
        if index >= self.tokens.len() {
            &TokenKind::EOF
        } else {
            &self.tokens[index].kind
        }
    }

    fn advance(&mut self) -> TokenKind {
        let kind = self.tokens[self.pos].kind.clone();
        self.pos += 1;
        kind
    }

    fn expect_kind(&mut self, expected: &TokenKind) {
        let current = self.current().clone();
        if &current == expected {
            self.advance();
        } else {
            panic!(
                "Line {}: Parser error! Expected {:?} but found {:?}",
                self.current_line(),
                expected,
                current
            );
        }
    }

    fn skip_newlines(&mut self) {
        while *self.current() == TokenKind::Newline {
            self.advance();
        }
    }

    fn parse_program(&mut self) -> Program {
        let mut statements: Vec<Stmt> = Vec::new();

        self.skip_newlines();
        while *self.current() != TokenKind::EOF {
            statements.push(self.parse_statement());
            self.skip_newlines();
        }

        statements
    }

    /// read an indented block — everything pushed in one tab level
    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect_kind(&TokenKind::Indent);
        let mut statements: Vec<Stmt> = Vec::new();

        self.skip_newlines();
        while *self.current() != TokenKind::Dedent && *self.current() != TokenKind::EOF {
            statements.push(self.parse_statement());
            self.skip_newlines();
        }

        if *self.current() == TokenKind::Dedent {
            self.advance();
        }

        statements
    }

    /// a block if indented, otherwise one single statement
    fn parse_block_or_single(&mut self) -> Vec<Stmt> {
        if *self.current() == TokenKind::Indent {
            self.parse_block()
        } else {
            vec![self.parse_statement()]
        }
    }

    fn parse_statement(&mut self) -> Stmt {
        // return 5  or  return
        if *self.current() == TokenKind::Return {
            return self.parse_return_statement();
        }

        // if condition ... else ...
        if *self.current() == TokenKind::If {
            return self.parse_if_statement();
        }

        // while condition ...
        if *self.current() == TokenKind::While {
            return self.parse_while_loop();
        }

        // for i in range(1 through 6)
        if *self.current() == TokenKind::For {
            return self.parse_for_loop();
        }

        // name = value  OR  name(args) { block }  OR  just an expression
        if let TokenKind::Identifier(name) = self.current().clone() {
            if *self.peek(1) == TokenKind::Equal {
                return self.parse_variable_declaration(name);
            }

            if *self.peek(1) == TokenKind::LeftParen {
                if self.looks_like_function_definition() {
                    return self.parse_function_def(name);
                }
            }
        }

        let expr = self.parse_expression();
        self.consume_optional_semicolon();
        Stmt::ExpressionStatement(expr)
    }

    /// tell function definitions apart from function calls
    /// a definition MUST have an indented block after the closing paren!
    fn looks_like_function_definition(&self) -> bool {
        let mut index = self.pos + 2;
        let mut depth = 1;

        while index < self.tokens.len() {
            match &self.tokens[index].kind {
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => {
                    depth -= 1;
                    if depth == 0 {
                        let mut next_index = index + 1;
                        while next_index < self.tokens.len()
                            && self.tokens[next_index].kind == TokenKind::Newline
                        {
                            next_index += 1;
                        }

                        return self.tokens[next_index].kind == TokenKind::Indent;
                    }
                }
                _ => {}
            }
            index += 1;
        }

        false
    }

    fn parse_function_def(&mut self, name: String) -> Stmt {
        self.advance(); // identifier
        self.expect_kind(&TokenKind::LeftParen);

        let mut params: Vec<String> = Vec::new();
        if *self.current() != TokenKind::RightParen {
            loop {
                match self.advance() {
                    TokenKind::Identifier(param) => params.push(param),
                    other => panic!(
                        "Line {}: Function parameters need names, found {:?}",
                        self.current_line(),
                        other
                    ),
                }

                if *self.current() == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect_kind(&TokenKind::RightParen);
        self.skip_newlines();

        let body = if *self.current() == TokenKind::Indent {
            self.parse_block()
        } else {
            Vec::new()
        };

        Stmt::FunctionDef { name, params, body }
    }

    fn parse_for_loop(&mut self) -> Stmt {
        self.expect_kind(&TokenKind::For);

        let variable = match self.advance() {
            TokenKind::Identifier(name) => name,
            other => panic!(
                "Line {}: Expected a loop variable name, found {:?}",
                self.current_line(),
                other
            ),
        };

        self.expect_kind(&TokenKind::In);
        let iterable = self.parse_expression();
        self.skip_newlines();

        let body = if *self.current() == TokenKind::Indent {
            self.parse_block()
        } else {
            Vec::new()
        };

        Stmt::ForLoop {
            variable,
            iterable,
            body,
        }
    }

    fn parse_while_loop(&mut self) -> Stmt {
        self.expect_kind(&TokenKind::While);
        let condition = self.parse_expression();
        self.skip_newlines();

        let body = self.parse_block_or_single();

        Stmt::WhileLoop { condition, body }
    }

    fn parse_if_statement(&mut self) -> Stmt {
        self.expect_kind(&TokenKind::If);
        let condition = self.parse_expression();
        self.skip_newlines();

        let then_branch = self.parse_block_or_single();
        self.skip_newlines();

        let else_branch = if *self.current() == TokenKind::Else {
            self.advance();
            self.skip_newlines();
            Some(self.parse_block_or_single())
        } else {
            None
        };

        Stmt::IfStatement {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn parse_return_statement(&mut self) -> Stmt {
        self.expect_kind(&TokenKind::Return);

        let value = if self.is_expression_start() {
            Some(self.parse_expression())
        } else {
            None
        };

        self.consume_optional_semicolon();
        Stmt::ReturnStatement { value }
    }

    fn is_expression_start(&self) -> bool {
        matches!(
            self.current(),
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

    fn parse_variable_declaration(&mut self, name: String) -> Stmt {
        self.advance(); // identifier
        self.expect_kind(&TokenKind::Equal);
        let value = self.parse_expression();
        self.consume_optional_semicolon();

        Stmt::VariableDeclaration { name, value }
    }

    fn consume_optional_semicolon(&mut self) {
        if *self.current() == TokenKind::Semicolon {
            self.advance();
        }
        if *self.current() == TokenKind::Newline {
            self.advance();
        }
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_or()
    }

    // or has the lowest priority — it decides last
    fn parse_or(&mut self) -> Expr {
        let mut expr = self.parse_and();

        while *self.current() == TokenKind::Or {
            self.advance();
            let right = self.parse_and();
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_and(&mut self) -> Expr {
        let mut expr = self.parse_equality();

        while *self.current() == TokenKind::And {
            self.advance();
            let right = self.parse_equality();
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_equality(&mut self) -> Expr {
        let mut expr = self.parse_comparison();

        loop {
            match self.current() {
                TokenKind::EqualEqual => {
                    self.advance();
                    let right = self.parse_comparison();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Equal,
                        right: Box::new(right),
                    };
                }
                TokenKind::BangEqual => {
                    self.advance();
                    let right = self.parse_comparison();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::NotEqual,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut expr = self.parse_additive();

        loop {
            match self.current() {
                TokenKind::Less => {
                    self.advance();
                    let right = self.parse_additive();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Less,
                        right: Box::new(right),
                    };
                }
                TokenKind::Greater => {
                    self.advance();
                    let right = self.parse_additive();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Greater,
                        right: Box::new(right),
                    };
                }
                TokenKind::LessEqual => {
                    self.advance();
                    let right = self.parse_additive();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::LessEqual,
                        right: Box::new(right),
                    };
                }
                TokenKind::GreaterEqual => {
                    self.advance();
                    let right = self.parse_additive();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::GreaterEqual,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_additive(&mut self) -> Expr {
        let mut expr = self.parse_multiplicative();

        loop {
            match self.current() {
                TokenKind::Plus => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Add,
                        right: Box::new(right),
                    };
                }
                TokenKind::Minus => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Sub,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_multiplicative(&mut self) -> Expr {
        let mut expr = self.parse_unary();

        loop {
            match self.current() {
                TokenKind::Star => {
                    self.advance();
                    let right = self.parse_unary();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Mul,
                        right: Box::new(right),
                    };
                }
                TokenKind::Slash => {
                    self.advance();
                    let right = self.parse_unary();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Div,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_unary(&mut self) -> Expr {
        if *self.current() == TokenKind::Not {
            self.advance();
            let operand = self.parse_unary();
            return Expr::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            };
        }

        if *self.current() == TokenKind::Minus {
            self.advance();
            let operand = self.parse_unary();
            return Expr::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            };
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current().clone() {
            TokenKind::StringLiteral(s) => {
                self.advance();
                Expr::StringLiteral(s)
            }
            TokenKind::CharLiteral(ch) => {
                self.advance();
                Expr::CharLiteral(ch)
            }
            TokenKind::NumberLiteral(n) => {
                self.advance();
                Expr::NumberLiteral(n)
            }
            TokenKind::FloatLiteral(f) => {
                self.advance();
                Expr::FloatLiteral(f)
            }
            TokenKind::True => {
                self.advance();
                Expr::BoolLiteral(true)
            }
            TokenKind::False => {
                self.advance();
                Expr::BoolLiteral(false)
            }
            TokenKind::LeftBracket => {
                self.advance();
                if *self.current() == TokenKind::RightBracket {
                    self.advance();
                    Expr::ArrayLiteral(Vec::new())
                } else {
                    panic!(
                        "Line {}: Only empty arrays [] are supported right now",
                        self.current_line()
                    );
                }
            }
            TokenKind::Range => {
                self.advance();
                self.parse_range_expr()
            }
            TokenKind::Identifier(name) => {
                self.advance();

                if *self.current() == TokenKind::LeftParen {
                    self.advance();
                    let mut args: Vec<Expr> = Vec::new();

                    while *self.current() != TokenKind::RightParen {
                        args.push(self.parse_expression());
                        if *self.current() == TokenKind::Comma {
                            self.advance();
                        }
                    }

                    self.expect_kind(&TokenKind::RightParen);
                    Expr::FunctionCall { name, args }
                } else {
                    Expr::Variable(name)
                }
            }
            other => panic!(
                "Line {}: I don't understand this token in an expression: {:?}",
                self.current_line(),
                other
            ),
        }
    }

    fn parse_range_expr(&mut self) -> Expr {
        self.expect_kind(&TokenKind::LeftParen);

        let start = self.parse_expression();
        self.expect_kind(&TokenKind::Through);
        let end = self.parse_expression();
        self.expect_kind(&TokenKind::RightParen);

        Expr::Range {
            start: Box::new(start),
            end: Box::new(end),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Program {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}
