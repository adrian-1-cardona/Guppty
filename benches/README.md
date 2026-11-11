# Guppty Reproducible Benchmark Harness

This directory contains the committed benchmark runner, workloads, raw data provenance, and analysis scripts for Guppty.

## Quick Start

To run the full reproducible benchmark suite:

```bash
cargo bench --bench benchmark_suite
```

To run a fast smoke verification (used in CI):

```bash
cargo bench --bench benchmark_suite -- --quick
```

Options:
- `--samples <N>`: number of samples per workload (default: 5).
- `--warmups <N>`: number of warmup runs per workload (default: 2).
- `--output <path>`: destination path for raw JSON dataset (default: `benches/results/raw_benchmark_results.json`).

## Workloads

Located in `benches/workloads/`:
1. `recursion.gup`: Recursive Fibonacci calculation (`fib(18)`), measuring function call and frame push/pop overhead.
2. `loops.gup`: Iterative counter loop, measuring basic block iteration and integer arithmetic.
3. `closures.gup`: Factory closures with captured upvalues, measuring closure instantiation and heap-allocated upvalue read/writes.
4. `strings.gup`: String concatenation loop, measuring string allocation and manipulation.
5. `synthetic.gup`: Mixed control flow, branching, arrays, and function dispatch.

## Raw Data and Analysis

The runner records raw machine-readable JSON metrics and metadata under `benches/results/raw_benchmark_results.json`:
- Compiler commit SHA and Guppty version
- Host OS, architecture, and `rustc` toolchain version
- Sample count and warmup policy
- Per-sample raw execution times (ms)
- Distribution metrics: min, max, mean, median, standard deviation, and relative speedup.

To re-generate summary tables from the raw JSON:

```bash
python3 benches/analyze.py
```

## Supported and Unsupported Claims

### What the Benchmarks Support
- Direct comparison between Guppty's default bytecode VM and its tree-walking reference interpreter (`--interp`) on identical workloads and hardware.
- Distribution measurements with uncertainty bounds (variance and standard deviation), showing sample-to-sample variability.
- Verifiable provenance: every dataset points to an exact Git commit and hardware environment.

### What the Benchmarks Do NOT Support
- Claims of absolute performance compared to mature JIT-compiled runtimes (such as V8 or PyPy) or ahead-of-time compiled languages (C, Rust).
- Cross-hardware generalization: results measured on one CPU or operating system reflect that architecture and power mode, not universal performance.
