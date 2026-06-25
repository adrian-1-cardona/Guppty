// === interpreter.rs ===
// the interpreter is the actor — it walks the AST tree and DOES the code!
// it keeps boxes of variables (environments) and knows when to open new boxes (scopes).
// closures remember which box they were born in — that is super important magic!

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::environment::{Environment, GuppyFunction};
use crate::lexer::is_print_function;
use crate::value::Value;

/// when return happens we throw this little flag to stop running the function
#[derive(Debug, Clone)]
struct ReturnSignal(Value);

/// when a statement finishes we say either "keep going" or "return this value!"
enum ExecResult {
    Continue,
    Return(Value),
}

pub fn interpret(program: Program) {
    let env = Environment::new();

    // first pass: register all top-level functions so they can call each other
    for statement in &program {
        if let Stmt::FunctionDef { name, params, body } = statement {
            let function = GuppyFunction {
                params: params.clone(),
                body: body.clone(),
                closure: env.clone(),
            };
            env.borrow_mut()
                .define(name.clone(), Value::GuppyFunction(function));
        }
    }

    // second pass: run everything that is not a function definition
    for statement in &program {
        if !matches!(statement, Stmt::FunctionDef { .. }) {
            execute_statement(statement, env.clone());
        }
    }
}

fn execute_statement(stmt: &Stmt, env: Rc<RefCell<Environment>>) -> ExecResult {
    match stmt {
        // run an expression for side effects (like out("hi"))
        Stmt::ExpressionStatement(expr) => {
            evaluate_expression(expr, env);
            ExecResult::Continue
        }

        // x = 5  — update an existing name anywhere in the chain, or make a new one here
        Stmt::VariableDeclaration { name, value } => {
            let evaluated = evaluate_expression(value, env.clone());
            let mut env_ref = env.borrow_mut();

            if env_ref.exists(name) {
                env_ref
                    .assign(name, evaluated)
                    .unwrap_or_else(|msg| panic!("{}", msg));
            } else {
                env_ref.define(name.clone(), evaluated);
            }

            ExecResult::Continue
        }

        // if the condition is truthy run the then branch, else run else branch
        Stmt::IfStatement {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition_value = evaluate_expression(condition, env.clone());

            if condition_value.is_truthy() {
                execute_block(then_branch, env)
            } else if let Some(else_body) = else_branch {
                execute_block(else_body, env)
            } else {
                ExecResult::Continue
            }
        }

        // keep running the body while the condition stays truthy
        Stmt::WhileLoop { condition, body } => {
            loop {
                let condition_value = evaluate_expression(condition, env.clone());
                if !condition_value.is_truthy() {
                    break;
                }

                match execute_block(body, env.clone()) {
                    ExecResult::Return(value) => return ExecResult::Return(value),
                    ExecResult::Continue => {}
                }
            }

            ExecResult::Continue
        }

        // for i in range(1 through 3) — each loop gets the loop variable in the same scope
        Stmt::ForLoop {
            variable,
            iterable,
            body,
        } => {
            let values = evaluate_iterable(iterable, env.clone());

            for value in values {
                env.borrow_mut().define(variable.clone(), value);

                match execute_block(body, env.clone()) {
                    ExecResult::Return(return_value) => return ExecResult::Return(return_value),
                    ExecResult::Continue => {}
                }
            }

            ExecResult::Continue
        }

        // return sends a value back and stops the function immediately
        Stmt::ReturnStatement { value } => {
            let return_value = match value {
                Some(expr) => evaluate_expression(expr, env),
                None => Value::Nothing,
            };
            ExecResult::Return(return_value)
        }

        // define a function and remember the current box (closure!)
        Stmt::FunctionDef { name, params, body } => {
            let function = GuppyFunction {
                params: params.clone(),
                body: body.clone(),
                closure: env.clone(),
            };
            env.borrow_mut()
                .define(name.clone(), Value::GuppyFunction(function));
            ExecResult::Continue
        }
    }
}

fn execute_block(statements: &[Stmt], env: Rc<RefCell<Environment>>) -> ExecResult {
    // a block gets its own inner box so local variables do not leak out!
    let block_env = Environment::with_parent(env);

    for stmt in statements {
        match execute_statement(stmt, block_env.clone()) {
            ExecResult::Return(value) => return ExecResult::Return(value),
            ExecResult::Continue => {}
        }
    }

    ExecResult::Continue
}

fn evaluate_iterable(expr: &Expr, env: Rc<RefCell<Environment>>) -> Vec<Value> {
    match expr {
        Expr::Range { start, end } => {
            let start_val = evaluate_expression(start, env.clone());
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

fn evaluate_expression(expr: &Expr, env: Rc<RefCell<Environment>>) -> Value {
    match expr {
        Expr::StringLiteral(text) => Value::GuppyString(text.clone()),
        Expr::CharLiteral(ch) => Value::GuppyChar(*ch),
        Expr::NumberLiteral(n) => Value::GuppyNumber(*n),
        Expr::FloatLiteral(f) => Value::GuppyFloat(*f),
        Expr::BoolLiteral(b) => Value::GuppyBool(*b),
        Expr::ArrayLiteral(items) => {
            let values = items
                .iter()
                .map(|item| evaluate_expression(item, env.clone()))
                .collect();
            Value::GuppyArray(values)
        }
        Expr::Variable(name) => env
            .borrow()
            .get(name)
            .unwrap_or_else(|msg| panic!("{}", msg)),
        Expr::UnaryOp { op, operand } => {
            let value = evaluate_expression(operand, env);
            evaluate_unary_op(*op, &value)
        }
        Expr::BinaryOp { left, op, right } => {
            // and / or short-circuit — skip the second side when we already know the answer!
            if *op == BinaryOp::And {
                let left_val = evaluate_expression(left, env.clone());
                if !left_val.is_truthy() {
                    return Value::GuppyBool(false);
                }
                let right_val = evaluate_expression(right, env);
                return Value::GuppyBool(right_val.is_truthy());
            }

            if *op == BinaryOp::Or {
                let left_val = evaluate_expression(left, env.clone());
                if left_val.is_truthy() {
                    return Value::GuppyBool(true);
                }
                let right_val = evaluate_expression(right, env);
                return Value::GuppyBool(right_val.is_truthy());
            }

            let left_val = evaluate_expression(left, env.clone());
            let right_val = evaluate_expression(right, env);
            evaluate_binary_op(&left_val, *op, &right_val)
        }
        Expr::FunctionCall { name, args } => {
            let evaluated_args: Vec<Value> = args
                .iter()
                .map(|arg| evaluate_expression(arg, env.clone()))
                .collect();

            if is_print_function(name) {
                if evaluated_args.is_empty() {
                    println!();
                } else {
                    let output: Vec<String> = evaluated_args
                        .iter()
                        .map(|v| v.to_display_string())
                        .collect();
                    println!("{}", output.join(" "));
                }
                return Value::Nothing;
            }

            let function = env
                .borrow()
                .get(name)
                .unwrap_or_else(|msg| panic!("{}", msg));

            match function {
                Value::GuppyFunction(function) => call_function(function, evaluated_args),
                other => panic!(
                    "'{}' is not a function! It is {}",
                    name,
                    other.to_display_string()
                ),
            }
        }
        Expr::Range { .. } => {
            panic!("range() can only be used inside a for loop like: for i in range(1 through 6)");
        }
    }
}

fn call_function(function: GuppyFunction, args: Vec<Value>) -> Value {
    if function.params.len() != args.len() {
        panic!(
            "Wrong number of arguments! Expected {} but got {}",
            function.params.len(),
            args.len()
        );
    }

    // open a new box inside the box where the function was born (closure magic!)
    let call_env = Environment::with_parent(function.closure);

    for (param, arg) in function.params.iter().zip(args) {
        call_env.borrow_mut().define(param.clone(), arg);
    }

    match execute_block(&function.body, call_env) {
        ExecResult::Return(value) => value,
        ExecResult::Continue => Value::Nothing,
    }
}

fn evaluate_unary_op(op: UnaryOp, value: &Value) -> Value {
    match op {
        UnaryOp::Not => Value::GuppyBool(!value.is_truthy()),
        UnaryOp::Negate => {
            let number = value
                .as_number()
                .unwrap_or_else(|msg| panic!("{}", msg));
            if matches!(value, Value::GuppyFloat(_)) {
                Value::GuppyFloat(-number)
            } else {
                Value::GuppyNumber(-number as i64)
            }
        }
    }
}

fn evaluate_binary_op(left: &Value, op: BinaryOp, right: &Value) -> Value {
    // string plus anything makes a bigger string!
    if op == BinaryOp::Add {
        if matches!(left, Value::GuppyString(_)) || matches!(right, Value::GuppyString(_)) {
            return Value::GuppyString(format!(
                "{}{}",
                left.to_display_string(),
                right.to_display_string()
            ));
        }
    }

    // comparisons work on numbers AND strings AND booleans
    if matches!(
        op,
        BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual
    ) {
        return Value::GuppyBool(compare_values(left, op, right));
    }

    // regular math needs numbers
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
        _ => unreachable!("comparison operators handled above"),
    };

    let is_float =
        matches!(left, Value::GuppyFloat(_)) || matches!(right, Value::GuppyFloat(_));

    if is_float {
        Value::GuppyFloat(result)
    } else {
        Value::GuppyNumber(result as i64)
    }
}

fn compare_values(left: &Value, op: BinaryOp, right: &Value) -> bool {
    // if both sides are numbers, compare as numbers
    if left.as_number().is_ok() && right.as_number().is_ok() {
        let left_num = left.as_number().unwrap();
        let right_num = right.as_number().unwrap();
        return match op {
            BinaryOp::Equal => left_num == right_num,
            BinaryOp::NotEqual => left_num != right_num,
            BinaryOp::Less => left_num < right_num,
            BinaryOp::Greater => left_num > right_num,
            BinaryOp::LessEqual => left_num <= right_num,
            BinaryOp::GreaterEqual => left_num >= right_num,
            _ => false,
        };
    }

    // otherwise compare how they look when printed (strings, bools, etc.)
    let left_text = left.to_display_string();
    let right_text = right.to_display_string();

    match op {
        BinaryOp::Equal => left_text == right_text,
        BinaryOp::NotEqual => left_text != right_text,
        BinaryOp::Less => left_text < right_text,
        BinaryOp::Greater => left_text > right_text,
        BinaryOp::LessEqual => left_text <= right_text,
        BinaryOp::GreaterEqual => left_text >= right_text,
        _ => false,
    }
}
#[allow(dead_code)]
impl ReturnSignal {
    fn value(self) -> Value {
        self.0
    }
}
