@echo off
echo Setting up Solana development environment...

:: Install Rust if not already installed
where rustc >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Installing Rust...
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | cmd
    echo Rust installed successfully!
) else (
    echo Rust is already installed.
)

:: Install Solana CLI tools
echo Installing Solana CLI tools...
curl -sSfL https://release.solana.com/v1.17.3/solana-install-init-x86_64-pc-windows-msvc.exe -o solana-install-init.exe
.\solana-install-init.exe v1.17.3

:: Add Solana to PATH
echo Adding Solana to PATH...
setx PATH "%PATH%;%USERPROFILE%\.local\share\solana\install\active_release\bin"

echo Solana development environment setup complete!
echo Please restart your command prompt or terminal for the changes to take effect.