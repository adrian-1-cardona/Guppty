# Contributing to Guppty

Thanks for helping improve Guppty.

## Setup

Fastest path (checks for Rust and installs it only if needed):

```bash
curl -fsSL https://raw.githubusercontent.com/adrian-1-cardona/guppty/main/install.sh | bash
```

Or install Rust from <https://rustup.rs>, then from this repo:

```bash
cargo test
```

## Making your own programs

```bash
guppty new hello
guppty compile hello.gup
guppty hello.gup
```

## Development Notes

- Keep language syntax changes centered in `src/syntax.rs` when possible.
- Add or update `.gup` examples in `examples/` for user-visible behavior.
- Add matching expected output in `examples/expected/` when an example changes.
- Run `cargo test` before opening a pull request.
- Do not commit Cargo build output. `target/` is generated locally and ignored.

## Pull Requests

Small, focused pull requests are easiest to review. Include a short explanation
of what changed, why it changed, and which command you used to test it.

By contributing, you agree that your contribution will be licensed under the MIT
License.
