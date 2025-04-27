use anchor_lang::prelude::*;
use anchor_spl::token::Token;

pub mod errors;
pub mod state;
pub mod instructions;

use instructions::*;
use state::*;

declare_id!("LoCK111111111111111111111111111111111111111");

#[program]
pub mod locking_vault {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        platform_fee_bps: u16,
        min_deposit: u64,
        lock_periods: [u16; 5],
        multipliers: [u16; 5],
    ) -> Result<()> {
        instructions::admin::initialize_vault(ctx, platform_fee_bps, min_deposit, lock_periods, multipliers)
    }

    pub fn create_lock_position(
        ctx: Context<CreateLockPosition>,
        amount: u64,
        asset_type: AssetType,
        lock_period: u16,
    ) -> Result<()> {
        instructions::deposits::create_lock_position(ctx, amount, asset_type, lock_period)
    }

    pub fn withdraw_locked(
        ctx: Context<WithdrawLocked>,
        amount: u64,
        withdraw_type: WithdrawType,
    ) -> Result<()> {
        instructions::withdrawals::withdraw_locked(ctx, amount, withdraw_type)
    }

    pub fn claim_lock_rewards(
        ctx: Context<ClaimLockRewards>,
    ) -> Result<()> {
        instructions::rewards::claim_lock_rewards(ctx)
    }

    pub fn update_lock_periods(
        ctx: Context<UpdateLockPeriods>,
        new_periods: [u16; 5],
        new_multipliers: [u16; 5],
    ) -> Result<()> {
        instructions::admin::update_lock_periods(ctx, new_periods, new_multipliers)
    }

    pub fn update_base_apy(
        ctx: Context<UpdateBaseAPY>,
        new_base_apy: u16,
    ) -> Result<()> {
        instructions::admin::update_base_apy(ctx, new_base_apy)
    }

    pub fn pause_vault(ctx: Context<PauseVault>) -> Result<()> {
        instructions::admin::pause_vault(ctx)
    }

    pub fn unpause_vault(ctx: Context<UnpauseVault>) -> Result<()> {
        instructions::admin::unpause_vault(ctx)
    }
}