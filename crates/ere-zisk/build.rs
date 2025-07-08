use std::process::Command;

use build_utils::gen_name_and_sdk_version;

fn main() {
    let out = Command::new("cargo-zisk")
        .arg("--version")
        .output()
        .expect("failed to execute cargo-zisk --version");

    if !out.status.success() {
        panic!("cargo-zisk exited with {}", out.status);
    }

    // Example stdout:  "cargo-zisk 0.8.1 (f9a3655 2025-05-19T16:57:59.442084351Z)"
    let text = std::str::from_utf8(&out.stdout).expect("cargo-zisk --version output");
    let version = text
        .split_whitespace()
        .nth(1)
        .expect("unexpected --version format");

    gen_name_and_sdk_version("zisk", version);
}
