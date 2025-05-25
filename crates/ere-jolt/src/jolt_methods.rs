use zkvm_interface::Input;

pub fn preprocess_prover(
    program: &jolt::host::Program,
) -> jolt::JoltProverPreprocessing<4, jolt::F, jolt::PCS, jolt::ProofTranscript> {
    use jolt::{Jolt, JoltProverPreprocessing, MemoryLayout, RV32IJoltVM};
    let (bytecode, memory_init) = program.decode();
    let memory_layout = MemoryLayout::new(4096, 4096);
    let preprocessing: JoltProverPreprocessing<4, jolt::F, jolt::PCS, jolt::ProofTranscript> =
        RV32IJoltVM::prover_preprocess(
            bytecode,
            memory_layout,
            memory_init,
            1 << 20,
            1 << 20,
            1 << 24,
        );
    preprocessing
}

pub fn preprocess_verifier(
    program: &jolt::host::Program,
) -> jolt::JoltVerifierPreprocessing<4, jolt::F, jolt::PCS, jolt::ProofTranscript> {
    use jolt::{Jolt, JoltVerifierPreprocessing, MemoryLayout, RV32IJoltVM};

    let (bytecode, memory_init) = program.decode();
    let memory_layout = MemoryLayout::new(4096, 4096);
    let preprocessing: JoltVerifierPreprocessing<4, jolt::F, jolt::PCS, jolt::ProofTranscript> =
        RV32IJoltVM::verifier_preprocess(
            bytecode,
            memory_layout,
            memory_init,
            1 << 20,
            1 << 20,
            1 << 24,
        );
    preprocessing
}

pub fn verify_generic(
    proof: jolt::JoltHyperKZGProof,
    // TODO: input should be private input
    _inputs: Input,
    _outputs: Input,
    preprocessing: jolt::JoltVerifierPreprocessing<4, jolt::F, jolt::PCS, jolt::ProofTranscript>,
) -> bool {
    use jolt::{Jolt, RV32IJoltVM, tracer};

    let preprocessing = std::sync::Arc::new(preprocessing);
    let preprocessing = (*preprocessing).clone();
    let io_device = tracer::JoltDevice::new(
        preprocessing.memory_layout.max_input_size,
        preprocessing.memory_layout.max_output_size,
    );

    // TODO: FIXME
    // io_device.inputs = inputs.bytes().to_vec();
    // io_device.outputs = outputs.bytes().to_vec();

    RV32IJoltVM::verify(
        preprocessing,
        proof.proof,
        proof.commitments,
        io_device,
        None,
    )
    .is_ok()
}

pub fn prove_generic(
    program: &jolt::host::Program,
    preprocessing: jolt::JoltProverPreprocessing<4, jolt::F, jolt::PCS, jolt::ProofTranscript>,
    _inputs: &Input,
) -> (Vec<u8>, jolt::JoltHyperKZGProof) {
    use jolt::{Jolt, RV32IJoltVM};

    let mut program = program.clone();

    // Convert inputs to a flat vector
    // TODO: FIXME
    let input_bytes = Vec::new();

    let (io_device, trace) = program.trace(&input_bytes);

    let (jolt_proof, jolt_commitments, output_io_device, _) =
        RV32IJoltVM::prove(io_device, trace, preprocessing);

    let proof = jolt::JoltHyperKZGProof {
        proof: jolt_proof,
        commitments: jolt_commitments,
    };
    (output_io_device.outputs.clone(), proof)
}
