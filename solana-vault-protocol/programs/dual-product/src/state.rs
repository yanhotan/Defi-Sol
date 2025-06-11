use anchor_lang::prelude::*;

#[account]
pub struct DualProductConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub platform_fee_bps: u16,
    pub min_deposit_amount: u64,
    pub lst_ratio: u16,  // Ratio of LST in basis points (e.g., 5000 = 50%)
    pub usdc_ratio: u16, // Ratio of USDC in basis points
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct UserDualPosition {
    pub owner: Pubkey,
    pub lst_amount: u64,
    pub usdc_amount: u64,
    pub in_lp: bool,      // Whether position is in LP pool
    pub deposit_timestamp: i64,
    pub last_reward_claim: i64,
    pub bump: u8,
}

#[account]
pub struct PoolState {
    pub total_lst: u64,
    pub total_usdc: u64,
    pub total_shares: u64,
    pub lst_per_share: u64,  // Multiplied by 1e9
    pub usdc_per_share: u64, // Multiplied by 1e9
    pub last_update: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum RewardSource {
    LST,
    LP,
    Both,
}

#[account]
pub struct DualConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub platform_fee_bps: u16,
    pub min_dual_amount: u64,
    pub total_dual_positions: u64,
    pub users_count: u64,
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct DualPosition {
    pub owner: Pubkey,
    pub wsol_amount: u64,
    pub usdc_amount: u64,
    pub start_time: i64,
    pub last_reward_claim: i64,
    pub lock_period: u64,  // Lock period in seconds
    pub apy_tier: u8,      // APY tier based on lock period
    pub bump: u8,
}

#[account]
pub struct DualPool {
    pub total_wsol: u64,
    pub total_usdc: u64,
    pub base_apy_points: u16,  // Base APY in basis points
    pub tier1_threshold: u64,  // 1 month lock threshold
    pub tier2_threshold: u64,  // 3 month lock threshold
    pub tier3_threshold: u64,  // 6 month lock threshold
    pub tier1_multiplier: u16, // Multiplier for 1 month lock (in bps)
    pub tier2_multiplier: u16, // Multiplier for 3 month lock (in bps)
    pub tier3_multiplier: u16, // Multiplier for 6 month lock (in bps)
    pub last_update: i64,
    pub rewards_available: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum RewardType {
    Wsol,
    Usdc,
    Both,
}