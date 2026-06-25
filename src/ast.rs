// === ast.rs ===
// AST stands for "Abstract Syntax Tree."
// Think of it like a family tree, but for your code!

#[derive(Debug, Clone)]
pub enum Expr {
    StringLiteral(String),
    CharLiteral(char),
    NumberLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    ArrayLiteral(Vec<Expr>),
    Variable(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    ExpressionStatement(Expr),
    VariableDeclaration {
        name: String,
        value: Expr,
    },
    ForLoop {
        variable: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    FunctionDef {
        name: String,
        body: Vec<Stmt>,
    },
}

pub type Program = Vec<Stmt>;
