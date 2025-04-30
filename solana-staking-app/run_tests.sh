#!/bin/bash
set -e

# Set up directories
mkdir -p .anchor/wallet
mkdir -p target/deploy
mkdir -p programs/target/deploy

# Set up wallet for tests
if [ ! -f .anchor/wallet/wallet.json ]; then
  echo "Setting up test wallet..."
  solana-keygen new --no-bip39-passphrase -o .anchor/wallet/wallet.json
  chmod 600 .anchor/wallet/wallet.json
fi

# Configure Solana CLI
solana config set --url http://solana-validator:8899

# Get wallet pubkey and airdrop funds for tests
WALLET_PUBKEY=$(solana address -k .anchor/wallet/wallet.json)
echo "Test wallet address: $WALLET_PUBKEY"
solana airdrop 10 $WALLET_PUBKEY || echo "Airdrop failed, but continuing..."

# Ensure program files are accessible
echo "Ensuring program files are accessible..."
ls -la target/deploy/ || echo "Main deploy directory empty or not accessible"
ls -la programs/target/deploy/ || echo "Programs deploy directory empty or not accessible"

# Copy compiled programs to the expected location if needed
echo "Copying program files to expected locations..."
if [ -d "target/deploy" ] && [ "$(ls -A target/deploy)" ]; then
  cp -f target/deploy/*.so programs/target/deploy/ 2>/dev/null || echo "No program files to copy from target/deploy"
fi

# Check anchor configuration
echo "Checking Anchor.toml configuration..."
cat Anchor.toml

# Set appropriate permissions
chmod -R 777 target
chmod -R 777 programs/target

# Run the tests with verbose output
echo "Running Anchor tests..."
anchor test --skip-local-validator --detach --provider.cluster http://solana-validator:8899