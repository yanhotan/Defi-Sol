use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};
use crate::state::{LockingVaultConfig, UserLockPosition, LockPoolState, AssetType};
use crate::errors::LockingVaultError;

#[derive(Accounts)]
pub struct CreateLockPosition<'info> {
    #[account(
        seeds = [b"locking_vault_config"],
        bump = config.bump,
        constraint = !config.paused @ LockingVaultError::VaultPaused,
    )]
    pub config: Account<'info, LockingVaultConfig>,

    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<UserLockPosition>(),
        seeds = [b"user_lock_position", user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserLockPosition>,

    #[account(
        mut,
        seeds = [b"lock_pool_state"],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, LockPoolState>,

    #[account(mut)]
    pub user: Signer<'info>,

    // For USDC deposits
    pub usdc_mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    pub user_usdc_account: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub vault_usdc_account: Option<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_lock_position(
    ctx: Context<CreateLockPosition>,
    amount: u64,
    asset_type: AssetType,
    lock_period: u16,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user = &ctx.accounts.user;
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &mut ctx.accounts.pool_state;

    // Validate amount and lock period
    require!(amount > 0, LockingVaultError::InvalidAmount);
    require!(
        amount >= config.min_deposit_amount,
        LockingVaultError::BelowMinimumDeposit
    );

    // Find and validate lock period and multiplier
    let (period_idx, _) = config.available_lock_periods
        .iter()
        .enumerate()
        .find(|(_, &p)| p == lock_period)
        .ok_or(LockingVaultError::InvalidLockPeriod)?;

    let multiplier = config.lock_period_multipliers[period_idx];

    // Handle asset transfer based on type
    match asset_type {
        AssetType::SOL => {
            // Transfer SOL from user to vault
            require!(
                user.lamports() >= amount,
                LockingVaultError::InsufficientBalance
            );

            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    user.key,
                    &config.treasury,
                    amount,
                ),
                &[
                    user.to_account_info(),
                    ctx.accounts.treasury.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;

            pool_state.total_sol_locked = pool_state.total_sol_locked
                .checked_add(amount)
                .ok_or(LockingVaultError::MathOverflow)?;
        },
        AssetType::USDC => {
            // Validate USDC accounts are provided
            require!(
                ctx.accounts.usdc_mint.is_some() &&
                ctx.accounts.user_usdc_account.is_some() &&
                ctx.accounts.vault_usdc_account.is_some(),
                LockingVaultError::InvalidTokenAccount
            );

            // Transfer USDC tokens
            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.user_usdc_account.as_ref().unwrap().to_account_info(),
                        to: ctx.accounts.vault_usdc_account.as_ref().unwrap().to_account_info(),
                        authority: user.to_account_info(),
                    },
                ),
                amount,
            )?;

            pool_state.total_usdc_locked = pool_state.total_usdc_locked
                .checked_add(amount)
                .ok_or(LockingVaultError::MathOverflow)?;
        },
    }

    // Calculate unlock timestamp
    let current_time = Clock::get()?.unix_timestamp;
    let unlock_time = current_time
        .checked_add((lock_period as i64) * 24 * 60 * 60)  // Convert days to seconds
        .ok_or(LockingVaultError::MathOverflow)?;

    // Initialize user position
    user_position.owner = user.key();
    user_position.asset_type = asset_type;
    user_position.amount = amount;
    user_position.lock_period = lock_period;
    user_position.apy_multiplier = multiplier;
    user_position.start_timestamp = current_time;
    user_position.unlock_timestamp = unlock_time;
    user_position.last_reward_claim = current_time;
    user_position.bump = *ctx.bumps.get("user_position").unwrap();

    // Update pool state
    pool_state.last_update = current_time;

    Ok(())
}