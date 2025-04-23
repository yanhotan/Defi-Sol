use anchor_lang::prelude::*;
use crate::state::StablecoinVaultConfig;
use crate::errors::StablecoinVaultError;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<StablecoinVaultConfig>(),
        seeds = [b"stable_vault_config"],
        bump
    )]
    pub config: Account<'info, StablecoinVaultConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateLendingRatio<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"stable_vault_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, StablecoinVaultConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ToggleLending<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"stable_vault_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, StablecoinVaultConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct PauseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"stable_vault_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, StablecoinVaultConfig>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnpauseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"stable_vault_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, StablecoinVaultConfig>,
    
    pub authority: Signer<'info>,
}

pub fn initialize_vault(
    ctx: Context<InitializeVault>,
    platform_fee_bps: u16,
    min_deposit: u64,
    lending_ratio: u16,
) -> Result<()> {
    require!(platform_fee_bps <= 10000, StablecoinVaultError::InvalidAmount);
    require!(min_deposit > 0, StablecoinVaultError::InvalidAmount);
    require!(lending_ratio <= 10000, StablecoinVaultError::InvalidLendingRatio);

    let config = &mut ctx.accounts.config;
    let bump = *ctx.bumps.get("config").unwrap();

    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.treasury.key();
    config.platform_fee_bps = platform_fee_bps;
    config.min_deposit_amount = min_deposit;
    config.lending_enabled = false;  // Start with lending disabled
    config.paused = false;
    config.bump = bump;

    Ok(())
}

pub fn update_lending_ratio(
    ctx: Context<UpdateLendingRatio>,
    new_ratio: u16,
) -> Result<()> {
    require!(new_ratio <= 10000, StablecoinVaultError::InvalidLendingRatio);
    
    let config = &mut ctx.accounts.config;
    require!(config.lending_enabled, StablecoinVaultError::LendingDisabled);
    
    Ok(())
}

pub fn toggle_lending(
    ctx: Context<ToggleLending>,
    enabled: bool,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.lending_enabled = enabled;
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