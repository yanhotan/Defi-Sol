@echo off
echo Setting up wallet for Solana devnet development...

:: Check if Solana CLI is installed and accessible
where solana >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Solana CLI not found in PATH. Please run setup-solana.bat first and restart your computer.
    exit /b 1
)

:: Configure Solana CLI to use devnet
echo Configuring Solana to use devnet...
solana config set --url https://api.devnet.solana.com

:: Generate a new keypair if id.json doesn't exist
if not exist %USERPROFILE%\.config\solana\id.json (
    echo Creating new wallet keypair...
    solana-keygen new --no-bip39-passphrase
) else (
    echo Using existing keypair at %USERPROFILE%\.config\solana\id.json
)

:: Display public key
echo Your wallet public key:
solana address

:: Request airdrop
echo Requesting SOL airdrop from devnet...
solana airdrop 2

echo Wallet setup complete! You can now deploy your programs to devnet.
echo Use 'solana balance' to check your SOL balance.