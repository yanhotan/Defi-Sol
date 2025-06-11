use anchor_lang::prelude::*;

#[error_code]
pub enum VaultSolError {
    #[msg("Amount must be greater than zero")]
    InvalidAmount,

    #[msg("Amount below minimum stake threshold")]
    BelowMinimumStake,

    #[msg("Insufficient stake balance")]
    InsufficientBalance,

    #[msg("Invalid authority")]
    InvalidAuthority,

    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Product is paused")]
    VaultPaused,

    #[msg("Invalid APY configuration")]
    InvalidAPY,

    #[msg("Invalid fee configuration")]
    InvalidFee,

    #[msg("Rewards pool depleted")]
    InsufficientRewards,
}

