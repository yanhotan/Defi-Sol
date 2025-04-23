use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};
use crate::state::{StablecoinVaultConfig, UserStablePosition, StablePoolState};
use crate::errors::StablecoinVaultError;

#[derive(Accounts)]
pub struct DepositStable<'info> {
    #[account(
        seeds = [b"stable_vault_config"],
        bump = config.bump,
        constraint = !config.paused @ StablecoinVaultError::VaultPaused,
    )]
    pub config: Account<'info, StablecoinVaultConfig>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserStablePosition>(),
        seeds = [b"user_stable_position", user.key().as_ref()],
        bump
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

    // USDC Token accounts
    pub usdc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_usdc_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn deposit_stable(
    ctx: Context<DepositStable>,
    amount: u64,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user = &ctx.accounts.user;
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &mut ctx.accounts.pool_state;

    // Validate deposit amount
    require!(amount > 0, StablecoinVaultError::InvalidAmount);
    require!(
        amount >= config.min_deposit_amount,
        StablecoinVaultError::BelowMinimumDeposit
    );

    // Transfer USDC tokens to vault
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_usdc_account.to_account_info(),
                to: ctx.accounts.vault_usdc_account.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        amount,
    )?;

    // Calculate shares to mint
    let shares = if pool_state.total_shares == 0 {
        // Initial deposit
        amount
    } else {
        // Calculate based on proportion of pool
        (amount as u128)
            .checked_mul(pool_state.total_shares as u128)
            .ok_or(StablecoinVaultError::MathOverflow)?
            .checked_div(pool_state.total_deposits as u128)
            .ok_or(StablecoinVaultError::MathOverflow)? as u64
    };

    // Initialize user position if new
    if user_position.owner == Pubkey::default() {
        user_position.owner = user.key();
        user_position.bump = *ctx.bumps.get("user_position").unwrap();
    }

    // Update user position
    user_position.stablecoin_amount = user_position.stablecoin_amount
        .checked_add(amount)
        .ok_or(StablecoinVaultError::MathOverflow)?;
    user_position.shares = user_position.shares
        .checked_add(shares)
        .ok_or(StablecoinVaultError::MathOverflow)?;
    user_position.deposit_timestamp = Clock::get()?.unix_timestamp;

    // Update pool state
    pool_state.total_deposits = pool_state.total_deposits
        .checked_add(amount)
        .ok_or(StablecoinVaultError::MathOverflow)?;
    pool_state.total_shares = pool_state.total_shares
        .checked_add(shares)
        .ok_or(StablecoinVaultError::MathOverflow)?;
    pool_state.last_update = Clock::get()?.unix_timestamp;

    Ok(())
}