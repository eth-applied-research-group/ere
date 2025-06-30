#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::{fs, path::PathBuf, process::Command, time::Instant};
use zkvm_interface::{
    Compiler, Input, InputItem, ProgramExecutionReport, ProgramProvingReport, zkVM, zkVMError,
};

mod compile;
mod error;

use error::{ExecuteError, ProveError, VerifyError};

#[allow(non_camel_case_types)]
pub struct RV32_IM_SUCCINCT_ZKVM_ELF;

impl Compiler for RV32_IM_SUCCINCT_ZKVM_ELF {
    type Error = error::AirbenderError;
    type Program = Vec<u8>;

    fn compile(path_to_program: &std::path::Path) -> Result<Self::Program, Self::Error> {
        compile::compile_airbender_program(path_to_program).map_err(Into::into)
    }
}

pub struct EreAirbender {
    /// The compiled ELF program
    program: Vec<u8>,
}

impl EreAirbender {
    pub fn new(program: Vec<u8>) -> Self {
        Self { program }
    }

    fn write_program_to_temp(&self) -> Result<PathBuf, std::io::Error> {
        let dir = tempfile::TempDir::new()?;
        let elf_path = dir.path().join("program.elf");
        fs::write(&elf_path, &self.program)?;

        // We need to keep the TempDir alive
        let _ = dir.keep();
        Ok(elf_path)
    }

    fn write_inputs_to_temp(&self, inputs: &Input) -> Result<PathBuf, std::io::Error> {
        let dir = tempfile::TempDir::new()?;
        let input_path = dir.path().join("input.txt");

        // Airbender expects inputs as hex strings, one per line
        let mut input_lines = Vec::new();
        for input in inputs.iter() {
            match input {
                InputItem::Object(serializable) => {
                    // Serialize the object to bytes, then convert to hex
                    let bytes = bincode::serialize(serializable)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    let hex = hex::encode(bytes);
                    input_lines.push(hex);
                }
                InputItem::Bytes(bytes) => {
                    let hex = hex::encode(bytes);
                    input_lines.push(hex);
                }
            }
        }

        fs::write(&input_path, input_lines.join("\n"))?;

        // We need to keep the TempDir alive
        let _ = dir.keep();
        Ok(input_path)
    }
}

impl zkVM for EreAirbender {
    fn execute(
        &self,
        inputs: &zkvm_interface::Input,
    ) -> Result<zkvm_interface::ProgramExecutionReport, zkVMError> {
        let start = Instant::now();

        // Write program and inputs to temporary files
        let elf_path = self
            .write_program_to_temp()
            .map_err(|e| error::AirbenderError::Execute(ExecuteError::Io(e)))?;
        let input_path = self
            .write_inputs_to_temp(inputs)
            .map_err(|e| error::AirbenderError::Execute(ExecuteError::Io(e)))?;

        // Run airbender-cli execute
        let output = Command::new("airbender-cli")
            .args([
                "execute",
                "--bin",
                elf_path.to_str().unwrap(),
                "--input-file",
                input_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| error::AirbenderError::Execute(ExecuteError::CliCommand(e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(error::AirbenderError::Execute(ExecuteError::CliFailed(
                stderr.to_string(),
            ))
            .into());
        }

        // TODO: Parse execution stats from output
        // For now, return placeholder metrics
        Ok(ProgramExecutionReport {
            total_num_cycles: 0, // Would need to parse from CLI output
            region_cycles: indexmap::IndexMap::new(),
            execution_duration: start.elapsed(),
        })
    }

    fn prove(
        &self,
        inputs: &zkvm_interface::Input,
    ) -> Result<(Vec<u8>, zkvm_interface::ProgramProvingReport), zkVMError> {
        let start = Instant::now();

        // Create output directory
        let output_dir = tempfile::TempDir::new()
            .map_err(|e| error::AirbenderError::Prove(ProveError::Io(e)))?;
        fs::create_dir(output_dir.path().join("output"))
            .map_err(|e| error::AirbenderError::Prove(ProveError::Io(e)))?;

        // Write program and inputs to temporary files
        let elf_path = self
            .write_program_to_temp()
            .map_err(|e| error::AirbenderError::Prove(ProveError::Io(e)))?;
        let input_path = self
            .write_inputs_to_temp(inputs)
            .map_err(|e| error::AirbenderError::Prove(ProveError::Io(e)))?;

        // Run airbender-cli prove
        let output = Command::new("airbender-cli")
            .current_dir(output_dir.path())
            .args([
                "prove",
                "--bin",
                elf_path.to_str().unwrap(),
                "--input-file",
                input_path.to_str().unwrap(),
                "--until",
                "final-recursion",
                "--tmp-dir",
                "/tmp",
            ])
            .output()
            .map_err(|e| error::AirbenderError::Prove(ProveError::CliCommand(e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(
                error::AirbenderError::Prove(ProveError::CliFailed(stderr.to_string())).into(),
            );
        }

        // Read the proof from output/metadata.json
        let metadata_path = output_dir.path().join("output").join("metadata.json");
        let proof_bytes = fs::read(&metadata_path).map_err(|e| {
            error::AirbenderError::Prove(ProveError::ReadProof(metadata_path.clone(), e))
        })?;

        let proving_time = start.elapsed();

        Ok((proof_bytes, ProgramProvingReport::new(proving_time)))
    }

    fn verify(&self, proof: &[u8]) -> Result<(), zkVMError> {
        // Write proof to temporary metadata.json
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| error::AirbenderError::Verify(VerifyError::Io(e)))?;
        let metadata_path = temp_dir.path().join("metadata.json");
        fs::write(&metadata_path, proof)
            .map_err(|e| error::AirbenderError::Verify(VerifyError::Io(e)))?;

        // Run airbender-cli verify-all
        let output = Command::new("airbender-cli")
            .args(["verify-all", "--metadata", metadata_path.to_str().unwrap()])
            .output()
            .map_err(|e| error::AirbenderError::Verify(VerifyError::CliCommand(e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(
                error::AirbenderError::Verify(VerifyError::CliFailed(stderr.to_string())).into(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod compile_tests {
    use super::*;
    use std::path::PathBuf;

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

        match compile::compile_airbender_program(&test_guest_path) {
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

#[cfg(test)]
mod execute_tests {
    use super::*;
    use std::path::PathBuf;
    use zkvm_interface::Input;

    fn get_execute_test_guest_program_path() -> PathBuf {
        let workspace_dir = env!("CARGO_WORKSPACE_DIR");
        PathBuf::from(workspace_dir)
            .join("tests")
            .join("airbender")
            .join("execute")
            .join("basic")
            .canonicalize()
            .expect("Failed to find test guest program")
    }

    #[test]
    #[ignore = "Requires airbender-cli to be installed"]
    fn test_execute_airbender() {
        let test_guest_path = get_execute_test_guest_program_path();
        let elf_bytes = RV32_IM_SUCCINCT_ZKVM_ELF::compile(&test_guest_path)
            .expect("Failed to compile test guest");

        let zkvm = EreAirbender::new(elf_bytes);

        // Empty input for fibonacci program
        let input = Input::new();

        let result = zkvm.execute(&input);
        assert!(result.is_ok(), "Execution should succeed");
    }
}

#[cfg(test)]
mod prove_tests {
    use super::*;
    use std::path::PathBuf;
    use zkvm_interface::Input;

    fn get_prove_test_guest_program_path() -> PathBuf {
        let workspace_dir = env!("CARGO_WORKSPACE_DIR");
        PathBuf::from(workspace_dir)
            .join("tests")
            .join("airbender")
            .join("prove")
            .join("basic")
            .canonicalize()
            .expect("Failed to find test guest program")
    }

    #[test]
    #[ignore = "Requires airbender-cli to be installed"]
    fn test_prove_and_verify_airbender() {
        let test_guest_path = get_prove_test_guest_program_path();
        let elf_bytes = RV32_IM_SUCCINCT_ZKVM_ELF::compile(&test_guest_path)
            .expect("Failed to compile test guest");

        let zkvm = EreAirbender::new(elf_bytes);

        // Empty input for fibonacci program
        let input = Input::new();

        // Generate proof
        let (proof_bytes, report) = zkvm.prove(&input).expect("Proving should succeed");

        assert!(!proof_bytes.is_empty(), "Proof should not be empty");
        println!("Proving took: {:?}", report.proving_time);

        // Verify proof
        zkvm.verify(&proof_bytes)
            .expect("Verification should succeed");
    }
}
