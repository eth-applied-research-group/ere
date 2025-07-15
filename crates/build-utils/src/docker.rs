use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use thiserror::Error;
use tracing::info;

pub fn build_image(dockerfile_workspace_relative_path: &str, tag: &str) -> Result<(), Error> {
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

    let cargo_workspace_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();

    // Build base image
    info!("Building base Docker image...");
    let dockerfile_base_path = cargo_workspace_dir.join("docker/base/Dockerfile.base");
    let status = Command::new("docker")
        .args([
            "build",
            "-t",
            "ere-base:latest",
            "-f",
            dockerfile_base_path
                .to_str()
                .ok_or_else(|| Error::InvalidDockerfilePath(dockerfile_base_path.clone()))?,
            cargo_workspace_dir.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| Error::DockerBuildFailed(e.into()))?;
    if !status.success() {
        return Err(Error::ImageBuildFailed);
    }

    let dockerfile_path = cargo_workspace_dir.join(dockerfile_workspace_relative_path);
    let status = Command::new("docker")
        .args([
            "build",
            "-t",
            tag,
            "-f",
            dockerfile_path
                .to_str()
                .ok_or_else(|| Error::InvalidDockerfilePath(dockerfile_path.clone()))?,
            cargo_workspace_dir.to_str().unwrap(),
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
