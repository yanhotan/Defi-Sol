use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};
use crate::state::{DualProductConfig, UserDualPosition, PoolState};
use crate::errors::DualProductError;

#[derive(Accounts)]
pub struct WithdrawDual<'info> {
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
        constraint = !user_position.in_lp @ DualProductError::PositionInLP,
    )]
    pub user_position: Account<'info, UserDualPosition>,

    #[account(
        mut,
        seeds = [b"pool_state"],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut)]
    pub user: Signer<'info>,

    // LST Token accounts
    pub lst_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_lst_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_lst_account: Account<'info, TokenAccount>,

    // USDC Token accounts
    pub usdc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_usdc_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn withdraw_dual(
    ctx: Context<WithdrawDual>,
    lst_amount: u64,
    usdc_amount: u64,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &mut ctx.accounts.pool_state;

    // Validate withdrawal amounts
    require!(
        lst_amount <= user_position.lst_amount && 
        usdc_amount <= user_position.usdc_amount,
        DualProductError::InsufficientBalance
    );

    // Calculate fees
    let lst_fee = (lst_amount as u128)
        .checked_mul(config.platform_fee_bps as u128)
        .ok_or(DualProductError::MathOverflow)?
        .checked_div(10000)
        .ok_or(DualProductError::MathOverflow)? as u64;
    
    let usdc_fee = (usdc_amount as u128)
        .checked_mul(config.platform_fee_bps as u128)
        .ok_or(DualProductError::MathOverflow)?
        .checked_div(10000)
        .ok_or(DualProductError::MathOverflow)? as u64;

    let lst_withdraw = lst_amount.checked_sub(lst_fee)
        .ok_or(DualProductError::MathOverflow)?;
    let usdc_withdraw = usdc_amount.checked_sub(usdc_fee)
        .ok_or(DualProductError::MathOverflow)?;

    // Transfer LST tokens to user
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_lst_account.to_account_info(),
                to: ctx.accounts.user_lst_account.to_account_info(),
                authority: config.to_account_info(),
            },
        ),
        lst_withdraw,
    )?;

    // Transfer USDC tokens to user
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_usdc_account.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: config.to_account_info(),
            },
        ),
        usdc_withdraw,
    )?;

    // Update user position
    user_position.lst_amount = user_position.lst_amount
        .checked_sub(lst_amount)
        .ok_or(DualProductError::MathOverflow)?;
    user_position.usdc_amount = user_position.usdc_amount
        .checked_sub(usdc_amount)
        .ok_or(DualProductError::MathOverflow)?;

    // Update pool state
    pool_state.total_lst = pool_state.total_lst
        .checked_sub(lst_amount)
        .ok_or(DualProductError::MathOverflow)?;
    pool_state.total_usdc = pool_state.total_usdc
        .checked_sub(usdc_amount)
        .ok_or(DualProductError::MathOverflow)?;
    pool_state.last_update = Clock::get()?.unix_timestamp;

    Ok(())
}