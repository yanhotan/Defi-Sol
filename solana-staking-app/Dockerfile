FROM rust:1.74

# Install build dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    python3 \
    clang \
    cmake \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Solana CLI tools
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.17.2/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install Anchor with specific version
RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
RUN avm install 0.29.0
RUN avm use 0.29.0

# Create workspace directory
WORKDIR /workspace

# The source code will be mounted as a volume
CMD ["bash"]