use std::{path::PathBuf, process::ExitStatus};

use thiserror::Error;
use zkvm_interface::zkVMError;

impl From<AirbenderError> for zkVMError {
    fn from(value: AirbenderError) -> Self {
        zkVMError::Other(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum AirbenderError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    Execute(#[from] ExecuteError),

    #[error(transparent)]
    Prove(#[from] ProveError),

    #[error(transparent)]
    Verify(#[from] VerifyError),
}

/// Errors that can be encountered while compiling a Airbender program
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Program path does not exist or is not a directory: {0}")]
    InvalidProgramPath(PathBuf),
    #[error(
        "Cargo.toml not found in program directory: {program_dir}. Expected at: {manifest_path}"
    )]
    CargoTomlMissing {
        program_dir: PathBuf,
        manifest_path: PathBuf,
    },
    #[error("Could not find `[package].name` in guest Cargo.toml at {path}")]
    MissingPackageName { path: PathBuf },
    #[error("Compiled ELF not found at expected path: {0}")]
    ElfNotFound(PathBuf),
    #[error("`cargo prove build` failed with status: {status} for program at {path}")]
    CargoBuildFailed { status: ExitStatus, path: PathBuf },
    #[error("Failed to read file at {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse guest Cargo.toml at {path}: {source}")]
    ParseCargoToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("Failed to execute `cargo prove build` in {cwd}: {source}")]
    CargoProveBuild {
        cwd: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to create temporary output directory: {0}")]
    TempDir(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to run airbender-cli: {0}")]
    CliCommand(std::io::Error),

    #[error("Airbender CLI execution failed: {0}")]
    CliFailed(String),
}

#[derive(Debug, Error)]
pub enum ProveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to run airbender-cli: {0}")]
    CliCommand(std::io::Error),

    #[error("Airbender CLI proving failed: {0}")]
    CliFailed(String),

    #[error("Failed to read proof from {0}: {1}")]
    ReadProof(PathBuf, std::io::Error),

    #[error("Serialising proof with `bincode` failed: {0}")]
    Bincode(#[from] bincode::Error),
}

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to run airbender-cli: {0}")]
    CliCommand(std::io::Error),

    #[error("Airbender CLI verification failed: {0}")]
    CliFailed(String),

    #[error("Deserialising proof failed: {0}")]
    Bincode(#[from] bincode::Error),
}
