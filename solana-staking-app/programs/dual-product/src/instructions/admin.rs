use anchor_lang::prelude::*;
use crate::state::{DualProductConfig, DualConfig, DualPool};
use crate::errors::DualProductError;

#[derive(Accounts)]
pub struct InitializeProduct<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<DualProductConfig>(),
        seeds = [b"dual_product_config"],
        bump
    )]
    pub config: Account<'info, DualProductConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateRatios<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"dual_product_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, DualProductConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct PauseProduct<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"dual_product_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, DualProductConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction()]
pub struct UnpauseProduct<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"dual_product_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, DualProductConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeDualProduct<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<DualConfig>(),
        seeds = [b"dual_config"],
        bump
    )]
    pub config: Account<'info, DualConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<DualPool>(),
        seeds = [b"dual_pool"],
        bump
    )]
    pub pool: Account<'info, DualPool>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePoolParameters<'info> {
    #[account(
        seeds = [b"dual_config"],
        bump = config.bump,
        has_one = authority,
    )]
    pub config: Account<'info, DualConfig>,

    #[account(
        mut,
        seeds = [b"dual_pool"],
        bump = pool.bump,
    )]
    pub pool: Account<'info, DualPool>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct PauseDualProduct<'info> {
    #[account(
        mut,
        seeds = [b"dual_config"],
        bump = config.bump,
        has_one = authority,
    )]
    pub config: Account<'info, DualConfig>,

    pub authority: Signer<'info>,
}

pub fn initialize_product(
    ctx: Context<InitializeProduct>,
    platform_fee_bps: u16,
    min_deposit: u64,
    lst_ratio: u16,
    usdc_ratio: u16,
) -> Result<()> {
    require!(platform_fee_bps <= 10000, DualProductError::InvalidAmount);
    require!(min_deposit > 0, DualProductError::InvalidAmount);
    require!(
        lst_ratio + usdc_ratio == 10000,
        DualProductError::InvalidRatios
    );

    let config = &mut ctx.accounts.config;
    let bump = *ctx.bumps.get("config").unwrap();

    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.treasury.key();
    config.platform_fee_bps = platform_fee_bps;
    config.min_deposit_amount = min_deposit;
    config.lst_ratio = lst_ratio;
    config.usdc_ratio = usdc_ratio;
    config.paused = false;
    config.bump = bump;

    Ok(())
}

pub fn update_ratios(
    ctx: Context<UpdateRatios>,
    new_lst_ratio: u16,
    new_usdc_ratio: u16,
) -> Result<()> {
    require!(
        new_lst_ratio + new_usdc_ratio == 10000,
        DualProductError::InvalidRatios
    );

    let config = &mut ctx.accounts.config;
    config.lst_ratio = new_lst_ratio;
    config.usdc_ratio = new_usdc_ratio;

    Ok(())
}

pub fn pause_product(ctx: Context<PauseProduct>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.paused = true;
    Ok(())
}

pub fn unpause_product(ctx: Context<UnpauseProduct>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.paused = false;
    Ok(())
}

pub fn initialize_dual_product(
    ctx: Context<InitializeDualProduct>,
    platform_fee_bps: u16,
    min_dual_amount: u64,
) -> Result<()> {
    require!(platform_fee_bps <= 1000, DualProductError::InvalidFee); // Max 10% fee

    let config = &mut ctx.accounts.config;
    let pool = &mut ctx.accounts.pool;

    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.authority.key(); // Initially set to authority
    config.platform_fee_bps = platform_fee_bps;
    config.min_dual_amount = min_dual_amount;
    config.total_dual_positions = 0;
    config.users_count = 0;
    config.paused = false;
    config.bump = *ctx.bumps.get("dual_config").unwrap();

    pool.total_wsol = 0;
    pool.total_usdc = 0;
    pool.base_apy_points = 500; // 5% base APY
    pool.tier1_threshold = 30 * 24 * 60 * 60; // 30 days
    pool.tier2_threshold = 90 * 24 * 60 * 60; // 90 days
    pool.tier3_threshold = 180 * 24 * 60 * 60; // 180 days
    pool.tier1_multiplier = 10000; // 1x
    pool.tier2_multiplier = 15000; // 1.5x
    pool.tier3_multiplier = 20000; // 2x
    pool.last_update = Clock::get()?.unix_timestamp;
    pool.rewards_available = 0;
    pool.bump = *ctx.bumps.get("dual_pool").unwrap();

    Ok(())
}

pub fn update_pool_parameters(
    ctx: Context<UpdatePoolParameters>,
    base_apy: Option<u16>,
    tier1_multiplier: Option<u16>,
    tier2_multiplier: Option<u16>,
    tier3_multiplier: Option<u16>,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    if let Some(apy) = base_apy {
        require!(apy <= 10000, DualProductError::InvalidFee); // Max 100% APY
        pool.base_apy_points = apy;
    }

    if let Some(t1) = tier1_multiplier {
        pool.tier1_multiplier = t1;
    }

    if let Some(t2) = tier2_multiplier {
        require!(t2 > pool.tier1_multiplier, DualProductError::InvalidAPYTier);
        pool.tier2_multiplier = t2;
    }

    if let Some(t3) = tier3_multiplier {
        require!(t3 > pool.tier2_multiplier, DualProductError::InvalidAPYTier);
        pool.tier3_multiplier = t3;
    }

    pool.last_update = Clock::get()?.unix_timestamp;
    
    Ok(())
}

pub fn pause_dual_product(ctx: Context<PauseDualProduct>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.paused = true;
    Ok(())
}

pub fn unpause_dual_product(ctx: Context<PauseDualProduct>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.paused = false;
    Ok(())
}