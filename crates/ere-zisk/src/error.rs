use std::{io, path::PathBuf, process::ExitStatus};
use thiserror::Error;
use zkvm_interface::zkVMError;

impl From<ZiskError> for zkVMError {
    fn from(value: ZiskError) -> Self {
        zkVMError::Other(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ZiskError {
    #[error(transparent)]
    Compile(#[from] CompileError),

    #[error(transparent)]
    Execute(#[from] ExecuteError),
}

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
    #[error("Compiled ELF not found at expected path: {path}")]
    ElfNotFound {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("`cargo-zisk build --release` failed with status: {status} for program at {path}")]
    CargoZiskBuildFailed { status: ExitStatus, path: PathBuf },
    #[error("Failed to read file at {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to parse guest Cargo.toml at {path}: {source}")]
    ParseCargoToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("Failed to execute `cargo-zisk build --release` in {cwd}: {source}")]
    CargoZiskBuild {
        cwd: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("Failed to serialize input: {0}")]
    SerializeInput(Box<dyn std::error::Error + Send + Sync>),
    #[error("Failed to convert ELF to ZisK ROM: {0}")]
    Riscv2ziskFailed(String),
    #[error("Emulation doesn't terminate")]
    EmulationNotTerminate,
    #[error("Total steps not found in report")]
    TotalStepsNotFound,
}
