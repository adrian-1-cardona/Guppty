# Property-Based and Differential Fuzz Testing

This document details the property-based and differential fuzz testing infrastructure in Guppty.

## Motivation and Approach

Guppty maintains two independent execution backends:
1. The stack-based bytecode Virtual Machine (`vm.rs`)
2. The tree-walking reference interpreter (`interpreter.rs`)

The dual-backend architecture enables **differential testing**: the tree-walking interpreter serves as a reference semantic model against which the bytecode compiler and VM are validated.

## Fuzz Targets and Properties

Located in `tests/fuzz_differential.rs`:

1. **Arbitrary UTF-8 and Noise Property Test (`lexer_and_parser_never_panic_on_arbitrary_utf8`)**:
   - Generates random byte sequences, unclosed string quotes, arbitrary symbols, and invalid indentation.
   - Asserts that neither the lexer nor parser ever panics or crashes. Any invalid input must cleanly produce a located `GupError`.

2. **Grammar-Aware Differential Fuzzing (`differential_fuzz_valid_programs_match`)**:
   - Programmatically synthesizes valid Guppty programs spanning arithmetic expressions, conditionals, bounded while loops, function definitions, parameters, and closures.
   - Executes each program on both the VM and the interpreter.
   - Asserts exact equality on standard output, error streams, and process exit codes.

3. **Structured Mutation Fuzzing (`structured_mutations_never_panic`)**:
   - Takes syntactically valid programs and applies structured perturbations (token deletions, indentation corruption, truncated statements, invalid operator sequences).
   - Verifies that both compiler and runtime reject malformed variants without panics.

## Reproduction and CI Configuration

The fuzzer is fully deterministic and accepts environment variables:

- **Reproducing a failure from a seed**:
  ```bash
  GUPPTY_FUZZ_SEED=<seed> cargo test --test fuzz_differential
  ```
  When a failure occurs, the exact seed is printed and the failing source is saved to:
  `target/fuzz_artifacts/diff_mismatch_seed_<seed>.gup`.

- **Running extended fuzzing campaigns in CI or locally**:
  ```bash
  GUPPTY_FUZZ_ITERS=500 cargo test --test fuzz_differential
  ```
