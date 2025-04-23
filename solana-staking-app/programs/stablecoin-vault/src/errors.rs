use anchor_lang::prelude::*;

#[error_code]
pub enum StablecoinVaultError {
    #[msg("Amount must be greater than zero")]
    InvalidAmount,

    #[msg("Amount below minimum deposit threshold")]
    BelowMinimumDeposit,

    #[msg("Insufficient balance for withdrawal")]
    InsufficientBalance,

    #[msg("Invalid authority")]
    InvalidAuthority,

    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Product is paused")]
    VaultPaused,

    #[msg("Invalid yield source")]
    InvalidYieldSource,

    #[msg("Lending is currently disabled")]
    LendingDisabled,

    #[msg("Invalid lending ratio")]
    InvalidLendingRatio,
}