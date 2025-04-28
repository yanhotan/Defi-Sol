use anchor_lang::prelude::*;
use crate::state::{VaultConfig, StakePosition, RewardsPool};
use crate::errors::VaultSolError;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        seeds = [b"vault_sol_config"],
        bump = config.bump,
        constraint = !config.paused @ VaultSolError::VaultPaused,
    )]
    pub config: Account<'info, VaultConfig>,

    #[account(
        mut,
        seeds = [b"stake_position", user.key().as_ref()],
        bump = stake_position.bump,
        constraint = stake_position.owner == user.key(),
    )]
    pub stake_position: Account<'info, StakePosition>,

    #[account(
        mut,
        seeds = [b"rewards_pool"],
        bump = rewards_pool.bump,
    )]
    pub rewards_pool: Account<'info, RewardsPool>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let config = &ctx.accounts.config;
    let stake_position = &mut ctx.accounts.stake_position;
    
    // Get account infos first before borrowing rewards_pool mutably
    let rewards_pool_info = ctx.accounts.rewards_pool.to_account_info();
    let user_info = ctx.accounts.user.to_account_info();
    
    // Now we can safely mutably borrow rewards_pool
    let rewards_pool = &mut ctx.accounts.rewards_pool;
    
    // Calculate rewards based on staking duration
    let current_time = Clock::get()?.unix_timestamp;
    let time_staked = current_time
        .checked_sub(stake_position.last_reward_claim)
        .ok_or(VaultSolError::MathOverflow)?;
    
    require!(time_staked > 0, VaultSolError::InvalidAmount);

    // Calculate rewards based on amount, time, and APY
    let rewards = calculate_rewards(
        stake_position.amount,
        time_staked,
        rewards_pool.apy_points,
    )?;

    // Validate rewards pool has enough SOL balance
    require!(
        rewards_pool_info.lamports() >= rewards,
        VaultSolError::InsufficientRewards
    );

    // Check rewards pool has enough allocated rewards
    require!(
        rewards_pool.total_rewards
            .checked_sub(rewards_pool.distributed_rewards)
            .ok_or(VaultSolError::MathOverflow)?
            >= rewards,
        VaultSolError::InsufficientRewards
    );

    // Apply platform fee
    let fee_amount = (rewards as u128)
        .checked_mul(config.platform_fee_bps as u128)
        .ok_or(VaultSolError::MathOverflow)?
        .checked_div(10000)
        .ok_or(VaultSolError::MathOverflow)? as u64;

    let reward_amount = rewards
        .checked_sub(fee_amount)
        .ok_or(VaultSolError::MathOverflow)?;

    // Transfer rewards to user
    **rewards_pool_info.try_borrow_mut_lamports()? = rewards_pool_info
        .lamports()
        .checked_sub(reward_amount)
        .ok_or(VaultSolError::MathOverflow)?;

    **user_info.try_borrow_mut_lamports()? = user_info
        .lamports()
        .checked_add(reward_amount)
        .ok_or(VaultSolError::MathOverflow)?;

    // Update rewards pool state
    rewards_pool.distributed_rewards = rewards_pool.distributed_rewards
        .checked_add(rewards)
        .ok_or(VaultSolError::MathOverflow)?;

    // Update last claim timestamp
    stake_position.last_reward_claim = current_time;

    Ok(())
}

// Helper function to calculate rewards
fn calculate_rewards(
    amount: u64,
    time_staked: i64,
    apy_points: u16,
) -> Result<u64> {
    // Calculate rewards based on APY
    let rewards = (amount as u128)
        .checked_mul(time_staked as u128)
        .ok_or(VaultSolError::MathOverflow)?
        .checked_mul(apy_points as u128)
        .ok_or(VaultSolError::MathOverflow)?
        .checked_div(365 * 24 * 60 * 60 * 10000)  // Convert APY to per-second rate
        .ok_or(VaultSolError::MathOverflow)? as u64;

    Ok(rewards)
}