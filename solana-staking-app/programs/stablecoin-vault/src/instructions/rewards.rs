use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::{StablecoinVaultConfig, UserStablePosition, StablePoolState, YieldSource};
use crate::errors::StablecoinVaultError;

#[derive(Accounts)]
pub struct ClaimStableRewards<'info> {
    #[account(
        seeds = [b"stable_vault_config"],
        bump = config.bump,
        constraint = !config.paused @ StablecoinVaultError::VaultPaused,
    )]
    pub config: Account<'info, StablecoinVaultConfig>,

    #[account(
        mut,
        seeds = [b"user_stable_position", user.key().as_ref()],
        bump = user_position.bump,
        constraint = user_position.owner == user.key(),
    )]
    pub user_position: Account<'info, UserStablePosition>,

    #[account(
        seeds = [b"stable_pool_state"],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, StablePoolState>,

    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_usdc_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn claim_stable_rewards(
    ctx: Context<ClaimStableRewards>,
    source: YieldSource,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &ctx.accounts.pool_state;
    
    // Calculate rewards based on staking duration and source
    let current_time = Clock::get()?.unix_timestamp;
    let time_staked = current_time
        .checked_sub(user_position.last_reward_claim)
        .ok_or(StablecoinVaultError::MathOverflow)?;
    
    require!(time_staked > 0, StablecoinVaultError::InvalidAmount);

    match source {
        YieldSource::Lending => {
            require!(config.lending_enabled, StablecoinVaultError::LendingDisabled);
            
            // Calculate lending rewards based on share of pool
            let rewards = calculate_lending_rewards(
                user_position.shares,
                time_staked,
                pool_state,
            )?;

            transfer_rewards(ctx, rewards)?;
        },
        YieldSource::Treasury => {
            // Calculate treasury rewards based on fixed APY
            let rewards = calculate_treasury_rewards(
                user_position.stablecoin_amount,
                time_staked,
                pool_state.apy_points,
            )?;

            transfer_rewards(ctx, rewards)?;
        },
        YieldSource::Both => {
            if config.lending_enabled {
                // Calculate and transfer both types of rewards
                let lending_rewards = calculate_lending_rewards(
                    user_position.shares,
                    time_staked,
                    pool_state,
                )?;
                
                let treasury_rewards = calculate_treasury_rewards(
                    user_position.stablecoin_amount,
                    time_staked,
                    pool_state.apy_points,
                )?;

                let total_rewards = lending_rewards
                    .checked_add(treasury_rewards)
                    .ok_or(StablecoinVaultError::MathOverflow)?;

                transfer_rewards(ctx, total_rewards)?;
            } else {
                // Only transfer treasury rewards if lending is disabled
                let rewards = calculate_treasury_rewards(
                    user_position.stablecoin_amount,
                    time_staked,
                    pool_state.apy_points,
                )?;

                transfer_rewards(ctx, rewards)?;
            }
        }
    }

    // Update last claim timestamp
    user_position.last_reward_claim = current_time;

    Ok(())
}

// Helper function to calculate lending rewards
fn calculate_lending_rewards(
    shares: u64,
    time_staked: i64,
    pool_state: &StablePoolState,
) -> Result<u64> {
    // Calculate rewards based on share of pool and stable_per_share rate
    let base_reward = (shares as u128)
        .checked_mul(time_staked as u128)
        .ok_or(StablecoinVaultError::MathOverflow)?
        .checked_mul(pool_state.stable_per_share as u128)
        .ok_or(StablecoinVaultError::MathOverflow)?
        .checked_div(1_000_000_000)
        .ok_or(StablecoinVaultError::MathOverflow)? as u64;

    Ok(base_reward)
}

// Helper function to calculate treasury rewards
fn calculate_treasury_rewards(
    amount: u64,
    time_staked: i64,
    apy_points: u16,
) -> Result<u64> {
    // Calculate rewards based on fixed APY (in basis points)
    let rewards = (amount as u128)
        .checked_mul(time_staked as u128)
        .ok_or(StablecoinVaultError::MathOverflow)?
        .checked_mul(apy_points as u128)
        .ok_or(StablecoinVaultError::MathOverflow)?
        .checked_div(365 * 24 * 60 * 60 * 10000)  // Convert APY to per-second rate
        .ok_or(StablecoinVaultError::MathOverflow)? as u64;

    Ok(rewards)
}

// Helper function to transfer rewards
fn transfer_rewards(
    ctx: Context<ClaimStableRewards>,
    reward_amount: u64,
) -> Result<()> {
    require!(reward_amount > 0, StablecoinVaultError::InvalidAmount);

    let config = &ctx.accounts.config;

    // Calculate platform fee
    let fee_amount = (reward_amount as u128)
        .checked_mul(config.platform_fee_bps as u128)
        .ok_or(StablecoinVaultError::MathOverflow)?
        .checked_div(10000)
        .ok_or(StablecoinVaultError::MathOverflow)? as u64;

    let user_reward = reward_amount
        .checked_sub(fee_amount)
        .ok_or(StablecoinVaultError::MathOverflow)?;

    // Transfer rewards to user
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_usdc_account.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: config.to_account_info(),
            },
        ),
        user_reward,
    )?;

    Ok(())
}