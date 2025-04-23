use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{DualProductConfig, UserDualPosition, PoolState, RewardSource};
use crate::errors::DualProductError;

#[derive(Accounts)]
pub struct ClaimDualRewards<'info> {
    #[account(
        seeds = [b"dual_product_config"],
        bump = config.bump,
        constraint = !config.paused @ DualProductError::ProductPaused,
    )]
    pub config: Account<'info, DualProductConfig>,

    #[account(
        mut,
        seeds = [b"user_dual_position", user.key().as_ref()],
        bump = user_position.bump,
        constraint = user_position.owner == user.key(),
    )]
    pub user_position: Account<'info, UserDualPosition>,

    #[account(
        seeds = [b"pool_state"],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut)]
    pub user: Signer<'info>,
    
    // LST reward token accounts
    #[account(mut)]
    pub user_lst_reward_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_lst_reward_account: Account<'info, TokenAccount>,
    
    // USDC reward token accounts
    #[account(mut)]
    pub user_usdc_reward_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_usdc_reward_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn claim_dual_rewards(
    ctx: Context<ClaimDualRewards>,
    reward_source: RewardSource,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &ctx.accounts.pool_state;
    
    // Calculate rewards based on staking duration and reward source
    let current_time = Clock::get()?.unix_timestamp;
    let time_staked = current_time
        .checked_sub(user_position.last_reward_claim)
        .ok_or(DualProductError::MathOverflow)?;
    
    require!(time_staked > 0, DualProductError::InvalidAmount);

    match reward_source {
        RewardSource::LST => {
            // Calculate LST rewards based on staking duration
            let lst_rewards = calculate_lst_rewards(
                user_position.lst_amount,
                time_staked,
                pool_state.lst_per_share,
            )?;

            // Apply platform fee
            let lst_fee = (lst_rewards as u128)
                .checked_mul(config.platform_fee_bps as u128)
                .ok_or(DualProductError::MathOverflow)?
                .checked_div(10000)
                .ok_or(DualProductError::MathOverflow)? as u64;

            let lst_to_user = lst_rewards.checked_sub(lst_fee)
                .ok_or(DualProductError::MathOverflow)?;

            // Transfer LST rewards
            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: ctx.accounts.vault_lst_reward_account.to_account_info(),
                        to: ctx.accounts.user_lst_reward_account.to_account_info(),
                        authority: config.to_account_info(),
                    },
                ),
                lst_to_user,
            )?;
        },
        RewardSource::LP => {
            require!(user_position.in_lp, DualProductError::PositionInLP);
            
            // Calculate LP rewards
            let lp_rewards = calculate_lp_rewards(
                user_position.lst_amount,
                user_position.usdc_amount,
                time_staked,
                pool_state,
            )?;

            // Split rewards into LST and USDC components
            let (lst_lp_rewards, usdc_lp_rewards) = lp_rewards;

            // Apply platform fees
            let lst_fee = (lst_lp_rewards as u128)
                .checked_mul(config.platform_fee_bps as u128)
                .ok_or(DualProductError::MathOverflow)?
                .checked_div(10000)
                .ok_or(DualProductError::MathOverflow)? as u64;

            let usdc_fee = (usdc_lp_rewards as u128)
                .checked_mul(config.platform_fee_bps as u128)
                .ok_or(DualProductError::MathOverflow)?
                .checked_div(10000)
                .ok_or(DualProductError::MathOverflow)? as u64;

            let lst_to_user = lst_lp_rewards.checked_sub(lst_fee)
                .ok_or(DualProductError::MathOverflow)?;
            let usdc_to_user = usdc_lp_rewards.checked_sub(usdc_fee)
                .ok_or(DualProductError::MathOverflow)?;

            // Transfer LP rewards
            if lst_to_user > 0 {
                anchor_spl::token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        anchor_spl::token::Transfer {
                            from: ctx.accounts.vault_lst_reward_account.to_account_info(),
                            to: ctx.accounts.user_lst_reward_account.to_account_info(),
                            authority: config.to_account_info(),
                        },
                    ),
                    lst_to_user,
                )?;
            }

            if usdc_to_user > 0 {
                anchor_spl::token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        anchor_spl::token::Transfer {
                            from: ctx.accounts.vault_usdc_reward_account.to_account_info(),
                            to: ctx.accounts.user_usdc_reward_account.to_account_info(),
                            authority: config.to_account_info(),
                        },
                    ),
                    usdc_to_user,
                )?;
            }
        },
        RewardSource::Both => {
            // Recursively call for both reward sources
            claim_dual_rewards(ctx.clone(), RewardSource::LST)?;
            claim_dual_rewards(ctx.clone(), RewardSource::LP)?;
        },
    }

    // Update last claim timestamp
    user_position.last_reward_claim = current_time;

    Ok(())
}

// Helper function to calculate LST staking rewards
fn calculate_lst_rewards(
    lst_amount: u64,
    time_staked: i64,
    lst_per_share: u64,
) -> Result<u64> {
    // Simple reward calculation based on amount staked and time
    // In production, this would use more complex tokenomics
    let base_reward = (lst_amount as u128)
        .checked_mul(time_staked as u128)
        .ok_or(DualProductError::MathOverflow)?
        .checked_mul(lst_per_share as u128)
        .ok_or(DualProductError::MathOverflow)?
        .checked_div(1_000_000_000)
        .ok_or(DualProductError::MathOverflow)? as u64;

    Ok(base_reward)
}

// Helper function to calculate LP rewards
fn calculate_lp_rewards(
    lst_amount: u64,
    usdc_amount: u64,
    time_staked: i64,
    pool_state: &PoolState,
) -> Result<(u64, u64)> {
    // Calculate share of pool
    let total_value = (lst_amount as u128)
        .checked_add(usdc_amount as u128)
        .ok_or(DualProductError::MathOverflow)?;
    
    let pool_total = (pool_state.total_lst as u128)
        .checked_add(pool_state.total_usdc as u128)
        .ok_or(DualProductError::MathOverflow)?;
    
    let share_ratio = total_value
        .checked_mul(1_000_000_000)
        .ok_or(DualProductError::MathOverflow)?
        .checked_div(pool_total)
        .ok_or(DualProductError::MathOverflow)? as u64;

    // Calculate rewards for each token type
    let lst_reward = (share_ratio as u128)
        .checked_mul(time_staked as u128)
        .ok_or(DualProductError::MathOverflow)?
        .checked_mul(pool_state.lst_per_share as u128)
        .ok_or(DualProductError::MathOverflow)?
        .checked_div(1_000_000_000)
        .ok_or(DualProductError::MathOverflow)? as u64;

    let usdc_reward = (share_ratio as u128)
        .checked_mul(time_staked as u128)
        .ok_or(DualProductError::MathOverflow)?
        .checked_mul(pool_state.usdc_per_share as u128)
        .ok_or(DualProductError::MathOverflow)?
        .checked_div(1_000_000_000)
        .ok_or(DualProductError::MathOverflow)? as u64;

    Ok((lst_reward, usdc_reward))
}