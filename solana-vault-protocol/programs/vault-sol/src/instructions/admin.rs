use anchor_lang::prelude::*;
use crate::state::{VaultConfig, RewardsPool, LSTProvider};
use crate::errors::VaultSolError;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<VaultConfig>(),
        seeds = [b"vault_sol_config"],
        bump
    )]
    pub config: Account<'info, VaultConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<RewardsPool>(),
        seeds = [b"rewards_pool"],
        bump
    )]
    pub rewards_pool: Account<'info, RewardsPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAPY<'info> {
    #[account(
        mut,
        seeds = [b"rewards_pool"],
        bump = rewards_pool.bump,
    )]
    pub rewards_pool: Account<'info, RewardsPool>,
    
    // We need to check the authority against the config, since RewardsPool doesn't have authority field
    #[account(
        seeds = [b"vault_sol_config"],
        bump,
        constraint = config.authority == authority.key()
    )]
    pub config: Account<'info, VaultConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddRewards<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"vault_sol_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, VaultConfig>,

    #[account(
        mut,
        seeds = [b"rewards_pool"],
        bump = rewards_pool.bump,
    )]
    pub rewards_pool: Account<'info, RewardsPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct PauseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"vault_sol_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, VaultConfig>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnpauseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"vault_sol_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, VaultConfig>,

    pub authority: Signer<'info>,
}

pub fn initialize_vault(
    ctx: Context<InitializeVault>,
    platform_fee_bps: u16,
    min_stake: u64,
) -> Result<()> {
    require!(platform_fee_bps <= 10000, VaultSolError::InvalidFee);
    require!(min_stake > 0, VaultSolError::InvalidAmount);

    // Initialize config
    let config = &mut ctx.accounts.config;
    let config_bump = *ctx.bumps.get("config").unwrap();

    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.treasury.key();
    config.platform_fee_bps = platform_fee_bps;
    config.min_stake_amount = min_stake;
    config.total_staked = 0;
    config.stakers_count = 0;
    config.paused = false;
    config.active_provider = LSTProvider::None;  // Initialize with no LST provider
    config.bump = config_bump;

    // Initialize rewards pool
    let rewards_pool = &mut ctx.accounts.rewards_pool;
    let pool_bump = *ctx.bumps.get("rewards_pool").unwrap();

    rewards_pool.total_rewards = 0;
    rewards_pool.apy_points = 500;  // Start with 5% APY
    rewards_pool.last_update = Clock::get()?.unix_timestamp;
    rewards_pool.distributed_rewards = 0;
    rewards_pool.bump = pool_bump;

    Ok(())
}

pub fn update_apy(
    ctx: Context<UpdateAPY>,
    new_apy: u16,
) -> Result<()> {
    require!(new_apy <= 10000, VaultSolError::InvalidAPY); // Max 100% APY
    
    let rewards_pool = &mut ctx.accounts.rewards_pool;
    rewards_pool.apy_points = new_apy;
    rewards_pool.last_update = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn add_rewards(
    ctx: Context<AddRewards>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, VaultSolError::InvalidAmount);
    
    // Get account info references first, before mutably borrowing rewards_pool
    let authority_info = ctx.accounts.authority.to_account_info();
    let rewards_pool_info = ctx.accounts.rewards_pool.to_account_info();
    
    // Now we can safely mutably borrow rewards_pool
    let rewards_pool = &mut ctx.accounts.rewards_pool;
    
    // Initialize rewards pool if it's the first time
    if rewards_pool.total_rewards == 0 {
        rewards_pool.apy_points = 500; // Default 5% APY
        rewards_pool.distributed_rewards = 0;
        rewards_pool.bump = *ctx.bumps.get("rewards_pool").unwrap();
    }
    
    // Transfer SOL to rewards pool
    require!(
        ctx.accounts.authority.lamports() >= amount,
        VaultSolError::InsufficientBalance
    );

    // Transfer lamports from authority to rewards pool
    **authority_info.try_borrow_mut_lamports()? = authority_info
        .lamports()
        .checked_sub(amount)
        .ok_or(VaultSolError::MathOverflow)?;

    **rewards_pool_info.try_borrow_mut_lamports()? = rewards_pool_info
        .lamports()
        .checked_add(amount)
        .ok_or(VaultSolError::MathOverflow)?;

    rewards_pool.total_rewards = rewards_pool.total_rewards
        .checked_add(amount)
        .ok_or(VaultSolError::MathOverflow)?;

    rewards_pool.last_update = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn pause_vault(ctx: Context<PauseVault>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.paused = true;
    Ok(())
}

pub fn unpause_vault(ctx: Context<UnpauseVault>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.paused = false;
    Ok(())
}