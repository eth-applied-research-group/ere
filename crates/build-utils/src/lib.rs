use std::{env, fs, path::Path};

// Generate a Rust source file that contains the name and version of the SDK.
pub fn gen_name_and_sdk_version(name: &str, sdk_dep_name: &str) {
    let workspace_dir = env::var("CARGO_WORKSPACE_DIR").unwrap();
    let lock = fs::read_to_string(Path::new(&workspace_dir).join("Cargo.lock"))
        .expect("Cargo.lock not found");

    let version = lock
        .split("[[package]]")
        .find(|pkg| pkg.contains(&format!("name = \"{sdk_dep_name}\"")))
        .and_then(|pkg| {
            pkg.lines()
                .find(|l| l.trim_start().starts_with("version ="))
                .and_then(|l| l.split('"').nth(1))
        })
        .unwrap_or_else(|| panic!("{sdk_dep_name} not found in Cargo.lock"));

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("sdk_version.rs");
    fs::write(
        &dest,
        format!("pub const NAME: &str = \"{name}\";\npub const SDK_VERSION: &str = \"{version}\";"),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=Cargo.lock");
}
