use anchor_lang::prelude::*;

pub mod state;
pub mod errors;
pub mod instructions;

use instructions::*;

declare_id!("VauLt5oL1111111111111111111111111111111111");

#[program]
pub mod vault_sol {
    use super::*;

    // Admin instructions
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        platform_fee_bps: u16,
        min_stake: u64,
    ) -> Result<()> {
        instructions::admin::initialize_vault(ctx, platform_fee_bps, min_stake)
    }

    pub fn update_apy(
        ctx: Context<UpdateAPY>,
        new_apy: u16,
    ) -> Result<()> {
        instructions::admin::update_apy(ctx, new_apy)
    }

    pub fn add_rewards(
        ctx: Context<AddRewards>,
        amount: u64,
    ) -> Result<()> {
        instructions::admin::add_rewards(ctx, amount)
    }

    pub fn pause_vault(ctx: Context<PauseVault>) -> Result<()> {
        instructions::admin::pause_vault(ctx)
    }

    pub fn unpause_vault(ctx: Context<UnpauseVault>) -> Result<()> {
        instructions::admin::unpause_vault(ctx)
    }

    // Staking instructions
    pub fn create_stake(
        ctx: Context<CreateStake>,
        amount: u64,
    ) -> Result<()> {
        instructions::staking::create_stake(ctx, amount)
    }

    pub fn withdraw_stake(
        ctx: Context<WithdrawStake>,
        amount: u64,
    ) -> Result<()> {
        instructions::staking::withdraw_stake(ctx, amount)
    }

    // Rewards instructions
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::rewards::claim_rewards(ctx)
    }
}

