#!/usr/bin/env bash
# Backfill GitHub issues for work already merged into Guppty.
# Run via the "Backfill Solved Issues" workflow (workflow_dispatch).

set -euo pipefail

REPO="${GITHUB_REPOSITORY:-adrian-1-cardona/Guppty}"
ASSIGNEE="adrian-1-cardona"

create_solved_issue() {
  local title="$1"
  local body="$2"
  local branch="$3"
  local pr_number="$4"
  local labels="$5"

  # Skip if an issue with this exact title already exists (idempotent reruns).
  if gh issue list --repo "$REPO" --state all --search "in:title \"$title\"" --json title --jq '.[].title' 2>/dev/null | grep -Fxq "$title"; then
    echo "Skipping (already exists): $title"
    return 0
  fi

  local issue_url
  issue_url=$(gh issue create \
    --repo "$REPO" \
    --title "$title" \
    --body "$body" \
    --assignee "$ASSIGNEE" \
    --label "$labels")

  local issue_number
  issue_number=$(basename "$issue_url")

  gh issue comment "$issue_number" --repo "$REPO" --body "✅ **Solved!** Merged in PR #${pr_number} on branch \`${branch}\`. Nice work :D"
  gh issue close "$issue_number" --repo "$REPO" --reason completed

  echo "Created and closed #$issue_number — $title"
}

# ── PR #3 — template ──────────────────────────────────────────────────────────

create_solved_issue \
  "Set up Cargo.toml and the guppty binary config" \
  "**What I needed:** a real Rust project so Guppty is not just vibes in a folder.

**What I did:** added \`Cargo.toml\` with package metadata and a \`guppty\` binary entry point so \`cargo build\` actually gives us something to run.

**Solved on branch:** \`template\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/3" \
  "template" "3" "enhancement,solved"

create_solved_issue \
  "Scaffold the modular interpreter architecture (lexer → parser → interpreter)" \
  "**What I needed:** clean separation instead of one giant file.

**What I did:** set up \`src/lexer.rs\`, \`src/parser.rs\`, \`src/interpreter.rs\`, \`src/ast.rs\`, \`src/token.rs\`, and \`src/value.rs\` — the full pipeline from source code to output.

**Solved on branch:** \`template\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/3" \
  "template" "3" "enhancement,solved"

create_solved_issue \
  "Wire up main.rs CLI: read a .gup file and run the pipeline" \
  "**What I needed:** type \`guppty hello.gup\` and have it work.

**What I did:** \`main.rs\` reads the file from the command line, runs lexer → parser → interpreter, and prints helpful errors if you forget the filename.

**Solved on branch:** \`template\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/3" \
  "template" "3" "enhancement,solved"

create_solved_issue \
  "Add design docs for Guppty grammar and syntax" \
  "**What I needed:** write down how the language is supposed to look before the interpreter gets wild.

**What I did:** added \`design/grammar.md\` and \`design/syntax.md\` so we have a reference for tokens, statements, and example syntax.

**Solved on branch:** \`template\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/3" \
  "template" "3" "documentation,solved"

create_solved_issue \
  "Add the first hello.gup example program" \
  "**What I needed:** proof that Guppty can print something.

**What I did:** added \`examples/hello.gup\` with a simple \`out(\"Hello World\")\` so we have a smoke test from day one.

**Solved on branch:** \`template\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/3" \
  "template" "3" "enhancement,solved"

create_solved_issue \
  "Define AST node types for expressions and statements" \
  "**What I needed:** a tree structure the interpreter can walk through.

**What I did:** built out \`src/ast.rs\` with expression and statement enums so the parser has something real to produce.

**Solved on branch:** \`template\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/3" \
  "template" "3" "enhancement,solved"

# ── PR #4 — confighello ─────────────────────────────────────────────────────

create_solved_issue \
  "Implement the lexer — turn source code into tokens" \
  "**What I needed:** the interpreter to understand actual Guppty code, not just placeholders.

**What I did:** built a real lexer in \`src/lexer.rs\` that tokenizes identifiers, strings, parens, semicolons, and whitespace.

**Solved on branch:** \`confighello\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/4" \
  "confighello" "4" "enhancement,solved"

create_solved_issue \
  "Implement the parser — build an AST from tokens" \
  "**What I needed:** figure out what the tokens actually mean together.

**What I did:** \`src/parser.rs\` now parses function calls, string literals, and optional semicolons into a proper AST.

**Solved on branch:** \`confighello\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/4" \
  "confighello" "4" "enhancement,solved"

create_solved_issue \
  "Implement the interpreter and the out() built-in" \
  "**What I needed:** code that actually runs and prints to the terminal.

**What I did:** \`src/interpreter.rs\` walks the AST and handles \`out(...)\` so \`hello.gup\` prints Hello World for real.

**Solved on branch:** \`confighello\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/4" \
  "confighello" "4" "enhancement,solved"

create_solved_issue \
  "Add runtime Value types for interpreter results" \
  "**What I needed:** a way to represent data while the program runs.

**What I did:** expanded \`src/value.rs\` with string values and display logic so \`out()\` can print them cleanly.

**Solved on branch:** \`confighello\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/4" \
  "confighello" "4" "enhancement,solved"

create_solved_issue \
  "Expand token types for the full core language" \
  "**What I needed:** more than just strings and identifiers.

**What I did:** updated \`src/token.rs\` with all the token variants the lexer and parser need for function calls and literals.

**Solved on branch:** \`confighello\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/4" \
  "confighello" "4" "enhancement,solved"

create_solved_issue \
  "Write the full language grammar specification" \
  "**What I needed:** grammar rules that match what the interpreter actually does.

**What I did:** updated \`design/grammar.md\` with program structure, statements, expressions, tokens, and the \`out\` built-in docs.

**Solved on branch:** \`confighello\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/4" \
  "confighello" "4" "documentation,solved"

create_solved_issue \
  "Add Cargo.lock for reproducible builds" \
  "**What I needed:** everyone building Guppty gets the same dependency versions.

**What I did:** committed \`Cargo.lock\` so \`cargo build\` is consistent across machines.

**Solved on branch:** \`confighello\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/4" \
  "confighello" "4" "chore,solved"

# ── PR #5 — cursor/extend-interpreter-new-syntax-2440 ───────────────────────

create_solved_issue \
  "Support // line comments in the lexer" \
  "**What I needed:** comment my code without breaking the interpreter.

**What I did:** the lexer strips \`//\` comments (inline and full-line) and ignores them — just like in \`design/syntax.md\`.

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "enhancement,solved"

create_solved_issue \
  "Add indentation-based blocks (Python-style)" \
  "**What I needed:** for loops and functions with indented bodies, not just one-liners.

**What I did:** lexer tracks indent/dedent tokens and the parser builds block statements from them.

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "enhancement,solved"

create_solved_issue \
  "Implement for loops with range(1 through 6) syntax" \
  "**What I needed:** loops that match the syntax doc — \`for i in range(1 through 6)\`.

**What I did:** parser + interpreter handle the \`for\`/\`in\`/\`range\`/\`through\` syntax and iterate over the range. See \`examples/for_loop.gup\`.

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "enhancement,solved"

create_solved_issue \
  "Add variables — numbers, floats, bools, chars, strings, and []" \
  "**What I needed:** store data like \`x = 5\`, \`bool = true\`, \`z = \"hello\"\`, \`f = 1.25\`, and empty arrays.

**What I did:** variable declarations, a runtime environment, and new Value types. Check \`examples/variables.gup\`.

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "enhancement,solved"

create_solved_issue \
  "Implement math operators: +, -, *, /" \
  "**What I needed:** do actual math like \`out(x + y)\` and get 11.

**What I did:** binary expression parsing and evaluation for all four operators, with number and float support. See \`examples/math.gup\`.

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "enhancement,solved"

create_solved_issue \
  "Add user-defined functions with indented blocks" \
  "**What I needed:** define \`math()\` and call it from \`main()\` like in the syntax doc.

**What I did:** function definitions via \`name()\` + indented block, two-pass registration + execution, and function calls. See \`examples/functions.gup\`.

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "enhancement,solved"

create_solved_issue \
  "Add example programs for every new language feature" \
  "**What I needed:** quick demos so I can test each feature without writing code from scratch every time.

**What I did:** added \`examples/for_loop.gup\`, \`variables.gup\`, \`math.gup\`, \`comments.gup\`, \`functions.gup\`, and \`all_features.gup\`.

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "documentation,solved"

create_solved_issue \
  "Stop tracking Rust target/ build artifacts in git" \
  "**What I needed:** git to stop being flooded with compiled .o files nobody wants to read.

**What I did:** added \`.gitignore\` for \`target/\` and removed the accidentally committed build artifacts. Much cleaner repo now :D

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "chore,solved"

create_solved_issue \
  "updated the syntax — expand design/syntax.md with full language examples" \
  "**What I needed:** the syntax doc to show loops, variables, functions, and comments — not just hello world.

**What I did:** updated \`design/syntax.md\` with for loops, function examples, variable types, and comment syntax so the interpreter has a spec to match.

**Solved on branch:** \`cursor/extend-interpreter-new-syntax-2440\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/5" \
  "cursor/extend-interpreter-new-syntax-2440" "5" "documentation,solved"

# ── PR #6 — cursor/guppty-examples-program-55f6 ────────────────────────────

create_solved_issue \
  "Add examples/program.gup — the main demo program" \
  "**What I needed:** one simple file that shows Guppty working end-to-end.

**What I did:** created \`examples/program.gup\` — prints a greeting, a status message, and does \`2 + 3\` (outputs 5). Perfect starter demo :D

**Solved on branch:** \`cursor/guppty-examples-program-55f6\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/6" \
  "cursor/guppty-examples-program-55f6" "6" "enhancement,solved"

create_solved_issue \
  "update readme so it has cargo build instructions" \
  "**What I needed:** someone cloning the repo to know exactly how to build and run Guppty.

**What I did:** rewrote \`README.md\` with quick start steps — \`cargo build\`, run \`./target/debug/guppty examples/program.gup\`, expected output, and a table of all example files.

**Solved on branch:** \`cursor/guppty-examples-program-55f6\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/6" \
  "cursor/guppty-examples-program-55f6" "6" "documentation,solved"

create_solved_issue \
  "Add HOW_TO_RUN.md with step-by-step guide and troubleshooting" \
  "**What I needed:** super easy instructions for people who have never touched Rust before.

**What I did:** wrote \`HOW_TO_RUN.md\` — Rust setup, build steps, how to run every example, \`cargo install --path .\`, and a troubleshooting section.

**Solved on branch:** \`cursor/guppty-examples-program-55f6\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/6" \
  "cursor/guppty-examples-program-55f6" "6" "documentation,solved"

create_solved_issue \
  "Document cargo install --path . for global guppty command" \
  "**What I needed:** type \`guppty\` from anywhere instead of \`./target/debug/guppty\` every time.

**What I did:** added install instructions to README and HOW_TO_RUN so you can run \`cargo install --path .\` and use \`guppty\` globally.

**Solved on branch:** \`cursor/guppty-examples-program-55f6\`
**PR:** https://github.com/adrian-1-cardona/Guppty/pull/6" \
  "cursor/guppty-examples-program-55f6" "6" "documentation,solved"

echo ""
echo "Done! All solved issues backfilled and assigned to @${ASSIGNEE} :D"
