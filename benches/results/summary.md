# Benchmark Results (0.2.0)

- Commit: `47698195e92181d5f0db36538180df5e05c14ff5`
- Toolchain: `rustc 1.96.0 (ac68faa20 2026-05-25)`
- Target: `macos (aarch64)`
- Samples: `1` (Warmups: `0`)

| Workload | VM Median (ms) | VM Mean ± Std (ms) | Interp Median (ms) | Interp Mean ± Std (ms) | Speedup (VM) |
|---|---|---|---|---|---|
| recursion | 216.81 | 216.81 ± 0.00 | 15.01 | 15.01 ± 0.00 | 0.07x |
| loops | 8.73 | 8.73 ± 0.00 | 7.72 | 7.72 ± 0.00 | 0.89x |
| closures | 5.92 | 5.92 ± 0.00 | 7.21 | 7.21 ± 0.00 | 1.22x |
| strings | 3.70 | 3.70 ± 0.00 | 3.85 | 3.85 ± 0.00 | 1.04x |
| synthetic | 4.60 | 4.60 ± 0.00 | 6.16 | 6.16 ± 0.00 | 1.34x |

