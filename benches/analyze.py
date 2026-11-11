#!/usr/bin/env python3
"""
Guppty Benchmark Analyzer
Reads raw benchmark JSON and outputs Markdown summary table and metrics.
"""

import json
import sys
from pathlib import Path

def main():
    results_path = Path(__file__).parent / "results" / "raw_benchmark_results.json"
    if len(sys.argv) > 1:
        results_path = Path(sys.argv[1])

    if not results_path.exists():
        print(f"Error: {results_path} does not exist.", file=sys.stderr)
        sys.exit(1)

    with open(results_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    print(f"# Benchmark Summary (Guppty {data.get('guppty_version')})")
    print(f"- Commit: `{data.get('commit')}`")
    print(f"- Rustc:  `{data.get('rustc')}`")
    print(f"- Target: `{data.get('os')} ({data.get('arch')})`")
    print(f"- Samples: {data.get('samples')} (Warmups: {data.get('warmups')})\n")

    print("| Workload | VM Median (ms) | VM Mean ± Std | Interp Median (ms) | Interp Mean ± Std | Speedup (VM) |")
    print("|---|---|---|---|---|---|")

    for name, bench in data.get("benchmarks", {}).items():
        vm = bench.get("vm", {})
        interp = bench.get("interp", {})
        speedup = bench.get("speedup_vm", 1.0)

        vm_med = vm.get("median_ms", 0.0)
        vm_mean = vm.get("mean_ms", 0.0)
        vm_std = vm.get("stddev_ms", 0.0)

        interp_med = interp.get("median_ms", 0.0)
        interp_mean = interp.get("mean_ms", 0.0)
        interp_std = interp.get("stddev_ms", 0.0)

        print(f"| {name} | {vm_med:.2f} | {vm_mean:.2f} ± {vm_std:.2f} | {interp_med:.2f} | {interp_mean:.2f} ± {interp_std:.2f} | {speedup:.2f}x |")

if __name__ == "__main__":
    main()
