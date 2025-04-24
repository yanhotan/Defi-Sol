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
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install Solana CLI tools
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.17.2/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install Anchor using NPM
RUN npm install -g @coral-xyz/anchor-cli@0.26.0

# Create workspace directory
WORKDIR /workspace

# The source code will be mounted as a volume
CMD ["bash"]