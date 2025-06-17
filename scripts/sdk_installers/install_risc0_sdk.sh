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

echo "Installing Risc0 Toolchain v3.0.0-rc.1 from source..."

ensure_tool_installed "git" "to clone the risc0 repository"
ensure_tool_installed "cargo" "to build and install risc0 tools"

# Clone risc0 repository
RISC0_INSTALL_DIR="/tmp/risc0-install-$$"
echo "Cloning risc0 repository to ${RISC0_INSTALL_DIR}..."
git clone https://github.com/risc0/risc0.git "${RISC0_INSTALL_DIR}"

# Checkout the specific version tag
cd "${RISC0_INSTALL_DIR}"
echo "Checking out version v3.0.0-rc.1..."
git checkout v3.0.0-rc.1

# Install rzup from source
echo "Installing rzup from source..."
cargo install --path rzup

# Add rzup to PATH if needed
RZUP_BIN_DIR="${HOME}/.cargo/bin"
if [ -d "${RZUP_BIN_DIR}" ] && [[ ":$PATH:" != *":${RZUP_BIN_DIR}:"* ]]; then
    echo "Adding ${RZUP_BIN_DIR} to PATH for current script session."
    export PATH="${RZUP_BIN_DIR}:$PATH"
fi

# Build rust toolchain
echo "Building rust toolchain..."
rzup toolchain build rust

# Install cargo-risczero
echo "Installing cargo-risczero..."
cargo install --path risc0/cargo-risczero

# Clean up
cd /
rm -rf "${RISC0_INSTALL_DIR}"
echo "Cleaned up temporary installation directory."

# Verify Risc0 installation
echo "Verifying Risc0 installation..."
ensure_tool_installed "cargo" "as cargo-risczero needs it"
cargo risczero --version || (echo "Error: cargo risczero command failed!" >&2 && exit 1)

echo "Risc0 Toolchain installation (latest release) successful."
echo "The rzup installer might have updated your shell configuration files (e.g., ~/.bashrc, ~/.zshrc)."
echo "To ensure rzup and Risc0 tools are available in your current shell session if this was a new installation,"
echo "you may need to source your shell profile (e.g., 'source ~/.bashrc') or open a new terminal." 