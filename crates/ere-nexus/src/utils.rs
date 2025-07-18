use std::fs;
use toml::{Table, Value};

pub fn get_cargo_package_name(crate_path: &std::path::Path) -> Option<String> {
    println!("aaabbbbb---------1");
    let cargo_contents = fs::read_to_string(crate_path.join("Cargo.toml")).ok()?;

    println!("aaabbbbb---------2");
    println!("cargo_toml: {}", cargo_contents);
    // let value: Toml = cargo_toml.parse().ok()?;
    let cargo_toml: Table = toml::from_str(&cargo_contents).ok()?;

    println!("aaabbbbb---------");
    cargo_toml
        .get("package")?
        .get("name")?
        .as_str()
        .map(|s| s.to_string())
}
