# Guppty Runtime Resource Limits & Execution Model

This document specifies the execution model and configurable resource limits in Guppty.

---

## 1. Motivation

Guppty programs may be run on user-authored scripts, property-based fuzzer inputs, or untrusted student submissions. Without resource bounds:
- Unbounded recursion causes host stack overflow or VM frame runaway.
- Unbounded loops (`while true`) hang the execution thread indefinitely.
- Massive range iterations (`range(1 through 1000000000)`) allocate gigabytes of memory or exhaust heap.
- Unbounded print loops exhaust stdout buffers and disk space.

Guppty implements checked runtime resource limits that stop pathological programs predictably with located `RuntimeError` diagnostics rather than crashing or hanging the host process.

---

## 2. Resource Limits

| Resource | Field | Default | Conservative (`--conservative`) | CLI Flag | Environment Variable |
|---|---|---|---|---|---|
| **Call Stack Depth** | `max_call_depth` | `1,000` | `256` | `--max-call-depth <N>` | `GUPPTY_MAX_CALL_DEPTH` |
| **Value Stack Size** | `max_stack_size` | `10,000` | `1,024` | `--max-stack-size <N>` | `GUPPTY_MAX_STACK_SIZE` |
| **Range Size** | `max_range_size` | `1,000,000` | `10,000` | `--max-range-size <N>` | `GUPPTY_MAX_RANGE_SIZE` |
| **Execution Steps** | `max_steps` | `None` (unlimited) | `100,000` | `--max-steps <N>` | `GUPPTY_MAX_STEPS` |
| **Output Volume** | `max_output_bytes` | `None` (unlimited) | `1,000,000` bytes | `--max-output-bytes <N>` | `GUPPTY_MAX_OUTPUT_BYTES` |

---

## 3. Execution Model & Dual-Backend Parity

Both the bytecode VM and the tree-walking interpreter enforce equivalent resource limits:

1. **Call Stack Depth**:
   - Checked on every function/closure invocation (`call_closure` in VM, `call_function` in interpreter).
   - If call depth reaches `max_call_depth`, a `RuntimeError` is raised:
     ```text
     RuntimeError: Maximum call stack depth of <N> exceeded.
     help: Check for runaway or infinite recursion, or increase the maximum call depth limit.
     ```

2. **Value Stack**:
   - VM checks that the operand stack length does not exceed `max_stack_size` on push.
   - Prevents memory runaway from deeply nested expression stacks:
     ```text
     RuntimeError: Value stack overflow: exceeded limit of <N> values.
     help: Simplify deep expressions or loops, or increase the maximum stack size limit.
     ```

3. **Streamed Range Iteration & Bounds**:
   - In the tree-walking interpreter, `for x in range(start through end)` streams iteration with a step counter, completely eliminating eager `Vec<Value>` materialization.
   - In the VM, range sizes are validated against `max_range_size` before array construction:
     ```text
     RuntimeError: Range size of <N> exceeds maximum allowed limit of <limit>.
     help: Narrow the range bounds or increase the maximum range size limit.
     ```

4. **Execution Steps**:
   - Incremented on every bytecode instruction in the VM and on every statement in the interpreter.
   - Guards against infinite loops and non-terminating executions:
     ```text
     RuntimeError: Execution step limit of <N> exceeded.
     help: Check for infinite loops or increase the execution step limit.
     ```

5. **Output Volume**:
   - Tracks cumulative bytes printed through `out(...)`.
   - Prevents stdout flooding:
     ```text
     RuntimeError: Output volume limit of <N> bytes exceeded.
     help: Reduce print volume or increase the output volume limit.
     ```

---

## 4. Presets

- **Conservative Mode (`--conservative` / `GUPPTY_CONSERVATIVE=1`)**: Enables tight execution step budgets and smaller stacks for sandboxed evaluation of untrusted code.
- **Unlimited Mode (`--unlimited`)**: Disables step and output limits and sets stack depths to `usize::MAX` for trusted, long-running research experiments.
