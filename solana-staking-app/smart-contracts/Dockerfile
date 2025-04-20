# Base image with Rust toolchain
FROM rust:1.72-slim-bookworm as builder

# Install essential packages
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    python3 \
    clang \
    libclang-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Install Solana CLI tools
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.17.2/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install Anchor (Solana smart contract framework)
RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
RUN avm install 0.29.0
RUN avm use 0.29.0

# Create app directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock files
COPY Cargo.toml .
COPY Cargo.lock* .

# Create an empty lib.rs file to trick Cargo into building dependencies
RUN mkdir -p src && \
    echo "// Empty file to build dependencies" > src/lib.rs

# Build dependencies to cache them in a separate Docker layer
RUN cargo build --release && \
    rm src/lib.rs

# Copy source code
COPY . .

# Build the smart contract
RUN cargo build-sbf --release

# Create runtime image with Solana tools
FROM ubuntu:22.04 as runtime

# Install dependencies for running Solana tools
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libudev-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Solana CLI tools
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.17.2/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Create app directory
WORKDIR /app

# Copy built smart contract from builder stage
COPY --from=builder /app/target/sbf-solana-solana/release/*.so /app/
COPY scripts /app/scripts

# Make scripts executable
RUN chmod +x /app/scripts/*.sh

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV SOLANA_CLUSTER="https://api.devnet.solana.com"

# Display Solana version and cluster info when container starts
CMD ["bash", "-c", "echo 'Solana version:' && solana --version && echo 'Connected to cluster:' && solana config get && echo 'Run deploy.sh to deploy the smart contract'"]