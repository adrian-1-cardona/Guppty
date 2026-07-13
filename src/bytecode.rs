// === bytecode.rs ===
// This file is the "recipe book" for the virtual machine (VM).
//
// Imagine you are baking cookies:
//   - The AST is the picture of cookies on the box.
//   - Bytecode is the numbered steps: "step 1: add sugar", "step 2: mix".
//   - The VM is the little robot that reads the steps and bakes.
//
// A **chunk** is one full recipe: byte steps + a box of ingredients (constants).

use crate::error::Span;
use crate::value::Value;

/// One tiny instruction for the VM robot.
/// Each variant is a different kind of step in the recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
  // --- Push values onto the stack (the robot's work table) ---
  Constant, // push constants[index]
  Nil,      // push "nothing"
  True,     // push boolean true
  False,    // push boolean false

  // --- Stack cleanup ---
  Pop, // throw away the top value

  // --- Variables: locals live in the current function frame ---
  GetLocal, // read local slot
  SetLocal, // write local slot
  DefineLocal, // make a new local slot in the current scope

  // --- Variables: globals live in the big shared cupboard ---
  GetGlobal,    // read global by name (index into name table)
  SetGlobal,    // write global by name
  DefineGlobal, // create global by name

  // --- Closures: "remember" variables from outer functions ---
  GetUpvalue,    // read captured variable
  SetUpvalue,    // write captured variable
  CloseUpvalue,  // close over locals when leaving a block

  // --- Guppy's special "does this name exist anywhere?" store ---
  GetVariable,   // read a variable by searching scopes at runtime
  StoreVariable, // assign if found, else define in current scope

  // --- Math and logic ---
  Add,
  Sub,
  Mul,
  Div,
  Negate,
  Not,
  Equal,
  NotEqual,
  Less,
  Greater,
  LessEqual,
  GreaterEqual,

  // --- Jumping around (for if/while/for) ---
  EnterScope,  // we walked into an indented block
  ExitScope,   // we walked out of an indented block
  Jump,        // always jump forward/back by offset
  JumpIfFalse, // jump only if top value is "falsy"
  Loop,        // jump backward (handy for while loops)

  // --- Functions ---
  Call,    // call function with N arguments
  Return,  // go back to whoever called us
  Closure, // build a function value with upvalues
  Print,   // out() — print to stdout

  // --- Collections ---
  MakeArray,  // build [a, b, c] from N stack values
  BuildRange, // pop end, pop start -> push number array
  GetIndex,   // pop index, pop array -> push element
  ArrayLen,   // pop array -> push its length as number
}

/// Things we keep in the constant pool (ingredients referenced by index).
#[derive(Debug, Clone)]
pub enum Constant {
  Value(Value),
  String(String),
  Function(RcCompiledFunction),
}

/// A compiled function body — params + bytecode + how many upvalues it needs.
#[derive(Debug, Clone)]
pub struct CompiledFunction {
  pub arity: usize,
  pub chunk: Chunk,
  pub upvalues: Vec<UpvalueDescriptor>,
  pub local_names: Vec<String>,
}

/// Tells the VM how to wire each upvalue when making a closure.
#[derive(Debug, Clone, Copy)]
pub struct UpvalueDescriptor {
  pub index: u8,
  pub is_local: bool,
}

/// A type alias so we can share compiled functions cheaply (Rc = shared pointer).
pub type RcCompiledFunction = std::rc::Rc<std::cell::RefCell<CompiledFunction>>;

/// A closed-over variable used by closures at runtime.
#[derive(Debug, Clone)]
pub struct ClosureUpvalue {
  pub open_slot: Option<usize>,
  pub closed: Option<crate::value::Value>,
}

/// A runnable closure bundles compiled code plus captured outer variables.
#[derive(Debug, Clone)]
pub struct ClosureValue {
  pub function: RcCompiledFunction,
  pub upvalues: Vec<std::rc::Rc<std::cell::RefCell<ClosureUpvalue>>>,
}

/// The full recipe: bytecode bytes + constants + source locations for errors.
#[derive(Debug, Clone)]
pub struct Chunk {
  pub code: Vec<u8>,
  pub constants: Vec<Constant>,
  pub spans: Vec<Span>,
}

impl Chunk {
  /// Start with an empty recipe.
  pub fn new() -> Self {
    Chunk {
      code: Vec::new(),
      constants: Vec::new(),
      spans: Vec::new(),
    }
  }

  /// Add a brand-new ingredient to the pool and return its shelf number.
  pub fn add_constant(&mut self, constant: Constant) -> usize {
    let index = self.constants.len();
    self.constants.push(constant);
    index
  }

  /// Write one byte of bytecode and remember which source line it came from.
  pub fn write(&mut self, byte: u8, span: Span) {
    self.code.push(byte);
    self.spans.push(span);
  }

  /// Patch a jump offset after we know how far to jump.
  pub fn patch_u16(&mut self, offset: usize, value: u16) {
    self.code[offset] = (value >> 8) as u8;
    self.code[offset + 1] = (value & 0xff) as u8;
  }

  /// How many bytes of code we have so far (used for jump math).
  pub fn current_offset(&self) -> usize {
    self.code.len()
  }
}

impl Default for Chunk {
  fn default() -> Self {
    Self::new()
  }
}

/// Read one opcode byte and turn it into an OpCode enum value.
pub fn decode_opcode(byte: u8) -> Option<OpCode> {
  match byte {
    0 => Some(OpCode::Constant),
    1 => Some(OpCode::Nil),
    2 => Some(OpCode::True),
    3 => Some(OpCode::False),
    4 => Some(OpCode::Pop),
    5 => Some(OpCode::GetLocal),
    6 => Some(OpCode::SetLocal),
    7 => Some(OpCode::DefineLocal),
    8 => Some(OpCode::GetGlobal),
    9 => Some(OpCode::SetGlobal),
    10 => Some(OpCode::DefineGlobal),
    11 => Some(OpCode::GetUpvalue),
    12 => Some(OpCode::SetUpvalue),
    13 => Some(OpCode::CloseUpvalue),
    14 => Some(OpCode::GetVariable),
    15 => Some(OpCode::StoreVariable),
    16 => Some(OpCode::Add),
    17 => Some(OpCode::Sub),
    18 => Some(OpCode::Mul),
    19 => Some(OpCode::Div),
    20 => Some(OpCode::Negate),
    21 => Some(OpCode::Not),
    22 => Some(OpCode::Equal),
    23 => Some(OpCode::NotEqual),
    24 => Some(OpCode::Less),
    25 => Some(OpCode::Greater),
    26 => Some(OpCode::LessEqual),
    27 => Some(OpCode::GreaterEqual),
    28 => Some(OpCode::EnterScope),
    29 => Some(OpCode::ExitScope),
    30 => Some(OpCode::Jump),
    31 => Some(OpCode::JumpIfFalse),
    32 => Some(OpCode::Loop),
    33 => Some(OpCode::Call),
    34 => Some(OpCode::Return),
    35 => Some(OpCode::Closure),
    36 => Some(OpCode::Print),
    37 => Some(OpCode::MakeArray),
    38 => Some(OpCode::BuildRange),
    39 => Some(OpCode::GetIndex),
    40 => Some(OpCode::ArrayLen),
    _ => None,
  }
}

/// Turn an OpCode enum into the byte we store in the chunk.
pub fn encode_opcode(op: OpCode) -> u8 {
  match op {
    OpCode::Constant => 0,
    OpCode::Nil => 1,
    OpCode::True => 2,
    OpCode::False => 3,
    OpCode::Pop => 4,
    OpCode::GetLocal => 5,
    OpCode::SetLocal => 6,
    OpCode::DefineLocal => 7,
    OpCode::GetGlobal => 8,
    OpCode::SetGlobal => 9,
    OpCode::DefineGlobal => 10,
    OpCode::GetUpvalue => 11,
    OpCode::SetUpvalue => 12,
    OpCode::CloseUpvalue => 13,
    OpCode::GetVariable => 14,
    OpCode::StoreVariable => 15,
    OpCode::Add => 16,
    OpCode::Sub => 17,
    OpCode::Mul => 18,
    OpCode::Div => 19,
    OpCode::Negate => 20,
    OpCode::Not => 21,
    OpCode::Equal => 22,
    OpCode::NotEqual => 23,
    OpCode::Less => 24,
    OpCode::Greater => 25,
    OpCode::LessEqual => 26,
    OpCode::GreaterEqual => 27,
    OpCode::EnterScope => 28,
    OpCode::ExitScope => 29,
    OpCode::Jump => 30,
    OpCode::JumpIfFalse => 31,
    OpCode::Loop => 32,
    OpCode::Call => 33,
    OpCode::Return => 34,
    OpCode::Closure => 35,
    OpCode::Print => 36,
    OpCode::MakeArray => 37,
    OpCode::BuildRange => 38,
    OpCode::GetIndex => 39,
    OpCode::ArrayLen => 40,
  }
}
