use pico_sdk::client::DefaultProverClient;
use std::process::Command;
use zkvm_interface::{Compiler, ProgramExecutionReport, ProgramProvingReport, zkVM};

mod error;
use error::PicoError;

#[allow(non_camel_case_types)]
pub struct PICO_TARGET;

impl Compiler for PICO_TARGET {
    type Error = PicoError;

    type Program = Vec<u8>;

    fn compile(path: &std::path::Path) -> Result<Self::Program, Self::Error> {
        // 1. Check guest path
        if !path.exists() {
            return Err(PicoError::PathNotFound(path.to_path_buf()));
        }

        // 2. Run `cargo pico build`
        let status = Command::new("cargo")
            .current_dir(path)
            .env("RUST_LOG", "info")
            .args(["pico", "build"])
            .status()?; // From<io::Error> → Spawn

        if !status.success() {
            return Err(PicoError::CargoFailed { status });
        }

        // 3. Locate the ELF file
        let elf_path = path.join("elf/riscv32im-pico-zkvm-elf");

        if !elf_path.exists() {
            return Err(PicoError::ElfNotFound(elf_path));
        }

        // 4. Read the ELF file
        let elf_bytes = std::fs::read(&elf_path).map_err(|e| PicoError::ReadElf {
            path: elf_path,
            source: e,
        })?;

        Ok(elf_bytes)
    }
}

pub struct ErePico {
    program: <PICO_TARGET as Compiler>::Program,
}

impl zkVM<PICO_TARGET> for ErePico {
    type Error = PicoError;

    fn new(program_bytes: <PICO_TARGET as Compiler>::Program) -> Self {
        ErePico {
            program: program_bytes,
        }
    }

    fn execute(
        &self,
        inputs: &zkvm_interface::Input,
    ) -> Result<zkvm_interface::ProgramExecutionReport, Self::Error> {
        let client = DefaultProverClient::new(&self.program);

        let mut stdin = client.new_stdin_builder();
        for input in inputs.chunked_iter() {
            stdin.write_slice(input);
        }
        let num_cycles = client.emulate(stdin);

        Ok(ProgramExecutionReport::new(num_cycles))
    }

    fn prove(
        &self,
        inputs: &zkvm_interface::Input,
    ) -> Result<(Vec<u8>, zkvm_interface::ProgramProvingReport), Self::Error> {
        let client = DefaultProverClient::new(&self.program);

        let mut stdin = client.new_stdin_builder();
        for input in inputs.chunked_iter() {
            stdin.write_slice(input);
        }
        let now = std::time::Instant::now();
        let meta_proof = client.prove(stdin).expect("Failed to generate proof");
        let elapsed = now.elapsed();

        let mut proof_serialized = Vec::new();
        for p in meta_proof.0.proofs().iter() {
            bincode::serialize_into(&mut proof_serialized, p).unwrap();
        }
        for p in meta_proof.1.proofs().iter() {
            bincode::serialize_into(&mut proof_serialized, p).unwrap();
        }

        for p in meta_proof.0.pv_stream.iter() {
            bincode::serialize_into(&mut proof_serialized, p).unwrap();
        }
        for p in meta_proof.1.pv_stream.iter() {
            bincode::serialize_into(&mut proof_serialized, p).unwrap();
        }

        Ok((proof_serialized, ProgramProvingReport::new(elapsed)))
    }

    fn verify(&self, proof: &[u8]) -> Result<(), Self::Error> {
        let client = DefaultProverClient::new(&self.program);
        let _vk = client.riscv_vk();
        todo!("Verification method missing from sdk")
    }
}

#[cfg(test)]
mod tests {
    use crate::PICO_TARGET;
    use std::path::PathBuf;
    use zkvm_interface::Compiler;

    fn get_compile_test_guest_program_path() -> PathBuf {
        let workspace_dir = env!("CARGO_WORKSPACE_DIR");
        let path = PathBuf::from(workspace_dir)
            .join("tests")
            .join("pico")
            .join("compile")
            .join("basic")
            .join("app");

        println!(
            "Attempting to find test guest program at: {}",
            path.display()
        );
        println!("Workspace dir is: {}", workspace_dir);

        path.canonicalize()
            .expect("Failed to find or canonicalize test guest program at <CARGO_WORKSPACE_DIR>/tests/pico/compile/basic/app")
    }

    #[test]
    fn test_compile_trait() {
        let test_guest_path = get_compile_test_guest_program_path();
        println!("Using test guest path: {}", test_guest_path.display());

        match PICO_TARGET::compile(&test_guest_path) {
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
