use crate::{compile::compile_risczero_program, error::RiscZeroError};
use risc0_zkvm::{ExecutorEnv, Receipt, default_executor};
use std::time::Instant;
use zkvm_interface::{
    Compiler, Input, InputItem, ProgramExecutionReport, ProgramProvingReport, ProverResourceType,
    zkVM, zkVMError,
};

include!(concat!(env!("OUT_DIR"), "/name_and_sdk_version.rs"));

mod compile;
mod error;
mod prove;

pub use compile::Risc0Program;

#[allow(non_camel_case_types)]
pub struct RV32_IM_RISCZERO_ZKVM_ELF;

impl Compiler for RV32_IM_RISCZERO_ZKVM_ELF {
    type Error = RiscZeroError;

    type Program = Risc0Program;

    fn compile(path_to_program: &std::path::Path) -> Result<Self::Program, Self::Error> {
        compile_risczero_program(path_to_program).map_err(RiscZeroError::from)
    }
}

pub struct EreRisc0 {
    program: <RV32_IM_RISCZERO_ZKVM_ELF as Compiler>::Program,
    resource_type: ProverResourceType,
}

impl EreRisc0 {
    pub fn new(
        program: <RV32_IM_RISCZERO_ZKVM_ELF as Compiler>::Program,
        resource_type: ProverResourceType,
    ) -> Result<Self, zkVMError> {
        match resource_type {
            ProverResourceType::Cpu => {
                if cfg!(feature = "cuda") || cfg!(feature = "metal") {
                    panic!("CPU mode requires both 'cuda' and 'metal' features to be disabled");
                }
            }
            ProverResourceType::Gpu => {
                if !cfg!(feature = "cuda") && !cfg!(feature = "metal") {
                    panic!("GPU selected but neither 'cuda' nor 'metal' feature is enabled");
                }
                if cfg!(feature = "cuda") && cfg!(feature = "metal") {
                    panic!("GPU selected but both 'cuda' and 'metal' feature are enabled");
                }

                #[cfg(feature = "cuda")]
                prove::bento::docker_compose_bento_up()?;
            }
            ProverResourceType::Network(_) => {
                panic!(
                    "Network proving not yet implemented for RISC Zero. Use CPU or GPU resource type."
                );
            }
        }

        Ok(Self {
            program,
            resource_type,
        })
    }
}

impl zkVM for EreRisc0 {
    fn execute(&self, inputs: &Input) -> Result<ProgramExecutionReport, zkVMError> {
        let executor = default_executor();
        let mut env = ExecutorEnv::builder();
        for input in inputs.iter() {
            match input {
                InputItem::Object(serialize) => {
                    env.write(serialize).unwrap();
                }
                InputItem::Bytes(items) => {
                    env.write_frame(items);
                }
            }
        }
        let env = env.build().map_err(|err| zkVMError::Other(err.into()))?;

        let start = Instant::now();
        let session_info = executor
            .execute(env, &self.program.elf)
            .map_err(|err| zkVMError::Other(err.into()))?;
        Ok(ProgramExecutionReport {
            total_num_cycles: session_info.cycles() as u64,
            execution_duration: start.elapsed(),
            ..Default::default()
        })
    }

    fn prove(&self, inputs: &Input) -> Result<(Vec<u8>, ProgramProvingReport), zkVMError> {
        let (receipt, proving_time) = match self.resource_type {
            ProverResourceType::Cpu => prove::local::prove(&self.program, inputs)?,
            ProverResourceType::Gpu => {
                #[cfg(not(feature = "cuda"))]
                let result = prove::local::prove(&self.program, inputs)?;

                #[cfg(feature = "cuda")]
                let result = prove::bento::prove(&self.program, inputs)?;

                result
            }
            ProverResourceType::Network(_) => unreachable!(),
        };

        let encoded = borsh::to_vec(&receipt).map_err(|err| zkVMError::Other(Box::new(err)))?;
        Ok((encoded, ProgramProvingReport::new(proving_time)))
    }

    fn verify(&self, proof: &[u8]) -> Result<(), zkVMError> {
        let decoded: Receipt =
            borsh::from_slice(proof).map_err(|err| zkVMError::Other(Box::new(err)))?;

        decoded
            .verify(self.program.image_id)
            .map_err(|err| zkVMError::Other(Box::new(err)))
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn sdk_version(&self) -> &'static str {
        SDK_VERSION
    }
}

impl Drop for EreRisc0 {
    fn drop(&mut self) {
        #[cfg(feature = "cuda")]
        prove::bento::docker_compose_bento_down().unwrap_or_else(|err| eprintln!("{err}"))
    }
}

#[cfg(test)]
mod prove_tests {
    use std::path::PathBuf;

    use super::*;
    use zkvm_interface::Input;

    fn get_prove_test_guest_program_path() -> PathBuf {
        let workspace_dir = env!("CARGO_WORKSPACE_DIR");
        PathBuf::from(workspace_dir)
            .join("tests")
            .join("risczero")
            .join("compile")
            .join("project_structure_build")
            .canonicalize()
            .expect("Failed to find or canonicalize test Risc0 methods crate")
    }

    fn get_compiled_test_r0_elf_for_prove() -> Result<Risc0Program, RiscZeroError> {
        let test_guest_path = get_prove_test_guest_program_path();
        RV32_IM_RISCZERO_ZKVM_ELF::compile(&test_guest_path)
    }

    #[test]
    fn test_prove_r0_dummy_input() {
        let program = get_compiled_test_r0_elf_for_prove().unwrap();

        let mut input_builder = Input::new();
        let n: u32 = 42;
        let a: u16 = 42;
        input_builder.write(n);
        input_builder.write(a);

        let zkvm = EreRisc0::new(program, ProverResourceType::Cpu).unwrap();

        let (proof_bytes, _) = zkvm
            .prove(&input_builder)
            .unwrap_or_else(|err| panic!("Proving error in test: {err:?}"));

        assert!(!proof_bytes.is_empty(), "Proof bytes should not be empty.");

        let verify_results = zkvm.verify(&proof_bytes).is_ok();
        assert!(verify_results);

        // TODO: Check public inputs
    }

    #[test]
    // TODO: Note: SP1 will panic here
    // #[should_panic]
    fn test_prove_r0_fails_on_bad_input_causing_execution_failure() {
        let elf_bytes = get_compiled_test_r0_elf_for_prove().unwrap();

        let empty_input = Input::new();

        let zkvm = EreRisc0::new(elf_bytes, ProverResourceType::Cpu).unwrap();
        let prove_result = zkvm.prove(&empty_input);
        assert!(prove_result.is_err());
    }

    #[test]
    #[cfg(feature = "cuda")]
    fn test_prove_r0_dummy_input_cuda() {
        let elf_bytes = get_compiled_test_r0_elf_for_prove().unwrap();

        let empty_input = Input::new();

        let zkvm = EreRisc0::new(elf_bytes, ProverResourceType::Gpu).unwrap();
        let prove_result = zkvm.prove(&empty_input);
        assert!(prove_result.is_err());
    }
}

#[cfg(test)]
mod execute_tests {
    use std::path::PathBuf;

    use super::*;
    use zkvm_interface::Input;

    fn get_compiled_test_r0_elf() -> Result<Risc0Program, RiscZeroError> {
        let test_guest_path = get_execute_test_guest_program_path();
        RV32_IM_RISCZERO_ZKVM_ELF::compile(&test_guest_path)
    }

    fn get_execute_test_guest_program_path() -> PathBuf {
        let workspace_dir = env!("CARGO_WORKSPACE_DIR");
        PathBuf::from(workspace_dir)
            .join("tests")
            .join("risczero")
            .join("compile")
            .join("project_structure_build")
            .canonicalize()
            .expect("Failed to find or canonicalize test Risc0 methods crate")
    }

    #[test]
    fn test_execute_r0_dummy_input() {
        let program = get_compiled_test_r0_elf().unwrap();

        let mut input_builder = Input::new();
        let n: u32 = 42;
        let a: u16 = 42;
        input_builder.write(n);
        input_builder.write(a);

        let zkvm = EreRisc0::new(program, ProverResourceType::Cpu).unwrap();

        zkvm.execute(&input_builder)
            .unwrap_or_else(|err| panic!("Execution error: {err:?}"));
    }

    #[test]
    fn test_execute_r0_no_input_for_guest_expecting_input() {
        let program = get_compiled_test_r0_elf().unwrap();

        let empty_input = Input::new();

        let zkvm = EreRisc0::new(program, ProverResourceType::Cpu).unwrap();
        let result = zkvm.execute(&empty_input);

        assert!(
            result.is_err(),
            "execute should fail if guest expects input but none is provided."
        );
    }
}
