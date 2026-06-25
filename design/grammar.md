# Guppty Grammar

This grammar matches the interpreter in `src/`. Keywords live in `src/syntax.rs` so you can change them in one place.

## Program

```
program        → statement* EOF
```

## Statements

```
statement      → expression_stmt
               | variable_decl
               | function_def
               | if_stmt
               | while_stmt
               | for_stmt
               | return_stmt

expression_stmt→ expression (";" | NEWLINE)?

variable_decl  → IDENTIFIER "=" expression (";" | NEWLINE)?

function_def   → IDENTIFIER "(" parameters? ")" NEWLINE INDENT statement* DEDENT

if_stmt        → "if" expression NEWLINE block
                 ("else" NEWLINE block)?

while_stmt     → "while" expression NEWLINE block

for_stmt       → "for" IDENTIFIER "in" expression NEWLINE INDENT statement* DEDENT

return_stmt    → "return" expression? (";" | NEWLINE)?

block          → INDENT statement* DEDENT
               | statement
```

## Expressions (precedence low → high)

```
expression     → or_expr
or_expr        → and_expr ("or" and_expr)*
and_expr       → equality ("and" equality)*
equality       → comparison (("==" | "!=") comparison)*
comparison     → additive (("<" | ">" | "<=" | ">=") additive)*
additive       → multiplicative (("+" | "-") multiplicative)*
multiplicative → unary (("*" | "/") unary)*
unary          → ("-" | "not") unary | primary
primary        → literal
               | IDENTIFIER
               | IDENTIFIER "(" arguments? ")"
               | "range" "(" expression "through" expression ")"
               | "[" "]"
```

## Literals

```
literal        → STRING | CHAR | NUMBER | FLOAT | "true" | "false"
```

## Tokens

- **Keywords:** `for`, `in`, `range`, `through`, `if`, `else`, `while`, `return`, `and`, `or`, `not`, `true`, `false`
- **Operators:** `+`, `-`, `*`, `/`, `=`, `==`, `!=`, `<`, `>`, `<=`, `>=`
- **Structure:** indentation (`INDENT` / `DEDENT`), `//` line comments

## Built-in Functions

| Function | Description |
|----------|-------------|
| `out(...)` | Print values to stdout (space-separated) |

## Scoping

- Blocks (`if`, `while`, `for`, function bodies) create a new scope.
- Functions capture their defining environment (closures).
- `return` exits the current function with an optional value.

## Notes

- Indentation uses spaces or tabs (Python-style).
- `//` starts a line or inline comment.
- Function definitions require an indented body after `name(...)`.
