use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::{LockingVaultConfig, UserLockPosition, LockPoolState, AssetType, WithdrawType};
use crate::errors::LockingVaultError;

#[derive(Accounts)]
pub struct WithdrawLocked<'info> {
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
        mut,
        seeds = [b"lock_pool_state"],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, LockPoolState>,

    #[account(mut)]
    pub user: Signer<'info>,

    // For USDC withdrawals
    #[account(mut)]
    pub user_usdc_account: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub vault_usdc_account: Option<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn withdraw_locked(
    ctx: Context<WithdrawLocked>,
    amount: u64,
    withdraw_type: WithdrawType,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &mut ctx.accounts.pool_state;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate withdrawal amount
    require!(amount > 0, LockingVaultError::InvalidAmount);
    require!(
        amount <= user_position.amount,
        LockingVaultError::InsufficientBalance
    );

    // Calculate any penalties for early withdrawal
    let (withdrawal_amount, penalty_amount) = match withdraw_type {
        WithdrawType::Normal => {
            require!(
                current_time >= user_position.unlock_timestamp,
                LockingVaultError::PositionLocked
            );
            (amount, 0)
        },
        WithdrawType::Early => {
            require!(
                current_time < user_position.unlock_timestamp,
                LockingVaultError::PositionUnlocked
            );
            
            // Calculate early withdrawal penalty (20%)
            let penalty = (amount as u128)
                .checked_mul(2000)
                .ok_or(LockingVaultError::MathOverflow)?
                .checked_div(10000)
                .ok_or(LockingVaultError::MathOverflow)? as u64;

            let withdraw = amount
                .checked_sub(penalty)
                .ok_or(LockingVaultError::MathOverflow)?;

            (withdraw, penalty)
        },
    };

    // Process withdrawal based on asset type
    match user_position.asset_type {
        AssetType::SOL => {
            // Transfer SOL back to user
            **ctx.accounts.treasury.try_borrow_mut_lamports()? = ctx
                .accounts
                .treasury
                .lamports()
                .checked_sub(withdrawal_amount)
                .ok_or(LockingVaultError::MathOverflow)?;

            **ctx.accounts.user.try_borrow_mut_lamports()? = ctx
                .accounts
                .user
                .lamports()
                .checked_add(withdrawal_amount)
                .ok_or(LockingVaultError::MathOverflow)?;

            pool_state.total_sol_locked = pool_state.total_sol_locked
                .checked_sub(amount)
                .ok_or(LockingVaultError::MathOverflow)?;
        },
        AssetType::USDC => {
            // Validate USDC accounts are provided
            require!(
                ctx.accounts.user_usdc_account.is_some() &&
                ctx.accounts.vault_usdc_account.is_some(),
                LockingVaultError::InvalidTokenAccount
            );

            // Transfer USDC tokens back to user
            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.vault_usdc_account.as_ref().unwrap().to_account_info(),
                        to: ctx.accounts.user_usdc_account.as_ref().unwrap().to_account_info(),
                        authority: config.to_account_info(),
                    },
                ),
                withdrawal_amount,
            )?;

            pool_state.total_usdc_locked = pool_state.total_usdc_locked
                .checked_sub(amount)
                .ok_or(LockingVaultError::MathOverflow)?;
        },
    }

    // Update position amount
    user_position.amount = user_position.amount
        .checked_sub(amount)
        .ok_or(LockingVaultError::MathOverflow)?;

    // Update pool penalties collected
    if penalty_amount > 0 {
        pool_state.total_penalties = pool_state.total_penalties
            .checked_add(penalty_amount)
            .ok_or(LockingVaultError::MathOverflow)?;
    }

    // Update pool state
    pool_state.last_update = current_time;

    Ok(())
}