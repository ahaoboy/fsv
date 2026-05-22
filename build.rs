use std::path::Path;
use std::process::Command;

fn main() {
    let index_html = Path::new("dist/index.html");

    if !index_html.exists() {
        let shell = if cfg!(target_os = "windows") { "powershell" } else { "sh" };

        println!("cargo:warning=dist/index.html not found, running pnpm i...");
        let status = Command::new(shell)
            .args(["-c", "pnpm i"])
            .env("CI", "true")
            .status()
            .expect("Failed to run pnpm i");

        if !status.success() {
            panic!("pnpm i failed");
        }

        println!("cargo:warning=running pnpm run build...");
        let status = Command::new(shell)
            .args(["-c", "pnpm run build"])
            .env("CI", "true")
            .status()
            .expect("Failed to run pnpm run build");

        if !status.success() {
            panic!("pnpm run build failed");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=dist/index.html");
}
