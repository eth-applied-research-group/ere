use zkvm_interface::zkVMError;

impl From<JoltError> for zkVMError {
    fn from(value: JoltError) -> Self {
        zkVMError::Other(Box::new(value))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JoltError {
    #[error("Proof verification failed")]
    ProofVerificationFailed,
}
