#!/bin/bash

# Create necessary directories
echo "Creating necessary target directories..."
mkdir -p /workspace/programs/target/src
mkdir -p /workspace/programs/target/deploy
mkdir -p /workspace/programs/target/release

# Run the tests
echo "Running anchor tests..."
anchor test --skip-local-validator