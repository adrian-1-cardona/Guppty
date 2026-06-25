# How to Run Guppty (Super Easy)

This guide shows you how to run:

```bash
guppty examples/program.gup
```

---

## Step 1: Make sure Rust is ready

Open your terminal and run:

```bash
source "$HOME/.cargo/env"
```

If that does nothing scary, you are good.

---

## Step 2: Go to the Guppty folder

```bash
cd /path/to/guppty
```

(Use the real folder where this project lives on your computer.)

---

## Step 3: Build Guppty

```bash
cargo build
```

Wait until it says `Finished`.

This makes the `guppty` program in `target/debug/guppty`.

---

## Step 4: Run the example program

**Option A — use the built program directly:**

```bash
./target/debug/guppty examples/program.gup
```

**Option B — install `guppty` so you can type it anywhere:**

```bash
cargo install --path .
```

Then:

```bash
guppty examples/program.gup
```

**Option C — run without installing (good for quick tests):**

```bash
cargo run -- examples/program.gup
```

---

## What you should see

```
Hi! I am program.gup
Guppty is working!
5
```

If you see that, it worked.

---

## Try other examples

```bash
guppty examples/hello.gup
guppty examples/math.gup
guppty examples/variables.gup
```

---

## If something goes wrong

| Problem | Fix |
|--------|-----|
| `command not found: guppty` | Run `cargo install --path .` or use `./target/debug/guppty` |
| `couldn't read the file` | Check you are in the project folder and typed the path right |
| `cargo: command not found` | Install Rust from https://rustup.rs |

---

## What happens inside? (the short version)

```
program.gup  →  lexer  →  parser  →  interpreter  →  output on screen
(your file)     (chop)    (organize)   (do the work)
```

That is all Guppty does. You write a `.gup` file, Guppty reads it, and runs it.
