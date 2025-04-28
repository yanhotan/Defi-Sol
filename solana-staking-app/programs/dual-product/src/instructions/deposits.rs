use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Token, TokenAccount, Mint, Transfer},
    associated_token::AssociatedToken,
};
use crate::state::{DualConfig, DualPool, DualPosition};
use crate::errors::DualProductError;

#[derive(Accounts)]
pub struct CreateDualPosition<'info> {
    #[account(
        mut,
        seeds = [b"dual_config"],
        bump = config.bump
    )]
    pub config: Account<'info, DualConfig>,

    #[account(
        mut,
        seeds = [b"dual_pool"],
        bump = pool.bump
    )]
    pub pool: Account<'info, DualPool>,

    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<DualPosition>(),
        seeds = [b"user_position", user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, DualPosition>,

    #[account(mut)]
    pub user: Signer<'info>,

    // WSOL token account
    #[account(mut)]
    pub user_wsol_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub pool_wsol_account: Account<'info, TokenAccount>,

    // USDC token account
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub pool_usdc_account: Account<'info, TokenAccount>,

    pub wsol_mint: Account<'info, Mint>,
    pub usdc_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddToPosition<'info> {
    #[account(
        mut,
        seeds = [b"dual_config"],
        bump = config.bump
    )]
    pub config: Account<'info, DualConfig>,

    #[account(
        mut,
        seeds = [b"dual_pool"],
        bump = pool.bump
    )]
    pub pool: Account<'info, DualPool>,

    #[account(
        mut,
        seeds = [b"user_position", user.key().as_ref()],
        bump = user_position.bump,
        constraint = user_position.owner == user.key() @ DualProductError::InvalidTokenAccountOwner
    )]
    pub user_position: Account<'info, DualPosition>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_wsol_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub pool_wsol_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub pool_usdc_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn create_dual_position(
    ctx: Context<CreateDualPosition>,
    wsol_amount: u64,
    usdc_amount: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(!config.paused, DualProductError::ProductPaused);
    require!(
        wsol_amount >= config.min_dual_amount,
        DualProductError::BelowMinimumAmount
    );

    // Transfer WSOL
    let wsol_transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_wsol_account.to_account_info(),
            to: ctx.accounts.pool_wsol_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(wsol_transfer_ctx, wsol_amount)?;

    // Transfer USDC
    let usdc_transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_usdc_account.to_account_info(),
            to: ctx.accounts.pool_usdc_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(usdc_transfer_ctx, usdc_amount)?;

    // Update pool state
    let pool = &mut ctx.accounts.pool;
    pool.total_wsol = pool.total_wsol.checked_add(wsol_amount).unwrap();
    pool.total_usdc = pool.total_usdc.checked_add(usdc_amount).unwrap();

    // Initialize user position
    let user_position = &mut ctx.accounts.user_position;
    user_position.owner = ctx.accounts.user.key();
    user_position.wsol_amount = wsol_amount;
    user_position.usdc_amount = usdc_amount;
    user_position.start_time = Clock::get()?.unix_timestamp;
    user_position.last_reward_claim = Clock::get()?.unix_timestamp;
    user_position.bump = *ctx.bumps.get("user_position").unwrap();

    // Update config
    config.total_dual_positions = config.total_dual_positions.checked_add(1).unwrap();
    config.users_count = config.users_count.checked_add(1).unwrap();

    Ok(())
}

pub fn add_to_position(
    ctx: Context<AddToPosition>,
    wsol_amount: u64,
    usdc_amount: u64,
) -> Result<()> {
    let config = &ctx.accounts.config;
    require!(!config.paused, DualProductError::ProductPaused);

    // Transfer WSOL
    let wsol_transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_wsol_account.to_account_info(),
            to: ctx.accounts.pool_wsol_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(wsol_transfer_ctx, wsol_amount)?;

    // Transfer USDC
    let usdc_transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_usdc_account.to_account_info(),
            to: ctx.accounts.pool_usdc_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(usdc_transfer_ctx, usdc_amount)?;

    // Update pool state
    let pool = &mut ctx.accounts.pool;
    pool.total_wsol = pool.total_wsol.checked_add(wsol_amount).unwrap();
    pool.total_usdc = pool.total_usdc.checked_add(usdc_amount).unwrap();

    // Update user position
    let user_position = &mut ctx.accounts.user_position;
    user_position.wsol_amount = user_position.wsol_amount.checked_add(wsol_amount).unwrap();
    user_position.usdc_amount = user_position.usdc_amount.checked_add(usdc_amount).unwrap();

    Ok(())
}