// === interpreter.rs ===
// The interpreter walks through the AST and actually runs the program.

use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Program, Stmt};
use crate::value::Value;

#[derive(Debug, Clone)]
struct Function {
    body: Vec<Stmt>,
}

struct Environment {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

pub fn interpret(program: Program) {
    let mut env = Environment::new();

    // First pass: register all function definitions
    for statement in &program {
        if let Stmt::FunctionDef { name, body } = statement {
            env.functions.insert(
                name.clone(),
                Function {
                    body: body.clone(),
                },
            );
        }
    }

    // Second pass: execute non-function-def statements
    for statement in &program {
        if !matches!(statement, Stmt::FunctionDef { .. }) {
            execute_statement(statement, &mut env);
        }
    }
}

fn execute_statement(stmt: &Stmt, env: &mut Environment) {
    match stmt {
        Stmt::ExpressionStatement(expr) => {
            evaluate_expression(expr, env);
        }
        Stmt::VariableDeclaration { name, value } => {
            let evaluated = evaluate_expression(value, env);
            env.variables.insert(name.clone(), evaluated);
        }
        Stmt::ForLoop {
            variable,
            iterable,
            body,
        } => {
            let values = evaluate_iterable(iterable, env);

            for value in values {
                env.variables.insert(variable.clone(), value);
                for body_stmt in body {
                    execute_statement(body_stmt, env);
                }
            }
        }
        Stmt::FunctionDef { .. } => {
            // Already registered in the first pass
        }
    }
}

fn evaluate_iterable(expr: &Expr, env: &mut Environment) -> Vec<Value> {
    match expr {
        Expr::Range { start, end } => {
            let start_val = evaluate_expression(start, env);
            let end_val = evaluate_expression(end, env);

            let start_num = start_val
                .as_number()
                .unwrap_or_else(|msg| panic!("{}", msg));
            let end_num = end_val
                .as_number()
                .unwrap_or_else(|msg| panic!("{}", msg));

            let mut values = Vec::new();
            let mut current = start_num as i64;
            let end = end_num as i64;

            if current <= end {
                while current <= end {
                    values.push(Value::GuppyNumber(current));
                    current += 1;
                }
            } else {
                while current >= end {
                    values.push(Value::GuppyNumber(current));
                    current -= 1;
                }
            }

            values
        }
        other => {
            let value = evaluate_expression(other, env);
            vec![value]
        }
    }
}

fn evaluate_expression(expr: &Expr, env: &mut Environment) -> Value {
    match expr {
        Expr::StringLiteral(text) => Value::GuppyString(text.clone()),
        Expr::CharLiteral(ch) => Value::GuppyChar(*ch),
        Expr::NumberLiteral(n) => Value::GuppyNumber(*n),
        Expr::FloatLiteral(f) => Value::GuppyFloat(*f),
        Expr::BoolLiteral(b) => Value::GuppyBool(*b),
        Expr::ArrayLiteral(items) => {
            let values = items
                .iter()
                .map(|item| evaluate_expression(item, env))
                .collect();
            Value::GuppyArray(values)
        }
        Expr::Variable(name) => env
            .variables
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("Variable '{}' is not defined yet!", name)),
        Expr::BinaryOp { left, op, right } => {
            let left_val = evaluate_expression(left, env);
            let right_val = evaluate_expression(right, env);
            evaluate_binary_op(&left_val, *op, &right_val)
        }
        Expr::FunctionCall { name, args } => {
            let evaluated_args: Vec<Value> = args
                .iter()
                .map(|arg| evaluate_expression(arg, env))
                .collect();

            match name.as_str() {
                "out" => {
                    if evaluated_args.is_empty() {
                        println!();
                    } else {
                        let output: Vec<String> = evaluated_args
                            .iter()
                            .map(|v| v.to_display_string())
                            .collect();
                        println!("{}", output.join(" "));
                    }
                    Value::Nothing
                }
                unknown => {
                    if let Some(function) = env.functions.get(unknown).cloned() {
                        let mut call_env = Environment::new();
                        call_env.functions = env.functions.clone();
                        call_env.variables = env.variables.clone();

                        for body_stmt in &function.body {
                            execute_statement(body_stmt, &mut call_env);
                        }

                        env.variables = call_env.variables;
                        Value::Nothing
                    } else {
                        panic!(
                            "I don't know a function called '{}'! Define it first or use a built-in like out().",
                            unknown
                        );
                    }
                }
            }
        }
        Expr::Range { .. } => {
            panic!("range() can only be used inside a for loop like: for i in range(1 through 6)");
        }
    }
}

fn evaluate_binary_op(left: &Value, op: BinaryOp, right: &Value) -> Value {
    match (left, right) {
        (Value::GuppyString(a), Value::GuppyString(b)) if op == BinaryOp::Add => {
            Value::GuppyString(format!("{}{}", a, b))
        }
        _ => {
            let left_num = left
                .as_number()
                .unwrap_or_else(|msg| panic!("{}", msg));
            let right_num = right
                .as_number()
                .unwrap_or_else(|msg| panic!("{}", msg));

            let result = match op {
                BinaryOp::Add => left_num + right_num,
                BinaryOp::Sub => left_num - right_num,
                BinaryOp::Mul => left_num * right_num,
                BinaryOp::Div => {
                    if right_num == 0.0 {
                        panic!("Cannot divide by zero!");
                    }
                    left_num / right_num
                }
            };

            let is_float = matches!(left, Value::GuppyFloat(_))
                || matches!(right, Value::GuppyFloat(_));

            if is_float {
                Value::GuppyFloat(result)
            } else {
                Value::GuppyNumber(result as i64)
            }
        }
    }
}
