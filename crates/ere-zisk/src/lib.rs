use crate::{compile::compile_zisk_program, error::ZiskError};
use std::path::{Path, PathBuf};
use zkvm_interface::Compiler;

mod compile;
mod error;

#[allow(non_camel_case_types)]
pub struct RV64_IMA_ZISK_ZKVM_ELF;

impl Compiler for RV64_IMA_ZISK_ZKVM_ELF {
    type Error = ZiskError;

    type Program = PathBuf;

    fn compile(path_to_program: &Path) -> Result<Self::Program, Self::Error> {
        compile_zisk_program(path_to_program).map_err(ZiskError::CompileError)
    }
}
