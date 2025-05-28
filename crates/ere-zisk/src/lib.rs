use crate::{
    compile::compile_zisk_program,
    error::{ExecuteError, ZiskError},
};
use std::path::{Path, PathBuf};
use zisk_core::Riscv2zisk;
use ziskemu::{Emu, EmuOptions};
use zkvm_interface::{
    Compiler, Input, ProgramExecutionReport, ProgramProvingReport, zkVM, zkVMError,
};

mod compile;
mod error;

#[allow(non_camel_case_types)]
pub struct RV64_IMA_ZISK_ZKVM_ELF;

impl Compiler for RV64_IMA_ZISK_ZKVM_ELF {
    type Error = ZiskError;

    /// Path to compiled ELF.
    type Program = PathBuf;

    fn compile(path_to_program: &Path) -> Result<Self::Program, Self::Error> {
        compile_zisk_program(path_to_program).map_err(ZiskError::Compile)
    }
}

pub struct EreZisk {
    elf_path: PathBuf,
}

impl EreZisk {
    pub fn new(elf_path: PathBuf) -> Self {
        Self { elf_path }
    }
}

impl zkVM for EreZisk {
    fn execute(&self, input: &Input) -> Result<ProgramExecutionReport, zkVMError> {
        // Convert elf to ZisK ROM.
        let rom = Riscv2zisk::new(self.elf_path.clone()).run().map_err(|e| {
            // FIXME: Error to string because error is not `Send` nor `Sync`.
            ZiskError::Execute(ExecuteError::Riscv2ziskFailed(e.to_string()))
        })?;

        // Serialize input and concat all bytes into single vector.
        let inputs = input
            .iter()
            .try_fold(Vec::new(), |mut acc, item| {
                acc.extend(item.as_bytes().map_err(ExecuteError::SerializeInput)?);
                Ok(acc)
            })
            .map_err(ZiskError::Execute)?;

        // Turn on `stats` to get cycle count.
        let options = EmuOptions {
            stats: true,
            ..Default::default()
        };

        // Execute
        let mut emu = Emu::new(&rom);
        emu.run(inputs, &options, None::<Box<dyn Fn(_)>>);

        if !emu.terminated() {
            return Err(ZiskError::Execute(ExecuteError::EmulationNotTerminate).into());
        }

        // FIXME: An ugly hack to get cycle count because `emu.ctx.stats`
        //        doesn't have an public getter for it.
        let total_num_cycles = emu
            .ctx
            .stats
            .report()
            .split_once("total steps = ")
            .and_then(|(_, stats)| {
                stats
                    .split_whitespace()
                    .next()
                    .and_then(|steps| steps.parse::<u64>().ok())
            })
            .ok_or(ZiskError::Execute(ExecuteError::TotalStepsNotFound))?;

        Ok(ProgramExecutionReport::new(total_num_cycles))
    }

    fn prove(&self, _: &Input) -> Result<(Vec<u8>, ProgramProvingReport), zkVMError> {
        todo!()
    }

    fn verify(&self, _: &[u8]) -> Result<(), zkVMError> {
        todo!()
    }
}

#[cfg(test)]
mod execute_tests {
    use super::*;

    fn get_compiled_test_zisk_elf() -> Result<PathBuf, ZiskError> {
        let test_guest_path = get_execute_test_guest_program_path();
        RV64_IMA_ZISK_ZKVM_ELF::compile(&test_guest_path)
    }

    fn get_execute_test_guest_program_path() -> PathBuf {
        let workspace_dir = env!("CARGO_WORKSPACE_DIR");
        PathBuf::from(workspace_dir)
            .join("tests")
            .join("zisk")
            .join("execute")
            .join("basic")
            .canonicalize()
            .expect("Failed to find or canonicalize test guest program at <CARGO_WORKSPACE_DIR>/tests/execute/zisk")
    }

    #[test]
    fn test_execute_zisk_dummy_input() {
        let elf_path = get_compiled_test_zisk_elf()
            .expect("Failed to compile test ZisK guest for execution test");

        let mut input_builder = Input::new();
        let n: u32 = 42;
        let a: u16 = 42;
        input_builder.write(n);
        input_builder.write(a);

        let zkvm = EreZisk::new(elf_path);

        let result = zkvm.execute(&input_builder);

        if let Err(e) = &result {
            panic!("Execution error: {:?}", e);
        }
    }

    // FIXME: The emulator panics due to some memory reading issue, but ideally
    //        it shouldn't panic but instead returning error indicating the
    //        non-zero exit code.
    #[test]
    #[should_panic]
    fn test_execute_zisk_no_input_for_guest_expecting_input() {
        let elf_path = get_compiled_test_zisk_elf()
            .expect("Failed to compile test ZisK guest for execution test");

        let empty_input = Input::new();

        let zkvm = EreZisk::new(elf_path);
        let _ = zkvm.execute(&empty_input);
    }
}
