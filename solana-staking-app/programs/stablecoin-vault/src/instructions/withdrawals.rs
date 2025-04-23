use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::{StablecoinVaultConfig, UserStablePosition, StablePoolState};
use crate::errors::StablecoinVaultError;

#[derive(Accounts)]
pub struct WithdrawStable<'info> {
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
        mut,
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

pub fn withdraw_stable(
    ctx: Context<WithdrawStable>,
    amount: u64,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &mut ctx.accounts.pool_state;

    // Validate withdrawal amount
    require!(amount > 0, StablecoinVaultError::InvalidAmount);
    require!(
        amount <= user_position.stablecoin_amount,
        StablecoinVaultError::InsufficientBalance
    );

    // Calculate shares to burn
    let shares_to_burn = (amount as u128)
        .checked_mul(user_position.shares as u128)
        .ok_or(StablecoinVaultError::MathOverflow)?
        .checked_div(user_position.stablecoin_amount as u128)
        .ok_or(StablecoinVaultError::MathOverflow)? as u64;

    // Calculate fees
    let fee_amount = (amount as u128)
        .checked_mul(config.platform_fee_bps as u128)
        .ok_or(StablecoinVaultError::MathOverflow)?
        .checked_div(10000)
        .ok_or(StablecoinVaultError::MathOverflow)? as u64;

    let withdrawal_amount = amount
        .checked_sub(fee_amount)
        .ok_or(StablecoinVaultError::MathOverflow)?;

    // Transfer USDC back to user
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_usdc_account.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: config.to_account_info(),
            },
        ),
        withdrawal_amount,
    )?;

    // Update user position
    user_position.stablecoin_amount = user_position.stablecoin_amount
        .checked_sub(amount)
        .ok_or(StablecoinVaultError::MathOverflow)?;
    user_position.shares = user_position.shares
        .checked_sub(shares_to_burn)
        .ok_or(StablecoinVaultError::MathOverflow)?;

    // Update pool state
    pool_state.total_deposits = pool_state.total_deposits
        .checked_sub(amount)
        .ok_or(StablecoinVaultError::MathOverflow)?;
    pool_state.total_shares = pool_state.total_shares
        .checked_sub(shares_to_burn)
        .ok_or(StablecoinVaultError::MathOverflow)?;
    pool_state.last_update = Clock::get()?.unix_timestamp;

    Ok(())
}