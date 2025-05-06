#!/bin/bash
set -e

echo "=== Starting Solana DeFi Test Suite ==="

# Define colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function for step headers
step() {
  echo -e "\n${GREEN}==== $1 ====${NC}"
}

# Function to check validator connectivity
check_validator() {
  step "Checking Solana Validator Connection"
  
  max_attempts=30
  attempt=1
  
  while [ $attempt -le $max_attempts ]; do
    echo "Attempt $attempt/$max_attempts: Testing connection to solana-validator:8899..."
    if curl -s http://solana-validator:8899 -X POST -H "Content-Type: application/json" \
       -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' | grep -q "ok"; then
      echo -e "${GREEN}✓ Solana validator is up and running!${NC}"
      return 0
    else
      if [ $attempt -eq $max_attempts ]; then
        echo -e "${RED}✗ Max attempts reached. Validator connection failed.${NC}"
        return 1
      else
        echo -e "${YELLOW}⟳ Validator not available yet. Waiting 5 seconds...${NC}"
        sleep 5
        attempt=$((attempt+1))
      fi
    fi
  done
}

# Function to set up and fund wallet
setup_wallet() {
  step "Setting Up Test Wallet"
  
  # Configure Solana CLI
  echo "Configuring Solana CLI..."
  solana config set --url http://solana-validator:8899
  
  # Create wallet directories
  mkdir -p /root/.config/solana
  mkdir -p /workspace/.anchor/wallet
  
  # Generate a new keypair if it doesn't exist
  if [ ! -f /root/.config/solana/id.json ]; then
    echo "Generating new Solana keypair..."
    solana-keygen new --no-bip39-passphrase --force -o /root/.config/solana/id.json
  fi
  
  # Copy to anchor folder
  cp /root/.config/solana/id.json /workspace/.anchor/wallet/wallet.json
  chmod 600 /workspace/.anchor/wallet/wallet.json
  
  # Get wallet address
  WALLET_ADDRESS=$(solana address)
  echo -e "Test wallet address: ${GREEN}$WALLET_ADDRESS${NC}"
  
  # Request airdrop with retry logic
  echo "Requesting airdrop of 15 SOL"
  airdrop_success=false
  for i in {1..5}; do
    echo "Airdrop attempt $i..."
    if solana airdrop 15 "$WALLET_ADDRESS"; then
      airdrop_success=true
      break
    else
      echo -e "${YELLOW}Airdrop attempt $i failed. Waiting before retry...${NC}"
      sleep 5
    fi
  done
  
  if [ "$airdrop_success" = true ]; then
    echo -e "${GREEN}✓ Airdrop successful!${NC}"
    solana balance "$WALLET_ADDRESS"
    return 0
  else
    echo -e "${RED}✗ All airdrop attempts failed.${NC}"
    return 1
  fi
}

# Function to update program IDs in Anchor.toml
update_program_ids() {
  step "Updating Program IDs in Anchor.toml"
  
  # Base directory for keypair files
  DEPLOY_DIR="/workspace/target/deploy"
  PROGRAMS_DEPLOY_DIR="/workspace/programs/target/deploy"
  
  # Function to get program ID from keypair file
  get_program_id() {
    local keypair_file="$1"
    if [ -f "$keypair_file" ]; then
      solana address -k "$keypair_file"
    else
      echo -e "${RED}Keypair file not found: $keypair_file${NC}"
      return 1
    fi
  }
  
  # Try to find keypair files in both deploy directories
  find_keypair() {
    local name="$1"
    if [ -f "$DEPLOY_DIR/${name}-keypair.json" ]; then
      echo "$DEPLOY_DIR/${name}-keypair.json"
    elif [ -f "$PROGRAMS_DEPLOY_DIR/${name}-keypair.json" ]; then
      echo "$PROGRAMS_DEPLOY_DIR/${name}-keypair.json"
    else
      echo -e "${RED}Keypair file not found for $name${NC}"
      return 1
    fi
  }
  
  echo "Looking for program keypairs..."
  
  # Extract program IDs
  VAULT_SOL_KEYPAIR=$(find_keypair "vault_sol")
  VAULT_SOL_ID=$(get_program_id "$VAULT_SOL_KEYPAIR")
  
  LOCKING_VAULT_KEYPAIR=$(find_keypair "locking_vault")
  LOCKING_VAULT_ID=$(get_program_id "$LOCKING_VAULT_KEYPAIR")
  
  STABLECOIN_VAULT_KEYPAIR=$(find_keypair "stablecoin_vault")
  STABLECOIN_VAULT_ID=$(get_program_id "$STABLECOIN_VAULT_KEYPAIR")
  
  DUAL_PRODUCT_KEYPAIR=$(find_keypair "dual_product")
  DUAL_PRODUCT_ID=$(get_program_id "$DUAL_PRODUCT_KEYPAIR")
  
  echo -e "Found program IDs:"
  echo -e "vault_sol: ${GREEN}$VAULT_SOL_ID${NC}"
  echo -e "locking_vault: ${GREEN}$LOCKING_VAULT_ID${NC}"
  echo -e "stablecoin_vault: ${GREEN}$STABLECOIN_VAULT_ID${NC}"
  echo -e "dual_product: ${GREEN}$DUAL_PRODUCT_ID${NC}"
  
  # Create backup of Anchor.toml
  cp /workspace/Anchor.toml /workspace/Anchor.toml.backup
  
  # Update Anchor.toml
  sed -i "s|vault_sol = \".*\"|vault_sol = \"$VAULT_SOL_ID\"|g" /workspace/Anchor.toml
  sed -i "s|locking_vault = \".*\"|locking_vault = \"$LOCKING_VAULT_ID\"|g" /workspace/Anchor.toml
  sed -i "s|stablecoin_vault = \".*\"|stablecoin_vault = \"$STABLECOIN_VAULT_ID\"|g" /workspace/Anchor.toml
  sed -i "s|dual_product = \".*\"|dual_product = \"$DUAL_PRODUCT_ID\"|g" /workspace/Anchor.toml
  
  echo -e "${GREEN}✓ Anchor.toml has been updated with correct program IDs.${NC}"
  cat /workspace/Anchor.toml
}

# Function to check the program files
verify_programs() {
  step "Verifying Program Files"
  
  echo "Ensuring program files are accessible..."
  ls -la /workspace/target/deploy/ || echo -e "${YELLOW}Main deploy directory empty or not accessible${NC}"
  ls -la /workspace/programs/target/deploy/ || echo -e "${YELLOW}Programs deploy directory empty or not accessible${NC}"
  
  # Copy compiled programs to the expected locations if needed
  echo "Copying program files to expected locations..."
  if [ -d "/workspace/target/deploy" ] && [ "$(ls -A /workspace/target/deploy)" ]; then
    cp -f /workspace/target/deploy/*.so /workspace/programs/target/deploy/ 2>/dev/null || echo "No program files to copy from target/deploy"
  fi
  
  # Set appropriate permissions
  chmod -R 777 /workspace/target
  chmod -R 777 /workspace/programs/target
}

# Function to deploy programs to the validator with direct deployment
deploy_programs() {
  step "Deploying Programs to Validator"
  
  echo "Looking for program files in deploy directories..."
  PROGRAM_FILES=$(find /workspace/programs/target/deploy -name "*.so" 2>/dev/null)
  
  if [ -z "$PROGRAM_FILES" ]; then
    echo -e "${RED}No program (.so) files found in the deploy directory!${NC}"
    return 1
  fi
  
  echo "Found program files: $PROGRAM_FILES"
  
  # Configure Solana for deployment with longer timeout
  solana config set --url http://solana-validator:8899
  solana config set --commitment processed
  
  # Wait for validator to be fully ready with more slots
  echo "Waiting for validator to be fully ready..."
  for i in {1..60}; do
    if solana slot > /dev/null 2>&1; then
      current_slot=$(solana slot)
      echo "Current slot: $current_slot"
      if [ "$current_slot" -gt 100 ]; then
        echo "Validator has processed enough slots, proceeding with deployment"
        break
      fi
    fi
    echo "Waiting for validator to process more slots... (attempt $i/60)"
    sleep 5
  done
  
  # Ensure enough SOL for deployments
  WALLET_ADDRESS=$(solana address)
  CURRENT_BALANCE=$(solana balance | awk '{print $1}')
  
  if (( $(echo "$CURRENT_BALANCE < 10" | bc -l) )); then
    echo "Balance too low, requesting more SOL..."
    solana airdrop 10 "$WALLET_ADDRESS"
    sleep 5
  fi
  
  for program in $PROGRAM_FILES; do
    program_name=$(basename $program .so)
    keypair_file=$(find /workspace/programs/target/deploy -name "${program_name}-keypair.json" 2>/dev/null)
    
    if [ -z "$keypair_file" ]; then
      echo -e "${RED}No keypair found for $program_name!${NC}"
      continue
    fi
    
    echo "Deploying $program_name..."
    
    # Direct deployment with retries
    deploy_success=false
    for attempt in {1..10}; do
      echo "Deployment attempt $attempt/10..."
      
      # Clear any previous failed deployments
      program_id=$(solana-keygen pubkey "$keypair_file")
      solana program close "$program_id" > /dev/null 2>&1 || true
      sleep 2
      
      if solana program deploy \
        --program-id "$keypair_file" \
        "$program" \
        --commitment processed; then
        
        deploy_success=true
        echo -e "${GREEN}✓ Program $program_name deployed successfully on attempt $attempt!${NC}"
        
        # Verify deployment
        if solana program show "$program_id" > /dev/null 2>&1; then
          echo "Deployment verified!"
          break
        else
          echo "Deployment verification failed, retrying..."
          deploy_success=false
        fi
      fi
      
      echo "Deployment failed on attempt $attempt. Waiting before retry..."
      sleep 10
    done
    
    if ! $deploy_success; then
      echo -e "${RED}All deployment attempts failed for $program_name${NC}"
      # Continue with other programs even if this one failed
    fi
    
    # Longer wait between program deployments
    echo "Waiting for validator to process deployment..."
    sleep 30
  done
}

# Main execution flow
main() {
  check_validator || {
    echo -e "${RED}Failed to connect to validator. Exiting tests.${NC}"
    exit 1
  }
  
  setup_wallet || {
    echo -e "${YELLOW}Wallet setup had issues but continuing...${NC}"
  }
  
  verify_programs
  
  update_program_ids || {
    echo -e "${RED}Failed to update program IDs. Exiting tests.${NC}"
    exit 1
  }
  
  deploy_programs || {
    echo -e "${YELLOW}Program deployment had issues but continuing...${NC}"
  }
  
  step "Running Tests"
  cd /workspace
  echo "Current directory: $(pwd)"
  
  # Run Anchor tests
  echo "Running Anchor tests with skip-local-validator flag..."
  anchor test --skip-local-validator --provider.cluster http://solana-validator:8899
}

# Run the main function
main