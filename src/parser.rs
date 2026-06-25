// === parser.rs ===
// The parser figures out what tokens MEAN together and builds the AST.

use crate::ast::{BinaryOp, Expr, Program, Stmt};
use crate::token::Token;

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        if self.pos >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.pos]
        }
    }

    fn peek(&self, offset: usize) -> &Token {
        let index = self.pos + offset;
        if index >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[index]
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: &Token) {
        let current = self.current().clone();
        if &current == expected {
            self.advance();
        } else {
            panic!(
                "Parser error! Expected {:?} but found {:?}",
                expected, current
            );
        }
    }

    fn skip_newlines(&mut self) {
        while *self.current() == Token::Newline {
            self.advance();
        }
    }

    fn parse_program(&mut self) -> Program {
        let mut statements: Vec<Stmt> = Vec::new();

        self.skip_newlines();
        while *self.current() != Token::EOF {
            let stmt = self.parse_statement();
            statements.push(stmt);
            self.skip_newlines();
        }

        statements
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(&Token::Indent);
        let mut statements: Vec<Stmt> = Vec::new();

        self.skip_newlines();
        while *self.current() != Token::Dedent && *self.current() != Token::EOF {
            statements.push(self.parse_statement());
            self.skip_newlines();
        }

        if *self.current() == Token::Dedent {
            self.advance();
        }

        statements
    }

    fn parse_statement(&mut self) -> Stmt {
        // for i in range(1 through 6)
        if *self.current() == Token::For {
            return self.parse_for_loop();
        }

        // name = value;
        if let Token::Identifier(name) = self.current().clone() {
            if *self.peek(1) == Token::Equal {
                return self.parse_variable_declaration(name);
            }

            // name() followed by indented block => function definition
            if *self.peek(1) == Token::LeftParen
                && *self.peek(2) == Token::RightParen
                && (*self.peek(3) == Token::Newline || *self.peek(3) == Token::Indent)
            {
                let next = self.peek(4);
                if *self.peek(3) == Token::Indent || *next == Token::Indent {
                    return self.parse_function_def(name);
                }
            }
        }

        let expr = self.parse_expression();
        self.consume_optional_semicolon();
        Stmt::ExpressionStatement(expr)
    }

    fn parse_function_def(&mut self, name: String) -> Stmt {
        self.advance(); // identifier already known
        self.expect(&Token::LeftParen);
        self.expect(&Token::RightParen);
        self.skip_newlines();

        let body = if *self.current() == Token::Indent {
            self.parse_block()
        } else {
            Vec::new()
        };

        Stmt::FunctionDef { name, body }
    }

    fn parse_for_loop(&mut self) -> Stmt {
        self.expect(&Token::For);

        let variable = match self.advance() {
            Token::Identifier(name) => name,
            other => panic!("Expected a loop variable name, found {:?}", other),
        };

        self.expect(&Token::In);
        let iterable = self.parse_expression();
        self.skip_newlines();

        let body = if *self.current() == Token::Indent {
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

    fn parse_variable_declaration(&mut self, name: String) -> Stmt {
        self.advance(); // identifier
        self.expect(&Token::Equal);
        let value = self.parse_expression();
        self.consume_optional_semicolon();

        Stmt::VariableDeclaration { name, value }
    }

    fn consume_optional_semicolon(&mut self) {
        if *self.current() == Token::Semicolon {
            self.advance();
        }
        if *self.current() == Token::Newline {
            self.advance();
        }
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Expr {
        let mut expr = self.parse_multiplicative();

        loop {
            match self.current() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Add,
                        right: Box::new(right),
                    };
                }
                Token::Minus => {
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
        let mut expr = self.parse_primary();

        loop {
            match self.current() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_primary();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: BinaryOp::Mul,
                        right: Box::new(right),
                    };
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_primary();
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

    fn parse_primary(&mut self) -> Expr {
        match self.current().clone() {
            Token::StringLiteral(s) => {
                self.advance();
                Expr::StringLiteral(s)
            }
            Token::CharLiteral(ch) => {
                self.advance();
                Expr::CharLiteral(ch)
            }
            Token::NumberLiteral(n) => {
                self.advance();
                Expr::NumberLiteral(n)
            }
            Token::FloatLiteral(f) => {
                self.advance();
                Expr::FloatLiteral(f)
            }
            Token::True => {
                self.advance();
                Expr::BoolLiteral(true)
            }
            Token::False => {
                self.advance();
                Expr::BoolLiteral(false)
            }
            Token::LeftBracket => {
                self.advance();
                if *self.current() == Token::RightBracket {
                    self.advance();
                    Expr::ArrayLiteral(Vec::new())
                } else {
                    panic!("Only empty arrays [] are supported right now");
                }
            }
            Token::Range => {
                self.advance();
                self.parse_range_expr()
            }
            Token::Identifier(name) => {
                self.advance();

                if *self.current() == Token::LeftParen {
                    self.advance();
                    let mut args: Vec<Expr> = Vec::new();

                    while *self.current() != Token::RightParen {
                        args.push(self.parse_expression());
                        if *self.current() == Token::Comma {
                            self.advance();
                        }
                    }

                    self.expect(&Token::RightParen);
                    Expr::FunctionCall { name, args }
                } else {
                    Expr::Variable(name)
                }
            }
            other => panic!("I don't understand this token in an expression: {:?}", other),
        }
    }

    fn parse_range_expr(&mut self) -> Expr {
        self.expect(&Token::LeftParen);

        let start = self.parse_expression();
        self.expect(&Token::Through);
        let end = self.parse_expression();
        self.expect(&Token::RightParen);

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
