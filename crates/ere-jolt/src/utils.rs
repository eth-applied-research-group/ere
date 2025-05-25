use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError};
use jolt::JoltHyperKZGProof;
use std::io::Cursor;
use std::{fs, path::Path};
use toml::Value;

use crate::JoltError;

/// Reads the `[package] name` out of a Cargo.toml.
///
/// * `manifest_path` – absolute or relative path to a Cargo.toml.
/// * Returns → `String` with the package name (`fib`, `my_guest`, …).
pub(crate) fn package_name_from_manifest(manifest_path: &Path) -> Result<String, JoltError> {
    let manifest = fs::read_to_string(manifest_path).unwrap();
    let value: Value = manifest.parse::<Value>().unwrap();

    value
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(Value::as_str)
        .map(|s| s.to_owned())
        .ok_or_else(|| panic!("no [package] name found in {}", manifest_path.display()))
}

/// Serializes the public input (as raw bytes) and proof into a single byte vector
pub fn serialize_public_input_with_proof(
    public_input: &[u8],
    proof: &JoltHyperKZGProof,
) -> Result<Vec<u8>, SerializationError> {
    let mut buffer = Vec::new();

    // First, serialize the length of the public input as u64
    let public_input_size = public_input.len() as u64;
    public_input_size.serialize_compressed(&mut buffer)?;

    // Append the public input directly (it's already bytes)
    buffer.extend_from_slice(public_input);

    // Now serialize the proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)?;

    // Append the serialized proof to the buffer
    buffer.extend_from_slice(&proof_bytes);

    Ok(buffer)
}

/// Deserializes a byte vector into a public input (Vec<u8>) and proof
pub fn deserialize_public_input_with_proof(
    bytes: &[u8],
) -> Result<(Vec<u8>, JoltHyperKZGProof), SerializationError> {
    let mut cursor = Cursor::new(bytes);

    // Read the size of the public input
    let public_input_size: u64 = CanonicalDeserialize::deserialize_compressed(&mut cursor)?;

    // Get the current position after reading the size
    let current_position = cursor.position() as usize;
    let public_input_end = current_position + public_input_size as usize;

    if public_input_end > bytes.len() {
        return Err(SerializationError::InvalidData);
    }

    // Extract the public input bytes directly
    let public_input = bytes[current_position..public_input_end].to_vec();

    // The rest is the proof
    let proof_bytes = &bytes[public_input_end..];
    let proof = JoltHyperKZGProof::deserialize_compressed(&mut Cursor::new(proof_bytes))?;

    Ok((public_input, proof))
}
