#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use compile::compile_sp1_program;
use sp1_sdk::{
    CpuProver, CudaProver, Prover, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin,
    SP1VerifyingKey,
};
use tracing::info;
use zkvm_interface::{
    Compiler, Input, InputItem, ProgramExecutionReport, ProgramProvingReport,
    ProverResourceType, zkVM, zkVMError,
};

mod compile;

mod error;
use error::{ExecuteError, ProveError, SP1Error, VerifyError};

enum ProverType {
    Cpu(CpuProver),
    Gpu(CudaProver),
}

impl ProverType {
    fn setup(
        &self,
        program: &<RV32_IM_SUCCINCT_ZKVM_ELF as Compiler>::Program,
    ) -> (SP1ProvingKey, SP1VerifyingKey) {
        match self {
            ProverType::Cpu(cpu_prover) => cpu_prover.setup(program),
            ProverType::Gpu(cuda_prover) => cuda_prover.setup(program),
        }
    }

    fn execute(
        &self,
        program: &<RV32_IM_SUCCINCT_ZKVM_ELF as Compiler>::Program,
        input: &SP1Stdin,
    ) -> Result<(sp1_sdk::SP1PublicValues, sp1_sdk::ExecutionReport), SP1Error> {
        let cpu_executor_builder = match self {
            ProverType::Cpu(cpu_prover) => cpu_prover.execute(program, input),
            ProverType::Gpu(cuda_prover) => cuda_prover.execute(program, input),
        };

        cpu_executor_builder
            .run()
            .map_err(|e| SP1Error::Execute(ExecuteError::Client(e.into())))
    }
    fn prove(
        &self,
        pk: &SP1ProvingKey,
        input: &SP1Stdin,
    ) -> Result<SP1ProofWithPublicValues, SP1Error> {
        match self {
            ProverType::Cpu(cpu_prover) => cpu_prover.prove(pk, input).core().run(),
            ProverType::Gpu(cuda_prover) => cuda_prover.prove(pk, input).core().run(),
        }
        .map_err(|e| SP1Error::Prove(ProveError::Client(e.into())))
    }

    fn verify(
        &self,
        proof: &SP1ProofWithPublicValues,
        vk: &SP1VerifyingKey,
    ) -> Result<(), SP1Error> {
        match self {
            ProverType::Cpu(cpu_prover) => cpu_prover.verify(proof, vk),
            ProverType::Gpu(cuda_prover) => cuda_prover.verify(proof, vk),
        }
        .map_err(|e| SP1Error::Verify(VerifyError::Client(e.into())))
    }
}

#[allow(non_camel_case_types)]
pub struct RV32_IM_SUCCINCT_ZKVM_ELF;
pub struct EreSP1 {
    program: <RV32_IM_SUCCINCT_ZKVM_ELF as Compiler>::Program,
    /// Proving key
    pk: SP1ProvingKey,
    /// Verification key
    vk: SP1VerifyingKey,
    /// Proof and Verification orchestrator
    client: ProverType,
}

impl Compiler for RV32_IM_SUCCINCT_ZKVM_ELF {
    type Error = SP1Error;

    type Program = Vec<u8>;

    fn compile(path_to_program: &std::path::Path) -> Result<Self::Program, Self::Error> {
        compile_sp1_program(path_to_program).map_err(SP1Error::from)
    }
}

impl EreSP1 {
    pub fn new(
        program: <RV32_IM_SUCCINCT_ZKVM_ELF as Compiler>::Program,
        resource: ProverResourceType,
    ) -> Self {
        let client = match resource {
            ProverResourceType::Cpu => ProverType::Cpu(ProverClient::builder().cpu().build()),
            ProverResourceType::Gpu => ProverType::Gpu(ProverClient::builder().cuda().build()),
        };
        let (pk, vk) = client.setup(&program);

        Self {
            program,
            client,
            pk,
            vk,
        }
    }
}

impl zkVM for EreSP1 {
    fn execute(
        &self,
        inputs: &Input,
    ) -> Result<zkvm_interface::ProgramExecutionReport, zkVMError> {
        let mut stdin = SP1Stdin::new();
        for input in inputs.iter() {
            match input {
                InputItem::Object(serialize) => stdin.write(serialize),
                InputItem::Bytes(items) => stdin.write_slice(items),
            }
        }

        let (_, exec_report) = self.client.execute(&self.program, &stdin)?;
        let total_num_cycles = exec_report.total_instruction_count();
        let region_cycles: indexmap::IndexMap<_, _> =
            exec_report.cycle_tracker.into_iter().collect();

        let mut ere_report = ProgramExecutionReport::new(total_num_cycles);
        ere_report.region_cycles = region_cycles;

        Ok(ere_report)
    }

    fn prove(
        &self,
        inputs: &zkvm_interface::Input,
    ) -> Result<(Vec<u8>, zkvm_interface::ProgramProvingReport), zkVMError> {
        info!("Generating proof…");

        let mut stdin = SP1Stdin::new();
        for input in inputs.iter() {
            match input {
                InputItem::Object(serialize) => stdin.write(serialize),
                InputItem::Bytes(items) => stdin.write_slice(items),
            };
        }

        let start = std::time::Instant::now();
        let proof_with_inputs = self.client.prove(&self.pk, &stdin)?;
        let proving_time = start.elapsed();

        let bytes = bincode::serialize(&proof_with_inputs)
            .map_err(|err| SP1Error::Prove(ProveError::Bincode(err)))?;

        Ok((bytes, ProgramProvingReport::new(proving_time)))
    }

    fn verify(&self, proof: &[u8]) -> Result<(), zkVMError> {
        info!("Verifying proof…");

        let proof: SP1ProofWithPublicValues = bincode::deserialize(proof)
            .map_err(|err| SP1Error::Verify(VerifyError::Bincode(err)))?;

        self.client
            .verify(&proof, &self.vk)
            .map_err(zkVMError::from)
    }
}

#[cfg(test)]
mod execute_tests {
    use std::path::PathBuf;

    use super::*;
    use zkvm_interface::Input;

    fn get_compiled_test_sp1_elf() -> Result<Vec<u8>, SP1Error> {
        let test_guest_path = get_execute_test_guest_program_path();
        RV32_IM_SUCCINCT_ZKVM_ELF::compile(&test_guest_path)
    }

    fn get_execute_test_guest_program_path() -> PathBuf {
        let workspace_dir = env!("CARGO_WORKSPACE_DIR");
        PathBuf::from(workspace_dir)
            .join("tests")
            .join("sp1")
            .join("execute")
            .join("basic")
            .canonicalize()
            .expect("Failed to find or canonicalize test guest program at <CARGO_WORKSPACE_DIR>/tests/execute/sp1")
    }

    #[test]
    fn test_execute_sp1_dummy_input() {
        let elf_bytes = get_compiled_test_sp1_elf()
            .expect("Failed to compile test SP1 guest for execution test");

        let mut input_builder = Input::new();
        let n: u32 = 42;
        let a: u16 = 42;
        input_builder.write(n);
        input_builder.write(a);

        let zkvm = EreSP1::new(elf_bytes, ProverResourceType::Cpu);

        let result = zkvm.execute(&input_builder);

        if let Err(e) = &result {
            panic!("Execution error: {:?}", e);
        }
    }

    #[test]
    fn test_execute_sp1_no_input_for_guest_expecting_input() {
        let elf_bytes = get_compiled_test_sp1_elf()
            .expect("Failed to compile test SP1 guest for execution test");

        let empty_input = Input::new();

        let zkvm = EreSP1::new(elf_bytes, ProverResourceType::Cpu);
        let result = zkvm.execute(&empty_input);

        assert!(
            result.is_err(),
            "execute should fail if guest expects input but none is provided."
        );
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
            .join("sp1")
            .join("prove")
            .join("basic")
            .canonicalize()
            .expect("Failed to find or canonicalize test guest program at <CARGO_WORKSPACE_DIR>/tests/execute/sp1")
    }

    fn get_compiled_test_sp1_elf_for_prove() -> Result<Vec<u8>, SP1Error> {
        let test_guest_path = get_prove_test_guest_program_path();
        RV32_IM_SUCCINCT_ZKVM_ELF::compile(&test_guest_path)
    }

    #[test]
    fn test_prove_sp1_dummy_input() {
        let elf_bytes = get_compiled_test_sp1_elf_for_prove()
            .expect("Failed to compile test SP1 guest for proving test");

        let mut input_builder = Input::new();
        let n: u32 = 42;
        let a: u16 = 42;
        input_builder.write(n);
        input_builder.write(a);

        let zkvm = EreSP1::new(elf_bytes, ProverResourceType::Cpu);

        let proof_bytes = match zkvm.prove(&input_builder) {
            Ok((prove_result, _)) => prove_result,
            Err(err) => {
                panic!("Proving error in test: {:?}", err);
            }
        };

        assert!(!proof_bytes.is_empty(), "Proof bytes should not be empty.");

        let verify_results = zkvm.verify(&proof_bytes).is_ok();
        assert!(verify_results);

        // TODO: Check public inputs
    }

    #[test]
    #[should_panic]
    fn test_prove_sp1_fails_on_bad_input_causing_execution_failure() {
        let elf_bytes = get_compiled_test_sp1_elf_for_prove()
            .expect("Failed to compile test SP1 guest for proving test");

        let empty_input = Input::new();

        let zkvm = EreSP1::new(elf_bytes, ProverResourceType::Cpu);
        let prove_result = zkvm.prove(&empty_input);
        assert!(prove_result.is_err())
    }
}
