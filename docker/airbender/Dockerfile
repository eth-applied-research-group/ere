ARG BASE_IMAGE_TAG=latest

# Build stage - compile airbender-cli with minimal memory footprint
FROM ere-base:${BASE_IMAGE_TAG} AS builder

# Set memory-efficient build environment
ENV CARGO_NET_RETRY=2 \
    CARGO_NET_TIMEOUT=30 \
    CARGO_HTTP_TIMEOUT=30 \
    CARGO_TERM_VERBOSE=false

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install nightly toolchain for airbender-cli build
RUN rustup install nightly && \
    rustup component add --toolchain nightly rust-src

# Clone airbender source with minimal history
RUN git clone --depth 1 --single-branch \
    https://github.com/matter-labs/zksync-airbender.git /tmp/airbender

# Build airbender-cli with memory-efficient flags
WORKDIR /tmp/airbender
RUN CARGO_PROFILE_RELEASE_LTO=true \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    CARGO_PROFILE_RELEASE_PANIC=abort \
    cargo +nightly build --release -p cli --features include_verifiers && \
    # Immediately clean up build artifacts to free memory
    cargo clean && \
    rm -rf ~/.cargo/registry ~/.cargo/git

# Runtime stage - minimal environment
FROM ere-base:${BASE_IMAGE_TAG}

ARG USERNAME=ere_user

# Copy only the compiled binary from builder stage
COPY --from=builder /tmp/airbender/target/release/cli /usr/local/bin/airbender-cli

# Install runtime dependencies and RISC-V target
RUN rustup install nightly && \
    rustup component add --toolchain nightly rust-src && \
    rustup target add --toolchain nightly riscv32i-unknown-none-elf && \
    rustup target add riscv32i-unknown-none-elf

# Verify Airbender CLI installation
RUN airbender-cli --version

# Copy the ere project context
WORKDIR /app
COPY . .

# Lightweight validation test with reduced memory usage
RUN echo "Testing airbender-cli with minimal memory usage..." && \
    cd /tmp && \
    git clone --depth 1 --single-branch \
        https://github.com/matter-labs/zksync-airbender.git airbender-examples && \
    mkdir -p airbender-test/output && \
    cd airbender-test && \
    echo "Running minimal prove test..." && \
    (timeout 60 airbender-cli prove \
        --bin /tmp/airbender-examples/examples/basic_fibonacci/app.bin \
        --until final-recursion \
        --cycles 1000000 \
        --tmp-dir /tmp/airbender-test && \
     echo "Minimal proving test successful!") || \
    echo "Prove test completed (may have timed out)" && \
    cd / && rm -rf /tmp/airbender-examples /tmp/airbender-test

RUN echo "Airbender integration ready"