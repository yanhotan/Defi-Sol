use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::StakeLendError,
    instructions::StakeLendInstruction,
    state::{
        Pool, PoolData, LendingPoolData, 
        UserPosition, LiquidationData, pda,
        ProtocolConfig, HealthStatus
    },
    utils::math::{
        calculate_health_factor,
        calculate_liquidation_amount,
        calculate_bonus_amount,
        get_current_timestamp,
    },
};

/// Health factor threshold for healthy positions
pub const HEALTHY_FACTOR_THRESHOLD: u64 = 120; // 120% (represented as percentage * 100)

/// Health factor threshold for liquidation
pub const LIQUIDATION_THRESHOLD: u64 = 105; // 105% (represented as percentage * 100)

/// Minimum health factor for new positions
pub const MIN_INITIAL_HEALTH_FACTOR: u64 = 150; // 150% (represented as percentage * 100)

/// Maximum bonus for liquidators (in basis points)
pub const MAX_LIQUIDATION_BONUS_BPS: u16 = 1000; // 10%

/// Evaluate the health of a user position in the protocol
pub fn evaluate_user_health(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_position_account = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;

    // Verify the user is a signer
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load user position data
    let mut user_position = match UserPosition::try_from_slice(&user_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify ownership
    if user_position.owner != *user_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Get latest asset prices from oracle
    let prices = crate::utils::oracle::get_asset_prices(oracle_account)?;

    // Calculate health factor
    let health_factor = calculate_health_factor(
        &user_position.collateral_assets,
        &user_position.debt_assets,
        &prices,
    )?;

    // Update health status
    let previous_status = user_position.health_status;
    
    user_position.health_status = if health_factor >= HEALTHY_FACTOR_THRESHOLD {
        HealthStatus::Healthy
    } else if health_factor >= LIQUIDATION_THRESHOLD {
        HealthStatus::Warning
    } else {
        HealthStatus::Liquidatable
    };
    
    user_position.health_factor = health_factor;
    user_position.last_health_check = get_current_timestamp();
    
    // Save updated position data
    borsh::to_writer(&mut user_position_account.data.borrow_mut()[..], &user_position)?;
    
    msg!(
        "User health evaluated: factor={}, status={:?} (previous: {:?})",
        health_factor,
        user_position.health_status,
        previous_status
    );
    
    Ok(())
}

/// Process liquidation of an unhealthy position
pub fn process_liquidation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    repay_amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let liquidator_account = next_account_info(accounts_iter)?;
    let liquidator_repay_token_account = next_account_info(accounts_iter)?;
    let liquidator_receive_token_account = next_account_info(accounts_iter)?;
    let borrower_position_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let pool_treasury = next_account_info(accounts_iter)?;
    let protocol_config_account = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify the liquidator is a signer
    if !liquidator_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load user position data
    let mut borrower_position = match UserPosition::try_from_slice(&borrower_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool authority PDA
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Load protocol config for liquidation parameters
    let config = match ProtocolConfig::try_from_slice(&protocol_config_account.data.borrow()) {
        Ok(config) => config,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Check if position is liquidatable
    // First, refresh health factor with latest oracle prices
    let prices = crate::utils::oracle::get_asset_prices(oracle_account)?;
    let health_factor = calculate_health_factor(
        &borrower_position.collateral_assets,
        &borrower_position.debt_assets,
        &prices,
    )?;

    if health_factor >= LIQUIDATION_THRESHOLD {
        return Err(StakeLendError::PositionNotLiquidatable.into());
    }
    
    // Calculate liquidation specifics (amount to repay, collateral to receive)
    let liquidation_data = calculate_liquidation_amount(
        &borrower_position,
        repay_amount,
        &prices,
        config.liquidation_bonus_bps,
    )?;

    // Transfer repayment tokens from liquidator to pool
    let repay_ix = spl_token::instruction::transfer(
        &token_program.key,
        &liquidator_repay_token_account.key,
        &pool_treasury.key,
        &liquidator_account.key,
        &[],
        liquidation_data.repay_amount,
    )?;
    
    invoke(
        &repay_ix,
        &[
            liquidator_repay_token_account.clone(),
            pool_treasury.clone(),
            liquidator_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Transfer collateral to liquidator
    let collateral_ix = spl_token::instruction::transfer(
        &token_program.key,
        &pool_treasury.key,
        &liquidator_receive_token_account.key,
        &pool_authority.key,
        &[],
        liquidation_data.collateral_amount,
    )?;
    
    invoke_signed(
        &collateral_ix,
        &[
            pool_treasury.clone(),
            liquidator_receive_token_account.clone(),
            pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    // Update borrower position
    borrower_position.update_after_liquidation(&liquidation_data)?;
    
    // Recalculate health factor after liquidation
    let new_health_factor = calculate_health_factor(
        &borrower_position.collateral_assets,
        &borrower_position.debt_assets,
        &prices,
    )?;
    
    borrower_position.health_factor = new_health_factor;
    borrower_position.health_status = if new_health_factor >= HEALTHY_FACTOR_THRESHOLD {
        HealthStatus::Healthy
    } else if new_health_factor >= LIQUIDATION_THRESHOLD {
        HealthStatus::Warning
    } else {
        HealthStatus::Liquidatable
    };
    
    // Record liquidation event
    borrower_position.liquidation_history.push(liquidation_data.clone());
    
    // Save updated position data
    borsh::to_writer(&mut borrower_position_account.data.borrow_mut()[..], &borrower_position)?;
    
    msg!(
        "Liquidation processed: repaid={}, collateral_liquidated={}, bonus={}",
        liquidation_data.repay_amount,
        liquidation_data.collateral_amount,
        liquidation_data.bonus_amount
    );
    
    Ok(())
}

/// Process automatic de-risking of protocol pools
pub fn process_protocol_derisking(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_account = next_account_info(accounts_iter)?;
    let protocol_config_account = next_account_info(accounts_iter)?;
    let market_risk_account = next_account_info(accounts_iter)?;

    // Verify admin signature
    if !admin_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load protocol config to verify admin
    let mut config = match ProtocolConfig::try_from_slice(&protocol_config_account.data.borrow()) {
        Ok(config) => config,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    if config.admin != *admin_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Get market risk metrics
    let market_risk = crate::utils::oracle::get_market_risk_metrics(market_risk_account)?;
    
    // Evaluate market conditions and adjust risk parameters accordingly
    if market_risk.volatility_index > config.high_volatility_threshold {
        // Increase required collateralization in high volatility
        config.required_collateral_ratio = config.required_collateral_ratio
            .saturating_add(config.volatility_adjustment_step);
            
        // Reduce maximum LTV
        config.max_ltv_bps = config.max_ltv_bps
            .saturating_sub(config.volatility_adjustment_step);
            
        // Increase liquidation threshold (requiring earlier liquidations)
        config.liquidation_threshold_bps = config.liquidation_threshold_bps
            .saturating_sub(config.volatility_adjustment_step);
            
        msg!("De-risking protocol due to high volatility: increased collateral ratio, reduced max LTV");
    } else if market_risk.volatility_index < config.low_volatility_threshold {
        // In low volatility environments, gradually return to normal parameters
        if config.required_collateral_ratio > config.base_required_collateral_ratio {
            config.required_collateral_ratio = config.required_collateral_ratio
                .saturating_sub(config.volatility_adjustment_step);
        }
        
        if config.max_ltv_bps < config.base_max_ltv_bps {
            config.max_ltv_bps = config.max_ltv_bps
                .saturating_add(config.volatility_adjustment_step);
        }
        
        if config.liquidation_threshold_bps < config.base_liquidation_threshold_bps {
            config.liquidation_threshold_bps = config.liquidation_threshold_bps
                .saturating_add(config.volatility_adjustment_step);
        }
        
        msg!("Normalizing risk parameters due to low market volatility");
    }
    
    // Save updated config
    borsh::to_writer(&mut protocol_config_account.data.borrow_mut()[..], &config)?;
    
    msg!("Protocol risk parameters updated based on market conditions");
    Ok(())
}

/// Validate a new borrow position meets minimum health requirements
pub fn validate_position_health(
    collateral_value: u64,
    borrow_value: u64,
    config: &ProtocolConfig,
) -> Result<u64, ProgramError> {
    // Calculate initial health factor
    let health_factor = if borrow_value == 0 {
        // If no borrowing, position is maximally healthy
        u64::MAX
    } else {
        collateral_value
            .checked_mul(100)
            .ok_or(StakeLendError::MathOverflow)?
            .checked_div(borrow_value)
            .ok_or(StakeLendError::MathOverflow)?
    };
    
    // Check if position meets minimum health requirements
    if health_factor < MIN_INITIAL_HEALTH_FACTOR {
        return Err(StakeLendError::InsufficientCollateral.into());
    }
    
    Ok(health_factor)
}

/// Process stress test on a specific pool
pub fn process_stress_test(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    shock_percentage: u16,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_account = next_account_info(accounts_iter)?;
    let protocol_config_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;

    // Verify admin signature
    if !admin_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load protocol config to verify admin
    let config = match ProtocolConfig::try_from_slice(&protocol_config_account.data.borrow()) {
        Ok(config) => config,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    if config.admin != *admin_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Load pool data
    let pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Get latest asset prices
    let mut prices = crate::utils::oracle::get_asset_prices(oracle_account)?;
    
    // Apply price shock for stress test
    for price in prices.values_mut() {
        // Apply negative price shock
        *price = price
            .checked_mul((10000 - shock_percentage) as u64)
            .ok_or(StakeLendError::MathOverflow)?
            .checked_div(10000)
            .ok_or(StakeLendError::MathOverflow)?;
    }
    
    // Evaluate pool health under stress conditions
    match &pool.data {
        PoolData::Lending(lending_data) => {
            // Calculate total collateral value and total borrows under stress
            let stressed_collateral_value = lending_data.calculate_stressed_collateral_value(&prices)?;
            let total_borrows_value = lending_data.calculate_total_borrows_value(&prices)?;
            
            // Calculate protocol safety margin
            let safety_margin = if total_borrows_value == 0 {
                // No borrows means infinite safety margin
                u64::MAX
            } else {
                stressed_collateral_value
                    .checked_mul(100)
                    .ok_or(StakeLendError::MathOverflow)?
                    .checked_div(total_borrows_value)
                    .ok_or(StakeLendError::MathOverflow)?
            };
            
            msg!(
                "Stress test results ({}% price shock): safety_margin={}%, stressed_collateral={}, total_borrows={}",
                shock_percentage,
                safety_margin,
                stressed_collateral_value,
                total_borrows_value
            );
            
            // Log risk assessment
            if safety_margin < 100 {
                msg!("CRITICAL RISK: Pool is undercollateralized under stress test");
            } else if safety_margin < 110 {
                msg!("HIGH RISK: Pool has low safety margin under stress test");
            } else if safety_margin < 120 {
                msg!("MODERATE RISK: Pool has acceptable but concerning safety margin");
            } else {
                msg!("LOW RISK: Pool has good safety margin even under stress");
            }
        },
        _ => {
            // Non-lending pools don't have borrowing risk in the same way
            msg!("Stress test not applicable for non-lending pools");
        }
    }
    
    Ok(())
}