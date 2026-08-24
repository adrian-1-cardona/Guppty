mod ast;
mod bytecode;
mod compiler;
mod environment;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod syntax;
mod token;
mod value;
mod vm;

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const EMBEDDED_MAGIC: &[u8] = b"\nGUPPTY_EMBEDDED_PROGRAM_V1\n";

fn main() {
    if let Some((filename, source)) = embedded_program() {
        run_source(&filename, &source, false);
        return;
    }

    let args: Vec<String> = env::args().skip(1).collect();
    if let Err(message) = dispatch(&args) {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn dispatch(args: &[String]) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        print_help();
        return Err("Run `guppty help` to see what you can do.".to_string());
    };

    match command {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "version" | "--version" | "-V" => {
            println!("guppty {VERSION}");
            Ok(())
        }
        "new" => create_project(args.get(1).map(String::as_str)),
        "init" => init_project(Path::new("."), None),
        "run" => {
            let use_interpreter = args.iter().any(|arg| arg == "--interp");
            let requested = positional_after_command(args, &["--interp"]);
            let path = resolve_source(requested.as_deref())?;
            run_file(&path, use_interpreter);
            Ok(())
        }
        "check" => {
            let requested = positional_after_command(args, &[]);
            let path = resolve_source(requested.as_deref())?;
            let source = read_source(&path)?;
            compile_source(&path.display().to_string(), &source);
            println!("✓ {} is ready", path.display());
            Ok(())
        }
        "build" => build_project(args),
        option if option.starts_with('-') => Err(format!("Unknown option: {option}")),
        filename => {
            let path = PathBuf::from(filename);
            let use_interpreter = args.iter().any(|arg| arg == "--interp");
            run_file(&path, use_interpreter);
            Ok(())
        }
    }
}

fn print_help() {
    println!(
        "Guppty {VERSION} — make and run .gup programs\n\n\
Usage:\n  \
  guppty new <name>       Create a fresh Guppty project\n  \
  guppty init             Create a project in this folder\n  \
  guppty run [file.gup]   Run a project or source file\n  \
  guppty check [file.gup] Check a program without running it\n  \
  guppty build [file.gup] Build a standalone executable\n  \
  guppty <file.gup>       Run a source file directly\n  \
  guppty version          Show the installed version\n\n\
Options:\n  \
  --interp                Use the tree-walking interpreter\n  \
  -o, --output <path>     Choose the build output path"
    );
}

fn create_project(name: Option<&str>) -> Result<(), String> {
    let name = name.ok_or_else(|| "Usage: guppty new <project-name>".to_string())?;
    validate_project_name(name)?;
    let root = PathBuf::from(name);
    if root.exists() {
        return Err(format!("A file or folder named '{name}' already exists."));
    }
    init_project(&root, Some(name))?;
    println!("\nNext steps:\n  cd {name}\n  guppty run\n  guppty build");
    Ok(())
}

fn init_project(root: &Path, supplied_name: Option<&str>) -> Result<(), String> {
    let inferred_name = root
        .canonicalize()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "my-guppty-program".to_string());
    let name = supplied_name.unwrap_or(&inferred_name);
    validate_project_name(name)?;

    let manifest = root.join("guppty.toml");
    let source = root.join("src/main.gup");
    if manifest.exists() || source.exists() {
        return Err("This folder already contains a Guppty project.".to_string());
    }

    fs::create_dir_all(root.join("src"))
        .map_err(|error| format!("Could not create the project: {error}"))?;
    fs::write(
        &manifest,
        format!("[project]\nname = \"{name}\"\nentry = \"src/main.gup\"\n"),
    )
    .map_err(|error| format!("Could not write {}: {error}", manifest.display()))?;
    fs::write(
        &source,
        format!("// Welcome to {name}!\nmessage = \"Hello from {name}!\"\nout(message)\n"),
    )
    .map_err(|error| format!("Could not write {}: {error}", source.display()))?;
    fs::write(root.join(".gitignore"), "build/\n")
        .map_err(|error| format!("Could not write .gitignore: {error}"))?;

    println!("✓ Created {name}");
    println!("  {}", manifest.display());
    println!("  {}", source.display());
    Ok(())
}

fn validate_project_name(name: &str) -> Result<(), String> {
    let valid = !name.is_empty()
        && !name.starts_with(['-', '.'])
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err("Project names may contain letters, numbers, dashes, and underscores.".to_string())
    }
}

fn build_project(args: &[String]) -> Result<(), String> {
    let mut requested_source: Option<&str> = None;
    let mut output: Option<PathBuf> = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "-o" | "--output" => {
                index += 1;
                let path = args
                    .get(index)
                    .ok_or_else(|| "--output needs a path".to_string())?;
                output = Some(PathBuf::from(path));
            }
            option if option.starts_with('-') => return Err(format!("Unknown option: {option}")),
            source if requested_source.is_none() => requested_source = Some(source),
            extra => return Err(format!("Unexpected argument: {extra}")),
        }
        index += 1;
    }

    let source_path = resolve_source(requested_source)?;
    let source = read_source(&source_path)?;
    compile_source(&source_path.display().to_string(), &source);

    let output_path = output.unwrap_or_else(|| default_build_path(&source_path));
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }

    let current_exe = env::current_exe()
        .map_err(|error| format!("Could not locate the Guppty executable: {error}"))?;
    fs::copy(&current_exe, &output_path)
        .map_err(|error| format!("Could not create {}: {error}", output_path.display()))?;
    let mut artifact = OpenOptions::new()
        .append(true)
        .open(&output_path)
        .map_err(|error| format!("Could not finish {}: {error}", output_path.display()))?;
    artifact
        .write_all(source.as_bytes())
        .and_then(|_| artifact.write_all(&(source.len() as u64).to_le_bytes()))
        .and_then(|_| artifact.write_all(EMBEDDED_MAGIC))
        .map_err(|error| format!("Could not finish {}: {error}", output_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&output_path, fs::Permissions::from_mode(0o755))
            .map_err(|error| format!("Could not make the build executable: {error}"))?;
    }

    println!("✓ Built {}", output_path.display());
    println!("  Run it with: {}", display_executable_path(&output_path));
    Ok(())
}

fn default_build_path(source: &Path) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("program");
    let project_name = project_name_from_manifest().unwrap_or_else(|| stem.to_string());
    let extension = if cfg!(windows) { ".exe" } else { "" };
    PathBuf::from(format!("build/{project_name}{extension}"))
}

fn project_name_from_manifest() -> Option<String> {
    let manifest = fs::read_to_string("guppty.toml").ok()?;
    manifest_value(&manifest, "name")
}

fn resolve_source(requested: Option<&str>) -> Result<PathBuf, String> {
    if let Some(path) = requested {
        return Ok(PathBuf::from(path));
    }
    let manifest = fs::read_to_string("guppty.toml").map_err(|_| {
        "No file was provided and no guppty.toml was found. Run `guppty new <name>` first."
            .to_string()
    })?;
    Ok(PathBuf::from(
        manifest_value(&manifest, "entry").unwrap_or_else(|| "src/main.gup".to_string()),
    ))
}

fn manifest_value(manifest: &str, key: &str) -> Option<String> {
    manifest.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        if candidate.trim() == key {
            Some(value.trim().trim_matches('"').to_string())
        } else {
            None
        }
    })
}

fn positional_after_command(args: &[String], flags: &[&str]) -> Option<String> {
    args.iter()
        .skip(1)
        .find(|arg| !arg.starts_with('-') && !flags.contains(&arg.as_str()))
        .cloned()
}

fn display_executable_path(path: &Path) -> String {
    if path.components().count() == 2 && !path.is_absolute() {
        format!("./{}", path.display())
    } else {
        path.display().to_string()
    }
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| {
        format!(
            "Oops! I couldn't read the file '{}': {error}",
            path.display()
        )
    })
}

fn run_file(path: &Path, use_interpreter: bool) {
    let source = read_source(path).unwrap_or_else(|message| exit_with_message(message));
    run_source(&path.display().to_string(), &source, use_interpreter);
}

fn compile_source(filename: &str, source: &str) {
    let tokens = lexer::lex(source).unwrap_or_else(|error| {
        exit_with_message(error.render(filename, source));
    });
    let program = parser::parse(tokens).unwrap_or_else(|error| {
        exit_with_message(error.render(filename, source));
    });
    let _ = compiler::compile(&program).unwrap_or_else(|error| {
        exit_with_message(error.render(filename, source));
    });
}

fn run_source(filename: &str, source: &str, use_interpreter: bool) {
    let tokens = lexer::lex(source).unwrap_or_else(|error| {
        exit_with_message(error.render(filename, source));
    });
    let program = parser::parse(tokens).unwrap_or_else(|error| {
        exit_with_message(error.render(filename, source));
    });

    if use_interpreter {
        interpreter::interpret(program).unwrap_or_else(|error| {
            exit_with_message(error.render(filename, source));
        });
    } else {
        let script = compiler::compile(&program).unwrap_or_else(|error| {
            exit_with_message(error.render(filename, source));
        });
        vm::run(script).unwrap_or_else(|error| {
            exit_with_message(error.render(filename, source));
        });
    }
}

fn embedded_program() -> Option<(String, String)> {
    let executable = env::current_exe().ok()?;
    let bytes = fs::read(executable).ok()?;
    if !bytes.ends_with(EMBEDDED_MAGIC) || bytes.len() < EMBEDDED_MAGIC.len() + 8 {
        return None;
    }
    let length_end = bytes.len() - EMBEDDED_MAGIC.len();
    let length_start = length_end - 8;
    let length = u64::from_le_bytes(bytes[length_start..length_end].try_into().ok()?) as usize;
    let source_start = length_start.checked_sub(length)?;
    let source = String::from_utf8(bytes[source_start..length_start].to_vec()).ok()?;
    Some(("<built Guppty program>".to_string(), source))
}

fn exit_with_message(message: String) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}
