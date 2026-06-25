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

// =============================================================================
// UNIT TESTS — we check that the parser builds the right tree from tokens!
// If these break, the interpreter might do the wrong thing. Super important!
// =============================================================================
#[cfg(test)]
mod tests {
    // parser stuff from this file
    use super::*;
    // we need the lexer to turn text into tokens first (parser eats tokens)
    use crate::lexer;
    // we need AST shapes to check "did we build the right tree?"
    use crate::ast::{BinaryOp, Expr, Stmt};

    // -------------------------------------------------------------------------
    // helper: shorthand — text in, program tree out
    // why? every test does lex then parse, so don't repeat that every time
    // -------------------------------------------------------------------------
    fn parse_source(source: &str) -> Program {
        parse(lexer::lex(source))
    }

    // -------------------------------------------------------------------------
    // TEST: x = 5 should become a VariableDeclaration statement
    // why? storing stuff in boxes (variables) is how programs remember things
    // -------------------------------------------------------------------------
    #[test]
    fn parses_variable_declaration() {
        let program = parse_source("x = 5");

        // we should have exactly one statement
        assert_eq!(program.len(), 1);

        // and it should be "put 5 in a box named x"
        match &program[0] {
            Stmt::VariableDeclaration { name, value } => {
                assert_eq!(name, "x");
                assert!(matches!(value, Expr::NumberLiteral(5)));
            }
            other => panic!("expected variable declaration, got {:?}", other),
        }
    }

    // -------------------------------------------------------------------------
    // TEST: out("hello") is an ExpressionStatement with a FunctionCall inside
    // why? printing is the #1 thing beginners do — it must parse right!
    // -------------------------------------------------------------------------
    #[test]
    fn parses_out_function_call() {
        let program = parse_source(r#"out("hello")"#);

        assert_eq!(program.len(), 1);

        match &program[0] {
            Stmt::ExpressionStatement(Expr::FunctionCall { name, args }) => {
                assert_eq!(name, "out");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Expr::StringLiteral(s) if s == "hello"));
            }
            other => panic!("expected out() call, got {:?}", other),
        }
    }

    // -------------------------------------------------------------------------
    // TEST: 2 + 3 * 4 should multiply BEFORE adding (math rules!)
    // why? wrong order = wrong answer = angry users
    // -------------------------------------------------------------------------
    #[test]
    fn parses_operator_precedence() {
        let program = parse_source("out(2 + 3 * 4)");

        match &program[0] {
            Stmt::ExpressionStatement(Expr::FunctionCall { args, .. }) => {
                match &args[0] {
                    Expr::BinaryOp { left, op, right } => {
                        // the top level should be PLUS (add happens last)
                        assert_eq!(*op, BinaryOp::Add);
                        assert!(matches!(left.as_ref(), Expr::NumberLiteral(2)));
                        // the right side should be 3 * 4 (multiply first)
                        match right.as_ref() {
                            Expr::BinaryOp {
                                op: BinaryOp::Mul,
                                ..
                            } => {}
                            other => panic!("expected multiply on the right, got {:?}", other),
                        }
                    }
                    other => panic!("expected binary op, got {:?}", other),
                }
            }
            other => panic!("expected expression statement, got {:?}", other),
        }
    }

    // -------------------------------------------------------------------------
    // TEST: for i in range(1 through 3) should become a ForLoop statement
    // why? loops let you do something many times without copy-pasting code
    // -------------------------------------------------------------------------
    #[test]
    fn parses_for_loop() {
        let source = "for i in range(1 through 3)\n    out(i)";
        let program = parse_source(source);

        match &program[0] {
            Stmt::ForLoop {
                variable,
                iterable,
                body,
            } => {
                assert_eq!(variable, "i");
                assert!(matches!(
                    iterable,
                    Expr::Range { start, end }
                    if matches!(start.as_ref(), Expr::NumberLiteral(1))
                        && matches!(end.as_ref(), Expr::NumberLiteral(3))
                ));
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected for loop, got {:?}", other),
        }
    }

    // -------------------------------------------------------------------------
    // TEST: greet() with an indented body is a FunctionDef
    // why? functions are reusable recipe cards — define once, use many times
    // -------------------------------------------------------------------------
    #[test]
    fn parses_function_definition() {
        let source = "greet()\n    out(\"hi\")";
        let program = parse_source(source);

        match &program[0] {
            Stmt::FunctionDef { name, body } => {
                assert_eq!(name, "greet");
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected function def, got {:?}", other),
        }
    }

    // -------------------------------------------------------------------------
    // TEST: true, false, 'a', "hi", [] all parse as the right literal types
    // why? different kinds of data need different boxes in the interpreter
    // -------------------------------------------------------------------------
    #[test]
    fn parses_literals() {
        let program = parse_source("a = true\nb = false\nc = 'x'\nd = \"hi\"\ne = []");

        assert_eq!(program.len(), 5);

        assert!(matches!(
            &program[0],
            Stmt::VariableDeclaration {
                value: Expr::BoolLiteral(true),
                ..
            }
        ));
        assert!(matches!(
            &program[1],
            Stmt::VariableDeclaration {
                value: Expr::BoolLiteral(false),
                ..
            }
        ));
        assert!(matches!(
            &program[2],
            Stmt::VariableDeclaration {
                value: Expr::CharLiteral('x'),
                ..
            }
        ));
        assert!(matches!(
            &program[3],
            Stmt::VariableDeclaration {
                value: Expr::StringLiteral(s),
                ..
            } if s == "hi"
        ));
        assert!(matches!(
            &program[4],
            Stmt::VariableDeclaration {
                value: Expr::ArrayLiteral(items),
                ..
            } if items.is_empty()
        ));
    }
}
