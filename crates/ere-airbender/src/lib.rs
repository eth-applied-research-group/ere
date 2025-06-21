#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use zkvm_interface::{Compiler, zkVM, zkVMError};

mod compile;
mod error;

#[allow(non_camel_case_types)]
pub struct RV32_IM_SUCCINCT_ZKVM_ELF;

impl Compiler for RV32_IM_SUCCINCT_ZKVM_ELF {
    type Error = error::AirbenderError;
    type Program = Vec<u8>;

    fn compile(path_to_program: &std::path::Path) -> Result<Self::Program, Self::Error> {
        compile::compile_airbender_program(path_to_program).map_err(Into::into)
    }
}

// Placeholder zkVM implementation - not needed for compile tests
pub struct EreAirbender;

impl zkVM for EreAirbender {
    fn execute(
        &self,
        _inputs: &zkvm_interface::Input,
    ) -> Result<zkvm_interface::ProgramExecutionReport, zkVMError> {
        unimplemented!("Airbender execution not yet implemented")
    }

    fn prove(
        &self,
        _inputs: &zkvm_interface::Input,
    ) -> Result<(Vec<u8>, zkvm_interface::ProgramProvingReport), zkVMError> {
        unimplemented!("Airbender proving not yet implemented")
    }

    fn verify(&self, _proof: &[u8]) -> Result<(), zkVMError> {
        unimplemented!("Airbender verification not yet implemented")
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
