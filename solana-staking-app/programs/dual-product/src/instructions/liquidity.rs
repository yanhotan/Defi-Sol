use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{DualProductConfig, UserDualPosition, PoolState};
use crate::errors::DualProductError;

#[derive(Accounts)]
pub struct AddToLP<'info> {
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

    // LP Token accounts and AMM accounts would be added here
    // This is a simplified version without actual LP integration
}

#[derive(Accounts)]
pub struct RemoveFromLP<'info> {
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
        constraint = user_position.in_lp @ DualProductError::PositionInLP,
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

    // LP Token accounts and AMM accounts would be added here
    // This is a simplified version without actual LP integration
}

pub fn add_to_lp(ctx: Context<AddToLP>) -> Result<()> {
    let user_position = &mut ctx.accounts.user_position;
    let pool_state = &mut ctx.accounts.pool_state;

    // Calculate shares to mint based on contribution
    let share_amount = if pool_state.total_shares == 0 {
        // Initial liquidity provision
        (user_position.lst_amount as u128)
            .checked_add(user_position.usdc_amount as u128)
            .ok_or(DualProductError::MathOverflow)? as u64
    } else {
        // Calculate based on proportion of existing liquidity
        let lst_share = (user_position.lst_amount as u128)
            .checked_mul(pool_state.total_shares as u128)
            .ok_or(DualProductError::MathOverflow)?
            .checked_div(pool_state.total_lst as u128)
            .ok_or(DualProductError::MathOverflow)? as u64;

        let usdc_share = (user_position.usdc_amount as u128)
            .checked_mul(pool_state.total_shares as u128)
            .ok_or(DualProductError::MathOverflow)?
            .checked_div(pool_state.total_usdc as u128)
            .ok_or(DualProductError::MathOverflow)? as u64;

        std::cmp::min(lst_share, usdc_share)
    };

    // Update pool state
    pool_state.total_shares = pool_state.total_shares
        .checked_add(share_amount)
        .ok_or(DualProductError::MathOverflow)?;
    
    // Mark position as in LP
    user_position.in_lp = true;

    Ok(())
}

pub fn remove_from_lp(ctx: Context<RemoveFromLP>) -> Result<()> {
    let user_position = &mut ctx.accounts.user_position;
    
    // In a real implementation, this would:
    // 1. Calculate share of LP tokens
    // 2. Remove liquidity from AMM
    // 3. Update user_position with resulting token amounts
    // 4. Update pool state
    
    // For now, just mark as removed from LP
    user_position.in_lp = false;

    Ok(())
}