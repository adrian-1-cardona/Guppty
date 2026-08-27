# Reproduce Guppty

Hey! This guide is here so you can clone Guppty, run the same checks, and know exactly what you
used. No guessing and no mystery setup :D

## What to record

Always record the release tag and commit. Run these before an experiment:

```bash
git describe --tags --always --dirty
rustc --version --verbose
cargo --version
```

Guppty 0.2.0 supports Rust 1.82 or newer. The reference checks below were also verified on
Apple Silicon macOS 26 with Rust 1.96.0. CI repeats them on Linux, macOS, and Windows.

## Build and check everything

```bash
git clone https://github.com/adrian-1-cardona/Guppty.git
cd Guppty
git checkout <the-release-tag-or-commit-you-are-reporting>
cargo build --release --locked
cargo test --all-targets --locked
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The build usually takes under a minute on a current laptop. The test suite prints one result for
unit, integration, project-workflow, and dual-backend checks. Every result should say `ok`.

## Run the language

```bash
cargo run --locked -- examples/program.gup
cargo run --locked -- examples/program.gup --interp
```

Both commands should print the same program output. The first uses the bytecode VM and the second
uses the tree-walking interpreter.

## Save experiment identity

Keep this next to every result file:

```text
Guppty version: <output from cargo run -- version>
Guppty commit: <output from git rev-parse HEAD>
Rust version: <output from rustc --version --verbose>
Operating system: <name and version>
Command: <the exact command you ran>
```

## If something gets grumpy

- `cargo ... --locked` fails: make sure `Cargo.lock` is present and you checked out the exact tag.
- Rust is too old: install Rust 1.82 or newer with rustup.
- VM and interpreter output differ: please open an issue with the source file, both outputs, and
  the commit SHA.
- A clean rebuild is needed: remove only the generated `target` folder, then build again.

## Limits and honest claims

Guppty is a young research language, not a production sandbox. Results from its included workloads
show how these two Guppty backends behave on the recorded machine; they do not prove that one
execution strategy is faster for every language or computer. Generated programs can also expose
cases that the hand-written examples do not cover yet. Pin the version, keep raw results, and state
those limits beside any published conclusion.
