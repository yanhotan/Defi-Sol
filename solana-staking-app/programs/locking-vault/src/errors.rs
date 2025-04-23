use anchor_lang::prelude::*;

#[error_code]
pub enum LockingVaultError {
    #[msg("Amount must be greater than zero")]
    InvalidAmount,

    #[msg("Amount below minimum deposit threshold")]
    BelowMinimumDeposit,

    #[msg("Invalid lock period")]
    InvalidLockPeriod,

    #[msg("Position is still locked")]
    PositionLocked,

    #[msg("Invalid asset type")]
    InvalidAssetType,

    #[msg("Invalid withdraw type")]
    InvalidWithdrawType,

    #[msg("Invalid authority")]
    InvalidAuthority,

    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Product is paused")]
    VaultPaused,

    #[msg("Invalid APY multiplier")]
    InvalidMultiplier,

    #[msg("Invalid lock periods configuration")]
    InvalidLockPeriods,

    #[msg("Position already unlocked")]
    PositionUnlocked,
}