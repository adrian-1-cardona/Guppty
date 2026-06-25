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

Right now Guppty is a Rust interpreter. A C compiler may come later.

## Example files

| File | What it does |
|------|----------------|
| `examples/program.gup` | Start here — simple demo |
| `examples/hello.gup` | Prints "Hello World!" |
| `examples/math.gup` | Numbers and math |
| `examples/variables.gup` | Variables |
| `examples/for_loop.gup` | Loops |
| `examples/functions.gup` | Functions |
| `examples/comments.gup` | Comments |
| `examples/all_features.gup` | Everything together |

## Development

```bash
cargo build
cargo run -- examples/program.gup
```
