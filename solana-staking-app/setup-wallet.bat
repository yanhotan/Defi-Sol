@echo off
echo Creating Solana wallet for development...
docker run --rm -v "%cd%\.anchor\wallet:/wallet" solanalabs/solana:latest solana-keygen new -o /wallet/wallet.json --no-bip39-passphrase
echo Wallet created at .anchor\wallet\wallet.json