use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::{LockingVaultConfig, UserLockPosition, LockPoolState, AssetType};
use crate::errors::LockingVaultError;

#[derive(Accounts)]
pub struct ClaimLockRewards<'info> {
    #[account(
        seeds = [b"locking_vault_config"],
        bump = config.bump,
        constraint = !config.paused @ LockingVaultError::VaultPaused,
    )]
    pub config: Account<'info, LockingVaultConfig>,

    #[account(
        mut,
        seeds = [b"user_lock_position", user.key().as_ref()],
        bump = user_position.bump,
        constraint = user_position.owner == user.key(),
    )]
    pub user_position: Account<'info, UserLockPosition>,

    #[account(
        seeds = [b"lock_pool_state"],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, LockPoolState>,

    #[account(mut)]
    pub user: Signer<'info>,

    // For USDC rewards
    #[account(mut)]
    pub user_usdc_account: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub vault_usdc_account: Option<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn claim_lock_rewards(ctx: Context<ClaimLockRewards>) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &ctx.accounts.pool_state;
    
    // Calculate rewards based on staking duration
    let current_time = Clock::get()?.unix_timestamp;
    let time_staked = current_time
        .checked_sub(user_position.last_reward_claim)
        .ok_or(LockingVaultError::MathOverflow)?;
    
    require!(time_staked > 0, LockingVaultError::InvalidAmount);

    // Calculate rewards based on amount, time, base APY, and position multiplier
    let rewards = calculate_lock_rewards(
        user_position.amount,
        time_staked,
        pool_state.base_apy_points,
        user_position.apy_multiplier,
    )?;

    // Apply platform fee
    let fee_amount = (rewards as u128)
        .checked_mul(config.platform_fee_bps as u128)
        .ok_or(LockingVaultError::MathOverflow)?
        .checked_div(10000)
        .ok_or(LockingVaultError::MathOverflow)? as u64;

    let reward_amount = rewards
        .checked_sub(fee_amount)
        .ok_or(LockingVaultError::MathOverflow)?;

    // Process rewards based on asset type
    match user_position.asset_type {
        AssetType::SOL => {
            // Transfer SOL rewards to user
            **ctx.accounts.treasury.try_borrow_mut_lamports()? = ctx
                .accounts
                .treasury
                .lamports()
                .checked_sub(reward_amount)
                .ok_or(LockingVaultError::MathOverflow)?;

            **ctx.accounts.user.try_borrow_mut_lamports()? = ctx
                .accounts
                .user
                .lamports()
                .checked_add(reward_amount)
                .ok_or(LockingVaultError::MathOverflow)?;
        },
        AssetType::USDC => {
            // Validate USDC accounts are provided
            require!(
                ctx.accounts.user_usdc_account.is_some() &&
                ctx.accounts.vault_usdc_account.is_some(),
                LockingVaultError::InvalidTokenAccount
            );

            // Transfer USDC rewards
            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.vault_usdc_account.as_ref().unwrap().to_account_info(),
                        to: ctx.accounts.user_usdc_account.as_ref().unwrap().to_account_info(),
                        authority: config.to_account_info(),
                    },
                ),
                reward_amount,
            )?;
        },
    }

    // Update last claim timestamp
    user_position.last_reward_claim = current_time;

    Ok(())
}

// Helper function to calculate locked rewards
fn calculate_lock_rewards(
    amount: u64,
    time_staked: i64,
    base_apy: u16,
    multiplier: u16,
) -> Result<u64> {
    // Calculate effective APY with multiplier
    let effective_apy = (base_apy as u128)
        .checked_mul(multiplier as u128)
        .ok_or(LockingVaultError::MathOverflow)?
        .checked_div(10000)  // Multiplier is in bps
        .ok_or(LockingVaultError::MathOverflow)? as u16;

    // Calculate rewards based on effective APY
    let rewards = (amount as u128)
        .checked_mul(time_staked as u128)
        .ok_or(LockingVaultError::MathOverflow)?
        .checked_mul(effective_apy as u128)
        .ok_or(LockingVaultError::MathOverflow)?
        .checked_div(365 * 24 * 60 * 60 * 10000)  // Convert APY to per-second rate
        .ok_or(LockingVaultError::MathOverflow)? as u64;

    Ok(rewards)
}