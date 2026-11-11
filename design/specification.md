# Guppty Language Specification

**Version:** 0.2.0 (Normative Standard)  
**Status:** Canonical Reference  
**Last Updated:** November 2026  
**Reference:** GitHub Issues #68, #76, #79, #82, #88, #89, #90  

---

> [!IMPORTANT]
> **Normative Authority Notice**  
> This document is the single normative and authoritative specification of the Guppty programming language. Other documentation files (including `README.md`, `design/grammar.md`, `design/syntax.md`, and the web documentation) are non-normative summaries, tutorials, or guides. In any event of discrepancy, this specification governs.

---

## 1. Conformance & Implementation Architecture

- **[SPEC-CONF-01] Dual-Engine Parity**: Any conforming Guppty implementation (including the default bytecode virtual machine and the tree-walking reference interpreter) must produce identical output, side effects, and error classifications for every valid or invalid program.
- **[SPEC-CONF-02] Source Encoding**: Guppty source text is encoded as valid UTF-8. Non-UTF-8 sequences are rejected during lexical analysis.

---

## 2. Lexical Grammar & Tokens

- **[SPEC-LEX-01] Whitespace & Lines**: Whitespace consists of ASCII space (`0x20`), horizontal tabs (`0x09`), and line breaks (LF `\n` or CRLF `\r\n`). Horizontal tabs are strictly forbidden in indentation (see `[SPEC-INDENT-02]`).
- **[SPEC-LEX-02] Comments**: Line comments begin with `//` and extend to the end of the current physical line. Inline comments following statements are stripped before AST construction. Comment characters inside string literals (`"//"`) do not initiate a comment.
- **[SPEC-LEX-03] Identifiers**: An identifier begins with an ASCII letter (`a`-`z`, `A`-`Z`) or underscore (`_`), followed by zero or more ASCII letters, digits (`0`-`9`), or underscores. Identifiers are case-sensitive.
- **[SPEC-LEX-04] Keywords**: The following 13 tokens are reserved keywords and cannot be used as variable or function names:
  ```text
  for  in  range  through  if  else  while  return  and  or  not  true  false
  ```
- **[SPEC-LEX-05] Operators & Punctuators**:
  - Arithmetic: `+`, `-`, `*`, `/`
  - Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
  - Assignment: `=`
  - Delimiters: `(`, `)`, `[`, `]`, `,`, `;`

---

## 3. Indentation & Block Structure

- **[SPEC-INDENT-01] Strict 4-Space Indentation**: Block indentation levels are defined strictly by multiples of four ASCII spaces (`    `, `        `, etc.). Each indent step emits an `INDENT` token; each matching unindent step emits a `DEDENT` token.
- **[SPEC-INDENT-02] Prohibition of Tabs**: Tab characters (`\t`) within line indentation prefixes are strictly rejected with an `IndentationError`.
- **[SPEC-INDENT-03] Dedent Alignment**: A dedent must match an indentation level previously opened by an active block. A dedent to an unaligned column produces an `IndentationError`.
- **[SPEC-INDENT-04] Partial Indents**: Indentation with character counts not divisible by 4 (e.g. 2 spaces, 5 spaces) is rejected with an `IndentationError`.

---

## 4. Types & Value Semantics

Guppty defines eight core value types:

1. **Number (`[SPEC-TYPE-NUM]`)**: Signed 64-bit two's complement integer (`i64`). Integer literals without a decimal point parse into `Number`.
2. **Float (`[SPEC-TYPE-FLOAT]`)**: IEEE 754 64-bit double precision floating-point number (`f64`). Numeric literals with a decimal point parse into `Float`.
3. **Bool (`[SPEC-TYPE-BOOL]`)**: Boolean values `true` or `false`.
4. **String (`[SPEC-TYPE-STR]`)**: UTF-8 string literals enclosed in double quotes (`"hello"`).
5. **Char (`[SPEC-TYPE-CHAR]`)**: Single Unicode scalar value enclosed in single quotes (`'c'`). Single quotes containing 0 or >1 characters produce a `SyntaxError`.
6. **Nothing (`[SPEC-TYPE-NIL]`)**: The unit value representing absent or completed computation.
7. **Array (`[SPEC-TYPE-ARR]`)**: Ordered sequences of values (`[1, 2, 3]`). Indexing is 0-based (`arr[0]`). Out-of-bounds indexing raises an `IndexError`.
8. **Function / Closure (`[SPEC-TYPE-FN]`)**: First-class callable functions capturing their lexical scope.

---

## 5. Numeric Semantics & Arithmetic

- **[SPEC-NUM-01] Exact Integer Arithmetic**: Operations between two `Number` values (`+`, `-`, `*`, `/`) produce an exact signed 64-bit `Number`.
- **[SPEC-NUM-02] Float Promotion**: If either operand of an arithmetic operation is a `Float`, the other operand is promoted to `Float` and the result is a `Float`.
- **[SPEC-NUM-03] Division by Zero**: Division by zero on integers (`n / 0`) or floats (`f / 0.0`) raises a runtime `MathError`.
- **[SPEC-NUM-04] String Concatenation**: If either operand of the `+` operator is a `String`, both operands are converted to their display strings and concatenated into a `String`.

---

## 6. Truthiness & Logical Operators

- **[SPEC-TRUTH-01] Truthiness**:
  - The following values are **falsy**: `false`, `Nothing`.
  - All other values are **truthy**: `true`, `0`, non-zero integers, `0.0`, non-zero floats, `""`, non-empty strings, `[]`, non-empty arrays, functions.
- **[SPEC-LOGIC-01] Short-Circuit Evaluation**:
  - `a and b`: Evaluates `a`. If `a` is falsy, returns `false` without evaluating `b`. Otherwise evaluates and returns truthiness of `b`.
  - `a or b`: Evaluates `a`. If `a` is truthy, returns `true` without evaluating `b`. Otherwise evaluates and returns truthiness of `b`.
  - `not a`: Inverts the truthiness of `a`, returning a `Bool`.

---

## 7. Operator Precedence & Associativity

- **[SPEC-OP-01] Precedence Table** (from lowest binding power to highest):

| Level | Operators | Description | Associativity |
|---|---|---|---|
| 1 | `or` | Logical disjunction | Left |
| 2 | `and` | Logical conjunction | Left |
| 3 | `==`, `!=` | Equality / inequality | Left |
| 4 | `<`, `<=`, `>`, `>=` | Relational ordering | Left |
| 5 | `+`, `-` | Addition, subtraction, string concat | Left |
| 6 | `*`, `/` | Multiplication, division | Left |
| 7 | `-` (unary), `not` | Arithmetic negation, logical negation | Right |
| 8 | `f(...)`, `a[i]`, `(...)` | Calls, indexing, grouping | Left |

---

## 8. Control Flow & Loops

- **[SPEC-FLOW-01] Conditional Execution (`if` / `else`)**:
  - `if condition \n [INDENT] then_body [DEDENT]`: Executes `then_body` if `condition` is truthy.
  - Optional `else \n [INDENT] else_body [DEDENT]`: Executes `else_body` if `condition` is falsy.
- **[SPEC-FLOW-02] While Loops**:
  - `while condition \n [INDENT] body [DEDENT]`: Repeatedly evaluates `condition` and executes `body` while truthy.
- **[SPEC-FLOW-03] For-in Range Loops**:
  - `for variable in range(start through end) \n [INDENT] body [DEDENT]`.
  - Range endpoints are **inclusive**.
  - Upward iteration: If `start <= end`, steps upwards by `+1` from `start` through `end`.
  - Downward iteration: If `start > end`, steps downwards by `-1` from `start` through `end`.
  - Iteration over arrays: `for item in array` iterates over sequential items from index 0.

---

## 9. Functions, Closures, and Scope

- **[SPEC-FUNC-01] Function Definitions**: Functions are defined with signature and an indented body:
  ```gup
  name(param1, param2)
      return param1 + param2
  ```
- **[SPEC-FUNC-02] Block Scoping**: Variable bindings declared inside a block or loop body are scoped locally to that block and shadow outer bindings of the same name.
- **[SPEC-FUNC-03] Closures & Upvalue Escapes**: Functions capture free variables from their enclosing lexical scopes. Captured variables remain accessible even after the enclosing scope has returned.

---

## 10. Standard Built-in Functions

- **[SPEC-BUILTIN-01] `out(...)`**: Variadic output function. Evaluates each argument and writes their display string representation to standard output, space-separated, followed by a newline. Returns `Nothing`.

---

## 11. Error Taxonomy & Diagnostics

- **[SPEC-ERR-01] Standard Diagnostic Format**: All diagnostic messages emitted by Guppty must include:
  1. Source location: `filename:line:column:`
  2. Error kind: `SyntaxError`, `IndentationError`, `NameError`, `TypeError`, `ValueError`, `MathError`, `ArgumentError`, `IndexError`, or `RuntimeError`.
  3. Source excerpt with carets highlighting the offending span.
  4. Help suggestion prefixed by `= help:`.

---

## 12. Implementation Limits & Defaults

- **[SPEC-LIMIT-01] Default Resource Limits**:
  - Maximum call depth: `1,000` stack frames.
  - Maximum value stack depth: `10,000` values.
  - Maximum range size: `1,000,000` items.
  - Execution steps: unbounded by default (configurable).

---

## 13. Specification Compatibility & Changelog

| Version | Date | Status | Major Additions & Notes |
|---|---|---|---|
| **0.2.0** | 2026-11 | Current Normative Standard | Full normative specification; checked i64 integer semantics (#76); array syntax and indexing (#79); bytecode limits (#82); reproducible benchmark workloads (#84); citable research artifact (#85); dual-backend conformance (#90). |
| **0.1.0** | 2026-09 | Historical Prototype | Initial prototype language implementation. Non-normative. |
