// =============================================================================
// PROPERTY-BASED AND DIFFERENTIAL FUZZ TESTING FOR GUPPTY BACKENDS
// =============================================================================
// This test suite verifies:
// 1. Lexer & parser never panic on arbitrary UTF-8 or malformed source.
// 2. Grammar-aware generated programs produce identical results on both
//    the VM backend and the tree-walking interpreter backend.
// 3. Structured mutations of programs never crash or panic either backend.
// 4. Any discovered mismatch or crash is recorded with its seed and saved as
//    an artifact for 1-command reproduction.
// =============================================================================

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Deterministic, zero-dependency 64-bit pseudo-random generator.
struct FuzzRng {
    state: u64,
}

impl FuzzRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xdead_beef_cafe_babe
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            (self.next_u64() as usize) % max
        }
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    fn next_i64(&mut self, min: i64, max: i64) -> i64 {
        if max <= min {
            min
        } else {
            let range = (max - min) as u64;
            min + (self.next_u64() % range) as i64
        }
    }

    fn choose<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        &slice[self.next_usize(slice.len())]
    }
}

fn get_seed() -> u64 {
    if let Ok(val) = std::env::var("GUPPTY_FUZZ_SEED") {
        val.parse::<u64>()
            .expect("GUPPTY_FUZZ_SEED must be an integer")
    } else {
        42424242
    }
}

fn get_iterations(default: usize) -> usize {
    if let Ok(val) = std::env::var("GUPPTY_FUZZ_ITERS") {
        val.parse::<usize>()
            .expect("GUPPTY_FUZZ_ITERS must be an integer")
    } else {
        default
    }
}

fn run_guppty(args: &[&str]) -> (String, String, i32) {
    let bin = env!("CARGO_BIN_EXE_guppty");
    let output = Command::new(bin)
        .args(args)
        .output()
        .expect("run guppty process");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

fn save_artifact(seed: u64, name: &str, content: &str) -> PathBuf {
    let artifact_dir = PathBuf::from("target").join("fuzz_artifacts");
    let _ = fs::create_dir_all(&artifact_dir);
    let file_path = artifact_dir.join(format!("{name}_seed_{seed}.gup"));
    let _ = fs::write(&file_path, content);
    file_path
}

// -------------------------------------------------------------------------
// 1. PROPERTY TEST: Parser & Lexer never panic on arbitrary UTF-8 noise
// -------------------------------------------------------------------------
#[test]
fn lexer_and_parser_never_panic_on_arbitrary_utf8() {
    let seed = get_seed();
    let iters = get_iterations(60);
    let mut rng = FuzzRng::new(seed);

    let alphabet = [
        'a', 'b', 'c', '1', '2', '3', ' ', '\t', '\n', '(', ')', '[', ']', '{', '}', '"', '\'',
        '+', '-', '*', '/', '=', '!', '<', '>', '.', ':', '_', '#', '$', '%', '&', '|', '~', '^',
        '\0', '🦀', 'ñ', '語',
    ];

    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("guppty-fuzz-utf8-{}.gup", std::process::id()));

    for i in 0..iters {
        let len = rng.next_usize(200);
        let mut text = String::with_capacity(len);
        for _ in 0..len {
            text.push(*rng.choose(&alphabet));
        }

        fs::write(&temp_path, &text).expect("write temp fuzz file");

        let (_, stderr, _) = run_guppty(&["check", temp_path.to_str().unwrap()]);
        assert!(
            !stderr.contains("panicked at"),
            "Compiler panicked on arbitrary UTF-8 input (iteration {i}, seed {seed})!\nInput was:\n{text}\nStderr:\n{stderr}"
        );
    }

    let _ = fs::remove_file(temp_path);
}

// -------------------------------------------------------------------------
// 2. DIFFERENTIAL FUZZ: Valid grammar-generated programs match on both backends
// -------------------------------------------------------------------------
struct ProgramGenerator<'a> {
    rng: &'a mut FuzzRng,
}

impl<'a> ProgramGenerator<'a> {
    fn new(rng: &'a mut FuzzRng) -> Self {
        Self { rng }
    }

    fn gen_arithmetic_expr(&mut self, vars: &[&str], depth: usize) -> String {
        if depth >= 2 || self.rng.next_bool() {
            if !vars.is_empty() && self.rng.next_bool() {
                (*self.rng.choose(vars)).to_string()
            } else {
                self.rng.next_i64(-20, 20).to_string()
            }
        } else {
            let left = self.gen_arithmetic_expr(vars, depth + 1);
            let right = self.gen_arithmetic_expr(vars, depth + 1);
            let op = *self.rng.choose(&["+", "-", "*"]);
            format!("({left} {op} {right})")
        }
    }

    fn generate_program(&mut self) -> String {
        let mut lines = Vec::new();
        let var_names = ["a", "b", "c"];

        // Initialize variables with arithmetic
        lines.push(format!("a = {}", self.rng.next_i64(1, 10)));
        lines.push(format!("b = {}", self.rng.next_i64(1, 5)));
        lines.push(format!("c = {}", self.rng.next_i64(0, 3)));

        // Output initial state
        lines.push("out(a, b, c)".to_string());

        // Arithmetic operations
        let expr1 = self.gen_arithmetic_expr(&var_names, 0);
        lines.push(format!("a = {expr1}"));
        lines.push("out(a)".to_string());

        // Conditionals (4-space indent)
        let comp_op = *self.rng.choose(&["<", ">", "==", "!="]);
        lines.push(format!("if a {comp_op} b"));
        lines.push("    c = c + 1".to_string());
        lines.push("    out(c)".to_string());
        lines.push("else".to_string());
        lines.push("    c = c + 2".to_string());
        lines.push("    out(c)".to_string());

        // While loop with bounded iteration
        lines.push("k = 0".to_string());
        let loop_limit = self.rng.next_usize(5) + 1;
        lines.push(format!("while k < {loop_limit}"));
        lines.push("    k = k + 1".to_string());
        lines.push("    out(k)".to_string());

        // Function definition and invocation
        lines.push("compute(x, y)".to_string());
        lines.push("    return (x * 2) + y".to_string());
        lines.push("res = compute(a, b)".to_string());
        lines.push("out(res)".to_string());

        // Nested function / closure (4-space and 8-space indent)
        lines.push("makeMultiplier(factor)".to_string());
        lines.push("    mult(val)".to_string());
        lines.push("        return val * factor".to_string());
        lines.push("    return mult".to_string());
        lines.push("timesThree = makeMultiplier(3)".to_string());
        lines.push("out(timesThree(4))".to_string());

        lines.join("\n") + "\n"
    }
}

#[test]
fn differential_fuzz_valid_programs_match() {
    let start_time = Instant::now();
    let seed = get_seed();
    let iters = get_iterations(40);
    let mut rng = FuzzRng::new(seed);

    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("guppty-diff-fuzz-{}.gup", std::process::id()));

    for i in 0..iters {
        let mut gen = ProgramGenerator::new(&mut rng);
        let program = gen.generate_program();

        fs::write(&temp_path, &program).expect("write temp program");

        let file_arg = temp_path.to_str().unwrap();

        // Run default (VM)
        let (vm_stdout, vm_stderr, vm_code) = run_guppty(&[file_arg]);

        // Run Interpreter (--interp)
        let (interp_stdout, interp_stderr, interp_code) = run_guppty(&[file_arg, "--interp"]);

        if vm_code != interp_code || vm_stdout != interp_stdout {
            let artifact = save_artifact(seed + i as u64, "diff_mismatch", &program);
            panic!(
                "Differential mismatch on iteration {i} (seed {})!\nArtifact saved at: {}\n\
                 VM exit: {vm_code}, Interp exit: {interp_code}\n\
                 --- VM stdout ---\n{vm_stdout}\n--- Interp stdout ---\n{interp_stdout}\n\
                 --- VM stderr ---\n{vm_stderr}\n--- Interp stderr ---\n{interp_stderr}\n\
                 Reproduce with: GUPPTY_FUZZ_SEED={} cargo test --test fuzz_differential",
                seed + i as u64,
                artifact.display(),
                seed + i as u64
            );
        }

        assert_eq!(
            vm_code, 0,
            "Generated program returned non-zero exit code: {vm_stderr}"
        );
    }

    let _ = fs::remove_file(temp_path);
    println!(
        "Differential fuzz passed {} programs in {:?}",
        iters,
        start_time.elapsed()
    );
}

// -------------------------------------------------------------------------
// 3. MUTATION FUZZ: Mutated programs never trigger unhandled compiler panics
// -------------------------------------------------------------------------
#[test]
fn structured_mutations_never_panic() {
    let seed = get_seed();
    let iters = get_iterations(40);
    let mut rng = FuzzRng::new(seed);

    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("guppty-mutation-fuzz-{}.gup", std::process::id()));

    for i in 0..iters {
        let mut gen = ProgramGenerator::new(&mut rng);
        let base_program = gen.generate_program();

        // Mutate: delete, insert random tokens, or break indent
        let mut mutated = String::new();
        for line in base_program.lines() {
            if rng.next_usize(10) < 3 {
                // Mutate this line
                match rng.next_usize(4) {
                    0 => {
                        // Skip line (deletion)
                        continue;
                    }
                    1 => {
                        // Bad indentation
                        mutated.push_str("   ");
                        mutated.push_str(line);
                        mutated.push('\n');
                    }
                    2 => {
                        // Truncate line
                        let half = line.len() / 2;
                        mutated.push_str(&line[..half]);
                        mutated.push('\n');
                    }
                    _ => {
                        // Insert broken syntax
                        mutated.push_str(line);
                        mutated.push_str(" === @@@ !!!\n");
                    }
                }
            } else {
                mutated.push_str(line);
                mutated.push('\n');
            }
        }

        fs::write(&temp_path, &mutated).expect("write mutated file");
        let file_arg = temp_path.to_str().unwrap();

        let (_, vm_err, _) = run_guppty(&[file_arg]);
        let (_, interp_err, _) = run_guppty(&[file_arg, "--interp"]);

        if vm_err.contains("panicked at") || interp_err.contains("panicked at") {
            let artifact = save_artifact(seed + i as u64, "panic_mutation", &mutated);
            panic!(
                "Backend panicked on mutated input (iteration {i}, seed {})!\nArtifact: {}\nVM err: {vm_err}\nInterp err: {interp_err}",
                seed + i as u64,
                artifact.display()
            );
        }
    }

    let _ = fs::remove_file(temp_path);
}
