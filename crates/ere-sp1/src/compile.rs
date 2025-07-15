use std::{path::Path, process::Command};

use build_utils::docker;
use tempfile::TempDir;
use tracing::info;

use crate::error::CompileError;

pub fn compile(guest_program_full_path: &Path) -> Result<Vec<u8>, CompileError> {
    // Build the SP1 docker image
    let tag = "ere-build-sp1:latest";
    docker::build_image("docker/sp1/Dockerfile", tag)
        .map_err(|e| CompileError::DockerImageBuildFailed(Box::new(e)))?;

    // Compile the guest program using the SP1 docker image
    let guest_program_path_str = guest_program_full_path
        .to_str()
        .ok_or_else(|| CompileError::InvalidGuestPath(guest_program_full_path.to_path_buf()))?;
    let elf_output_dir = TempDir::new().map_err(CompileError::CreatingTempOutputDirectoryFailed)?;
    let elf_output_dir_str = elf_output_dir
        .path()
        .to_str()
        .ok_or_else(|| CompileError::InvalidTempOutputPath(elf_output_dir.path().to_path_buf()))?;

    info!("Mounting guest program at: {}", guest_program_path_str);
    Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &format!("{guest_program_path_str}:/app"),
            "-v",
            &format!("{elf_output_dir_str}:/output"),
            tag,
            "cargo",
            "prove",
            "build",
            "--output-directory",
            "/output",
            "--elf-name",
            "guest_program.elf",
        ])
        .status()
        .map_err(|e| CompileError::DockerImageBuildFailed(Box::new(e)))?;

    // Read the compiled ELF program from the output directory
    let elf = std::fs::read(elf_output_dir.path().join("guest_program.elf"))
        .map_err(CompileError::ReadCompiledELFProgram)?;

    Ok(elf)
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
            .join("sp1")
            .join("compile")
            .join("basic")
            .canonicalize()
            .expect("Failed to find or canonicalize test guest program at <CARGO_WORKSPACE_DIR>/tests/compile/sp1")
    }

    #[test]
    fn test_compile_sp1_program() {
        let test_guest_path = get_compile_test_guest_program_path();

        match compile(&test_guest_path) {
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
                    "compile_sp1_program direct call failed for dedicated guest: {:?}",
                    e
                );
            }
        }
    }
}
