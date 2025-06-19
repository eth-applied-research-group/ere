ARG BASE_IMAGE_TAG=latest
FROM ere-base:${BASE_IMAGE_TAG}

# The ere-base image provides Rust, Cargo, and common tools.
# ZisK requires Ubuntu 22.04 or higher (ere-base uses 22.04 by default).
# We operate as root for SDK and dependency installation.

# Install ZisK system dependencies (for Ubuntu)
# Taken from https://0xpolygonhermez.github.io/zisk/getting_started/installation.html
RUN apt-get update && apt-get install -y --no-install-recommends \
    xz-utils \
    jq \
    # build-essential is in ere-base
    # curl is in ere-base
    # git is in ere-base
    qemu-system \
    libomp-dev \
    libgmp-dev \
    nlohmann-json3-dev \
    protobuf-compiler \
    uuid-dev \
    libgrpc++-dev \
    libsecp256k1-dev \
    libsodium-dev \
    libpqxx-dev \
    nasm \
    libopenmpi-dev \
    openmpi-bin \
    openmpi-common \
    libclang-dev \
    clang \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy the ZisK SDK installer script from the workspace context
COPY scripts/sdk_installers/install_zisk_sdk.sh /tmp/install_zisk_sdk.sh
RUN chmod +x /tmp/install_zisk_sdk.sh

# Run the ZisK SDK installation script using ziskup.
# This script installs the 'zisk' Rust toolchain and cargo-zisk.
RUN /tmp/install_zisk_sdk.sh && rm /tmp/install_zisk_sdk.sh # Clean up the script

# The 'zisk' Rust toolchain is now installed.
# cargo-zisk is installed in /root/.zisk/bin.
# The ziskup script adds /root/.zisk/bin to PATH for its session.
# For the image environment, we need to ensure /root/.zisk/bin is persistently in PATH.
ENV ZISK_BIN_DIR="/root/.zisk/bin"
ENV PATH="${PATH}:${ZISK_BIN_DIR}"

# Verify cargo-zisk is accessible
RUN echo "Verifying Zisk installation in Dockerfile ..." && cargo-zisk --version

## I guess we will move away from what's in the readme and just rely on Dockerfiles running comamnds like this?
## (cf the SP1 Dockerfile)
#
# Copy the entire ere project context
# The WORKDIR is /app from the base image
WORKDIR /app
COPY . .

# Run tests
RUN echo "Running tests for ere-zisk library..." && \
    cargo test --release -p ere-zisk --lib -- --color always

CMD ["/bin/bash"]
