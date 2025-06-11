use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{VaultConfig, UserPosition, StakePosition};
use crate::errors::VaultSolError;

#[derive(Accounts)]
pub struct StakeSol<'info> {
    #[account(
        seeds = [b"vault_sol_config"],
        bump = config.bump,
        constraint = !config.paused @ VaultSolError::VaultPaused,
    )]
    pub config: Account<'info, VaultConfig>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserPosition>(),
        seeds = [b"user_position", user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,

    #[account(mut)]
    pub user: Signer<'info>,

    // LST Token accounts
    pub vsol_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_vsol_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ VaultSolError::InvalidAuthority
    )]
    pub treasury: SystemAccount<'info>,

    // System accounts
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnstakeSol<'info> {
    #[account(
        seeds = [b"vault_sol_config"],
        bump = config.bump,
        constraint = !config.paused @ VaultSolError::VaultPaused,
    )]
    pub config: Account<'info, VaultConfig>,

    #[account(
        mut,
        seeds = [b"user_position", user.key().as_ref()],
        bump = user_position.bump,
        constraint = user_position.owner == user.key(),
    )]
    pub user_position: Account<'info, UserPosition>,

    #[account(mut)]
    pub user: Signer<'info>,

    // LST Token accounts
    #[account(mut)]
    pub vsol_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_vsol_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,

    // System accounts
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateStake<'info> {
    #[account(
        seeds = [b"vault_sol_config"],
        bump = config.bump,
        constraint = !config.paused @ VaultSolError::VaultPaused,
    )]
    pub config: Account<'info, VaultConfig>,

    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<StakePosition>(),
        seeds = [b"stake_position", user.key().as_ref()],
        bump
    )]
    pub stake_position: Account<'info, StakePosition>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(
        seeds = [b"vault_sol_config"],
        bump = config.bump,
        constraint = !config.paused @ VaultSolError::VaultPaused,
    )]
    pub config: Account<'info, VaultConfig>,

    #[account(
        mut,
        seeds = [b"stake_position", user.key().as_ref()],
        bump = stake_position.bump,
        constraint = stake_position.owner == user.key(),
        close = user
    )]
    pub stake_position: Account<'info, StakePosition>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,
}

pub fn stake_sol(ctx: Context<StakeSol>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultSolError::InvalidAmount);
    
    let config = &ctx.accounts.config;
    let user = &ctx.accounts.user;
    let user_position = &mut ctx.accounts.user_position;
    
    // Transfer SOL from user to vault
    invoke(
        &system_instruction::transfer(
            user.key,
            &config.treasury,
            amount
        ),
        &[ 
            user.to_account_info(),
            ctx.accounts.treasury.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Update user position
    if user_position.owner == Pubkey::default() {
        user_position.owner = user.key();
        user_position.provider_used = config.active_provider;
        user_position.bump = *ctx.bumps.get("user_position").unwrap();
    }
    
    user_position.amount_staked = user_position.amount_staked.checked_add(amount)
        .ok_or(VaultSolError::MathOverflow)?;
    user_position.vsol_minted = user_position.vsol_minted.checked_add(amount)
        .ok_or(VaultSolError::MathOverflow)?;
    user_position.deposit_timestamp = Clock::get()?.unix_timestamp;

    // Mint vSOL to user
    anchor_spl::token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.vsol_mint.to_account_info(),
                to: ctx.accounts.user_vsol_account.to_account_info(),
                authority: config.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}

pub fn unstake_sol(ctx: Context<UnstakeSol>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultSolError::InvalidAmount);
    
    let config = &ctx.accounts.config;
    let user_position = &mut ctx.accounts.user_position;
    
    require!(
        user_position.vsol_minted >= amount,
        VaultSolError::InsufficientBalance
    );

    // Calculate fees
    let fee_amount = (amount as u128)
        .checked_mul(config.platform_fee_bps as u128)
        .ok_or(VaultSolError::MathOverflow)?
        .checked_div(10000)
        .ok_or(VaultSolError::MathOverflow)? as u64;
    
    let withdraw_amount = amount.checked_sub(fee_amount)
        .ok_or(VaultSolError::MathOverflow)?;

    // Burn vSOL
    anchor_spl::token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.vsol_mint.to_account_info(),
                from: ctx.accounts.user_vsol_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    // Transfer SOL back to user
    **ctx.accounts.treasury.try_borrow_mut_lamports()? = ctx
        .accounts
        .treasury
        .lamports()
        .checked_sub(withdraw_amount)
        .ok_or(VaultSolError::InsufficientBalance)?;
    **ctx.accounts.user.try_borrow_mut_lamports()? = ctx
        .accounts
        .user
        .lamports()
        .checked_add(withdraw_amount)
        .ok_or(VaultSolError::MathOverflow)?;

    // Update user position
    user_position.amount_staked = user_position.amount_staked
        .checked_sub(amount)
        .ok_or(VaultSolError::MathOverflow)?;
    user_position.vsol_minted = user_position.vsol_minted
        .checked_sub(amount)
        .ok_or(VaultSolError::MathOverflow)?;

    Ok(())
}

pub fn create_stake(
    ctx: Context<CreateStake>,
    amount: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let stake_position = &mut ctx.accounts.stake_position;
    let user = &ctx.accounts.user;

    // Validate amount
    require!(amount > 0, VaultSolError::InvalidAmount);
    require!(
        amount >= config.min_stake_amount,
        VaultSolError::BelowMinimumStake
    );
    require!(
        user.lamports() >= amount,
        VaultSolError::InsufficientBalance
    );

    // Transfer SOL from user to vault treasury
    anchor_lang::solana_program::program::invoke(
        &anchor_lang::solana_program::system_instruction::transfer(
            user.key,
            &ctx.accounts.treasury.key(),
            amount,
        ),
        &[ 
            user.to_account_info(),
            ctx.accounts.treasury.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Initialize stake position
    let current_time = Clock::get()?.unix_timestamp;
    stake_position.owner = user.key();
    stake_position.amount = amount;
    stake_position.start_time = current_time;
    stake_position.last_reward_claim = current_time;
    stake_position.bump = *ctx.bumps.get("stake_position").unwrap();

    // Update vault config
    config.total_staked = config.total_staked
        .checked_add(amount)
        .ok_or(VaultSolError::MathOverflow)?;
    config.stakers_count = config.stakers_count
        .checked_add(1)
        .ok_or(VaultSolError::MathOverflow)?;

    Ok(())
}

pub fn withdraw_stake(
    ctx: Context<WithdrawStake>,
    amount: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let stake_position = &ctx.accounts.stake_position;

    // Validate withdrawal amount
    require!(amount > 0, VaultSolError::InvalidAmount);
    require!(
        amount <= stake_position.amount,
        VaultSolError::InsufficientBalance
    );

    // Validate treasury account matches config
    require!(
        ctx.accounts.treasury.key() == config.treasury,
        VaultSolError::InvalidAuthority
    );

    // Transfer SOL back to user
    **ctx.accounts.treasury.try_borrow_mut_lamports()? = ctx
        .accounts
        .treasury
        .lamports()
        .checked_sub(amount)
        .ok_or(VaultSolError::MathOverflow)?;

    **ctx.accounts.user.try_borrow_mut_lamports()? = ctx
        .accounts
        .user
        .lamports()
        .checked_add(amount)
        .ok_or(VaultSolError::MathOverflow)?;

    // Update vault config
    config.total_staked = config.total_staked
        .checked_sub(amount)
        .ok_or(VaultSolError::MathOverflow)?;
    
    if amount == stake_position.amount {
        config.stakers_count = config.stakers_count
            .checked_sub(1)
            .ok_or(VaultSolError::MathOverflow)?;
    }

    Ok(())
}