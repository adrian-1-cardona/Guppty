// Shared helpers for integration tests that run real .gup files.

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// Guppty can run code through the VM or through the interpreter. The tests use
// this enum to run the same file both ways.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Vm,
    Interpreter,
}

impl Backend {
    // A friendly name to show in test messages so a human knows which
    // way was used when something goes wrong.
    pub fn name(self) -> &'static str {
        match self {
            Backend::Vm => "vm (default)",
            Backend::Interpreter => "interpreter (--interp)",
        }
    }

    // Both backends in a little list, handy for "do this for every way" loops.
    pub fn all() -> [Backend; 2] {
        [Backend::Vm, Backend::Interpreter]
    }
}

pub fn run_gup_file_with_backend(file: &Path, backend: Backend) -> (String, String, i32) {
    let bin = env!("CARGO_BIN_EXE_guppty");

    let mut command = Command::new(bin);
    command.arg(file);
    if backend == Backend::Interpreter {
        command.arg("--interp");
    }

    let output = command
        .output()
        .unwrap_or_else(|e| panic!("failed to run guppty on {}: {}", file.display(), e));

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);

    (stdout, stderr, code)
}
