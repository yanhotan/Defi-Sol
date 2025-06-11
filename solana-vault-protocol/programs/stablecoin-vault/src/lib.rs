use anchor_lang::prelude::*;

pub mod errors;
pub mod state;
pub mod instructions;

use instructions::*;
use state::*;

declare_id!("StbL111111111111111111111111111111111111111");

#[program]
pub mod stablecoin_vault {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        platform_fee_bps: u16,
        min_deposit: u64,
        lending_ratio: u16,
    ) -> Result<()> {
        instructions::admin::initialize_vault(ctx, platform_fee_bps, min_deposit, lending_ratio)
    }

    pub fn deposit_stable(
        ctx: Context<DepositStable>,
        amount: u64,
    ) -> Result<()> {
        instructions::deposits::deposit_stable(ctx, amount)
    }

    pub fn withdraw_stable(
        ctx: Context<WithdrawStable>,
        amount: u64,
    ) -> Result<()> {
        instructions::withdrawals::withdraw_stable(ctx, amount)
    }

    pub fn claim_stable_rewards(
        ctx: Context<ClaimStableRewards>,
        source: YieldSource,
    ) -> Result<()> {
        instructions::rewards::claim_stable_rewards(ctx, source)
    }

    pub fn update_lending_ratio(
        ctx: Context<UpdateLendingRatio>,
        new_ratio: u16,
    ) -> Result<()> {
        instructions::admin::update_lending_ratio(ctx, new_ratio)
    }

    pub fn toggle_lending(
        ctx: Context<ToggleLending>,
        enabled: bool,
    ) -> Result<()> {
        instructions::admin::toggle_lending(ctx, enabled)
    }

    pub fn pause_vault(ctx: Context<PauseVault>) -> Result<()> {
        instructions::admin::pause_vault(ctx)
    }

    pub fn unpause_vault(ctx: Context<UnpauseVault>) -> Result<()> {
        instructions::admin::unpause_vault(ctx)
    }
}