#!/bin/bash
set -e

# TODO: Pull this out into its own script file
# Common utility functions for shell scripts

# Checks if a tool is installed and available in PATH.
# Usage: is_tool_installed <tool_name>
# Returns 0 if found, 1 otherwise.
is_tool_installed() {
    command -v "$1" &> /dev/null
}

# Ensures a tool is installed. Exits with an error if not.
# Usage: ensure_tool_installed <tool_name> [optional_purpose_message]
# Example: ensure_tool_installed curl "to download files"
ensure_tool_installed() {
    local tool_name="$1"
    local purpose_message="$2"

    if ! is_tool_installed "${tool_name}"; then
        echo "Error: Required tool '${tool_name}' could not be found." >&2
        if [ -n "${purpose_message}" ]; then
            echo "       It is needed ${purpose_message}." >&2
        fi
        echo "       Please install it first and ensure it is in your PATH." >&2
        exit 1
    fi
} 

echo "Installing Risc0 Toolchain from git revision to match project dependencies..."

ensure_tool_installed "cargo" "to install Risc0 tools from git"
ensure_tool_installed "git" "to clone the Risc0 repository"

# Define the git revision that matches what's used in the project
# This ensures r0vm version matches the risc0-zkvm crate version used in Cargo.toml
RISC0_GIT_REV="352dea62857ba57331053cd0986a12c1a4708732"

echo "Installing cargo-risczero from git revision ${RISC0_GIT_REV}..."
echo "This will install both cargo-risczero and r0vm executables..."
cargo install --git https://github.com/risc0/risc0.git --rev ${RISC0_GIT_REV} --force cargo-risczero

# Verify Risc0 installation
echo "Verifying Risc0 installation..."
cargo risczero --version || (echo "Error: cargo risczero command failed!" >&2 && exit 1)
r0vm --version || (echo "Error: r0vm command failed!" >&2 && exit 1)

echo "Risc0 Toolchain installation from git revision ${RISC0_GIT_REV} successful."
echo "Both cargo-risczero and r0vm have been installed from the same git revision"
echo "to ensure compatibility with the risc0-zkvm crate used in this project." 