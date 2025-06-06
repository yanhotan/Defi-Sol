FROM rust:1.72

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
    wget \
    bzip2 \
    && rm -rf /var/lib/apt/lists/*

# Install OpenSSL 1.1 from Debian backports
RUN mkdir -p /etc/apt/keyrings && \
    wget -qO - https://snapshot.debian.org/archive/debian/20230320T000000Z/pool/main/o/openssl/libssl1.1_1.1.1n-0+deb11u4_amd64.deb > /tmp/libssl1.1.deb && \
    dpkg -i /tmp/libssl1.1.deb && \
    rm -f /tmp/libssl1.1.deb

# Set OpenSSL environment variables
ENV LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV OPENSSL_INCLUDE_DIR=/usr/include

# Setup compatible Rust version for Solana
# Use Rust 1.68.2 which is known to work with Solana BPF
RUN rustup install 1.68.2
RUN rustup default 1.68.2
RUN rustup component add rust-src --toolchain 1.68.2

# Also install nightly for solana-specific needs
RUN rustup toolchain install nightly
RUN rustup component add rust-src --toolchain nightly

# Install Solana CLI tools - using version 1.16.19
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.16.19/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install necessary LLVM version for Solana BPF
RUN apt-get update && \
    apt-get install -y software-properties-common && \
    wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add - && \
    add-apt-repository "deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-14 main" && \
    apt-get update && \
    apt-get install -y llvm-14 clang-14 && \
    rm -rf /var/lib/apt/lists/*

# Configure BPF environment variables
ENV SBF_LLVM_PATH="/usr/lib/llvm-14"
ENV LLVM_CONFIG="/usr/bin/llvm-config-14"

# Let Solana install the target during build - don't try to add it directly
# This ensures the correct target is used based on the Solana version

# Install Anchor using NPM
RUN npm install -g @coral-xyz/anchor-cli

# Create workspace directory
WORKDIR /workspace

# Configure Solana to use local validator
RUN solana config set --url http://solana-validator:8899

# The source code will be mounted as a volume
CMD ["bash"]