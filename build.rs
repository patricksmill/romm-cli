use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/commands/");
    println!("cargo:rerun-if-changed=src/frontend/cli.rs");

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .or_else(|_| env::var("CARGO_MANIFEST_DIR").map(|d| PathBuf::from(d).join("target")))
        .unwrap_or_else(|_| PathBuf::from("target"));

    let gen_name = if cfg!(windows) {
        "romm-complete-gen.exe"
    } else {
        "romm-complete-gen"
    };
    let generator = target_dir.join(&profile).join(gen_name);

    if !generator.exists() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let status = Command::new(&generator).current_dir(&manifest_dir).status();

    if let Ok(s) = status {
        if !s.success() {
            eprintln!("warning: {gen_name} exited with {s}; committed completions/ left unchanged");
        }
    }
}
