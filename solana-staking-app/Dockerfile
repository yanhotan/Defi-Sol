FROM rust:1.73

# Install dependencies including libelf-dev and llvm
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    python3 \
    wget \
    libelf-dev \
    zlib1g-dev \
    llvm-dev \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash -
RUN apt-get update && apt-get install -y nodejs && rm -rf /var/lib/apt/lists/*

# Install Solana CLI tools
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* && \
    mkdir -p /root/.local/share/solana/install/ && \
    curl -sSfL --max-time 300 https://release.solana.com/v1.14.17/solana-release-x86_64-unknown-linux-gnu.tar.bz2 -o /tmp/solana.tar.bz2 && \
    tar -xjf /tmp/solana.tar.bz2 -C /root/.local/share/solana/install/ && \
    mv /root/.local/share/solana/install/solana-release/* /root/.local/share/solana/install/ && \
    rm /tmp/solana.tar.bz2

ENV PATH="/root/.local/share/solana/install/bin:${PATH}"

# Install Anchor CLI
RUN npm install -g @project-serum/anchor-cli

# Verify Solana installation and print version
RUN solana --version

# Install cargo-build-sbf (using the Solana-provided version instead of cargo install)
RUN ln -s /root/.local/share/solana/install/bin/cargo-build-bpf /root/.cargo/bin/ || true
RUN ln -s /root/.local/share/solana/install/bin/cargo-build-sbf /root/.cargo/bin/ || true

# Set up working directory
WORKDIR /workspace

# Show Rust version
RUN rustc --version
RUN cargo --version