// === debugger_bench.rs ===
// Benchmark runner comparing:
//   1. Standard VM (fast loop, zero debug overhead)
//   2. Debugger VM (passive tracing / continue mode)
//   3. Debugger VM with active breakpoints (measuring breakpoint checks)
//   4. Tree-walking interpreter (--interp)

use std::fs;
use std::process::Command;
use std::time::Instant;

struct BenchmarkResult {
    name: &'static str,
    vm_mean_us: f64,
    debug_passive_mean_us: f64,
    debug_active_mean_us: f64,
    interp_mean_us: f64,
}

fn measure_command(bin: &str, args: &[&str], runs: usize) -> f64 {
    // Warmup
    for _ in 0..3 {
        let _ = Command::new(bin).args(args).output();
    }

    let mut times = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        let status = Command::new(bin)
            .args(args)
            .output()
            .expect("run command failed");
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0; // microseconds
        assert!(status.status.success());
        times.push(elapsed);
    }

    times.iter().sum::<f64>() / times.len() as f64
}

#[test]
fn run_debugger_benchmarks() {
    let bin = env!("CARGO_BIN_EXE_guppty");
    let runs = 25;

    // Create a compute-heavy recursion workload
    let temp_dir = std::env::temp_dir();
    let fib_deep_path = temp_dir.join("bench_fib_deep.gup");
    fs::write(
        &fib_deep_path,
        "fib(n)\n    if n <= 1\n        return n\n    return fib(n - 1) + fib(n - 2)\n\nout(fib(18))\n",
    )
    .unwrap();

    // Create a compute-heavy loop workload
    let loop_heavy_path = temp_dir.join("bench_loop_heavy.gup");
    fs::write(
        &loop_heavy_path,
        "sum = 0\ni = 1\nwhile i <= 5000\n    sum = sum + i\n    i = i + 1\nout(sum)\n",
    )
    .unwrap();

    let workloads: &[(&'static str, String)] = &[
        ("Deep Recursion (fib 18)", fib_deep_path.to_str().unwrap().to_string()),
        ("Tight Loop (5k iters)", loop_heavy_path.to_str().unwrap().to_string()),
        ("Recursion (fib 6)", "examples/recursion_fib.gup".to_string()),
        ("Loop (while_sum)", "examples/while_sum.gup".to_string()),
        ("Closures (counter)", "examples/closure_counter.gup".to_string()),
        ("Strings (concat)", "examples/string_concat.gup".to_string()),
    ];

    println!("\n=========================================================================================");
    println!("GUPPTY EXECUTION & DEBUGGER BENCHMARK REPORT (N = {} runs per test)", runs);
    println!("=========================================================================================");

    let mut results = Vec::new();

    for (name, path) in workloads {
        let vm_mean = measure_command(bin, &["run", path], runs);
        let dbg_pass_mean = measure_command(bin, &["debug", path, "--commands", "c"], runs);
        let dbg_act_mean = measure_command(bin, &["debug", path, "--commands", "b 999; c"], runs);
        let interp_mean = measure_command(bin, &["run", path, "--interp"], runs);

        results.push(BenchmarkResult {
            name,
            vm_mean_us: vm_mean,
            debug_passive_mean_us: dbg_pass_mean,
            debug_active_mean_us: dbg_act_mean,
            interp_mean_us: interp_mean,
        });
    }

    println!("{:<26} | {:>12} | {:>14} | {:>14} | {:>12}",
        "Workload", "Standard VM", "Debug (Pass)", "Debug (Active)", "Interpreter");
    println!("---------------------------+--------------+----------------+----------------+-------------");

    for r in &results {
        println!("{:<26} | {:>9.2} µs | {:>11.2} µs | {:>11.2} µs | {:>9.2} µs",
            r.name,
            r.vm_mean_us,
            r.debug_passive_mean_us,
            r.debug_active_mean_us,
            r.interp_mean_us
        );
    }

    println!("=========================================================================================\n");

    for r in &results {
        let overhead_pass = ((r.debug_passive_mean_us - r.vm_mean_us) / r.vm_mean_us) * 100.0;
        let overhead_act = ((r.debug_active_mean_us - r.vm_mean_us) / r.vm_mean_us) * 100.0;
        let speedup_vs_interp = ((r.interp_mean_us - r.vm_mean_us) / r.interp_mean_us) * 100.0;
        println!(
            "{:<26}: Debug Pass Overhead: {:+.2}%, Active BP Check: {:+.2}%, VM vs Interp: {:+.2}%",
            r.name, overhead_pass, overhead_act, speedup_vs_interp
        );
    }

    let _ = fs::remove_file(fib_deep_path);
    let _ = fs::remove_file(loop_heavy_path);
}
