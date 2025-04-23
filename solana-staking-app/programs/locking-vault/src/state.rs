use anchor_lang::prelude::*;

#[account]
pub struct LockingVaultConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub platform_fee_bps: u16,
    pub min_deposit_amount: u64,
    pub available_lock_periods: [u16; 5],  // Lock periods in days [30, 90, 180, 270, 360]
    pub lock_period_multipliers: [u16; 5], // APY multipliers for each period in bps
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct UserLockPosition {
    pub owner: Pubkey,
    pub asset_type: AssetType,
    pub amount: u64,
    pub lock_period: u16,        // In days
    pub apy_multiplier: u16,     // In bps
    pub start_timestamp: i64,
    pub unlock_timestamp: i64,
    pub last_reward_claim: i64,
    pub bump: u8,
}

#[account]
pub struct LockPoolState {
    pub total_sol_locked: u64,
    pub total_usdc_locked: u64,
    pub base_apy_points: u16,    // Base APY in bps before multipliers
    pub last_update: i64,
    pub total_penalties: u64,    // Early withdrawal penalties collected
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    SOL,
    USDC,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum WithdrawType {
    Normal,     // After lock period
    Early,      // Before lock period (with penalty)
}