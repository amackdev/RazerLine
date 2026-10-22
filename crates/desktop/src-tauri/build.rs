use std::path::Path;
use std::process::Command;

fn main() {
    // If the frontend build output doesn't exist, build it automatically so the app is self-contained
    let dist_index = Path::new("../build/index.html");
    if !dist_index.exists() {
        println!("cargo:warning=Frontend build not found, running npm run build...");
        let status = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", "npm run build"])
                .current_dir("..")
                .status()
        } else {
            Command::new("npm")
                .args(["run", "build"])
                .current_dir("..")
                .status()
        };
        if let Err(e) = status {
            println!("cargo:warning=Failed to run npm run build: {e}");
        }
    }

    tauri_build::build()
}
