#!/bin/bash
set -e

# --- Utility functions (duplicated) ---
# Checks if a tool is installed and available in PATH.
is_tool_installed() {
    command -v "$1" &> /dev/null
}

# Ensures a tool is installed. Exits with an error if not.
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
# --- End of Utility functions ---

echo "Installing ZisK Toolchain and SDK using ziskup (prebuilt binaries)..."

# Prerequisites for ziskup and ZisK (some of these are for the SDK itself beyond ziskup)
ensure_tool_installed "curl" "to download the ziskup installer"
ensure_tool_installed "bash" "to run the ziskup installer"
ensure_tool_installed "rustup" "for managing Rust toolchains (ZisK installs its own)"
ensure_tool_installed "cargo" "as cargo-zisk is a cargo subcommand"

# Step 1: Download and run the script that installs the ziskup binary itself.
ZISKUP_INSTALLER_DOWNLOAD_PATH="/tmp/ziskup_install.sh"
echo "Downloading the script that installs the 'ziskup' binary to ${ZISKUP_INSTALLER_DOWNLOAD_PATH}..."
curl -L https://raw.githubusercontent.com/0xPolygonHermez/zisk/main/ziskup/install.sh -o "${ZISKUP_INSTALLER_DOWNLOAD_PATH}"
chmod +x "${ZISKUP_INSTALLER_DOWNLOAD_PATH}"

echo "Running the 'ziskup' binary installer script..."
"${ZISKUP_INSTALLER_DOWNLOAD_PATH}" # This script installs $HOME/.zisk/bin/ziskup
rm "${ZISKUP_INSTALLER_DOWNLOAD_PATH}" # Clean up the downloaded installer script

# Step 2: Ensure the installed ziskup binary is in PATH for this script session.
ZISK_BIN_DIR="${HOME}/.zisk/bin"
if [ -d "${ZISK_BIN_DIR}" ] && [[ ":$PATH:" != *":${ZISK_BIN_DIR}:"* ]]; then
    echo "Adding ${ZISK_BIN_DIR} to PATH for current script session."
    export PATH="${ZISK_BIN_DIR}:$PATH"
fi

# Ensure the ziskup binary itself is now installed and executable
ensure_tool_installed "ziskup" "ZisK SDK manager tool"

# Step 3: Now run the installed ziskup binary with non-interactive flags.
echo "Running 'ziskup --provingkey' to install ZisK components..."
# Export GH_RUNNER=true to ensure ziskup uses default non-interactive options.
export GH_RUNNER=true
ziskup --provingkey # This is the actual ziskup tool that processes --provingkey
unset GH_RUNNER

# Verify ZisK installation
echo "Verifying ZisK installation..."

echo "Checking for 'zisk' toolchain..."
if rustup toolchain list | grep -q "^zisk"; then
    echo "ZisK Rust toolchain found."
else
    echo "Error: ZisK Rust toolchain ('zisk') not found after installation!" >&2
    echo "       Attempting to run 'ziskup' again to ensure toolchain setup..."
    # Sometimes ziskup might need a second run or explicit toolchain setup if path issues occurred initially
    # However, for a script, we expect the first run to succeed. This is more of a diagnostic.
    if command -v ziskup &> /dev/null; then ziskup; fi
    if ! rustup toolchain list | grep -q "^zisk"; then
      echo "Critical Error: ZisK Rust toolchain still not found!" >&2
      exit 1
    fi
fi

echo "Checking for cargo-zisk CLI tool (using +zisk toolchain)..."
# cargo-zisk should be callable via `cargo +zisk zisk ...`
if cargo +zisk zisk --version; then
    echo "cargo-zisk CLI tool verified successfully."
else
    echo "Error: 'cargo +zisk zisk --version' failed." >&2
    echo "       Attempting verification with cargo-zisk directly (if in PATH from ${ZISK_BIN_DIR})..."
    if command -v cargo-zisk &> /dev/null && cargo-zisk --version; then
        echo "cargo-zisk found directly in PATH and verified."
    else
        echo "Error: cargo-zisk also not found directly or 'cargo +zisk zisk --version' failed." >&2
        echo "       Ensure ${ZISK_BIN_DIR} is effectively in PATH for new shells and check ziskup output." >&2
        exit 1
    fi
fi