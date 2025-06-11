#!/bin/bash
set -e

echo "Setting up wallet for testing..."

# Wait for validator to be ready
echo "Checking if solana-validator is accessible..."
max_attempts=30
attempt=1

while [ $attempt -le $max_attempts ]; do
  echo "Attempt $attempt/$max_attempts: Testing connection to solana-validator..."
  if curl -s http://solana-validator:8899 -X POST -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' | grep -q "ok"; then
    echo "Solana validator is up and running!"
    break
  else
    if [ $attempt -eq $max_attempts ]; then
      echo "Max attempts reached. Proceeding anyway, but wallet setup may fail."
    else
      echo "Validator not available yet. Waiting 5 seconds..."
      sleep 5
      attempt=$((attempt+1))
    fi
  fi
done

# Configure Solana CLI
echo "Configuring Solana CLI..."
solana config set --url http://solana-validator:8899

# Create wallet directory
mkdir -p /root/.config/solana

# Generate a new keypair if it doesn't exist
if [ ! -f /root/.config/solana/id.json ]; then
    echo "Generating new Solana keypair..."
    solana-keygen new --no-bip39-passphrase --force -o /root/.config/solana/id.json
fi

# Get the public key
WALLET_ADDRESS=$(solana address)
echo "Test wallet address: $WALLET_ADDRESS"

# Request multiple airdrops to get enough SOL
echo "Requesting multiple airdrops for a total of at least 15 SOL"
total_sol=0

# Try to get at least 15 SOL with multiple airdrops
while [ "$total_sol" -lt 15 ]; do
    echo "Current balance: $total_sol SOL - Requesting another 15 SOL airdrop..."
    
    # Try airdrop with retry logic
    airdrop_success=false
    for i in {1..5}; do
        echo "Airdrop attempt $i..."
        if solana airdrop 15 "$WALLET_ADDRESS"; then
            airdrop_success=true
            break
        else
            echo "Airdrop attempt $i failed. Waiting before retry..."
            sleep 5
        fi
    done
    
    if [ "$airdrop_success" = true ]; then
        echo "Airdrop successful!"
        # Check current balance
        total_sol=$(solana balance "$WALLET_ADDRESS" | grep -oE '[0-9]+' | head -1)
        echo "Current balance: $total_sol SOL"
    else
        echo "All airdrop attempts failed, but continuing..."
        break
    fi
done

echo "Final wallet balance: $(solana balance "$WALLET_ADDRESS")"

# Also create anchor wallet directory and copy the keypair
mkdir -p /workspace/.anchor/wallet
cp /root/.config/solana/id.json /workspace/.anchor/wallet/wallet.json

echo "Wallet setup complete."