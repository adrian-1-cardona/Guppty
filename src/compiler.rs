// === compiler.rs ===
// The compiler turns the AST (family tree of code) into bytecode (numbered steps).
//
// Think of it like writing a treasure map:
//   - The parser drew the island (AST).
//   - The compiler writes "walk 3 steps north, dig here" (bytecode).
//   - The VM follows the map.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{BinaryOp, Expr, ExprKind, Program, Stmt, StmtKind, UnaryOp};
use crate::bytecode::{
  encode_opcode, Chunk, CompiledFunction, Constant, OpCode, RcCompiledFunction, UpvalueDescriptor,
};
use crate::error::{GupError, Span};
use crate::lexer::is_print_function;
use crate::value::Value;

/// One local variable slot in the current function.
struct Local {
  name: String,
  depth: usize,
  is_captured: bool,
}

/// One captured outer variable for a closure.
#[derive(Debug, Clone, Copy)]
struct Upvalue {
  index: u8,
  is_local: bool,
}

/// Compiler state for one function (the top-level script counts as a function too).
struct FunctionContext {
  function: Rc<RefCell<CompiledFunction>>,
  locals: Vec<Local>,
  upvalues: Vec<Upvalue>,
  scope_depth: usize,
  enclosing: Option<usize>,
  dynamic_locals: HashSet<String>,
  has_nested_functions: bool,
}

/// The full compiler walks the AST and fills bytecode chunks.
struct Compiler {
  contexts: Vec<FunctionContext>,
  current: usize,
  globals: HashMap<String, bool>,
}

/// Public entry: compile a whole program into one runnable script function.
pub fn compile(program: &Program) -> Result<RcCompiledFunction, GupError> {
  let mut globals = HashMap::new();
  for stmt in program {
    if let StmtKind::FunctionDef { name, .. } = &stmt.kind {
      globals.insert(name.clone(), true);
    }
  }

  let mut compiler = Compiler::new(globals);
  let script = compiler.begin_function(0, None, program);

  // Pass 1: register top-level functions (same order as the tree-walking interpreter).
  for stmt in program {
    if let StmtKind::FunctionDef { name, params, body } = &stmt.kind {
      let function = compiler.compile_function(params, body, stmt.span)?;
      let constant = compiler.add_constant(Constant::Function(function));
      compiler.emit_bytes(OpCode::Closure, &[constant], stmt.span);
      compiler.define_global(name, stmt.span);
    }
  }

  // Pass 2: compile the rest of the top-level statements.
  for stmt in program {
    if !matches!(&stmt.kind, StmtKind::FunctionDef { .. }) {
      compiler.compile_statement(stmt)?;
    }
  }

  compiler.emit_byte(OpCode::Nil, Span::new(1, 1, 1));
  compiler.emit_byte(OpCode::Return, Span::new(1, 1, 1));
  compiler.end_function();

  Ok(script)
}

impl Compiler {
  fn new(globals: HashMap<String, bool>) -> Self {
    Compiler {
      contexts: Vec::new(),
      current: 0,
      globals,
    }
  }

  fn begin_function(
    &mut self,
    arity: usize,
    enclosing: Option<usize>,
    body: &[Stmt],
  ) -> Rc<RefCell<CompiledFunction>> {
    let has_nested_functions = body
      .iter()
      .any(|stmt| matches!(stmt.kind, StmtKind::FunctionDef { .. }));
    let function = Rc::new(RefCell::new(CompiledFunction {
      arity,
      chunk: Chunk::new(),
      upvalues: Vec::new(),
      local_names: Vec::new(),
    }));
    self.contexts.push(FunctionContext {
      function: function.clone(),
      locals: Vec::new(),
      upvalues: Vec::new(),
      scope_depth: 0,
      enclosing,
      dynamic_locals: HashSet::new(),
      has_nested_functions,
    });
    self.current = self.contexts.len() - 1;
    function
  }

  fn end_function(&mut self) -> Rc<RefCell<CompiledFunction>> {
    let ctx = &mut self.contexts[self.current];
    let upvalue_desc: Vec<UpvalueDescriptor> = ctx
      .upvalues
      .iter()
      .map(|u| UpvalueDescriptor {
        index: u.index,
        is_local: u.is_local,
      })
      .collect();
    ctx.function.borrow_mut().upvalues = upvalue_desc;
    let function = ctx.function.clone();
    let enclosing = ctx.enclosing;
    self.contexts.pop();
    self.current = enclosing.unwrap_or(0);
    function
  }

  fn chunk(&self) -> std::cell::Ref<'_, Chunk> {
    std::cell::Ref::map(self.contexts[self.current].function.borrow(), |f| &f.chunk)
  }

  fn chunk_mut(&mut self) -> std::cell::RefMut<'_, Chunk> {
    std::cell::RefMut::map(self.contexts[self.current].function.borrow_mut(), |f| &mut f.chunk)
  }

  fn emit_byte(&mut self, op: OpCode, span: Span) {
    self.chunk_mut().write(encode_opcode(op), span);
  }

  fn emit_bytes(&mut self, op: OpCode, operands: &[u8], span: Span) {
    self.emit_byte(op, span);
    for byte in operands {
      self.chunk_mut().write(*byte, span);
    }
  }

  fn emit_jump_placeholder(&mut self, op: OpCode, span: Span) -> usize {
    self.emit_byte(op, span);
    let offset = self.chunk().current_offset();
    self.chunk_mut().write(0, span);
    self.chunk_mut().write(0, span);
    offset
  }

  fn patch_jump(&mut self, offset: usize) {
    let jump = self.chunk().current_offset() - offset - 2;
    self.chunk_mut().patch_u16(offset, jump as u16);
  }

  fn emit_loop(&mut self, loop_start: usize, span: Span) {
    self.emit_byte(OpCode::Loop, span);
    let offset = self.chunk().current_offset() - loop_start + 2;
    self.chunk_mut().write((offset >> 8) as u8, span);
    self.chunk_mut().write((offset & 0xff) as u8, span);
  }

  fn add_constant(&mut self, constant: Constant) -> u8 {
    let index = self.chunk_mut().add_constant(constant);
    if index > u8::MAX as usize {
      panic!("Too many constants in one chunk.");
    }
    index as u8
  }

  fn make_constant(&mut self, value: Constant, span: Span) -> u8 {
    let index = self.add_constant(value);
    self.emit_bytes(OpCode::Constant, &[index], span);
    index
  }

  fn add_string_constant(&mut self, text: &str) -> u8 {
    self.add_constant(Constant::String(text.to_string()))
  }

  fn define_global(&mut self, name: &str, span: Span) {
    let constant = self.add_string_constant(name);
    self.emit_bytes(OpCode::DefineGlobal, &[constant], span);
    self.globals.insert(name.to_string(), true);
  }

  fn begin_scope(&mut self, span: Span) {
    self.contexts[self.current].scope_depth += 1;
    self.emit_byte(OpCode::EnterScope, span);
  }

  fn end_scope(&mut self, span: Span) {
    self.emit_byte(OpCode::ExitScope, span);
    let scope_depth = {
      let ctx = &mut self.contexts[self.current];
      ctx.scope_depth -= 1;
      ctx.scope_depth
    };
    loop {
      let should_pop = self
        .contexts[self.current]
        .locals
        .last()
        .is_some_and(|local| local.depth > scope_depth);
      if !should_pop {
        break;
      }
      let is_captured = self.contexts[self.current].locals.last().unwrap().is_captured;
      if is_captured {
        self.emit_byte(OpCode::CloseUpvalue, span);
      } else {
        self.emit_byte(OpCode::Pop, span);
      }
      self.contexts[self.current].locals.pop();
    }
  }

  fn declare_variable(&mut self, name: &str) {
    if self.contexts[self.current].scope_depth == 0 {
      return;
    }
    let depth = self.contexts[self.current].scope_depth;
    self.contexts[self.current].locals.push(Local {
      name: name.to_string(),
      depth,
      is_captured: false,
    });
    self.contexts[self.current]
      .function
      .borrow_mut()
      .local_names
      .push(name.to_string());
  }

  fn resolve_local_in(&self, ctx_index: usize, name: &str) -> Option<u8> {
    for (i, local) in self.contexts[ctx_index].locals.iter().enumerate().rev() {
      if local.name == name {
        return Some(i as u8);
      }
    }

    for (i, local_name) in self.contexts[ctx_index]
      .function
      .borrow()
      .local_names
      .iter()
      .enumerate()
    {
      if local_name == name {
        return Some(i as u8);
      }
    }

    None
  }

  fn resolve_local(&mut self, name: &str) -> Option<u8> {
    self.resolve_local_in(self.current, name)
  }

  fn add_upvalue(&mut self, index: u8, is_local: bool) -> Option<u8> {
    let ctx = &mut self.contexts[self.current];
    for (i, upvalue) in ctx.upvalues.iter().enumerate() {
      if upvalue.index == index && upvalue.is_local == is_local {
        return Some(i as u8);
      }
    }
    if ctx.upvalues.len() >= u8::MAX as usize {
      return None;
    }
    ctx.upvalues.push(Upvalue { index, is_local });
    Some((ctx.upvalues.len() - 1) as u8)
  }

  fn resolve_upvalue(&mut self, name: &str) -> Option<u8> {
    let enclosing = self.contexts[self.current].enclosing?;

    if let Some(local) = self.resolve_local_in(enclosing, name) {
      self.contexts[enclosing].locals[local as usize].is_captured = true;
      return self.add_upvalue(local, true);
    }

    if let Some(upvalue) = self.resolve_upvalue_in(enclosing, name) {
      let up = self.contexts[enclosing].upvalues[upvalue as usize];
      return self.add_upvalue(up.index, up.is_local);
    }

    None
  }

  fn resolve_upvalue_in(&mut self, ctx_index: usize, name: &str) -> Option<u8> {
    let enclosing = self.contexts[ctx_index].enclosing?;

    if let Some(local) = self.resolve_local_in(enclosing, name) {
      self.contexts[enclosing].locals[local as usize].is_captured = true;
      let ctx = &mut self.contexts[ctx_index];
      for (i, upvalue) in ctx.upvalues.iter().enumerate() {
        if upvalue.index == local && upvalue.is_local {
          return Some(i as u8);
        }
      }
      ctx.upvalues.push(Upvalue {
        index: local,
        is_local: true,
      });
      return Some((ctx.upvalues.len() - 1) as u8);
    }

    if let Some(upvalue) = self.resolve_upvalue_in(enclosing, name) {
      let up = self.contexts[enclosing].upvalues[upvalue as usize];
      let ctx = &mut self.contexts[ctx_index];
      for (i, existing) in ctx.upvalues.iter().enumerate() {
        if existing.index == up.index && existing.is_local == up.is_local {
          return Some(i as u8);
        }
      }
      ctx.upvalues.push(up);
      return Some((ctx.upvalues.len() - 1) as u8);
    }

    None
  }

  fn named_variable(&mut self, name: &str, span: Span, can_assign: bool) -> Result<(), GupError> {
    if let Some(local) = self.resolve_local(name) {
      if !can_assign && self.contexts[self.current].dynamic_locals.contains(name) {
        let constant = self.add_string_constant(name);
        self.emit_bytes(OpCode::GetVariable, &[constant], span);
        return Ok(());
      }
      let op = if can_assign {
        OpCode::SetLocal
      } else {
        OpCode::GetLocal
      };
      self.emit_bytes(op, &[local], span);
      return Ok(());
    }

    if let Some(upvalue) = self.resolve_upvalue(name) {
      let op = if can_assign {
        OpCode::SetUpvalue
      } else {
        OpCode::GetUpvalue
      };
      self.emit_bytes(op, &[upvalue], span);
      return Ok(());
    }

    let constant = self.add_string_constant(name);
    if can_assign {
      if self.globals.contains_key(name) {
        self.emit_bytes(OpCode::SetGlobal, &[constant], span);
      } else {
        self.emit_bytes(OpCode::StoreVariable, &[constant], span);
      }
    } else if self.globals.contains_key(name) {
      self.emit_bytes(OpCode::GetGlobal, &[constant], span);
    } else {
      self.emit_bytes(OpCode::GetVariable, &[constant], span);
    }
    Ok(())
  }

  fn compile_function(
    &mut self,
    params: &[String],
    body: &[Stmt],
    span: Span,
  ) -> Result<RcCompiledFunction, GupError> {
    let enclosing = self.current;
    self.begin_function(params.len(), Some(enclosing), body);

    self.contexts[self.current].scope_depth = 1;
    for param in params {
      self.declare_variable(param);
      let ctx = &mut self.contexts[self.current];
      if let Some(local) = ctx.locals.last_mut() {
        local.depth = 1;
      }
    }

    self.compile_block(body, span)?;
    self.emit_byte(OpCode::Nil, span);
    self.emit_byte(OpCode::Return, span);
    Ok(self.end_function())
  }

  fn compile_block(&mut self, statements: &[Stmt], span: Span) -> Result<(), GupError> {
    self.begin_scope(span);
    for stmt in statements {
      self.compile_statement(stmt)?;
    }
    self.end_scope(span);
    Ok(())
  }

  fn compile_statement(&mut self, stmt: &Stmt) -> Result<(), GupError> {
    match &stmt.kind {
      StmtKind::ExpressionStatement(expr) => {
        self.compile_expression(expr)?;
        self.emit_byte(OpCode::Pop, stmt.span);
      }

      StmtKind::VariableDeclaration { name, value } => {
        self.compile_expression(value)?;
        if let Some(local) = self.resolve_local(name) {
          self.emit_bytes(OpCode::SetLocal, &[local], stmt.span);
        } else if let Some(upvalue) = self.resolve_upvalue(name) {
          self.emit_bytes(OpCode::SetUpvalue, &[upvalue], stmt.span);
        } else if self.globals.contains_key(name) {
          let constant = self.add_string_constant(name);
          self.emit_bytes(OpCode::SetGlobal, &[constant], stmt.span);
        } else if self.contexts[self.current].enclosing.is_some()
          && self.contexts[self.current].has_nested_functions
        {
          self.declare_variable(name);
          let slot = (self.contexts[self.current].locals.len() - 1) as u8;
          self.emit_bytes(OpCode::SetLocal, &[slot], stmt.span);
        } else if self.contexts[self.current].enclosing.is_none()
          && self.contexts[self.current].scope_depth == 0
        {
          let constant = self.add_string_constant(name);
          self.emit_bytes(OpCode::StoreVariable, &[constant], stmt.span);
          self.globals.insert(name.clone(), true);
        } else {
          let constant = self.add_string_constant(name);
          self.emit_bytes(OpCode::StoreVariable, &[constant], stmt.span);
        }
      }

      StmtKind::IfStatement {
        condition,
        then_branch,
        else_branch,
      } => {
        self.compile_expression(condition)?;
        let then_jump = self.emit_jump_placeholder(OpCode::JumpIfFalse, stmt.span);
        self.emit_byte(OpCode::Pop, stmt.span);
        self.compile_block(then_branch, stmt.span)?;

        let else_jump = self.emit_jump_placeholder(OpCode::Jump, stmt.span);
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop, stmt.span);

        if let Some(else_body) = else_branch {
          self.compile_block(else_body, stmt.span)?;
        }
        self.patch_jump(else_jump);
      }

      StmtKind::WhileLoop { condition, body } => {
        let loop_start = self.chunk().current_offset();
        self.compile_expression(condition)?;
        let exit_jump = self.emit_jump_placeholder(OpCode::JumpIfFalse, stmt.span);
        self.emit_byte(OpCode::Pop, stmt.span);
        self.compile_block(body, stmt.span)?;
        self.emit_loop(loop_start, stmt.span);
        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop, stmt.span);
      }

      StmtKind::ForLoop {
        variable,
        iterable,
        body,
      } => {
        self.compile_for_loop(variable, iterable, body, stmt.span)?;
      }

      StmtKind::ReturnStatement { value } => {
        if let Some(expr) = value {
          self.compile_expression(expr)?;
        } else {
          self.make_constant(Constant::Value(Value::Nothing), stmt.span);
        }
        self.emit_byte(OpCode::Return, stmt.span);
      }

      StmtKind::FunctionDef { name, params, body } => {
        let function = self.compile_function(params, body, stmt.span)?;
        let constant = self.add_constant(Constant::Function(function));
        self.emit_bytes(OpCode::Closure, &[constant], stmt.span);
        let name_constant = self.add_string_constant(name);
        self.emit_bytes(OpCode::StoreVariable, &[name_constant], stmt.span);
        self.declare_variable(name);
      }
    }
    Ok(())
  }

  fn compile_for_loop(
    &mut self,
    variable: &str,
    iterable: &Expr,
    body: &[Stmt],
    span: Span,
  ) -> Result<(), GupError> {
    self.begin_scope(span);

    match &iterable.kind {
      ExprKind::Range { start, end } => {
        self.compile_expression(start)?;
        self.compile_expression(end)?;
        self.emit_byte(OpCode::BuildRange, span);
      }
      _ => {
        self.compile_expression(iterable)?;
        self.emit_bytes(OpCode::MakeArray, &[1], span);
      }
    }

    let array_slot = self.add_temp_local("__for_array", span);
    self.emit_bytes(OpCode::SetLocal, &[array_slot], span);

    let index_slot = self.add_temp_local("__for_index", span);
    self.make_constant(Constant::Value(Value::GuppyNumber(0)), span);
    self.emit_bytes(OpCode::SetLocal, &[index_slot], span);

    let loop_start = self.chunk().current_offset();

    // When index >= length we are done; otherwise run the body.
    self.emit_bytes(OpCode::GetLocal, &[index_slot], span);
    self.emit_bytes(OpCode::GetLocal, &[array_slot], span);
    self.emit_byte(OpCode::ArrayLen, span);
    self.emit_byte(OpCode::GreaterEqual, span);
    let body_jump = self.emit_jump_placeholder(OpCode::JumpIfFalse, span);
    self.emit_byte(OpCode::Pop, span);
    let exit_jump = self.emit_jump_placeholder(OpCode::Jump, span);
    self.patch_jump(body_jump);
    self.emit_byte(OpCode::Pop, span);

    self.emit_bytes(OpCode::GetLocal, &[array_slot], span);
    self.emit_bytes(OpCode::GetLocal, &[index_slot], span);
    self.emit_byte(OpCode::GetIndex, span);
    let var_constant = self.add_string_constant(variable);
    self.emit_bytes(OpCode::StoreVariable, &[var_constant], span);
    self.declare_variable(variable);

    self.compile_block(body, span)?;

    self.emit_bytes(OpCode::GetLocal, &[index_slot], span);
    self.make_constant(Constant::Value(Value::GuppyNumber(1)), span);
    self.emit_byte(OpCode::Add, span);
    self.emit_bytes(OpCode::SetLocal, &[index_slot], span);

    self.emit_loop(loop_start, span);
    self.patch_jump(exit_jump);

    self.end_scope(span);
    Ok(())
  }

  fn add_temp_local(&mut self, name: &str, _span: Span) -> u8 {
    self.declare_variable(name);
    (self.contexts[self.current].locals.len() - 1) as u8
  }

  fn compile_expression(&mut self, expr: &Expr) -> Result<(), GupError> {
    match &expr.kind {
      ExprKind::StringLiteral(text) => {
        self.make_constant(
          Constant::Value(Value::GuppyString(text.clone())),
          expr.span,
        );
      }
      ExprKind::CharLiteral(ch) => {
        self.make_constant(Constant::Value(Value::GuppyChar(*ch)), expr.span);
      }
      ExprKind::NumberLiteral(n) => {
        self.make_constant(Constant::Value(Value::GuppyNumber(*n)), expr.span);
      }
      ExprKind::FloatLiteral(f) => {
        self.make_constant(Constant::Value(Value::GuppyFloat(*f)), expr.span);
      }
      ExprKind::BoolLiteral(b) => {
        if *b {
          self.emit_byte(OpCode::True, expr.span);
        } else {
          self.emit_byte(OpCode::False, expr.span);
        }
      }
      ExprKind::ArrayLiteral(items) => {
        for item in items {
          self.compile_expression(item)?;
        }
        self.emit_bytes(OpCode::MakeArray, &[items.len() as u8], expr.span);
      }
      ExprKind::Variable(name) => {
        self.named_variable(name, expr.span, false)?;
      }
      ExprKind::UnaryOp { op, operand } => {
        self.compile_expression(operand)?;
        match op {
          UnaryOp::Not => self.emit_byte(OpCode::Not, expr.span),
          UnaryOp::Negate => self.emit_byte(OpCode::Negate, expr.span),
        }
      }
      ExprKind::BinaryOp { left, op, right } => {
        self.compile_binary_op(left, *op, right, expr.span)?;
      }
      ExprKind::FunctionCall { name, args } => {
        if is_print_function(name) {
          for arg in args {
            self.compile_expression(arg)?;
          }
          self.emit_bytes(OpCode::Print, &[args.len() as u8], expr.span);
        } else {
          self.named_variable(name, expr.span, false)?;
          for arg in args {
            self.compile_expression(arg)?;
          }
          self.emit_bytes(OpCode::Call, &[args.len() as u8], expr.span);
        }
      }
      ExprKind::Range { .. } => {
        return Err(GupError::runtime(
          expr.span,
          "range() can only be used inside a for loop, like: for i in range(1 through 6).",
        ));
      }
    }
    Ok(())
  }

  fn compile_binary_op(
    &mut self,
    left: &Expr,
    op: BinaryOp,
    right: &Expr,
    span: Span,
  ) -> Result<(), GupError> {
    if op == BinaryOp::And {
      self.compile_expression(left)?;
      let end_jump = self.emit_jump_placeholder(OpCode::JumpIfFalse, span);
      self.emit_byte(OpCode::Pop, span);
      self.compile_expression(right)?;
      self.patch_jump(end_jump);
      return Ok(());
    }

    if op == BinaryOp::Or {
      self.compile_expression(left)?;
      let else_jump = self.emit_jump_placeholder(OpCode::JumpIfFalse, span);
      let end_jump = self.emit_jump_placeholder(OpCode::Jump, span);
      self.patch_jump(else_jump);
      self.emit_byte(OpCode::Pop, span);
      self.compile_expression(right)?;
      self.patch_jump(end_jump);
      return Ok(());
    }

    self.compile_expression(left)?;
    self.compile_expression(right)?;

    let opcode = match op {
      BinaryOp::Add => OpCode::Add,
      BinaryOp::Sub => OpCode::Sub,
      BinaryOp::Mul => OpCode::Mul,
      BinaryOp::Div => OpCode::Div,
      BinaryOp::Equal => OpCode::Equal,
      BinaryOp::NotEqual => OpCode::NotEqual,
      BinaryOp::Less => OpCode::Less,
      BinaryOp::Greater => OpCode::Greater,
      BinaryOp::LessEqual => OpCode::LessEqual,
      BinaryOp::GreaterEqual => OpCode::GreaterEqual,
      BinaryOp::And | BinaryOp::Or => unreachable!(),
    };
    self.emit_byte(opcode, span);
    Ok(())
  }
}
