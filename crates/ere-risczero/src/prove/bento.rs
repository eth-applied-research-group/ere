use crate::Risc0Program;
use risc0_zkvm::{Receipt, VERSION, serde::to_vec};
use std::{
    env,
    ffi::OsStr,
    io::{self, Write},
    process::{Command, Stdio},
    time::Duration,
};
use zkvm_interface::{Input, InputItem, zkVMError};

const URL: &str = "http://localhost:8081";
const KEY: &str = "";

// Copied and modified from https://github.com/risc0/risc0/blob/main/bento/crates/bento-client/src/bento_cli.rs.
pub fn prove(program: &Risc0Program, inputs: &Input) -> Result<(Receipt, Duration), zkVMError> {
    let client =
        bonsai_sdk::blocking::Client::from_parts(URL.to_string(), KEY.to_string(), VERSION)
            .map_err(|err| zkVMError::Other(err.into()))?;

    // Serialize `inputs` in the same way `ExecutorEnv` does.
    let mut input_bytes = Vec::new();
    for input in inputs.iter() {
        match input {
            InputItem::Object(serialize) => {
                input_bytes.extend(bytemuck::cast_slice(&to_vec(serialize).unwrap()));
            }
            InputItem::Bytes(items) => {
                input_bytes.extend((items.len() as u32).to_le_bytes());
                input_bytes.extend(items);
            }
        }
    }

    client
        .upload_img(&program.image_id.to_string(), program.elf.clone())
        .map_err(|err| zkVMError::Other(err.into()))?;
    let input_id = client
        .upload_input(input_bytes)
        .map_err(|err| zkVMError::Other(err.into()))?;

    let now = std::time::Instant::now();

    let session = client
        .create_session(program.image_id.to_string(), input_id, vec![], false)
        .map_err(|err| zkVMError::Other(err.into()))?;

    loop {
        let res = session
            .status(&client)
            .map_err(|err| zkVMError::Other(err.into()))?;

        match res.status.as_ref() {
            "RUNNING" => {
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
            "SUCCEEDED" => {
                let receipt_bytes = client
                    .receipt_download(&session)
                    .map_err(|err| zkVMError::Other(err.into()))?;
                break Ok((bincode::deserialize(&receipt_bytes).unwrap(), now.elapsed()));
            }
            _ => {
                return Err(zkVMError::Other(
                    format!("Unexpected proving status: {}", res.status).into(),
                ));
            }
        }
    }
}

const BENTO_COMPOSE_FILE: &str = include_str!("../../compose.yml");

/// Execute `docker compose ... {command}` with `BENTO_COMPOSE_FILE`.
fn docker_compose_bento<I, S>(command: I) -> Result<(), io::Error>
where
    I: Clone + IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "INFO".to_string());
    let segment_size = env::var("SEGMENT_SIZE").unwrap_or_else(|_| "21".to_string());
    let risc0_keccak_po2 = env::var("RISC0_KECCAK_PO2").unwrap_or_else(|_| "17".to_string());
    let compose_file = BENTO_COMPOSE_FILE
        .replace("${RUST_LOG}", &rust_log)
        .replace("${SEGMENT_SIZE}", &segment_size)
        .replace("${RISC0_KECCAK_PO2}", &risc0_keccak_po2);

    let mut child = Command::new("docker")
        .args(["compose", "--file", "-"]) // Compose file from stdin.
        .args(command.clone())
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(compose_file.as_bytes())?;
    drop(stdin);

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "Failed to spawn `docker compose --file - ${}`",
            command
                .into_iter()
                .map(|s| s.as_ref().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )));
    }

    Ok(())
}

/// Execute `docker compose ... up --detach` with `BENTO_COMPOSE_FILE`.
pub fn docker_compose_bento_up() -> Result<(), zkVMError> {
    docker_compose_bento(["up", "--detach"]).map_err(|err| zkVMError::Other(Box::new(err)))
}

/// Execute `docker compose ... down --volumes` with `BENTO_COMPOSE_FILE`.
pub fn docker_compose_bento_down() -> Result<(), zkVMError> {
    docker_compose_bento(["down", "--volumes"]).map_err(|err| zkVMError::Other(Box::new(err)))
}
