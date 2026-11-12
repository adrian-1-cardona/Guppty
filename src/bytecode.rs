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

    // --- Variables: globals live in the big shared cupboard ---
    GetGlobal,    // read global by name (index into name table)
    SetGlobal,    // write global by name
    DefineGlobal, // create global by name

    // --- Closures: "remember" variables from outer functions ---
    GetUpvalue,   // read captured variable
    SetUpvalue,   // write captured variable
    CloseUpvalue, // close over locals when leaving a block

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
    pub name: Option<String>,
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

    /// Disassemble one instruction at the given byte offset.
    /// Returns the formatted instruction string and the byte offset of the next instruction.
    pub fn disassemble_instruction(
        &self,
        offset: usize,
        local_names: &[String],
    ) -> (String, usize) {
        if offset >= self.code.len() {
            return ("<end of chunk>".to_string(), offset);
        }
        let byte = self.code[offset];
        let Some(op) = decode_opcode(byte) else {
            return (format!("UnknownOpcode({byte:#04x})"), offset + 1);
        };

        match op {
            OpCode::Constant => {
                let idx = self.code.get(offset + 1).copied().unwrap_or(0) as usize;
                let c_str = self.format_constant(idx);
                (
                    format!("{:<16} {:<4} ({})", "Constant", idx, c_str),
                    offset + 2,
                )
            }
            OpCode::Nil => ("Nil".to_string(), offset + 1),
            OpCode::True => ("True".to_string(), offset + 1),
            OpCode::False => ("False".to_string(), offset + 1),
            OpCode::Pop => ("Pop".to_string(), offset + 1),
            OpCode::GetLocal => {
                let slot = self.code.get(offset + 1).copied().unwrap_or(0);
                let name = local_names
                    .get(slot as usize)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                (
                    format!("{:<16} {:<4} (slot {}: {})", "GetLocal", slot, slot, name),
                    offset + 2,
                )
            }
            OpCode::SetLocal => {
                let slot = self.code.get(offset + 1).copied().unwrap_or(0);
                let name = local_names
                    .get(slot as usize)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                (
                    format!("{:<16} {:<4} (slot {}: {})", "SetLocal", slot, slot, name),
                    offset + 2,
                )
            }
            OpCode::GetGlobal => {
                let idx = self.code.get(offset + 1).copied().unwrap_or(0) as usize;
                let name = self.format_constant(idx);
                (
                    format!("{:<16} {:<4} ({})", "GetGlobal", idx, name),
                    offset + 2,
                )
            }
            OpCode::SetGlobal => {
                let idx = self.code.get(offset + 1).copied().unwrap_or(0) as usize;
                let name = self.format_constant(idx);
                (
                    format!("{:<16} {:<4} ({})", "SetGlobal", idx, name),
                    offset + 2,
                )
            }
            OpCode::DefineGlobal => {
                let idx = self.code.get(offset + 1).copied().unwrap_or(0) as usize;
                let name = self.format_constant(idx);
                (
                    format!("{:<16} {:<4} ({})", "DefineGlobal", idx, name),
                    offset + 2,
                )
            }
            OpCode::GetUpvalue => {
                let slot = self.code.get(offset + 1).copied().unwrap_or(0);
                (format!("{:<16} {:<4}", "GetUpvalue", slot), offset + 2)
            }
            OpCode::SetUpvalue => {
                let slot = self.code.get(offset + 1).copied().unwrap_or(0);
                (format!("{:<16} {:<4}", "SetUpvalue", slot), offset + 2)
            }
            OpCode::CloseUpvalue => ("CloseUpvalue".to_string(), offset + 1),
            OpCode::GetVariable => {
                let idx = self.code.get(offset + 1).copied().unwrap_or(0) as usize;
                let name = self.format_constant(idx);
                (
                    format!("{:<16} {:<4} ({})", "GetVariable", idx, name),
                    offset + 2,
                )
            }
            OpCode::StoreVariable => {
                let idx = self.code.get(offset + 1).copied().unwrap_or(0) as usize;
                let name = self.format_constant(idx);
                (
                    format!("{:<16} {:<4} ({})", "StoreVariable", idx, name),
                    offset + 2,
                )
            }
            OpCode::Add => ("Add".to_string(), offset + 1),
            OpCode::Sub => ("Sub".to_string(), offset + 1),
            OpCode::Mul => ("Mul".to_string(), offset + 1),
            OpCode::Div => ("Div".to_string(), offset + 1),
            OpCode::Negate => ("Negate".to_string(), offset + 1),
            OpCode::Not => ("Not".to_string(), offset + 1),
            OpCode::Equal => ("Equal".to_string(), offset + 1),
            OpCode::NotEqual => ("NotEqual".to_string(), offset + 1),
            OpCode::Less => ("Less".to_string(), offset + 1),
            OpCode::Greater => ("Greater".to_string(), offset + 1),
            OpCode::LessEqual => ("LessEqual".to_string(), offset + 1),
            OpCode::GreaterEqual => ("GreaterEqual".to_string(), offset + 1),
            OpCode::EnterScope => ("EnterScope".to_string(), offset + 1),
            OpCode::ExitScope => ("ExitScope".to_string(), offset + 1),
            OpCode::Jump => {
                let hi = self.code.get(offset + 1).copied().unwrap_or(0) as u16;
                let lo = self.code.get(offset + 2).copied().unwrap_or(0) as u16;
                let jump = (hi << 8) | lo;
                let target = offset + 3 + jump as usize;
                (
                    format!("{:<16} {:<4} -> {:04x}", "Jump", jump, target),
                    offset + 3,
                )
            }
            OpCode::JumpIfFalse => {
                let hi = self.code.get(offset + 1).copied().unwrap_or(0) as u16;
                let lo = self.code.get(offset + 2).copied().unwrap_or(0) as u16;
                let jump = (hi << 8) | lo;
                let target = offset + 3 + jump as usize;
                (
                    format!("{:<16} {:<4} -> {:04x}", "JumpIfFalse", jump, target),
                    offset + 3,
                )
            }
            OpCode::Loop => {
                let hi = self.code.get(offset + 1).copied().unwrap_or(0) as u16;
                let lo = self.code.get(offset + 2).copied().unwrap_or(0) as u16;
                let jump = (hi << 8) | lo;
                let target = (offset + 3).saturating_sub(jump as usize);
                (
                    format!("{:<16} {:<4} -> {:04x}", "Loop", jump, target),
                    offset + 3,
                )
            }
            OpCode::Call => {
                let args = self.code.get(offset + 1).copied().unwrap_or(0);
                (format!("{:<16} {:<4}", "Call", args), offset + 2)
            }
            OpCode::Return => ("Return".to_string(), offset + 1),
            OpCode::Closure => {
                let idx = self.code.get(offset + 1).copied().unwrap_or(0) as usize;
                let desc = self.format_constant(idx);
                (
                    format!("{:<16} {:<4} ({})", "Closure", idx, desc),
                    offset + 2,
                )
            }
            OpCode::Print => {
                let args = self.code.get(offset + 1).copied().unwrap_or(0);
                (format!("{:<16} {:<4}", "Print", args), offset + 2)
            }
            OpCode::MakeArray => {
                let count = self.code.get(offset + 1).copied().unwrap_or(0);
                (format!("{:<16} {:<4}", "MakeArray", count), offset + 2)
            }
            OpCode::BuildRange => ("BuildRange".to_string(), offset + 1),
            OpCode::GetIndex => ("GetIndex".to_string(), offset + 1),
            OpCode::ArrayLen => ("ArrayLen".to_string(), offset + 1),
        }
    }

    /// Format a constant by index into a readable string.
    pub fn format_constant(&self, index: usize) -> String {
        match self.constants.get(index) {
            Some(Constant::Value(v)) => v.to_display_string(),
            Some(Constant::String(s)) => format!("\"{s}\""),
            Some(Constant::Function(f)) => {
                let name = f
                    .borrow()
                    .name
                    .clone()
                    .unwrap_or_else(|| "<anon>".to_string());
                format!("<fn {name}>")
            }
            None => "<invalid constant>".to_string(),
        }
    }

    /// Disassemble the entire chunk into a formatted string.
    pub fn disassemble(
        &self,
        name: &str,
        local_names: &[String],
        current_ip: Option<usize>,
    ) -> String {
        let mut out = String::new();
        out.push_str(&format!("== Bytecode for {} ==\n", name));
        let mut offset = 0;
        while offset < self.code.len() {
            let pointer = if current_ip == Some(offset) {
                "-> "
            } else {
                "   "
            };
            let (dis, next) = self.disassemble_instruction(offset, local_names);
            out.push_str(&format!("{}{:04x}  {}\n", pointer, offset, dis));
            offset = next;
        }
        out
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
        7 => Some(OpCode::GetGlobal),
        8 => Some(OpCode::SetGlobal),
        9 => Some(OpCode::DefineGlobal),
        10 => Some(OpCode::GetUpvalue),
        11 => Some(OpCode::SetUpvalue),
        12 => Some(OpCode::CloseUpvalue),
        13 => Some(OpCode::GetVariable),
        14 => Some(OpCode::StoreVariable),
        15 => Some(OpCode::Add),
        16 => Some(OpCode::Sub),
        17 => Some(OpCode::Mul),
        18 => Some(OpCode::Div),
        19 => Some(OpCode::Negate),
        20 => Some(OpCode::Not),
        21 => Some(OpCode::Equal),
        22 => Some(OpCode::NotEqual),
        23 => Some(OpCode::Less),
        24 => Some(OpCode::Greater),
        25 => Some(OpCode::LessEqual),
        26 => Some(OpCode::GreaterEqual),
        27 => Some(OpCode::EnterScope),
        28 => Some(OpCode::ExitScope),
        29 => Some(OpCode::Jump),
        30 => Some(OpCode::JumpIfFalse),
        31 => Some(OpCode::Loop),
        32 => Some(OpCode::Call),
        33 => Some(OpCode::Return),
        34 => Some(OpCode::Closure),
        35 => Some(OpCode::Print),
        36 => Some(OpCode::MakeArray),
        37 => Some(OpCode::BuildRange),
        38 => Some(OpCode::GetIndex),
        39 => Some(OpCode::ArrayLen),
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
        OpCode::GetGlobal => 7,
        OpCode::SetGlobal => 8,
        OpCode::DefineGlobal => 9,
        OpCode::GetUpvalue => 10,
        OpCode::SetUpvalue => 11,
        OpCode::CloseUpvalue => 12,
        OpCode::GetVariable => 13,
        OpCode::StoreVariable => 14,
        OpCode::Add => 15,
        OpCode::Sub => 16,
        OpCode::Mul => 17,
        OpCode::Div => 18,
        OpCode::Negate => 19,
        OpCode::Not => 20,
        OpCode::Equal => 21,
        OpCode::NotEqual => 22,
        OpCode::Less => 23,
        OpCode::Greater => 24,
        OpCode::LessEqual => 25,
        OpCode::GreaterEqual => 26,
        OpCode::EnterScope => 27,
        OpCode::ExitScope => 28,
        OpCode::Jump => 29,
        OpCode::JumpIfFalse => 30,
        OpCode::Loop => 31,
        OpCode::Call => 32,
        OpCode::Return => 33,
        OpCode::Closure => 34,
        OpCode::Print => 35,
        OpCode::MakeArray => 36,
        OpCode::BuildRange => 37,
        OpCode::GetIndex => 38,
        OpCode::ArrayLen => 39,
    }
}
