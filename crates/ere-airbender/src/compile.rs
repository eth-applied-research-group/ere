use std::{fs, path::Path, process::Command};

use toml::Value as TomlValue;
use tracing::info;

use crate::error::CompileError;


/// Compile the guest crate and return raw ELF bytes.
pub fn compile_airbender_program(program_crate_path: &Path) -> Result<Vec<u8>, CompileError> {
    info!("Compiling Airbender program at {}", program_crate_path.display());

    if !program_crate_path.exists() || !program_crate_path.is_dir() {
        return Err(CompileError::InvalidProgramPath(
            program_crate_path.to_path_buf(),
        ));
    }

    let guest_manifest_path = program_crate_path.join("Cargo.toml");
    if !guest_manifest_path.exists() {
        return Err(CompileError::CargoTomlMissing {
            program_dir: program_crate_path.to_path_buf(),
            manifest_path: guest_manifest_path.clone(),
        });
    }

    // ── read + parse Cargo.toml ───────────────────────────────────────────
    let manifest_content =
        fs::read_to_string(&guest_manifest_path).map_err(|e| CompileError::ReadFile {
            path: guest_manifest_path.clone(),
            source: e,
        })?;

    let manifest_toml: TomlValue =
        manifest_content
            .parse::<TomlValue>()
            .map_err(|e| CompileError::ParseCargoToml {
                path: guest_manifest_path.clone(),
                source: e,
            })?;

    let program_name = manifest_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| CompileError::MissingPackageName {
            path: guest_manifest_path.clone(),
        })?;

    info!("Parsed program name: {program_name}");

    // ── build the program ─────────────────────────────────────────────
    info!(
        "Running `cargo build --release` for program: {}",
        program_name
    );

    let status = Command::new("cargo")
        .current_dir(program_crate_path)
        .args([
            "+nightly",
            "build",
            "--release",
            "--target",
            "riscv32i-unknown-none-elf",
       ])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| CompileError::CargoProveBuild {
            cwd: program_crate_path.to_path_buf(),
            source: e,
        })?;

    if !status.success() {
        return Err(CompileError::CargoBuildFailed {
            status,
            path: program_crate_path.to_path_buf(),
        });
    }

    // The ELF file will be in target/riscv32i-unknown-none-elf/release/
    let elf_path = program_crate_path
        .join("target")
        .join("riscv32i-unknown-none-elf")
        .join("release")
        .join(program_name);
        
    if !elf_path.exists() {
        return Err(CompileError::ElfNotFound(elf_path));
    }

    let elf_bytes = fs::read(&elf_path).map_err(|e| CompileError::ReadFile {
        path: elf_path,
        source: e,
    })?;

    info!("Airbender program compiled OK – {} bytes", elf_bytes.len());
    Ok(elf_bytes)
}

#[cfg(test)]
mod tests {
    use zkvm_interface::Compiler;

    use crate::RV32_IM_SUCCINCT_ZKVM_ELF;

    use super::*;
    use std::path::PathBuf;

    // TODO: for now, we just get one test file
    // TODO: but this should get the whole directory and compile each test
    fn get_compile_test_guest_program_path() -> PathBuf {
        let workspace_dir = env!("CARGO_WORKSPACE_DIR");
        PathBuf::from(workspace_dir)
            .join("tests")
            .join("airbender")
            .join("compile")
            .join("basic")
            .canonicalize()
            .expect("Failed to find or canonicalize test guest program at <CARGO_WORKSPACE_DIR>/tests/compile/airbender")
    }

    #[test]
    fn test_compile_airbender_program() {
        let test_guest_path = get_compile_test_guest_program_path();

        match compile_airbender_program(&test_guest_path) {
            Ok(elf_bytes) => {
                assert!(!elf_bytes.is_empty(), "ELF bytes should not be empty.");
            }
            Err(e) => {
                panic!("compile failed for dedicated guest: {:?}", e);
            }
        }
    }

    #[test]
    fn test_compile_trait() {
        let test_guest_path = get_compile_test_guest_program_path();
        match RV32_IM_SUCCINCT_ZKVM_ELF::compile(&test_guest_path) {
            Ok(elf_bytes) => {
                assert!(!elf_bytes.is_empty(), "ELF bytes should not be empty.");
            }
            Err(e) => {
                panic!(
                    "compile_airbender_program direct call failed for dedicated guest: {:?}",
                    e
                );
            }
        }
    }
}
