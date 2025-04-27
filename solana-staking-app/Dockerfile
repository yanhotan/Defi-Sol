FROM rust:1.69

# Install build dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    python3 \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install Solana CLI tools - using an older, more stable version
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.14.17/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install Anchor using NPM - using a version compatible with Solana 1.14.17
RUN npm install -g @project-serum/anchor-cli@0.26.0

# Create workspace directory
WORKDIR /workspace

# Add a toolchain file to ensure consistent Rust versions
RUN rustup install 1.69.0
RUN rustup default 1.69.0
RUN rustup component add rust-src --toolchain 1.69.0

# Solana configuration
RUN solana config set --url http://solana-validator:8899

# The source code will be mounted as a volume
CMD ["bash"]