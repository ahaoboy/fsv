use std::path::Path;
use std::process::Command;

fn main() {
    let index_html = Path::new("dist/index.html");

    if !index_html.exists() {
        let shell = if cfg!(target_os = "windows") { "powershell" } else { "sh" };

        println!("cargo:warning=dist/index.html not found, running bun i...");
        let status = Command::new(shell)
            .args(["-c", "bun i"])
            .env("CI", "true")
            .status()
            .expect("Failed to run bun i");

        if !status.success() {
            panic!("bun i failed");
        }

        println!("cargo:warning=running bun run build...");
        let status = Command::new(shell)
            .args(["-c", "bun run build"])
            .env("CI", "true")
            .status()
            .expect("Failed to run bun run build");

        if !status.success() {
            panic!("bun run build failed");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=dist/index.html");
}
