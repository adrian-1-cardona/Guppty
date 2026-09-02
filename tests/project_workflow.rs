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

#[test]
fn project_commands_find_and_resolve_a_valid_manifest_from_nested_folders() {
    let workspace = temp_workspace("nested-manifest");
    let source = workspace.join("source files/main.gup");
    let nested = workspace.join("one/two");
    fs::create_dir_all(source.parent().unwrap()).expect("create source folder");
    fs::create_dir_all(&nested).expect("create nested folder");
    fs::write(&source, "out(\"found my project :D\")\n").expect("write source");
    fs::write(
        workspace.join("guppty.toml"),
        "# friendly comments are valid TOML\n[project]\nname = \"nested-project\"\nentry = \"source files/main.gup\"\n",
    )
    .expect("write manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_guppty"))
        .arg("run")
        .current_dir(nested)
        .output()
        .expect("run nested project");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "found my project :D\n"
    );

    fs::remove_dir_all(workspace).expect("clean temporary workspace");
}

#[test]
fn malformed_and_unknown_manifest_settings_fail_early() {
    let binary = env!("CARGO_BIN_EXE_guppty");
    for (label, manifest, expected) in [
        ("malformed", "[project\nname = 3", "project setting"),
        (
            "unknown",
            "[project]\nname = \"hello\"\nentry = \"main.gup\"\nsurprise = true\n",
            "surprise",
        ),
        (
            "wrong-entry",
            "[project]\nname = \"hello\"\nentry = \"main.txt\"\n",
            ".gup file",
        ),
    ] {
        let workspace = temp_workspace(label);
        fs::write(workspace.join("guppty.toml"), manifest).expect("write bad manifest");
        let output = Command::new(binary)
            .arg("run")
            .current_dir(&workspace)
            .output()
            .expect("run bad manifest");
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("guppty.toml"), "{stderr}");
        assert!(stderr.contains(expected), "{stderr}");
        fs::remove_dir_all(workspace).expect("clean temporary workspace");
    }
}
