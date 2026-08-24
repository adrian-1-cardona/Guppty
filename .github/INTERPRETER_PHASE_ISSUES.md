# Issues addressed by the Lox interpreter phase PR

This PR completes the **Crafting Interpreters (Lox-level) interpreter phase** for Guppty. Pair each item below with a GitHub issue (create if missing, then close when merged).

| # | Issue title | What this PR delivers |
|---|-------------|----------------------|
| 1 | Implement lexical scopes with nested environments | `src/environment.rs` — scope chain with parent pointers; blocks open new scopes |
| 2 | Fix variable assignment to respect enclosing scopes | Assign walks the scope chain; new names define in the current scope |
| 3 | Add block scoping for if, while, for, and function bodies | `execute_block()` wraps each body in a child environment |
| 4 | Add function parameters | `name(a, b)` syntax in parser + parameter binding at call time |
| 5 | Add return statements and function return values | `return` / `return expr`; callers receive values |
| 6 | Implement closures (functions capture defining environment) | `GuppyFunction.closure` stores the birth environment |
| 7 | Add if / else control flow | `if condition` + optional `else` with indented blocks |
| 8 | Add while loops | `while condition` with indented body |
| 9 | Add comparison operators (==, !=, <, >, <=, >=) | Lexer + parser precedence + interpreter |
| 10 | Add logical operators (and, or, not) with short-circuit | Keywords + short-circuit evaluation in interpreter |
| 11 | Add unary minus for negative numbers | `-5`, `-3.5` via `UnaryOp::Negate` |
| 12 | Improve string concatenation with + | String + anything coerces to string |
| 13 | Add truthiness rules for if / while | `Value::is_truthy()` — only `false` and `Nothing` are falsy |
| 14 | Make syntax keywords configurable in one place | `src/syntax.rs` — change keywords without touching lexer/parser logic |
| 15 | Update design/grammar.md for the full language | Full grammar for statements, expressions, scoping |
| 16 | Update design/syntax.md with control flow and closures | Examples for if, while, return, closures |
| 17 | Add example: if / else | `examples/if_else.gup` |
| 18 | Add example: while loop countdown | `examples/while_countdown.gup` |
| 19 | Add example: comparisons | `examples/comparisons.gup` |
| 20 | Add example: boolean logic | `examples/booleans_logic.gup` |
| 21 | Add example: function parameters | `examples/function_params.gup` |
| 22 | Add example: function return values | `examples/function_return.gup` |
| 23 | Add example: recursion (factorial) | `examples/recursion_factorial.gup` |
| 24 | Add example: recursion (fibonacci) | `examples/recursion_fib.gup` |
| 25 | Add example: closure make-adder | `examples/closure_make_adder.gup` |
| 26 | Add example: closure counter | `examples/closure_counter.gup` |
| 27 | Add example: string concatenation | `examples/string_concat.gup` |
| 28 | Add example: block scope | `examples/scope_block.gup` |
| 29 | Add example: nested if | `examples/nested_if.gup` |
| 30 | Add 30 example programs with expected output | `examples/` + `examples/expected/*.txt` |
| 31 | Add integration tests for all example programs | `tests/example_programs.rs` — `cargo test` |

**Branch:** `cursor/lox-interpreter-phase-993f`

**How to verify:**

```bash
cargo build
cargo test
./target/debug/guppty examples/closure_make_adder.gup
```
