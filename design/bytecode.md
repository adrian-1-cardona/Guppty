# Guppty Bytecode Specification and Limits

This document describes the virtual machine bytecode format, instruction encoding, operand widths,
and checked execution limits in Guppty.

## Bytecode Chunks

A compiled Guppty unit is stored in a `Chunk`:
- `code`: Vector of `u8` bytecode bytes.
- `constants`: Vector of `Constant` items (values, string names, nested compiled functions).
- `spans`: Parallel vector of source locations (`Span`) tracking each instruction byte back to user source.

## Operand Encodings and Limits

| Operand Kind | Width | Maximum Capacity | Checked Error on Overflow |
|---|---|---|---|
| Constant Pool Index | `u8` (1 byte) | 256 constants per chunk | Compile error: `Too many constants in one chunk (limit is 256).` |
| Local Variable Slot | `u8` (1 byte) | 256 locals per function | Compile error: `Too many local variables in function (limit is 256).` |
| Upvalue Slot | `u8` (1 byte) | 256 upvalues per closure | Compile error: `Too many upvalues in closure (limit is 256).` |
| Call Arguments Count | `u8` (1 byte) | 255 arguments per call | Compile error: `Too many function arguments (limit is 255, got N).` |
| Function Parameters | `u8` (1 byte) | 255 parameters | Compile error: `Too many parameters in function definition (limit is 255, got N).` |
| Array Literal Elements | `u8` (1 byte) | 255 elements per literal | Compile error: `Too many array elements in literal (limit is 255, got N).` |
| Jump Offset | `u16` (2 bytes, big-endian) | 65,535 bytes | Compile error: `Too much code to jump over (limit is 65535 bytes).` |
| Loop Offset | `u16` (2 bytes, big-endian) | 65,535 bytes | Compile error: `Loop body too large to jump back (limit is 65535 bytes).` |

## Opcode Table

| Opcode Byte | Mnemonic | Operands | Description |
|---|---|---|---|
| 0 | `Constant` | `[index: u8]` | Push `constants[index]` onto stack |
| 1 | `Nil` | none | Push `Nothing` onto stack |
| 2 | `True` | none | Push `true` onto stack |
| 3 | `False` | none | Push `false` onto stack |
| 4 | `Pop` | none | Discard top of stack |
| 5 | `GetLocal` | `[slot: u8]` | Push value from frame local slot |
| 6 | `SetLocal` | `[slot: u8]` | Store top of stack into frame local slot |
| 7 | `GetGlobal` | `[name_index: u8]` | Push global variable value |
| 8 | `SetGlobal` | `[name_index: u8]` | Update existing global variable |
| 9 | `DefineGlobal` | `[name_index: u8]` | Create global variable |
| 10 | `GetUpvalue` | `[slot: u8]` | Push captured upvalue |
| 11 | `SetUpvalue` | `[slot: u8]` | Update captured upvalue |
| 12 | `CloseUpvalue` | none | Close open upvalues for leaving block |
| 13 | `GetVariable` | `[name_index: u8]` | Dynamic lookup across active scopes |
| 14 | `StoreVariable` | `[name_index: u8]` | Dynamic store/declare |
| 15..26 | Math & Logic | none | Binary & unary operators (`Add`, `Sub`, `Mul`, `Div`, `Negate`, `Not`, `Equal`, `NotEqual`, `Less`, `Greater`, `LessEqual`, `GreaterEqual`) |
| 27 | `EnterScope` | none | Enter indented block scope |
| 28 | `ExitScope` | none | Exit indented block scope |
| 29 | `Jump` | `[offset: u16]` | Unconditional forward jump |
| 30 | `JumpIfFalse` | `[offset: u16]` | Forward jump if top of stack is falsy |
| 31 | `Loop` | `[offset: u16]` | Backward jump to loop header |
| 32 | `Call` | `[arg_count: u8]` | Invoke function with arguments |
| 33 | `Return` | none | Return from function |
| 34 | `Closure` | `[func_index: u8]` | Create closure instance from compiled function |
| 35 | `Print` | `[arg_count: u8]` | Built-in `out(...)` print |
| 36 | `MakeArray` | `[count: u8]` | Construct array from `count` stack elements |
| 37 | `BuildRange` | none | Build range array from start and end |
| 38 | `GetIndex` | none | Read element at index from array |
| 39 | `ArrayLen` | none | Push length of array |

## Bytecode Validation

Before execution, bytecode chunks are verified by `validate_chunk` / `validate_function`:
1. Every opcode must be recognized in `decode_opcode`.
2. All instruction operands must lie completely within chunk code bounds.
3. Constant, variable name, and closure indices must be valid references into the chunk constant pool.
4. Jump and loop targets must not underflow or exceed chunk code boundaries.
5. Malformed chunks produce a clean `GupError::runtime` rather than VM crashes, underflows, or panics.
