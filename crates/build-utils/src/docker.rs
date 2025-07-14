use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use thiserror::Error;
use tracing::info;

pub fn build_image(dockerfile_relative_path: &str, tag: &str) -> Result<(), Error> {
    // Check that Docker is installed and available
    if Command::new("docker")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        return Err(Error::DockerIsNotAvailable);
    }

    // Build the Docker image
    let cargo_workspace_dir = env!("CARGO_WORKSPACE_DIR");

    // Build base image
    info!("Building base Docker image...");
    let dockerfile_base_path =
        PathBuf::from(cargo_workspace_dir).join("docker/base/Dockerfile.base");

    info!("Building base Dockerfile at: {:?}", dockerfile_base_path);
    let status = Command::new("docker")
        .args([
            "build",
            "-t",
            "ere-base:latest",
            "-f",
            dockerfile_base_path
                .to_str()
                .ok_or_else(|| Error::InvalidDockerfilePath(dockerfile_base_path.clone()))?,
            cargo_workspace_dir,
        ])
        .status()
        .map_err(|e| Error::DockerBuildFailed(e.into()))?;
    if !status.success() {
        return Err(Error::ImageBuildFailed);
    }

    let dockerfile_path = PathBuf::from(cargo_workspace_dir).join(dockerfile_relative_path);
    let status = Command::new("docker")
        .args([
            "build",
            "-q",
            "-t",
            tag,
            "-f",
            dockerfile_path
                .to_str()
                .ok_or_else(|| Error::InvalidDockerfilePath(dockerfile_path.clone()))?,
            cargo_workspace_dir,
        ])
        .status()
        .map_err(|e| Error::DockerBuildFailed(e.into()))?;

    if !status.success() {
        return Err(Error::ImageBuildFailed);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Dockerfile path: {0}")]
    InvalidDockerfilePath(PathBuf),
    #[error("Docker image build failed: {0}")]
    DockerBuildFailed(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Docker image build failed")]
    ImageBuildFailed,
    #[error("Docker is not available. Please ensure Docker is installed and running.")]
    DockerIsNotAvailable,
}
