# Guppty

A small programming language you can run from the terminal.

## Quick start — run an example

```bash
# 1. Set up Rust (once)
source "$HOME/.cargo/env"

# 2. Build
cargo build

# 3. Run the example program
./target/debug/guppty examples/program.gup
```

You should see:

```
Hi! I am program.gup
Guppty is working!
5
```

## Install `guppty` (RECOMMDED AND EASIER ) at least I think it is for now 


After this, you can type `guppty` from anywhere:

```bash
cargo install --path .
guppty examples/program.gup
```

## More help

See **[HOW_TO_RUN.md](HOW_TO_RUN.md)** for step-by-step instructions, other examples, and troubleshooting.

## How it works

```
hello.gup → lexer → parser → interpreter → output
```

Guppty now supports a **Lox-level interpreter phase**: variables & scopes, functions & closures, control flow (`if`/`else`, `while`, `for`), numbers/strings/booleans, comparisons, and logical operators. Keywords are configurable in `src/syntax.rs`.

Right now Guppty is a Rust interpreter. A C compiler may come later.

## Example files (30 programs)

| File | What it does |
|------|----------------|
| `examples/program.gup` | Start here — simple demo |
| `examples/hello.gup` | Prints "Hello World!" |
| `examples/math.gup` | Numbers and math |
| `examples/variables.gup` | Variables and types |
| `examples/for_loop.gup` | For loops with `range` |
| `examples/while_countdown.gup` | While loops |
| `examples/if_else.gup` | If / else branches |
| `examples/comparisons.gup` | `==`, `!=`, `<`, `>`, etc. |
| `examples/booleans_logic.gup` | `and`, `or`, `not` |
| `examples/function_params.gup` | Functions with parameters |
| `examples/function_return.gup` | Return values |
| `examples/recursion_factorial.gup` | Recursive factorial |
| `examples/recursion_fib.gup` | Recursive fibonacci |
| `examples/closure_make_adder.gup` | Closure basics |
| `examples/closure_counter.gup` | Mutable closure state |
| `examples/closure_greeter.gup` | Personalized greeter |
| `examples/string_concat.gup` | String `+` |
| `examples/scope_block.gup` | Block scope |
| `examples/nested_if.gup` | Nested conditionals |
| `examples/all_features.gup` | Everything together |

See `examples/` for all 30 programs. Expected output lives in `examples/expected/`.

## Development

```bash
cargo build
cargo test          # runs all 30 example programs against expected output
cargo run -- examples/program.gup
```
