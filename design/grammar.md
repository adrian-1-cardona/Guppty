# Guppty Grammar

This describes the rules for how Guppty code is structured.
Think of it like the grammar rules for English, but for code!

## Program

A program is one or more statements, executed top to bottom.

```
program        → statement* EOF
```

## Statements

A statement is a complete instruction. It ends with an optional semicolon.

```
statement      → expression_stmt
expression_stmt → expression ";"?
```

## Expressions

An expression is something that produces a value.

```
expression     → function_call | string_literal
function_call  → IDENTIFIER "(" arguments? ")"
arguments      → expression ("," expression)*
string_literal → '"' <any characters> '"'
```

## Tokens

```
IDENTIFIER     → [a-zA-Z_][a-zA-Z0-9_]*
STRING         → '"' <any character except '"'>* '"'
SEMICOLON      → ";"
LEFT_PAREN     → "("
RIGHT_PAREN    → ")"
```

## Built-in Functions

| Function | What it does                | Example              |
|----------|-----------------------------|----------------------|
| `out`    | Prints text to the screen   | `out("Hello World")` |

## Notes

- Semicolons are **optional** at the end of statements
- Whitespace (spaces, tabs, newlines) is ignored between tokens
- Strings must be wrapped in double quotes `"`
