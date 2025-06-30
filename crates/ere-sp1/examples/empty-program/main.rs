use std::{env, path::PathBuf};

use ere_succinct::{EreSP1, RV32_IM_SUCCINCT_ZKVM_ELF};
use zkvm_interface::{Compiler, Input, ProverResourceType, zkVM};

fn main() {
    // TODO/temp: use clap -- we need to not rely on Rust-examples and be its own example crate at
    // the workspace level.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} [--cpu || --gpu]", args[0]);
        return;
    }
    let resource = match args[1].as_str() {
        "--cpu" => ProverResourceType::Cpu,
        "--gpu" => ProverResourceType::Gpu,
        _ => {
            eprintln!("Invalid argument: {}. Use --cpu or --gpu.", args[1]);
            return;
        }
    };

    let test_guest_path = PathBuf::from(env!("CARGO_WORKSPACE_DIR"))
            .join("tests")
            .join("sp1")
            .join("prove")
            .join("empty")
            .canonicalize()
            .expect("Failed to find or canonicalize test guest program at <CARGO_WORKSPACE_DIR>/tests/prove/sp1");

    let elf_bytes = RV32_IM_SUCCINCT_ZKVM_ELF::compile(&test_guest_path)
        .expect("Failed to compile test SP1 guest for execution test");

    let zkvm = EreSP1::new(elf_bytes, resource);

    let start = std::time::Instant::now();
    let result = zkvm.prove(&Input::new());
    match result {
        Ok(proof) => {
            let elapsed = start.elapsed();
            println!("Proving completed in: {:?}", elapsed);
            println!(
                "Proof size: {:.2} MiB",
                proof.0.len() as f64 / (1024.0 * 1024.0)
            );
        }
        Err(e) => {
            eprintln!("Error generating proof: {:?}", e);
            return;
        }
    }
}
