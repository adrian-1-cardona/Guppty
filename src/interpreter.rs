// === interpreter.rs ===
// The interpreter walks the AST and runs the code.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expr, ExprKind, Program, Stmt, StmtKind, UnaryOp};
use crate::environment::{Environment, GuppyFunction};
use crate::error::{GupError, Span};
use crate::lexer::is_print_function;
use crate::limits::ResourceLimits;
use crate::value::Value;

/// when a statement finishes we say either "keep going" or "return this value!"
enum ExecResult {
    Continue,
    Return(Value),
}

/// Execution state tracking runtime resource consumption.
pub struct InterpreterState {
    pub limits: ResourceLimits,
    pub call_depth: usize,
    pub step_count: usize,
    pub bytes_printed: usize,
}

#[allow(dead_code)]
pub fn interpret(program: Program) -> Result<(), GupError> {
    interpret_with_limits(program, ResourceLimits::default())
}

pub fn interpret_with_limits(program: Program, limits: ResourceLimits) -> Result<(), GupError> {
    let env = Environment::new();
    let mut state = InterpreterState {
        limits,
        call_depth: 0,
        step_count: 0,
        bytes_printed: 0,
    };

    // first pass: register all top-level functions so they can call each other
    for statement in &program {
        if let StmtKind::FunctionDef { name, params, body } = &statement.kind {
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
        if !matches!(&statement.kind, StmtKind::FunctionDef { .. }) {
            execute_statement(statement, env.clone(), &mut state)?;
        }
    }

    Ok(())
}

fn execute_statement(
    stmt: &Stmt,
    env: Rc<RefCell<Environment>>,
    state: &mut InterpreterState,
) -> Result<ExecResult, GupError> {
    state.step_count += 1;
    if let Some(max_steps) = state.limits.max_steps {
        if state.step_count > max_steps {
            return Err(GupError::runtime(
                stmt.span,
                format!("Execution step limit of {} exceeded.", max_steps),
            ));
        }
    }

    match &stmt.kind {
        StmtKind::ExpressionStatement(expr) => {
            evaluate_expression(expr, env, state)?;
            Ok(ExecResult::Continue)
        }

        StmtKind::VariableDeclaration { name, value } => {
            let evaluated = evaluate_expression(value, env.clone(), state)?;
            let mut env_ref = env.borrow_mut();

            if env_ref.exists(name) {
                env_ref
                    .assign(name, evaluated)
                    .map_err(|msg| GupError::runtime(stmt.span, msg))?;
            } else {
                env_ref.define(name.clone(), evaluated);
            }

            Ok(ExecResult::Continue)
        }

        StmtKind::IfStatement {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition_value = evaluate_expression(condition, env.clone(), state)?;

            if condition_value.is_truthy() {
                execute_block(then_branch, env, state)
            } else if let Some(else_body) = else_branch {
                execute_block(else_body, env, state)
            } else {
                Ok(ExecResult::Continue)
            }
        }

        StmtKind::WhileLoop { condition, body } => {
            loop {
                let condition_value = evaluate_expression(condition, env.clone(), state)?;
                if !condition_value.is_truthy() {
                    break;
                }

                match execute_block(body, env.clone(), state)? {
                    ExecResult::Return(value) => return Ok(ExecResult::Return(value)),
                    ExecResult::Continue => {}
                }
            }

            Ok(ExecResult::Continue)
        }

        StmtKind::ForLoop {
            variable,
            iterable,
            body,
        } => {
            match &iterable.kind {
                ExprKind::Range { start, end } => {
                    let start_val = evaluate_expression(start, env.clone(), state)?;
                    let end_val = evaluate_expression(end, env.clone(), state)?;

                    let start_num = start_val
                        .as_number()
                        .map_err(|msg| GupError::runtime(start.span, msg))?;
                    let end_num = end_val
                        .as_number()
                        .map_err(|msg| GupError::runtime(end.span, msg))?;

                    let start_i = start_num as i64;
                    let end_i = end_num as i64;
                    let count = end_i.abs_diff(start_i).saturating_add(1);
                    if count > state.limits.max_range_size as u64 {
                        return Err(GupError::runtime(
                            iterable.span,
                            format!(
                                "Range size of {} exceeds maximum allowed limit of {}.",
                                count, state.limits.max_range_size
                            ),
                        ));
                    }

                    // Stream range iteration without allocating a vector
                    let step: i64 = if start_i <= end_i { 1 } else { -1 };
                    let mut curr = start_i;
                    loop {
                        env.borrow_mut()
                            .define(variable.clone(), Value::GuppyNumber(curr));

                        match execute_block(body, env.clone(), state)? {
                            ExecResult::Return(return_value) => {
                                return Ok(ExecResult::Return(return_value));
                            }
                            ExecResult::Continue => {}
                        }

                        if curr == end_i {
                            break;
                        }
                        curr += step;
                    }

                    Ok(ExecResult::Continue)
                }
                _ => {
                    let evaluated = evaluate_expression(iterable, env.clone(), state)?;
                    let items = match evaluated {
                        Value::GuppyArray(items) => items,
                        other => {
                            return Err(GupError::runtime(
                                iterable.span,
                                format!(
                                    "Can only loop over a range or array, not {}.",
                                    other.to_display_string()
                                ),
                            ));
                        }
                    };

                    for value in items {
                        env.borrow_mut().define(variable.clone(), value);

                        match execute_block(body, env.clone(), state)? {
                            ExecResult::Return(return_value) => {
                                return Ok(ExecResult::Return(return_value));
                            }
                            ExecResult::Continue => {}
                        }
                    }

                    Ok(ExecResult::Continue)
                }
            }
        }

        StmtKind::ReturnStatement { value } => {
            let return_value = match value {
                Some(expr) => evaluate_expression(expr, env, state)?,
                None => Value::Nothing,
            };
            Ok(ExecResult::Return(return_value))
        }

        StmtKind::FunctionDef { name, params, body } => {
            let function = GuppyFunction {
                params: params.clone(),
                body: body.clone(),
                closure: env.clone(),
            };
            env.borrow_mut()
                .define(name.clone(), Value::GuppyFunction(function));
            Ok(ExecResult::Continue)
        }
    }
}

fn execute_block(
    statements: &[Stmt],
    env: Rc<RefCell<Environment>>,
    state: &mut InterpreterState,
) -> Result<ExecResult, GupError> {
    let block_env = Environment::with_parent(env);

    for stmt in statements {
        match execute_statement(stmt, block_env.clone(), state)? {
            ExecResult::Return(value) => return Ok(ExecResult::Return(value)),
            ExecResult::Continue => {}
        }
    }

    Ok(ExecResult::Continue)
}

fn evaluate_expression(
    expr: &Expr,
    env: Rc<RefCell<Environment>>,
    state: &mut InterpreterState,
) -> Result<Value, GupError> {
    match &expr.kind {
        ExprKind::StringLiteral(text) => Ok(Value::GuppyString(text.clone())),
        ExprKind::CharLiteral(ch) => Ok(Value::GuppyChar(*ch)),
        ExprKind::NumberLiteral(n) => Ok(Value::GuppyNumber(*n)),
        ExprKind::FloatLiteral(f) => Ok(Value::GuppyFloat(*f)),
        ExprKind::BoolLiteral(b) => Ok(Value::GuppyBool(*b)),
        ExprKind::ArrayLiteral(items) => {
            let values = items
                .iter()
                .map(|item| evaluate_expression(item, env.clone(), state))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::GuppyArray(values))
        }
        ExprKind::Variable(name) => env
            .borrow()
            .get(name)
            .map_err(|msg| GupError::runtime(expr.span, msg)),
        ExprKind::UnaryOp { op, operand } => {
            let value = evaluate_expression(operand, env, state)?;
            evaluate_unary_op(*op, &value, expr.span)
        }
        ExprKind::BinaryOp { left, op, right } => {
            // and / or short-circuit — skip the second side when the answer is known
            if *op == BinaryOp::And {
                let left_val = evaluate_expression(left, env.clone(), state)?;
                if !left_val.is_truthy() {
                    return Ok(Value::GuppyBool(false));
                }
                let right_val = evaluate_expression(right, env, state)?;
                return Ok(Value::GuppyBool(right_val.is_truthy()));
            }

            if *op == BinaryOp::Or {
                let left_val = evaluate_expression(left, env.clone(), state)?;
                if left_val.is_truthy() {
                    return Ok(Value::GuppyBool(true));
                }
                let right_val = evaluate_expression(right, env, state)?;
                return Ok(Value::GuppyBool(right_val.is_truthy()));
            }

            let left_val = evaluate_expression(left, env.clone(), state)?;
            let right_val = evaluate_expression(right, env, state)?;
            evaluate_binary_op(&left_val, *op, &right_val, expr.span)
        }
        ExprKind::FunctionCall { name, args } => {
            let evaluated_args: Vec<Value> = args
                .iter()
                .map(|arg| evaluate_expression(arg, env.clone(), state))
                .collect::<Result<Vec<_>, _>>()?;

            if is_print_function(name) {
                let line = if evaluated_args.is_empty() {
                    String::new()
                } else {
                    let output: Vec<String> = evaluated_args
                        .iter()
                        .map(|v| v.to_display_string())
                        .collect();
                    output.join(" ")
                };
                let line_bytes = line.len() + 1;
                if let Some(max_output) = state.limits.max_output_bytes {
                    if state.bytes_printed.saturating_add(line_bytes) > max_output {
                        return Err(GupError::runtime(
                            expr.span,
                            format!("Output volume limit of {} bytes exceeded.", max_output),
                        ));
                    }
                }
                state.bytes_printed += line_bytes;
                println!("{}", line);
                return Ok(Value::Nothing);
            }

            let function = env
                .borrow()
                .get(name)
                .map_err(|msg| GupError::runtime(expr.span, msg))?;

            match function {
                Value::GuppyFunction(function) => {
                    call_function(function, evaluated_args, expr.span, state)
                }
                other => Err(GupError::runtime(
                    expr.span,
                    format!(
                        "'{}' is not a function. It is {}.",
                        name,
                        other.to_display_string()
                    ),
                )),
            }
        }
        ExprKind::Range { .. } => Err(GupError::runtime(
            expr.span,
            "range() can only be used inside a for loop, like: for i in range(1 through 6).",
        )),
    }
}

fn call_function(
    function: GuppyFunction,
    args: Vec<Value>,
    call_span: Span,
    state: &mut InterpreterState,
) -> Result<Value, GupError> {
    if function.params.len() != args.len() {
        return Err(GupError::runtime(
            call_span,
            format!(
                "Wrong number of arguments. Expected {} but got {}.",
                function.params.len(),
                args.len()
            ),
        ));
    }

    if state.call_depth >= state.limits.max_call_depth {
        return Err(GupError::runtime(
            call_span,
            format!(
                "Maximum call stack depth of {} exceeded.",
                state.limits.max_call_depth
            ),
        ));
    }

    let call_env = Environment::with_parent(function.closure);

    for (param, arg) in function.params.iter().zip(args) {
        call_env.borrow_mut().define(param.clone(), arg);
    }

    state.call_depth += 1;
    let result = execute_block(&function.body, call_env, state);
    state.call_depth -= 1;

    match result? {
        ExecResult::Return(value) => Ok(value),
        ExecResult::Continue => Ok(Value::Nothing),
    }
}

fn evaluate_unary_op(op: UnaryOp, value: &Value, span: Span) -> Result<Value, GupError> {
    match op {
        UnaryOp::Not => Ok(Value::GuppyBool(!value.is_truthy())),
        UnaryOp::Negate => {
            let number = value
                .as_number()
                .map_err(|msg| GupError::runtime(span, msg))?;
            if matches!(value, Value::GuppyFloat(_)) {
                Ok(Value::GuppyFloat(-number))
            } else {
                Ok(Value::GuppyNumber(-number as i64))
            }
        }
    }
}

fn evaluate_binary_op(
    left: &Value,
    op: BinaryOp,
    right: &Value,
    span: Span,
) -> Result<Value, GupError> {
    // string plus anything makes a bigger string
    if op == BinaryOp::Add
        && (matches!(left, Value::GuppyString(_)) || matches!(right, Value::GuppyString(_)))
    {
        return Ok(Value::GuppyString(format!(
            "{}{}",
            left.to_display_string(),
            right.to_display_string()
        )));
    }

    if matches!(
        op,
        BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual
    ) {
        return Ok(Value::GuppyBool(compare_values(left, op, right)));
    }

    let left_num = left
        .as_number()
        .map_err(|msg| GupError::runtime(span, msg))?;
    let right_num = right
        .as_number()
        .map_err(|msg| GupError::runtime(span, msg))?;

    let result = match op {
        BinaryOp::Add => left_num + right_num,
        BinaryOp::Sub => left_num - right_num,
        BinaryOp::Mul => left_num * right_num,
        BinaryOp::Div => {
            if right_num == 0.0 {
                return Err(GupError::runtime(span, "Cannot divide by zero."));
            }
            left_num / right_num
        }
        _ => {
            return Err(GupError::runtime(
                span,
                "This operator cannot be used as a math operator here.",
            ));
        }
    };

    let is_float = matches!(left, Value::GuppyFloat(_)) || matches!(right, Value::GuppyFloat(_));

    if is_float {
        Ok(Value::GuppyFloat(result))
    } else {
        Ok(Value::GuppyNumber(result as i64))
    }
}

fn compare_values(left: &Value, op: BinaryOp, right: &Value) -> bool {
    if left.as_number().is_ok() && right.as_number().is_ok() {
        let left_num = left.as_number().unwrap_or(0.0);
        let right_num = right.as_number().unwrap_or(0.0);
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
