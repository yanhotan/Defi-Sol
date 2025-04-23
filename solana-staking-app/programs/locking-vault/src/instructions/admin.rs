use anchor_lang::prelude::*;
use crate::state::{LockingVaultConfig, LockPoolState};
use crate::errors::LockingVaultError;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<LockingVaultConfig>(),
        seeds = [b"locking_vault_config"],
        bump
    )]
    pub config: Account<'info, LockingVaultConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<LockPoolState>(),
        seeds = [b"lock_pool_state"],
        bump
    )]
    pub pool_state: Account<'info, LockPoolState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateLockPeriods<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"locking_vault_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, LockingVaultConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateBaseAPY<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"lock_pool_state"],
        bump = pool_state.bump,
    )]
    pub pool_state: Account<'info, LockPoolState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct PauseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"locking_vault_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, LockingVaultConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnpauseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"locking_vault_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, LockingVaultConfig>,
    
    pub authority: Signer<'info>,
}

pub fn initialize_vault(
    ctx: Context<InitializeVault>,
    platform_fee_bps: u16,
    min_deposit: u64,
    lock_periods: [u16; 5],
    multipliers: [u16; 5],
) -> Result<()> {
    require!(platform_fee_bps <= 10000, LockingVaultError::InvalidAmount);
    require!(min_deposit > 0, LockingVaultError::InvalidAmount);
    
    // Validate lock periods are in ascending order and multipliers
    for i in 1..5 {
        require!(
            lock_periods[i] > lock_periods[i-1],
            LockingVaultError::InvalidLockPeriods
        );
        require!(
            multipliers[i] > multipliers[i-1],
            LockingVaultError::InvalidMultiplier
        );
    }

    // Initialize config
    let config = &mut ctx.accounts.config;
    let config_bump = *ctx.bumps.get("config").unwrap();

    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.treasury.key();
    config.platform_fee_bps = platform_fee_bps;
    config.min_deposit_amount = min_deposit;
    config.available_lock_periods = lock_periods;
    config.lock_period_multipliers = multipliers;
    config.paused = false;
    config.bump = config_bump;

    // Initialize pool state
    let pool_state = &mut ctx.accounts.pool_state;
    let pool_bump = *ctx.bumps.get("pool_state").unwrap();

    pool_state.total_sol_locked = 0;
    pool_state.total_usdc_locked = 0;
    pool_state.base_apy_points = 500;  // Start with 5% base APY
    pool_state.total_penalties = 0;
    pool_state.last_update = Clock::get()?.unix_timestamp;
    pool_state.bump = pool_bump;

    Ok(())
}

pub fn update_lock_periods(
    ctx: Context<UpdateLockPeriods>,
    new_periods: [u16; 5],
    new_multipliers: [u16; 5],
) -> Result<()> {
    // Validate new periods are in ascending order and multipliers
    for i in 1..5 {
        require!(
            new_periods[i] > new_periods[i-1],
            LockingVaultError::InvalidLockPeriods
        );
        require!(
            new_multipliers[i] > new_multipliers[i-1],
            LockingVaultError::InvalidMultiplier
        );
    }
    
    let config = &mut ctx.accounts.config;
    config.available_lock_periods = new_periods;
    config.lock_period_multipliers = new_multipliers;

    Ok(())
}

pub fn update_base_apy(
    ctx: Context<UpdateBaseAPY>,
    new_base_apy: u16,
) -> Result<()> {
    require!(new_base_apy <= 10000, LockingVaultError::InvalidAmount); // Max 100% APY
    
    let pool_state = &mut ctx.accounts.pool_state;
    pool_state.base_apy_points = new_base_apy;
    pool_state.last_update = Clock::get()?.unix_timestamp;

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