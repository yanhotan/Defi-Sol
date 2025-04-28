@echo off
echo Building and deploying Solana DeFi programs to devnet...

:: Check if Solana CLI is available
where solana >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Solana CLI not found in PATH. Please run setup-solana.bat first and restart your computer.
    exit /b 1
)

:: Check current network configuration
echo Checking Solana configuration...
solana config get
echo.

:: Confirm devnet selection
echo WARNING: This script will deploy your programs to Solana devnet.
set /p CONTINUE="Continue? (y/n): "
if /i not "%CONTINUE%"=="y" (
    echo Deployment cancelled.
    exit /b 0
)

echo.
echo Building programs...

:: Create deployment directory if it doesn't exist
if not exist "solana-staking-app\target\deploy" mkdir "solana-staking-app\target\deploy"

:: Build each program
cd solana-staking-app\programs

echo Building locking-vault...
cd locking-vault
cargo build-bpf --bpf-out-dir ../../target/deploy
if %ERRORLEVEL% NEQ 0 (
    echo Error building locking-vault
    exit /b 1
)
cd ..

echo Building stablecoin-vault...
cd stablecoin-vault
cargo build-bpf --bpf-out-dir ../../target/deploy
if %ERRORLEVEL% NEQ 0 (
    echo Error building stablecoin-vault
    exit /b 1
)
cd ..

echo Building dual-product...
cd dual-product
cargo build-bpf --bpf-out-dir ../../target/deploy
if %ERRORLEVEL% NEQ 0 (
    echo Error building dual-product
    exit /b 1
)
cd ..

echo Building vault-sol...
cd vault-sol
cargo build-bpf --bpf-out-dir ../../target/deploy
if %ERRORLEVEL% NEQ 0 (
    echo Error building vault-sol
    exit /b 1
)
cd ..

cd ..

echo.
echo Build complete! Now deploying to devnet...

:: Deploy each program to devnet
echo Deploying locking-vault...
solana program deploy --program-id LoCK111111111111111111111111111111111111111 target\deploy\locking_vault.so
echo.

echo Deploying stablecoin-vault...
solana program deploy --program-id USDC111111111111111111111111111111111111111 target\deploy\stablecoin_vault.so
echo.

echo Deploying dual-product...
solana program deploy --program-id DuaL111111111111111111111111111111111111111 target\deploy\dual_product.so
echo.

echo Deploying vault-sol...
solana program deploy --program-id VauLt5oL11111111111111111111111111111111111 target\deploy\vault_sol.so
echo.

echo Deployment complete! Your programs are now available on Solana devnet.
echo You can view your programs at https://explorer.solana.com/?cluster=devnet

:: Return to the root directory
cd ..