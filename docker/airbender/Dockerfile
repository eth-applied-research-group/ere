ARG BASE_IMAGE_TAG=latest
FROM ere-base:${BASE_IMAGE_TAG}

ARG USERNAME=ere_user

# Ensure Cargo/Rustup environment variables are set from the base image for SDK script
# TODO: These should be inherited from ere-base.
ENV RUSTUP_HOME=${RUSTUP_HOME:-/usr/local/rustup} \
    CARGO_HOME=${CARGO_HOME:-/usr/local/cargo} \
    PATH=${PATH:-/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin}

# Install the RISC-V target required for airbender guest programs
# Airbender guest programs require nightly features
RUN rustup install nightly && \
    rustup component add --toolchain nightly rust-src && \
    rustup target add --toolchain nightly riscv32i-unknown-none-elf && \
    # Also add for stable toolchain in case it's needed
    rustup target add riscv32i-unknown-none-elf

# Copy the entire ere project context
# The WORKDIR is /app from the base image
WORKDIR /app
COPY . .

# Run tests
RUN echo "Running tests for ere-airbender library..." && \
    cargo test --release -p ere-airbender --lib -- --color always

# Optional: Keep container running for debugging
# CMD ["/bin/bash"]
