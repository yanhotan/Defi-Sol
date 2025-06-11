#!/bin/bash
set -e

echo "Starting test sequence..."

# Check if the Solana validator is accessible
echo "Checking if Solana validator is accessible..."
max_attempts=30
attempt=1

while [ $attempt -le $max_attempts ]; do
  echo "Attempt $attempt/$max_attempts: Testing connection to solana-validator:8899..."
  if curl -s http://solana-validator:8899 -X POST -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' | grep -q "ok"; then
    echo "Solana validator is up and running!"
    break
  else
    if [ $attempt -eq $max_attempts ]; then
      echo "Max attempts reached. Proceeding anyway, but tests may fail."
    else
      echo "Validator not available yet. Waiting 5 seconds..."
      sleep 5
      attempt=$((attempt+1))
    fi
  fi
done

# Set up directories
mkdir -p .anchor/wallet
mkdir -p target/deploy
mkdir -p programs/target/deploy

# Configure Solana CLI
echo "Configuring Solana CLI..."
solana config set --url http://solana-validator:8899
solana config get

# Set up wallet for tests
if [ ! -f .anchor/wallet/wallet.json ]; then
  echo "Setting up test wallet..."
  solana-keygen new --no-bip39-passphrase -o .anchor/wallet/wallet.json
  chmod 600 .anchor/wallet/wallet.json
fi

# Get wallet pubkey and airdrop funds for tests
WALLET_PUBKEY=$(solana address -k .anchor/wallet/wallet.json)
echo "Test wallet address: $WALLET_PUBKEY"

# Try airdrop with retries
echo "Requesting airdrop of 10 SOL"
airdrop_success=false
for i in {1..5}; do
    echo "Airdrop attempt $i..."
    if solana airdrop 10 "$WALLET_PUBKEY"; then
        airdrop_success=true
        break
    else
        echo "Airdrop attempt $i failed. Waiting before retry..."
        sleep 5
    fi
done

if [ "$airdrop_success" = true ]; then
    echo "Airdrop successful!"
else
    echo "All airdrop attempts failed, but continuing..."
fi

# Check wallet balance
echo "Checking wallet balance:"
solana balance "$WALLET_PUBKEY" || echo "Could not check balance, continuing..."

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