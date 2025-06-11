use anchor_lang::prelude::*;

#[account]
pub struct VaultConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub platform_fee_bps: u16,
    pub min_stake_amount: u64,
    pub total_staked: u64,
    pub stakers_count: u64,
    pub active_provider: LSTProvider,
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct StakePosition {
    pub owner: Pubkey,
    pub amount: u64,
    pub start_time: i64,
    pub last_reward_claim: i64,
    pub bump: u8,
}

#[account]
pub struct RewardsPool {
    pub total_rewards: u64,
    pub apy_points: u16,  // APY in basis points
    pub last_update: i64,
    pub distributed_rewards: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum LSTProvider {
    None,
    Marinade,
    Lido,
    JitoSol,
}

#[account]
pub struct UserPosition {
    pub owner: Pubkey,
    pub amount_staked: u64,
    pub vsol_minted: u64,
    pub provider_used: LSTProvider,
    pub deposit_timestamp: i64,
    pub bump: u8,
}

