use anchor_lang::prelude::*;

#[account]
pub struct StablecoinVaultConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub platform_fee_bps: u16,
    pub min_deposit_amount: u64,
    pub lending_enabled: bool,  // Whether vault can lend to protocols
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct UserStablePosition {
    pub owner: Pubkey,
    pub stablecoin_amount: u64,
    pub shares: u64,
    pub deposit_timestamp: i64,
    pub last_reward_claim: i64,
    pub bump: u8,
}

#[account]
pub struct StablePoolState {
    pub total_deposits: u64,
    pub total_shares: u64,
    pub apy_points: u16,  // Current APY in basis points
    pub stable_per_share: u64,  // Multiplied by 1e9
    pub last_update: i64,
    pub lending_ratio: u16,  // Max ratio that can be lent out (bps)
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum YieldSource {
    Lending,     // External lending protocols
    Treasury,    // Direct yield from treasury
    Both,
}