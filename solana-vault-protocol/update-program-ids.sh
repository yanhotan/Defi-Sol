#!/bin/bash
set -e

echo "Updating program IDs in Anchor.toml..."

# Ensure solana CLI is available
if ! command -v solana &> /dev/null; then
    echo "Error: Solana CLI is not installed or not in PATH"
    exit 1
fi

# Base directory for keypair files
DEPLOY_DIR="./target/deploy"
PROGRAMS_DEPLOY_DIR="./programs/target/deploy"

# Function to get program ID from keypair file
get_program_id() {
    local keypair_file="$1"
    if [ -f "$keypair_file" ]; then
        solana address -k "$keypair_file"
    else
        echo "Keypair file not found: $keypair_file"
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
        echo "Keypair file not found for $name"
        return 1
    fi
}

# Extract program IDs
VAULT_SOL_KEYPAIR=$(find_keypair "vault_sol")
VAULT_SOL_ID=$(get_program_id "$VAULT_SOL_KEYPAIR")

LOCKING_VAULT_KEYPAIR=$(find_keypair "locking_vault")
LOCKING_VAULT_ID=$(get_program_id "$LOCKING_VAULT_KEYPAIR")

STABLECOIN_VAULT_KEYPAIR=$(find_keypair "stablecoin_vault")
STABLECOIN_VAULT_ID=$(get_program_id "$STABLECOIN_VAULT_KEYPAIR")

DUAL_PRODUCT_KEYPAIR=$(find_keypair "dual_product")
DUAL_PRODUCT_ID=$(get_program_id "$DUAL_PRODUCT_KEYPAIR")

echo "Found program IDs:"
echo "vault_sol: $VAULT_SOL_ID"
echo "locking_vault: $LOCKING_VAULT_ID"
echo "stablecoin_vault: $STABLECOIN_VAULT_ID"
echo "dual_product: $DUAL_PRODUCT_ID"

# Update Anchor.toml
sed -i.bak "s|vault_sol = \".*\"|vault_sol = \"$VAULT_SOL_ID\"|g" Anchor.toml
sed -i.bak "s|locking_vault = \".*\"|locking_vault = \"$LOCKING_VAULT_ID\"|g" Anchor.toml
sed -i.bak "s|stablecoin_vault = \".*\"|stablecoin_vault = \"$STABLECOIN_VAULT_ID\"|g" Anchor.toml
sed -i.bak "s|dual_product = \".*\"|dual_product = \"$DUAL_PRODUCT_ID\"|g" Anchor.toml

echo "Anchor.toml has been updated with correct program IDs."