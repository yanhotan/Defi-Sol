#!/bin/bash
set -e

# Check for Solana CLI
if ! command -v solana &> /dev/null; then
    echo "Error: Solana CLI not found!"
    echo "Please install Solana CLI: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

echo "=== Solana Stake-to-Lend Protocol Test Suite ==="

# Run cargo tests first (unit tests)
echo "Running Rust unit tests..."
cargo test

# Check if we should run full integration tests with validator
echo "Would you like to run integration tests with local validator? (y/n)"
read -r RUN_INTEGRATION

if [[ "${RUN_INTEGRATION}" =~ ^[Yy] ]]; then
    echo "Starting local validator and running integration tests..."
    
    # Start local validator in background
    echo "Starting Solana local validator..."
    solana-test-validator --quiet --reset &
    VALIDATOR_PID=$!
    
    # Wait for validator to start
    echo "Waiting for validator to start..."
    sleep 5
    
    # Configure CLI to use local validator
    solana config set --url localhost
    
    # Build the program if needed
    if [ ! -f "./target/sbf-solana-solana/release/sol_stake_lend.so" ]; then
        echo "Building program..."
        cargo build-sbf --release
    fi
    
    # Deploy to local validator
    echo "Deploying program to local validator..."
    PROGRAM_OUTPUT=$(solana program deploy ./target/sbf-solana-solana/release/sol_stake_lend.so)
    PROGRAM_ID=$(echo "$PROGRAM_OUTPUT" | grep -o '[1-9A-HJ-NP-Za-km-z]\{32,44\}' | head -1)
    
    echo "Program deployed with ID: $PROGRAM_ID"
    
    # Run integration tests with deployed program ID
    echo "Running integration tests..."
    PROGRAM_ID=$PROGRAM_ID cargo test --test integration_tests -- --nocapture
    
    # Kill validator when done
    echo "Stopping local validator..."
    kill $VALIDATOR_PID
    
    echo "Integration tests complete!"
else
    echo "Skipping integration tests."
fi

# Run any additional test scripts as needed
echo "Testing complete! All tests passed."