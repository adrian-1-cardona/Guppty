use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

fn temp_workspace(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "guppty-workflow-{}-{}-{label}",
        process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create temporary workspace");
    path
}

#[test]
fn new_creates_a_runnable_project() {
    let workspace = temp_workspace("new");
    let binary = env!("CARGO_BIN_EXE_guppty");

    let created = Command::new(binary)
        .args(["new", "hello-world"])
        .current_dir(&workspace)
        .output()
        .expect("run guppty new");
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    assert!(workspace.join("hello-world/guppty.toml").is_file());
    assert!(workspace.join("hello-world/src/main.gup").is_file());
    assert!(workspace.join("hello-world/.gitignore").is_file());

    let run = Command::new(binary)
        .arg("run")
        .current_dir(workspace.join("hello-world"))
        .output()
        .expect("run generated project");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "Hello from hello-world!\n"
    );

    fs::remove_dir_all(workspace).expect("clean temporary workspace");
}

#[test]
fn check_validates_without_running_the_program() {
    let workspace = temp_workspace("check");
    let source = workspace.join("quiet.gup");
    fs::write(&source, "out(\"this should not run\")\n").expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_guppty"))
        .args(["check", source.to_str().unwrap()])
        .output()
        .expect("check program");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("is ready"));
    assert!(!stdout.contains("this should not run"));

    fs::remove_dir_all(workspace).expect("clean temporary workspace");
}

#[test]
fn build_creates_a_standalone_executable() {
    let workspace = temp_workspace("build");
    let source = workspace.join("ship-me.gup");
    fs::write(&source, "out(\"built with Guppty\")\n").expect("write source");
    let artifact = workspace.join(if cfg!(windows) {
        "ship-me.exe"
    } else {
        "ship-me"
    });

    let build = Command::new(env!("CARGO_BIN_EXE_guppty"))
        .args([
            "build",
            source.to_str().unwrap(),
            "--output",
            artifact.to_str().unwrap(),
        ])
        .output()
        .expect("build program");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(artifact.is_file());

    let run = Command::new(&artifact).output().expect("run built program");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "built with Guppty\n");

    fs::remove_dir_all(workspace).expect("clean temporary workspace");
}
