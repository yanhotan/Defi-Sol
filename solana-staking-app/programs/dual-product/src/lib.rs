use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

pub mod errors;
pub mod state;
pub mod instructions;

use instructions::*;
use state::*;

declare_id!("DuaL111111111111111111111111111111111111111");

#[program]
pub mod dual_product {
    use super::*;

    pub fn initialize_product(
        ctx: Context<InitializeProduct>,
        platform_fee_bps: u16,
        min_deposit: u64,
        lst_ratio: u16,
        usdc_ratio: u16,
    ) -> Result<()> {
        instructions::admin::initialize_product(ctx, platform_fee_bps, min_deposit, lst_ratio, usdc_ratio)
    }

    pub fn deposit_dual(
        ctx: Context<DepositDual>,
        lst_amount: u64,
        usdc_amount: u64,
        add_to_lp: bool,
    ) -> Result<()> {
        instructions::deposits::deposit_dual(ctx, lst_amount, usdc_amount, add_to_lp)
    }

    pub fn withdraw_dual(
        ctx: Context<WithdrawDual>,
        lst_amount: u64,
        usdc_amount: u64,
    ) -> Result<()> {
        instructions::withdrawals::withdraw_dual(ctx, lst_amount, usdc_amount)
    }

    pub fn add_to_lp(ctx: Context<AddToLP>) -> Result<()> {
        instructions::liquidity::add_to_lp(ctx)
    }

    pub fn remove_from_lp(ctx: Context<RemoveFromLP>) -> Result<()> {
        instructions::liquidity::remove_from_lp(ctx)
    }

    pub fn claim_dual_rewards(
        ctx: Context<ClaimDualRewards>,
        reward_source: RewardSource,
    ) -> Result<()> {
        instructions::rewards::claim_dual_rewards(ctx, reward_source)
    }

    pub fn update_ratios(
        ctx: Context<UpdateRatios>,
        new_lst_ratio: u16,
        new_usdc_ratio: u16,
    ) -> Result<()> {
        instructions::admin::update_ratios(ctx, new_lst_ratio, new_usdc_ratio)
    }

    pub fn pause_product(ctx: Context<PauseProduct>) -> Result<()> {
        instructions::admin::pause_product(ctx)
    }

    pub fn unpause_product(ctx: Context<UnpauseProduct>) -> Result<()> {
        instructions::admin::unpause_product(ctx)
    }
}