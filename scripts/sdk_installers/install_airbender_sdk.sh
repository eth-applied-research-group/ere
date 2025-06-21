#!/bin/bash
set -euo pipefail

echo "Installing Airbender SDK dependencies..."

# Install system dependencies
apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler

# Clone and build Airbender CLI
echo "Building Airbender CLI from source..."
git clone https://github.com/matter-labs/zksync-airbender.git /tmp/airbender
cd /tmp/airbender

# Build the CLI tool with verifiers feature
echo "Building CLI with verifiers..."
cargo build --release -p cli --features include_verifiers

# Install the CLI globally
cp target/release/cli /usr/local/bin/airbender-cli

# Clean up
cd /
rm -rf /tmp/airbender

echo "Airbender CLI installed successfully"
airbender-cli --version || echo "Airbender CLI installation complete"