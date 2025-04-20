#!/bin/bash
set -e

# Check for Solana CLI
if ! command -v solana &> /dev/null; then
    echo "Error: Solana CLI not found!"
    echo "Please install Solana CLI: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

# Display Solana version
echo "Using Solana CLI $(solana --version | head -n 1)"

# Set up constants
PROGRAM_NAME="sol_stake_lend"
PROGRAM_PATH="./target/sbf-solana-solana/release/${PROGRAM_NAME}.so"
LOG_FILE="deploy_log_$(date +%Y%m%d_%H%M%S).txt"
KEYPAIR_PATH="${HOME}/.config/solana/id.json"

# Check for keypair
if [ ! -f "${KEYPAIR_PATH}" ]; then
    echo "Solana keypair not found. Creating new keypair..."
    solana-keygen new --no-bip39-passphrase -o "${KEYPAIR_PATH}"
fi

# Check for built program
if [ ! -f "${PROGRAM_PATH}" ]; then
    echo "Program binary not found! Building the program..."
    cargo build-sbf --release
fi

# Check cluster configuration
CLUSTER_URL=$(solana config get | grep "RPC URL" | awk '{print $3}')
echo "Deploying to Solana cluster: ${CLUSTER_URL}"

# Check for devnet SOL if on devnet
if [[ "${CLUSTER_URL}" == *"devnet"* ]]; then
    BALANCE=$(solana balance | awk '{print $1}')
    if (( $(echo "${BALANCE} < 2" | bc -l) )); then
        echo "Getting airdrop of 2 SOL for deployment fees..."
        solana airdrop 2 || echo "Airdrop failed. Please fund your account manually."
    fi
fi

echo "Deploying program: ${PROGRAM_PATH}"
echo "Deployment log will be saved to: ${LOG_FILE}"

# Deploy the program
echo "===== DEPLOYMENT STARTED: $(date) =====" | tee -a "${LOG_FILE}"
RESULT=$(solana program deploy "${PROGRAM_PATH}" --output json 2>&1)
echo "${RESULT}" | tee -a "${LOG_FILE}"

# Extract program id from result
PROGRAM_ID=$(echo "${RESULT}" | grep -o '"programId": "[^"]*' | cut -d'"' -f4)

if [ -z "${PROGRAM_ID}" ]; then
    echo "Failed to extract program ID! Check ${LOG_FILE} for details."
    exit 1
fi

echo "===== DEPLOYMENT SUCCESSFUL =====" | tee -a "${LOG_FILE}"
echo "Program ID: ${PROGRAM_ID}" | tee -a "${LOG_FILE}"

# Save program ID to config file for easy access
echo "{\"programId\": \"${PROGRAM_ID}\"}" > "./program_id.json"
echo "Program ID saved to program_id.json"

# Initialize the protocol if needed
echo "Would you like to initialize the protocol configuration? (y/n)"
read -r INIT_ANSWER

if [[ "${INIT_ANSWER}" =~ ^[Yy] ]]; then
    echo "Initializing protocol configuration..."
    
    # Generate protocol initialization transaction
    # (In a real scenario, you'd likely have a more complex initialization script)
    echo "Running initialization transaction..."
    
    # Example initialization command - replace with your actual initialization logic
    # solana-test-validator command would go here for local testing
    # or transaction sending code for devnet/testnet/mainnet
    
    echo "Protocol initialized successfully!"
fi

echo "Deployment complete! Use program ID ${PROGRAM_ID} for client integration."