// =============================================================================
// REPRODUCIBLE BENCHMARK RUNNER FOR GUPPTY (VM vs INTERPRETER)
// =============================================================================
// Measures execution timings across canonical workloads, gathers system metadata,
// calculates distributions (mean, median, min, max, stddev), and generates
// machine-readable JSON datasets and Markdown summary reports.
// =============================================================================

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

struct BenchmarkConfig {
    samples: usize,
    warmups: usize,
    output_path: PathBuf,
}

struct MetricStats {
    min_ms: f64,
    max_ms: f64,
    mean_ms: f64,
    median_ms: f64,
    stddev_ms: f64,
    raw_samples_ms: Vec<f64>,
}

fn calculate_stats(mut samples: Vec<f64>) -> MetricStats {
    if samples.is_empty() {
        return MetricStats {
            min_ms: 0.0,
            max_ms: 0.0,
            mean_ms: 0.0,
            median_ms: 0.0,
            stddev_ms: 0.0,
            raw_samples_ms: Vec::new(),
        };
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min_ms = samples[0];
    let max_ms = samples[samples.len() - 1];
    let sum: f64 = samples.iter().sum();
    let mean_ms = sum / (samples.len() as f64);
    let median_ms = if samples.len() % 2 == 1 {
        samples[samples.len() / 2]
    } else {
        (samples[samples.len() / 2 - 1] + samples[samples.len() / 2]) / 2.0
    };
    let variance: f64 =
        samples.iter().map(|v| (v - mean_ms).powi(2)).sum::<f64>() / (samples.len() as f64);
    let stddev_ms = variance.sqrt();

    MetricStats {
        min_ms,
        max_ms,
        mean_ms,
        median_ms,
        stddev_ms,
        raw_samples_ms: samples,
    }
}

fn get_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn find_guppty_bin() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_guppty") {
        return PathBuf::from(path);
    }
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            let candidate = dir.join(if cfg!(windows) {
                "guppty.exe"
            } else {
                "guppty"
            });
            if candidate.exists() {
                return candidate;
            }
            if let Some(parent) = dir.parent() {
                let candidate2 = parent.join(if cfg!(windows) {
                    "guppty.exe"
                } else {
                    "guppty"
                });
                if candidate2.exists() {
                    return candidate2;
                }
            }
        }
    }
    let target_debug = PathBuf::from("target/debug").join(if cfg!(windows) {
        "guppty.exe"
    } else {
        "guppty"
    });
    if target_debug.exists() {
        return target_debug;
    }
    let target_release = PathBuf::from("target/release").join(if cfg!(windows) {
        "guppty.exe"
    } else {
        "guppty"
    });
    if target_release.exists() {
        return target_release;
    }
    PathBuf::from("guppty")
}

fn run_guppty(bin: &Path, file: &Path, is_interp: bool) -> (f64, bool) {
    let mut cmd = Command::new(bin);
    cmd.arg(file);
    if is_interp {
        cmd.arg("--interp");
    }
    let start = Instant::now();
    let output = cmd.output().expect("execute guppty command");
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    (elapsed, output.status.success())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut config = BenchmarkConfig {
        samples: 5,
        warmups: 2,
        output_path: PathBuf::from("benches/results/raw_benchmark_results.json"),
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" | "--dry-run" => {
                config.samples = 1;
                config.warmups = 0;
            }
            "--samples" if i + 1 < args.len() => {
                i += 1;
                config.samples = args[i].parse().unwrap_or(5);
            }
            "--warmups" if i + 1 < args.len() => {
                i += 1;
                config.warmups = args[i].parse().unwrap_or(2);
            }
            "--output" if i + 1 < args.len() => {
                i += 1;
                config.output_path = PathBuf::from(&args[i]);
            }
            _ => {}
        }
        i += 1;
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = find_guppty_bin();
    let workloads_dir = manifest_dir.join("benches/workloads");

    let workloads = ["recursion", "loops", "closures", "strings", "synthetic"];
    let commit = get_commit_hash();
    let rustc = get_rustc_version();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    println!("=== Guppty Reproducible Benchmark Harness ===");
    println!("Binary:   {}", bin.display());
    println!("Commit:   {commit}");
    println!("Rustc:    {rustc}");
    println!("Host:     {os} ({arch})");
    println!("Samples:  {}", config.samples);
    println!("Warmups:  {}", config.warmups);
    println!();

    let mut results_json = String::new();
    results_json.push_str("{\n");
    results_json.push_str("  \"guppty_version\": \"0.2.0\",\n");
    results_json.push_str(&format!("  \"commit\": \"{commit}\",\n"));
    results_json.push_str(&format!("  \"rustc\": \"{rustc}\",\n"));
    results_json.push_str(&format!("  \"os\": \"{os}\",\n"));
    results_json.push_str(&format!("  \"arch\": \"{arch}\",\n"));
    results_json.push_str(&format!("  \"samples\": {},\n", config.samples));
    results_json.push_str(&format!("  \"warmups\": {},\n", config.warmups));
    results_json.push_str("  \"benchmarks\": {\n");

    let mut summary_table = String::new();
    summary_table.push_str("| Workload | VM Median (ms) | VM Mean ± Std (ms) | Interp Median (ms) | Interp Mean ± Std (ms) | Speedup (VM) |\n");
    summary_table.push_str("|---|---|---|---|---|---|\n");

    for (w_idx, name) in workloads.iter().enumerate() {
        let file = workloads_dir.join(format!("{name}.gup"));
        if !file.exists() {
            eprintln!("Warning: workload file not found: {}", file.display());
            continue;
        }

        // Benchmark VM
        for _ in 0..config.warmups {
            run_guppty(&bin, &file, false);
        }
        let mut vm_samples = Vec::with_capacity(config.samples);
        for _ in 0..config.samples {
            let (elapsed, success) = run_guppty(&bin, &file, false);
            assert!(success, "VM execution failed on {name}");
            vm_samples.push(elapsed);
        }
        let vm_stats = calculate_stats(vm_samples);

        // Benchmark Interpreter
        for _ in 0..config.warmups {
            run_guppty(&bin, &file, true);
        }
        let mut interp_samples = Vec::with_capacity(config.samples);
        for _ in 0..config.samples {
            let (elapsed, success) = run_guppty(&bin, &file, true);
            assert!(success, "Interpreter execution failed on {name}");
            interp_samples.push(elapsed);
        }
        let interp_stats = calculate_stats(interp_samples);

        let speedup = if vm_stats.median_ms > 0.0 {
            interp_stats.median_ms / vm_stats.median_ms
        } else {
            1.0
        };

        summary_table.push_str(&format!(
            "| {name} | {:.2} | {:.2} ± {:.2} | {:.2} | {:.2} ± {:.2} | {:.2}x |\n",
            vm_stats.median_ms,
            vm_stats.mean_ms,
            vm_stats.stddev_ms,
            interp_stats.median_ms,
            interp_stats.mean_ms,
            interp_stats.stddev_ms,
            speedup
        ));

        let comma = if w_idx + 1 < workloads.len() { "," } else { "" };
        results_json.push_str(&format!("    \"{name}\": {{\n"));
        results_json.push_str("      \"vm\": {\n");
        results_json.push_str(&format!(
            "        \"median_ms\": {:.4},\n",
            vm_stats.median_ms
        ));
        results_json.push_str(&format!("        \"mean_ms\": {:.4},\n", vm_stats.mean_ms));
        results_json.push_str(&format!("        \"min_ms\": {:.4},\n", vm_stats.min_ms));
        results_json.push_str(&format!("        \"max_ms\": {:.4},\n", vm_stats.max_ms));
        results_json.push_str(&format!(
            "        \"stddev_ms\": {:.4},\n",
            vm_stats.stddev_ms
        ));
        results_json.push_str(&format!(
            "        \"samples_ms\": {:?}\n",
            vm_stats.raw_samples_ms
        ));
        results_json.push_str("      },\n");
        results_json.push_str("      \"interp\": {\n");
        results_json.push_str(&format!(
            "        \"median_ms\": {:.4},\n",
            interp_stats.median_ms
        ));
        results_json.push_str(&format!(
            "        \"mean_ms\": {:.4},\n",
            interp_stats.mean_ms
        ));
        results_json.push_str(&format!(
            "        \"min_ms\": {:.4},\n",
            interp_stats.min_ms
        ));
        results_json.push_str(&format!(
            "        \"max_ms\": {:.4},\n",
            interp_stats.max_ms
        ));
        results_json.push_str(&format!(
            "        \"stddev_ms\": {:.4},\n",
            interp_stats.stddev_ms
        ));
        results_json.push_str(&format!(
            "        \"samples_ms\": {:?}\n",
            interp_stats.raw_samples_ms
        ));
        results_json.push_str("      },\n");
        results_json.push_str(&format!("      \"speedup_vm\": {:.4}\n", speedup));
        results_json.push_str(&format!("    }}{comma}\n"));
    }
    results_json.push_str("  }\n");
    results_json.push_str("}\n");

    println!("{summary_table}");

    if let Some(parent) = config.output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&config.output_path, &results_json).expect("write results json");
    println!(
        "Raw machine-readable dataset written to: {}",
        config.output_path.display()
    );

    let summary_path = config.output_path.with_file_name("summary.md");
    let summary_content = format!(
        "# Benchmark Results (0.2.0)\n\n\
         - Commit: `{commit}`\n\
         - Toolchain: `{rustc}`\n\
         - Target: `{os} ({arch})`\n\
         - Samples: `{}` (Warmups: `{}`)\n\n\
         {}\n",
        config.samples, config.warmups, summary_table
    );
    fs::write(&summary_path, summary_content).expect("write summary.md");
    println!("Markdown summary written to: {}", summary_path.display());
}
