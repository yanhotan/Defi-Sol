use anchor_lang::prelude::*;

#[error_code]
pub enum DualProductError {
    #[msg("Invalid fee percentage")]
    InvalidFee,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Math overflow occurred")]
    MathOverflow,

    #[msg("Amount below minimum required")]
    BelowMinimumAmount,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Vault is paused")]
    VaultPaused,

    #[msg("Lock period not expired")]
    LockPeriodNotExpired,

    #[msg("Invalid lock period")]
    InvalidLockPeriod,

    #[msg("Invalid APY tier")]
    InvalidAPYTier,

    #[msg("Insufficient rewards")]
    InsufficientRewards,

    #[msg("Invalid asset ratio")]
    InvalidAssetRatio,

    #[msg("Position already in LP")]
    PositionAlreadyInLP,

    #[msg("Position not in LP")]
    PositionNotInLP,
}