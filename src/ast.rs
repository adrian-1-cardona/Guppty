// === ast.rs ===
// AST = abstract syntax tree = a family tree for your code!
// it shows which parts belong together so the interpreter knows what to do.

#[derive(Debug, Clone)]
pub enum Expr {
    StringLiteral(String),
    CharLiteral(char),
    NumberLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    ArrayLiteral(Vec<Expr>),
    Variable(String),
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
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
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    ExpressionStatement(Expr),
    VariableDeclaration {
        name: String,
        value: Expr,
    },
    Block {
        statements: Vec<Stmt>,
    },
    IfStatement {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    WhileLoop {
        condition: Expr,
        body: Vec<Stmt>,
    },
    ForLoop {
        variable: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    ReturnStatement {
        value: Option<Expr>,
    },
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
}

pub type Program = Vec<Stmt>;
