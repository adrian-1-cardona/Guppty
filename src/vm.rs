// === vm.rs ===
// The VM (virtual machine) is a little robot that reads bytecode and runs your program.
//
// Picture a stack of plates (the operand stack) and a recipe book (bytecode):
//   - Each step might put a plate on the stack or take one off.
//   - When you call a function, you start a new recipe page (call frame).
//   - Closures are like backpacks that carry outer variables (upvalues) along.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::{
  decode_opcode, ClosureUpvalue, ClosureValue, Constant, OpCode, RcCompiledFunction,
};
use crate::error::{GupError, Span};
use crate::value::Value;

/// One active function call — where we are in the recipe and our local plates.
#[derive(Debug, Clone)]
struct CallFrame {
  function: RcCompiledFunction,
  ip: usize,
  stack_start: usize,
  scope_depth: usize,
  upvalues: Vec<Rc<RefCell<ClosureUpvalue>>>,
  local_slots: HashMap<String, usize>,
}

/// The full virtual machine state.
pub struct Vm {
  stack: Vec<Value>,
  frames: Vec<CallFrame>,
  globals: HashMap<String, Value>,
  open_upvalues: Vec<Rc<RefCell<ClosureUpvalue>>>,
}

/// Run a compiled script function from start to finish.
pub fn run(function: RcCompiledFunction) -> Result<(), GupError> {
  let mut vm = Vm::new();
  vm.frames.push(CallFrame {
    function,
    ip: 0,
    stack_start: 0,
    scope_depth: 0,
    upvalues: Vec::new(),
    local_slots: HashMap::new(),
  });
  vm.run_loop()
}

impl Vm {
  fn new() -> Self {
    Vm {
      stack: Vec::new(),
      frames: Vec::new(),
      globals: HashMap::new(),
      open_upvalues: Vec::new(),
    }
  }

  fn current_frame(&self) -> &CallFrame {
    self.frames.last().expect("VM always has a call frame")
  }

  fn current_frame_mut(&mut self) -> &mut CallFrame {
    self.frames.last_mut().expect("VM always has a call frame")
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }

  fn pop(&mut self) -> Value {
    self.stack.pop().expect("Stack underflow in VM")
  }

  fn peek(&self, distance: usize) -> &Value {
    &self.stack[self.stack.len() - 1 - distance]
  }

  fn read_byte(&mut self) -> u8 {
    let ip = self.current_frame().ip;
    let byte = self.current_frame().function.borrow().chunk.code[ip];
    self.current_frame_mut().ip = ip + 1;
    byte
  }

  fn read_u16(&mut self) -> u16 {
    let high = self.read_byte() as u16;
    let low = self.read_byte() as u16;
    (high << 8) | low
  }

  fn current_span(&self) -> Span {
    let frame = self.current_frame();
    let ip = frame.ip.saturating_sub(1);
    frame
      .function
      .borrow()
      .chunk
      .spans
      .get(ip)
      .copied()
      .unwrap_or(Span::new(1, 1, 1))
  }

  fn runtime_error(&self, message: impl Into<String>) -> GupError {
    GupError::runtime(self.current_span(), message)
  }

  fn constant_name(&self, index: u8) -> Result<String, GupError> {
    match &self.current_frame().function.borrow().chunk.constants[index as usize] {
      Constant::String(name) => Ok(name.clone()),
      _ => Err(self.runtime_error("Expected a variable name in constants.")),
    }
  }

  fn constant_value(&self, index: u8) -> Result<Value, GupError> {
    match &self.current_frame().function.borrow().chunk.constants[index as usize] {
      Constant::Value(value) => Ok(value.clone()),
      _ => Err(self.runtime_error("Expected a value constant.")),
    }
  }

  fn ensure_local_slot(&mut self, slot: u8) {
    self.ensure_stack_index(self.slot_index(slot));
  }

  fn ensure_stack_index(&mut self, index: usize) {
    while self.stack.len() <= index {
      self.stack.push(Value::Nothing);
    }
  }

  fn slot_index(&self, slot: u8) -> usize {
    self.current_frame().stack_start + slot as usize
  }

  fn run_loop(&mut self) -> Result<(), GupError> {
    loop {
      if self.frames.is_empty() {
        return Ok(());
      }

      let ip = self.current_frame().ip;
      let code_len = self.current_frame().function.borrow().chunk.code.len();
      if ip >= code_len {
        self.frames.pop();
        continue;
      }

      let opcode_byte = self.current_frame().function.borrow().chunk.code[ip];
      let instruction = decode_opcode(opcode_byte)
        .ok_or_else(|| GupError::runtime(Span::new(1, 1, 1), "Unknown opcode in bytecode."))?;
      self.current_frame_mut().ip = ip + 1;

      match instruction {
        OpCode::Constant => {
          let index = self.read_byte();
          self.push(self.constant_value(index)?);
        }
        OpCode::Nil => self.push(Value::Nothing),
        OpCode::True => self.push(Value::GuppyBool(true)),
        OpCode::False => self.push(Value::GuppyBool(false)),
        OpCode::Pop => {
          self.pop();
        }
        OpCode::GetLocal => {
          let slot = self.read_byte();
          self.ensure_local_slot(slot);
          self.push(self.stack[self.slot_index(slot)].clone());
        }
        OpCode::SetLocal => {
          let slot = self.read_byte();
          let value = self.pop();
          self.ensure_local_slot(slot);
          let index = self.slot_index(slot);
          self.stack[index] = value;
        }
        OpCode::GetGlobal => {
          let index = self.read_byte();
          let name = self.constant_name(index)?;
          let value = self
            .globals
            .get(&name)
            .cloned()
            .ok_or_else(|| self.runtime_error(format!("Variable '{}' is not defined yet!", name)))?;
          self.push(value);
        }
        OpCode::SetGlobal => {
          let index = self.read_byte();
          let name = self.constant_name(index)?;
          let value = self.pop();
          if !self.globals.contains_key(&name) {
            return Err(self.runtime_error(format!("Variable '{}' is not defined yet!", name)));
          }
          self.globals.insert(name, value);
        }
        OpCode::DefineGlobal => {
          let index = self.read_byte();
          let name = self.constant_name(index)?;
          let value = self.pop();
          self.globals.insert(name, value);
        }
        OpCode::GetUpvalue => {
          let slot = self.read_byte() as usize;
          self.push(self.read_upvalue(slot)?);
        }
        OpCode::SetUpvalue => {
          let slot = self.read_byte() as usize;
          let value = self.pop();
          self.write_upvalue(slot, value)?;
        }
        OpCode::CloseUpvalue => {
          self.close_upvalues(self.stack.len() - 1);
          self.pop();
        }
        OpCode::StoreVariable => {
          let index = self.read_byte();
          let name = self.constant_name(index)?;
          let value = self.pop();
          self.store_variable(&name, value)?;
        }
        OpCode::GetVariable => {
          let index = self.read_byte();
          let name = self.constant_name(index)?;
          self.push(self.get_variable(&name)?);
        }
        OpCode::Add => self.binary_op(OpCode::Add)?,
        OpCode::Sub => self.binary_op(OpCode::Sub)?,
        OpCode::Mul => self.binary_op(OpCode::Mul)?,
        OpCode::Div => self.binary_op(OpCode::Div)?,
        OpCode::Negate => {
          let value = self.pop();
          self.push(self.negate_value(value)?);
        }
        OpCode::Not => {
          let value = self.pop();
          self.push(Value::GuppyBool(!value.is_truthy()));
        }
        OpCode::Equal => self.compare_values(OpCode::Equal)?,
        OpCode::NotEqual => self.compare_values(OpCode::NotEqual)?,
        OpCode::Less => self.compare_values(OpCode::Less)?,
        OpCode::Greater => self.compare_values(OpCode::Greater)?,
        OpCode::LessEqual => self.compare_values(OpCode::LessEqual)?,
        OpCode::GreaterEqual => self.compare_values(OpCode::GreaterEqual)?,
        OpCode::EnterScope => {
          self.current_frame_mut().scope_depth += 1;
        }
        OpCode::ExitScope => {
          self.current_frame_mut().scope_depth -= 1;
        }
        OpCode::Jump => {
          let offset = self.read_u16();
          self.current_frame_mut().ip += offset as usize;
        }
        OpCode::JumpIfFalse => {
          let offset = self.read_u16();
          if !self.peek(0).is_truthy() {
            self.current_frame_mut().ip += offset as usize;
          }
        }
        OpCode::Loop => {
          let offset = self.read_u16();
          self.current_frame_mut().ip -= offset as usize;
        }
        OpCode::Call => {
          let arg_count = self.read_byte() as usize;
          self.call_value(arg_count)?;
        }
        OpCode::Return => {
          let value = self.pop();
          let stack_start = self.current_frame().stack_start;
          self.close_upvalues(stack_start);
          self.frames.pop();
          if self.frames.is_empty() {
            return Ok(());
          }
          while self.stack.len() > stack_start {
            self.pop();
          }
          self.push(value);
        }
        OpCode::Closure => {
          let constant = self.read_byte();
          self.make_closure(constant)?;
        }
        OpCode::Print => {
          let arg_count = self.read_byte() as usize;
          self.print_args(arg_count)?;
        }
        OpCode::MakeArray => {
          let count = self.read_byte() as usize;
          let mut items = Vec::with_capacity(count);
          for _ in 0..count {
            items.push(self.pop());
          }
          items.reverse();
          self.push(Value::GuppyArray(items));
        }
        OpCode::BuildRange => {
          let end_val = self.pop();
          let start_val = self.pop();
          self.push(self.build_range(start_val, end_val)?);
        }
        OpCode::GetIndex => {
          let index_val = self.pop();
          let array_val = self.pop();
          self.push(self.get_index(array_val, index_val)?);
        }
        OpCode::ArrayLen => {
          let array_val = self.pop();
          let len = match array_val {
            Value::GuppyArray(items) => items.len() as i64,
            other => {
              return Err(self.runtime_error(format!(
                "Expected an array but got {}",
                other.to_display_string()
              )));
            }
          };
          self.push(Value::GuppyNumber(len));
        }
        OpCode::DefineLocal => {
          return Err(self.runtime_error("Internal compiler error: DefineLocal reached."));
        }
      }
    }
  }

  fn get_variable(&self, name: &str) -> Result<Value, GupError> {
    if let Some(index) = self.resolve_local_slot(name) {
      return Ok(self.stack[index].clone());
    }

    if let Some(slot) = self.resolve_upvalue_name(name) {
      return self.read_upvalue(slot);
    }

    if let Some(value) = self.globals.get(name) {
      return Ok(value.clone());
    }

    Err(self.runtime_error(format!("Variable '{}' is not defined yet!", name)))
  }

  fn store_variable(&mut self, name: &str, value: Value) -> Result<(), GupError> {
    if let Some(index) = self.resolve_local_slot(name) {
      self.ensure_stack_index(index);
      self.stack[index] = value;
      return Ok(());
    }

    if let Some(slot) = self.resolve_upvalue_name(name) {
      self.write_upvalue(slot, value)?;
      return Ok(());
    }

    if self.globals.contains_key(name) {
      self.globals.insert(name.to_string(), value);
      return Ok(());
    }

    if self.current_frame().scope_depth > 0 {
      let index = self.stack.len();
      self.push(value);
      self
        .current_frame_mut()
        .local_slots
        .insert(name.to_string(), index);
      return Ok(());
    }

    self.globals.insert(name.to_string(), value);
    Ok(())
  }

  fn resolve_local_slot(&self, name: &str) -> Option<usize> {
    for frame in self.frames.iter().rev() {
      if let Some(&index) = frame.local_slots.get(name) {
        return Some(index);
      }
      let function = frame.function.borrow();
      for (slot, local_name) in function.local_names.iter().enumerate() {
        if local_name == name {
          return Some(frame.stack_start + slot);
        }
      }
    }
    None
  }

  fn resolve_upvalue_name(&self, name: &str) -> Option<usize> {
    let frame = self.current_frame();
    let function = frame.function.borrow();
    for (slot, desc) in function.upvalues.iter().enumerate() {
      if desc.is_local {
        if function
          .local_names
          .get(desc.index as usize)
          .is_some_and(|n| n == name)
        {
          return Some(slot);
        }
      }
    }
    None
  }

  fn read_upvalue(&self, slot: usize) -> Result<Value, GupError> {
    let upvalue = self.current_frame().upvalues[slot].borrow();
    if let Some(open_slot) = upvalue.open_slot {
      Ok(self.stack[open_slot].clone())
    } else {
      Ok(upvalue
        .closed
        .clone()
        .unwrap_or(Value::Nothing))
    }
  }

  fn write_upvalue(&mut self, slot: usize, value: Value) -> Result<(), GupError> {
    let open_slot = self.current_frame().upvalues[slot].borrow().open_slot;
    if let Some(index) = open_slot {
      self.stack[index] = value;
    } else {
      self.current_frame().upvalues[slot].borrow_mut().closed = Some(value);
    }
    Ok(())
  }

  fn binary_op(&mut self, op: OpCode) -> Result<(), GupError> {
    let right = self.pop();
    let left = self.pop();

    if op == OpCode::Add {
      if matches!(left, Value::GuppyString(_)) || matches!(right, Value::GuppyString(_)) {
        self.push(Value::GuppyString(format!(
          "{}{}",
          left.to_display_string(),
          right.to_display_string()
        )));
        return Ok(());
      }
    }

    let left_num = left
      .as_number()
      .map_err(|msg| self.runtime_error(msg))?;
    let right_num = right
      .as_number()
      .map_err(|msg| self.runtime_error(msg))?;

    let result = match op {
      OpCode::Add => left_num + right_num,
      OpCode::Sub => left_num - right_num,
      OpCode::Mul => left_num * right_num,
      OpCode::Div => {
        if right_num == 0.0 {
          return Err(self.runtime_error("Cannot divide by zero."));
        }
        left_num / right_num
      }
      _ => return Err(self.runtime_error("Invalid math opcode.")),
    };

    let is_float =
      matches!(left, Value::GuppyFloat(_)) || matches!(right, Value::GuppyFloat(_));
    if is_float {
      self.push(Value::GuppyFloat(result));
    } else {
      self.push(Value::GuppyNumber(result as i64));
    }
    Ok(())
  }

  fn negate_value(&self, value: Value) -> Result<Value, GupError> {
    let number = value
      .as_number()
      .map_err(|msg| self.runtime_error(msg))?;
    if matches!(value, Value::GuppyFloat(_)) {
      Ok(Value::GuppyFloat(-number))
    } else {
      Ok(Value::GuppyNumber(-number as i64))
    }
  }

  fn compare_values(&mut self, op: OpCode) -> Result<(), GupError> {
    let right = self.pop();
    let left = self.pop();

    let result = if left.as_number().is_ok() && right.as_number().is_ok() {
      let left_num = left.as_number().unwrap_or(0.0);
      let right_num = right.as_number().unwrap_or(0.0);
      compare_numbers(op, left_num, right_num)
    } else {
      let left_text = left.to_display_string();
      let right_text = right.to_display_string();
      compare_strings(op, &left_text, &right_text)
    };

    self.push(Value::GuppyBool(result));
    Ok(())
  }

  fn build_range(&self, start_val: Value, end_val: Value) -> Result<Value, GupError> {
    let start_num = start_val
      .as_number()
      .map_err(|msg| self.runtime_error(msg))?;
    let end_num = end_val
      .as_number()
      .map_err(|msg| self.runtime_error(msg))?;

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

    Ok(Value::GuppyArray(values))
  }

  fn get_index(&self, array_val: Value, index_val: Value) -> Result<Value, GupError> {
    let array = match array_val {
      Value::GuppyArray(items) => items,
      other => {
        return Err(self.runtime_error(format!(
          "Expected an array but got {}",
          other.to_display_string()
        )));
      }
    };
    let index = index_val
      .as_number()
      .map_err(|msg| self.runtime_error(msg))? as i64;
    if index < 0 || index as usize >= array.len() {
      return Err(self.runtime_error("Array index out of bounds."));
    }
    Ok(array[index as usize].clone())
  }

  fn print_args(&mut self, arg_count: usize) -> Result<(), GupError> {
    if arg_count == 0 {
      println!();
    } else {
      let mut parts = Vec::with_capacity(arg_count);
      for _ in 0..arg_count {
        parts.push(self.pop().to_display_string());
      }
      parts.reverse();
      println!("{}", parts.join(" "));
    }
    self.push(Value::Nothing);
    Ok(())
  }

  fn call_value(&mut self, arg_count: usize) -> Result<(), GupError> {
    let callee = self.peek(arg_count).clone();
    match callee {
      Value::GuppyClosure(closure) => self.call_closure(closure, arg_count),
      other => Err(self.runtime_error(format!(
        "'{}' is not a function. It is {}.",
        "<call>",
        other.to_display_string()
      ))),
    }
  }

  fn call_closure(&mut self, closure: ClosureValue, arg_count: usize) -> Result<(), GupError> {
    let function = closure.function.borrow();
    if function.arity != arg_count {
      return Err(self.runtime_error(format!(
        "Wrong number of arguments. Expected {} but got {}.",
        function.arity, arg_count
      )));
    }
    drop(function);

    let arg_start = self.stack.len() - arg_count;
    if arg_start == 0 {
      return Err(self.runtime_error("Cannot call with empty stack."));
    }
    self.stack.remove(arg_start - 1);
    let stack_start = arg_start - 1;

    self.frames.push(CallFrame {
      function: closure.function,
      ip: 0,
      stack_start,
      scope_depth: 1,
      upvalues: closure.upvalues,
      local_slots: HashMap::new(),
    });
    Ok(())
  }

  fn make_closure(&mut self, constant: u8) -> Result<(), GupError> {
    let function = match &self.current_frame().function.borrow().chunk.constants[constant as usize]
    {
      Constant::Function(function) => function.clone(),
      _ => return Err(self.runtime_error("Expected a function constant.")),
    };

    let upvalue_desc = function.borrow().upvalues.clone();
    let mut upvalues = Vec::with_capacity(upvalue_desc.len());

    for desc in upvalue_desc {
      let upvalue = if desc.is_local {
        self.capture_upvalue(self.current_frame().stack_start + desc.index as usize)
      } else {
        self.current_frame().upvalues[desc.index as usize].clone()
      };
      upvalues.push(upvalue);
    }

    self.push(Value::GuppyClosure(ClosureValue {
      function,
      upvalues,
    }));
    Ok(())
  }

  fn capture_upvalue(&mut self, location: usize) -> Rc<RefCell<ClosureUpvalue>> {
    for upvalue in &self.open_upvalues {
      if upvalue.borrow().open_slot == Some(location) {
        return upvalue.clone();
      }
    }

    let created = Rc::new(RefCell::new(ClosureUpvalue {
      open_slot: Some(location),
      closed: None,
    }));
    self.open_upvalues.push(created.clone());
    created
  }

  fn close_upvalues(&mut self, last_index: usize) {
    let mut still_open = Vec::new();
    for upvalue in self.open_upvalues.drain(..) {
      let open_slot = upvalue.borrow().open_slot;
      if let Some(index) = open_slot {
        if index >= last_index {
          let value = self.stack[index].clone();
          let mut borrowed = upvalue.borrow_mut();
          borrowed.open_slot = None;
          borrowed.closed = Some(value);
          continue;
        }
      }
      still_open.push(upvalue);
    }
    self.open_upvalues = still_open;
  }
}

fn compare_numbers(op: OpCode, left: f64, right: f64) -> bool {
  match op {
    OpCode::Equal => left == right,
    OpCode::NotEqual => left != right,
    OpCode::Less => left < right,
    OpCode::Greater => left > right,
    OpCode::LessEqual => left <= right,
    OpCode::GreaterEqual => left >= right,
    _ => false,
  }
}

fn compare_strings(op: OpCode, left: &str, right: &str) -> bool {
  match op {
    OpCode::Equal => left == right,
    OpCode::NotEqual => left != right,
    OpCode::Less => left < right,
    OpCode::Greater => left > right,
    OpCode::LessEqual => left <= right,
    OpCode::GreaterEqual => left >= right,
    _ => false,
  }
}

// =============================================================================
// Hand-written bytecode tests
// =============================================================================
// These tests skip the lexer/parser/compiler entirely and feed bytecode
// straight to the VM. We assemble a tiny Chunk by hand (writing the recipe
// steps ourselves), run it, then peek at the operand stack to check the robot
// did the right thing. This gives fast, focused coverage of individual opcodes.
#[cfg(test)]
mod tests {
  use super::*;
  use crate::bytecode::{encode_opcode, Chunk, CompiledFunction};

  /// A tiny helper for assembling bytecode without repeating boilerplate.
  struct ChunkBuilder {
    chunk: Chunk,
  }

  impl ChunkBuilder {
    fn new() -> Self {
      ChunkBuilder {
        chunk: Chunk::new(),
      }
    }

    /// Every byte needs a source span; tests don't care, so use a dummy one.
    fn span() -> Span {
      Span::new(1, 1, 1)
    }

    /// Emit a bare opcode with no operands.
    fn op(&mut self, op: OpCode) -> &mut Self {
      self.chunk.write(encode_opcode(op), Self::span());
      self
    }

    /// Emit an opcode followed by a single byte operand (e.g. a slot or count).
    fn op_byte(&mut self, op: OpCode, operand: u8) -> &mut Self {
      self.chunk.write(encode_opcode(op), Self::span());
      self.chunk.write(operand, Self::span());
      self
    }

    /// Emit an opcode followed by a big-endian u16 operand (used by jumps).
    fn op_u16(&mut self, op: OpCode, operand: u16) -> &mut Self {
      self.chunk.write(encode_opcode(op), Self::span());
      self.chunk.write((operand >> 8) as u8, Self::span());
      self.chunk.write((operand & 0xff) as u8, Self::span());
      self
    }

    /// Add a value to the constant pool and emit `Constant <index>` to push it.
    fn push_value(&mut self, value: Value) -> &mut Self {
      let index = self.chunk.add_constant(Constant::Value(value));
      self.op_byte(OpCode::Constant, index as u8)
    }

    /// Add a string constant (e.g. a variable name) and return its pool index.
    fn string_constant(&mut self, text: &str) -> u8 {
      self.chunk.add_constant(Constant::String(text.to_string())) as u8
    }

    /// Wrap the assembled chunk in a runnable script function.
    fn finish(&mut self) -> RcCompiledFunction {
      Rc::new(RefCell::new(CompiledFunction {
        name: "<test>".to_string(),
        arity: 0,
        chunk: std::mem::replace(&mut self.chunk, Chunk::new()),
        upvalues: Vec::new(),
        local_names: Vec::new(),
      }))
    }
  }

  /// Run a hand-built function to completion and hand back the whole VM so a
  /// test can inspect both the operand stack and the globals cupboard.
  ///
  /// Our test chunks deliberately do NOT end in `Return`. When the instruction
  /// pointer runs off the end of the code, the VM pops the (only) frame and
  /// stops, leaving whatever we computed sitting on the stack for us to check.
  fn run_vm(function: RcCompiledFunction) -> Vm {
    let mut vm = Vm::new();
    vm.frames.push(CallFrame {
      function,
      ip: 0,
      stack_start: 0,
      scope_depth: 0,
      upvalues: Vec::new(),
      local_slots: HashMap::new(),
    });
    vm.run_loop().expect("VM run should succeed");
    vm
  }

  /// Like `run_vm` but keeps the `Result` so we can assert on failure paths.
  fn run_result(function: RcCompiledFunction) -> Result<Vec<Value>, GupError> {
    let mut vm = Vm::new();
    vm.frames.push(CallFrame {
      function,
      ip: 0,
      stack_start: 0,
      scope_depth: 0,
      upvalues: Vec::new(),
      local_slots: HashMap::new(),
    });
    vm.run_loop()?;
    Ok(vm.stack)
  }

  fn run_stack(function: RcCompiledFunction) -> Vec<Value> {
    run_vm(function).stack
  }

  fn top(stack: &[Value]) -> &Value {
    stack.last().expect("expected at least one value on the stack")
  }

  fn assert_number(value: &Value, expected: i64) {
    match value {
      Value::GuppyNumber(n) => assert_eq!(*n, expected, "wrong number on stack"),
      other => panic!("expected GuppyNumber({expected}), got {other:?}"),
    }
  }

  fn assert_float(value: &Value, expected: f64) {
    match value {
      Value::GuppyFloat(f) => {
        assert!((f - expected).abs() < 1e-9, "expected float {expected}, got {f}")
      }
      other => panic!("expected GuppyFloat({expected}), got {other:?}"),
    }
  }

  fn assert_bool(value: &Value, expected: bool) {
    match value {
      Value::GuppyBool(b) => assert_eq!(*b, expected, "wrong bool on stack"),
      other => panic!("expected GuppyBool({expected}), got {other:?}"),
    }
  }

  fn assert_string(value: &Value, expected: &str) {
    match value {
      Value::GuppyString(s) => assert_eq!(s, expected, "wrong string on stack"),
      other => panic!("expected GuppyString({expected:?}), got {other:?}"),
    }
  }

  #[test]
  fn constant_pushes_a_value() {
    let mut b = ChunkBuilder::new();
    b.push_value(Value::GuppyNumber(42));
    let stack = run_stack(b.finish());
    assert_eq!(stack.len(), 1);
    assert_number(top(&stack), 42);
  }

  #[test]
  fn adds_two_numbers() {
    let mut b = ChunkBuilder::new();
    b.push_value(Value::GuppyNumber(2))
      .push_value(Value::GuppyNumber(3))
      .op(OpCode::Add);
    assert_number(top(&run_stack(b.finish())), 5);
  }

  #[test]
  fn subtracts_multiplies_and_divides() {
    // 10 - 4 = 6
    let mut sub = ChunkBuilder::new();
    sub
      .push_value(Value::GuppyNumber(10))
      .push_value(Value::GuppyNumber(4))
      .op(OpCode::Sub);
    assert_number(top(&run_stack(sub.finish())), 6);

    // 6 * 7 = 42
    let mut mul = ChunkBuilder::new();
    mul
      .push_value(Value::GuppyNumber(6))
      .push_value(Value::GuppyNumber(7))
      .op(OpCode::Mul);
    assert_number(top(&run_stack(mul.finish())), 42);

    // 20 / 5 = 4 (two whole numbers in -> whole number out)
    let mut div = ChunkBuilder::new();
    div
      .push_value(Value::GuppyNumber(20))
      .push_value(Value::GuppyNumber(5))
      .op(OpCode::Div);
    assert_number(top(&run_stack(div.finish())), 4);
  }

  #[test]
  fn a_float_operand_makes_a_float_result() {
    // 1.5 + 2 = 3.5 (a float anywhere makes the whole answer a float)
    let mut b = ChunkBuilder::new();
    b.push_value(Value::GuppyFloat(1.5))
      .push_value(Value::GuppyNumber(2))
      .op(OpCode::Add);
    assert_float(top(&run_stack(b.finish())), 3.5);
  }

  #[test]
  fn dividing_by_zero_is_a_runtime_error() {
    let mut b = ChunkBuilder::new();
    b.push_value(Value::GuppyNumber(1))
      .push_value(Value::GuppyNumber(0))
      .op(OpCode::Div);
    let error = run_result(b.finish()).expect_err("dividing by zero should fail");
    assert!(
      error.message.contains("divide by zero"),
      "unexpected error message: {}",
      error.message
    );
  }

  #[test]
  fn negate_flips_the_sign() {
    let mut b = ChunkBuilder::new();
    b.push_value(Value::GuppyNumber(5)).op(OpCode::Negate);
    assert_number(top(&run_stack(b.finish())), -5);
  }

  #[test]
  fn not_inverts_truthiness() {
    // not true = false
    let mut t = ChunkBuilder::new();
    t.op(OpCode::True).op(OpCode::Not);
    assert_bool(top(&run_stack(t.finish())), false);

    // not nothing = true (Nothing is falsy)
    let mut n = ChunkBuilder::new();
    n.op(OpCode::Nil).op(OpCode::Not);
    assert_bool(top(&run_stack(n.finish())), true);
  }

  #[test]
  fn comparisons_push_booleans() {
    // 2 < 3 -> true
    let mut less = ChunkBuilder::new();
    less
      .push_value(Value::GuppyNumber(2))
      .push_value(Value::GuppyNumber(3))
      .op(OpCode::Less);
    assert_bool(top(&run_stack(less.finish())), true);

    // 2 > 3 -> false
    let mut greater = ChunkBuilder::new();
    greater
      .push_value(Value::GuppyNumber(2))
      .push_value(Value::GuppyNumber(3))
      .op(OpCode::Greater);
    assert_bool(top(&run_stack(greater.finish())), false);

    // 4 == 4 -> true
    let mut equal = ChunkBuilder::new();
    equal
      .push_value(Value::GuppyNumber(4))
      .push_value(Value::GuppyNumber(4))
      .op(OpCode::Equal);
    assert_bool(top(&run_stack(equal.finish())), true);
  }

  #[test]
  fn add_concatenates_when_a_string_is_involved() {
    // "foo" + "bar" = "foobar"
    let mut both = ChunkBuilder::new();
    both
      .push_value(Value::GuppyString("foo".to_string()))
      .push_value(Value::GuppyString("bar".to_string()))
      .op(OpCode::Add);
    assert_string(top(&run_stack(both.finish())), "foobar");

    // "n=" + 5 = "n=5" (the number is coerced to text)
    let mut mixed = ChunkBuilder::new();
    mixed
      .push_value(Value::GuppyString("n=".to_string()))
      .push_value(Value::GuppyNumber(5))
      .op(OpCode::Add);
    assert_string(top(&run_stack(mixed.finish())), "n=5");
  }

  #[test]
  fn boolean_and_nil_literals_land_on_the_stack() {
    let mut b = ChunkBuilder::new();
    b.op(OpCode::True).op(OpCode::False).op(OpCode::Nil);
    let stack = run_stack(b.finish());
    assert_eq!(stack.len(), 3);
    assert_bool(&stack[0], true);
    assert_bool(&stack[1], false);
    assert!(matches!(stack[2], Value::Nothing));
  }

  #[test]
  fn builds_an_array_and_reads_an_index() {
    let mut b = ChunkBuilder::new();
    b.push_value(Value::GuppyNumber(10))
      .push_value(Value::GuppyNumber(20))
      .push_value(Value::GuppyNumber(30))
      .op_byte(OpCode::MakeArray, 3)
      .push_value(Value::GuppyNumber(1))
      .op(OpCode::GetIndex);
    // [10, 20, 30][1] == 20
    assert_number(top(&run_stack(b.finish())), 20);
  }

  #[test]
  fn array_length_counts_items() {
    let mut b = ChunkBuilder::new();
    b.push_value(Value::GuppyNumber(7))
      .push_value(Value::GuppyNumber(8))
      .push_value(Value::GuppyNumber(9))
      .op_byte(OpCode::MakeArray, 3)
      .op(OpCode::ArrayLen);
    assert_number(top(&run_stack(b.finish())), 3);
  }

  #[test]
  fn build_range_counts_up_and_down() {
    // 1 through 4 -> [1, 2, 3, 4]
    let mut up = ChunkBuilder::new();
    up.push_value(Value::GuppyNumber(1))
      .push_value(Value::GuppyNumber(4))
      .op(OpCode::BuildRange);
    assert_eq!(
      top(&run_stack(up.finish())).to_display_string(),
      "[1, 2, 3, 4]"
    );

    // 3 through 1 -> [3, 2, 1] (ranges can walk backwards too)
    let mut down = ChunkBuilder::new();
    down
      .push_value(Value::GuppyNumber(3))
      .push_value(Value::GuppyNumber(1))
      .op(OpCode::BuildRange);
    assert_eq!(
      top(&run_stack(down.finish())).to_display_string(),
      "[3, 2, 1]"
    );
  }

  #[test]
  fn define_and_read_a_global() {
    let mut b = ChunkBuilder::new();
    let name = b.string_constant("score");
    b.push_value(Value::GuppyNumber(7))
      .op_byte(OpCode::DefineGlobal, name)
      .op_byte(OpCode::GetGlobal, name);
    let vm = run_vm(b.finish());
    // GetGlobal pushed the value back onto the stack...
    assert_number(top(&vm.stack), 7);
    // ...and it really lives in the globals cupboard.
    assert_number(vm.globals.get("score").expect("global should exist"), 7);
  }

  #[test]
  fn jump_skips_instructions() {
    // push 1, jump over "push 2", push 3  =>  stack ends up [1, 3]
    let mut b = ChunkBuilder::new();
    b.push_value(Value::GuppyNumber(1))
      .op_u16(OpCode::Jump, 2) // skip the next Constant (opcode + index = 2 bytes)
      .push_value(Value::GuppyNumber(2))
      .push_value(Value::GuppyNumber(3));
    let stack = run_stack(b.finish());
    assert_eq!(stack.len(), 2);
    assert_number(&stack[0], 1);
    assert_number(&stack[1], 3);
  }

  #[test]
  fn jump_if_false_only_jumps_when_falsy() {
    // condition is false -> jump over "push 100" and land on "push 200"
    let mut taken = ChunkBuilder::new();
    taken
      .op(OpCode::False)
      .op_u16(OpCode::JumpIfFalse, 2)
      .push_value(Value::GuppyNumber(100))
      .push_value(Value::GuppyNumber(200));
    let stack = run_stack(taken.finish());
    // JumpIfFalse peeks at the condition (it does not pop it), so `false`
    // is still sitting underneath the result.
    assert_eq!(stack.len(), 2);
    assert_bool(&stack[0], false);
    assert_number(&stack[1], 200);

    // condition is true -> no jump, so "push 100" runs as normal
    let mut fell_through = ChunkBuilder::new();
    fell_through
      .op(OpCode::True)
      .op_u16(OpCode::JumpIfFalse, 2)
      .push_value(Value::GuppyNumber(100));
    let stack = run_stack(fell_through.finish());
    assert_eq!(stack.len(), 2);
    assert_bool(&stack[0], true);
    assert_number(&stack[1], 100);
  }
}
