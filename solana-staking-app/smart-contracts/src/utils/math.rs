use std::collections::HashMap;
use solana_program::{
    program_error::ProgramError,
    msg,
    clock::Clock,
    sysvar::Sysvar,
    pubkey::Pubkey,
};

use crate::{
    error::StakeLendError,
    state::{
        UserPosition,
        LendingPoolData,
        AssetInfo,
        InterestRateParams,
        LiquidationData,
    },
};

/// Basis points denominator (10000 = 100%)
pub const BPS_DENOMINATOR: u64 = 10000;

/// Scale factor for fixed-point math operations
pub const SCALE_FACTOR: u64 = 1_000_000; // 6 decimals for precision

/// Maximum interest rate in basis points (300% per year)
pub const MAX_INTEREST_RATE_BPS: u16 = 30000;

/// Get the current timestamp in seconds
pub fn get_current_timestamp() -> u64 {
    Clock::get()
        .map(|clock| clock.unix_timestamp as u64)
        .unwrap_or_default()
}

/// Calculate fee based on amount and basis points
pub fn calculate_fee(amount: u64, fee_bps: u16) -> Result<u64, ProgramError> {
    if fee_bps > BPS_DENOMINATOR as u16 {
        return Err(StakeLendError::InvalidParameter.into());
    }
    
    amount
        .checked_mul(fee_bps as u64)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow.into())
}

/// Calculate token amount based on share of total shares
pub fn calculate_token_amount_from_shares(
    shares: u64,
    total_shares: u64,
    total_tokens: u64,
) -> Result<u64, ProgramError> {
    if total_shares == 0 {
        return Ok(shares); // 1:1 when no existing shares
    }
    
    shares
        .checked_mul(total_tokens)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(total_shares)
        .ok_or(StakeLendError::MathOverflow.into())
}

/// Calculate shares based on token amount
pub fn calculate_shares_from_token_amount(
    token_amount: u64,
    total_shares: u64,
    total_tokens: u64,
) -> Result<u64, ProgramError> {
    if total_tokens == 0 {
        return Ok(token_amount); // 1:1 when no existing tokens
    }
    
    token_amount
        .checked_mul(total_shares)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(total_tokens)
        .ok_or(StakeLendError::MathOverflow.into())
}

/// Calculate utilization rate (borrowed / total deposits)
pub fn calculate_utilization_rate(
    total_borrows: u64,
    total_deposits: u64,
) -> Result<u64, ProgramError> {
    if total_deposits == 0 {
        return Ok(0);
    }
    
    total_borrows
        .checked_mul(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(total_deposits)
        .ok_or(StakeLendError::MathOverflow.into())
}

/// Calculate borrow interest rate based on utilization
pub fn calculate_borrow_rate(
    utilization_rate: u64,
    rate_params: &InterestRateParams,
) -> Result<u64, ProgramError> {
    let optimal_utilization = rate_params.optimal_utilization_rate as u64;
    
    if utilization_rate <= optimal_utilization {
        // Below optimal: interpolate between base and optimal rates
        let rate_range = rate_params.optimal_borrow_rate.saturating_sub(rate_params.base_borrow_rate);
        let util_ratio = if optimal_utilization == 0 {
            0
        } else {
            utilization_rate
                .checked_mul(BPS_DENOMINATOR)
                .ok_or(StakeLendError::MathOverflow)?
                .checked_div(optimal_utilization)
                .ok_or(StakeLendError::MathOverflow)?
        };
        
        let additional_rate = rate_range
            .checked_mul(util_ratio as u64)
            .ok_or(StakeLendError::MathOverflow)?
            .checked_div(BPS_DENOMINATOR)
            .ok_or(StakeLendError::MathOverflow)?;
        
        Ok(rate_params.base_borrow_rate.saturating_add(additional_rate as u16) as u64)
    } else {
        // Above optimal: interpolate between optimal and max rates
        let rate_range = rate_params.max_borrow_rate.saturating_sub(rate_params.optimal_borrow_rate);
        let excess_utilization = utilization_rate.saturating_sub(optimal_utilization);
        let max_excess = BPS_DENOMINATOR.saturating_sub(optimal_utilization);
        let util_ratio = if max_excess == 0 {
            BPS_DENOMINATOR // Avoid division by zero
        } else {
            excess_utilization
                .checked_mul(BPS_DENOMINATOR)
                .ok_or(StakeLendError::MathOverflow)?
                .checked_div(max_excess)
                .ok_or(StakeLendError::MathOverflow)?
        };
        
        let additional_rate = rate_range
            .checked_mul(util_ratio as u64)
            .ok_or(StakeLendError::MathOverflow)?
            .checked_div(BPS_DENOMINATOR)
            .ok_or(StakeLendError::MathOverflow)?;
        
        Ok(rate_params.optimal_borrow_rate.saturating_add(additional_rate as u16) as u64)
    }
}

/// Calculate supply interest rate based on borrow rate and utilization
pub fn calculate_supply_rate(
    borrow_rate: u64,
    utilization_rate: u64,
    reserve_factor_bps: u16,
) -> Result<u64, ProgramError> {
    if reserve_factor_bps > BPS_DENOMINATOR as u16 {
        return Err(StakeLendError::InvalidParameter.into());
    }
    
    // Supply rate = Borrow rate * Utilization rate * (1 - Reserve factor)
    let borrow_rate_contribution = borrow_rate
        .checked_mul(utilization_rate)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let reserve_factor = BPS_DENOMINATOR
        .saturating_sub(reserve_factor_bps as u64);
    
    borrow_rate_contribution
        .checked_mul(reserve_factor)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow.into())
}

/// Update interest index based on interest rate and time elapsed
pub fn update_interest_index(
    current_index: u64,
    interest_rate_bps: u64,
    time_elapsed_seconds: u64,
) -> Result<u64, ProgramError> {
    // Convert annual interest rate to per-second rate
    // rate_per_second = annual_rate_bps / (BPS_DENOMINATOR * SECONDS_PER_YEAR)
    const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;
    
    let interest_factor = interest_rate_bps
        .checked_mul(time_elapsed_seconds)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_mul(SCALE_FACTOR)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(SECONDS_PER_YEAR)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // New index = current_index * (1 + interest_factor)
    let new_index = current_index
        .checked_mul(SCALE_FACTOR.saturating_add(interest_factor))
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(SCALE_FACTOR)
        .ok_or(StakeLendError::MathOverflow)?;
    
    Ok(new_index)
}

/// Calculate health factor for a user position
pub fn calculate_health_factor(
    collateral_assets: &HashMap<Pubkey, AssetInfo>,
    debt_assets: &HashMap<Pubkey, AssetInfo>,
    prices: &HashMap<Pubkey, u64>,
) -> Result<u64, ProgramError> {
    let mut total_collateral_value: u64 = 0;
    let mut total_debt_value: u64 = 0;
    
    // Calculate total collateral value
    for (asset_mint, asset_info) in collateral_assets {
        let price = prices.get(asset_mint).ok_or(StakeLendError::PriceNotFound)?;
        
        // Apply collateral factor
        let collateral_value = asset_info.amount
            .checked_mul(*price)
            .ok_or(StakeLendError::MathOverflow)?
            .checked_mul(asset_info.collateral_factor as u64)
            .ok_or(StakeLendError::MathOverflow)?
            .checked_div(BPS_DENOMINATOR)
            .ok_or(StakeLendError::MathOverflow)?;
        
        total_collateral_value = total_collateral_value
            .checked_add(collateral_value)
            .ok_or(StakeLendError::MathOverflow)?;
    }
    
    // Calculate total debt value
    for (asset_mint, asset_info) in debt_assets {
        let price = prices.get(asset_mint).ok_or(StakeLendError::PriceNotFound)?;
        
        let debt_value = asset_info.amount
            .checked_mul(*price)
            .ok_or(StakeLendError::MathOverflow)?;
        
        total_debt_value = total_debt_value
            .checked_add(debt_value)
            .ok_or(StakeLendError::MathOverflow)?;
    }
    
    // Calculate health factor (collateral / debt * 100)
    if total_debt_value == 0 {
        return Ok(u64::MAX); // No debt means maximally healthy
    }
    
    total_collateral_value
        .checked_mul(100)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(total_debt_value)
        .ok_or(StakeLendError::MathOverflow.into())
}

/// Calculate liquidation amount and bonus for liquidator
pub fn calculate_liquidation_amount(
    user_position: &UserPosition,
    requested_repay_amount: u64,
    prices: &HashMap<Pubkey, u64>,
    liquidation_bonus_bps: u16,
) -> Result<LiquidationData, ProgramError> {
    // Find debt asset to repay
    let (debt_mint, debt_info) = user_position.debt_assets
        .iter()
        .next() // For simplicity, just take the first debt asset
        .ok_or(StakeLendError::NoDebtFound)?;
    
    // Find collateral asset to seize
    let (collateral_mint, collateral_info) = user_position.collateral_assets
        .iter()
        .next() // For simplicity, just take the first collateral asset
        .ok_or(StakeLendError::NoCollateralFound)?;
    
    // Get asset prices
    let debt_price = prices.get(debt_mint).ok_or(StakeLendError::PriceNotFound)?;
    let collateral_price = prices.get(collateral_mint).ok_or(StakeLendError::PriceNotFound)?;
    
    // Calculate max repay amount (typically up to 50% of the debt)
    const MAX_LIQUIDATION_RATIO_BPS: u64 = 5000; // 50%
    
    let max_repay_amount = debt_info.amount
        .checked_mul(MAX_LIQUIDATION_RATIO_BPS)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // Determine actual repay amount
    let repay_amount = std::cmp::min(requested_repay_amount, max_repay_amount);
    
    // Calculate repayment value
    let repay_value = repay_amount
        .checked_mul(*debt_price)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // Calculate collateral value including bonus
    let bonus_factor = BPS_DENOMINATOR
        .checked_add(liquidation_bonus_bps as u64)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let collateral_value_with_bonus = repay_value
        .checked_mul(bonus_factor)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // Calculate collateral amount to seize
    let collateral_amount = collateral_value_with_bonus
        .checked_div(*collateral_price)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // Make sure collateral amount doesn't exceed what's available
    let actual_collateral_amount = std::cmp::min(collateral_amount, collateral_info.amount);
    
    // Calculate actual bonus value
    let base_collateral_value = repay_value;
    let actual_collateral_value = actual_collateral_amount
        .checked_mul(*collateral_price)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let bonus_amount = actual_collateral_value
        .saturating_sub(base_collateral_value);
    
    Ok(LiquidationData {
        debt_mint: *debt_mint,
        collateral_mint: *collateral_mint,
        repay_amount,
        collateral_amount: actual_collateral_amount,
        bonus_amount,
        timestamp: get_current_timestamp(),
    })
}

/// Calculate bonus amount for liquidators
pub fn calculate_bonus_amount(
    repay_value: u64,
    bonus_bps: u16,
) -> Result<u64, ProgramError> {
    if bonus_bps > 3000 { // Max 30% bonus
        return Err(StakeLendError::InvalidParameter.into());
    }
    
    repay_value
        .checked_mul(bonus_bps as u64)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow.into())
}

/// Calculate accumulated interest for a loan
pub fn calculate_accumulated_interest(
    principal: u64,
    original_index: u64,
    current_index: u64,
) -> Result<u64, ProgramError> {
    // Calculate interest based on index ratio: principal * (current_index / original_index - 1)
    // Ensure we don't go below 0
    if current_index <= original_index {
        return Ok(0);
    }
    
    let new_total = principal
        .checked_mul(current_index)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(original_index)
        .ok_or(StakeLendError::MathOverflow)?;
    
    Ok(new_total.saturating_sub(principal))
}

/// Calculate APY from APR
pub fn calculate_apy_from_apr(
    apr_bps: u64,
    compounds_per_year: u64,
) -> Result<u64, ProgramError> {
    if compounds_per_year == 0 {
        return Err(StakeLendError::InvalidParameter.into());
    }
    
    // APY = (1 + APR/compounds_per_year)^compounds_per_year - 1
    // In bps: APY_bps = ((1 + APR_bps/compounds_per_year/10000)^compounds_per_year - 1) * 10000
    
    // Calculate (1 + APR_bps/compounds_per_year/10000)
    // Use scale factor for precision
    let apr_per_period = apr_bps
        .checked_div(compounds_per_year)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let mut factor = SCALE_FACTOR.checked_add(
        apr_per_period.checked_mul(SCALE_FACTOR).ok_or(StakeLendError::MathOverflow)?
    ).ok_or(StakeLendError::MathOverflow)?;
    
    // Compound factor^compounds_per_year
    let mut result = SCALE_FACTOR;
    let mut exp = compounds_per_year;
    
    // Exponentiation by squaring algorithm
    while exp > 0 {
        if exp & 1 == 1 {
            result = result
                .checked_mul(factor)
                .ok_or(StakeLendError::MathOverflow)?
                .checked_div(SCALE_FACTOR)
                .ok_or(StakeLendError::MathOverflow)?;
        }
        
        factor = factor
            .checked_mul(factor)
            .ok_or(StakeLendError::MathOverflow)?
            .checked_div(SCALE_FACTOR)
            .ok_or(StakeLendError::MathOverflow)?;
            
        exp >>= 1;
    }
    
    // Calculate APY in bps = (result / SCALE_FACTOR - 1) * BPS_DENOMINATOR
    let apy_bps = result
        .saturating_sub(SCALE_FACTOR)
        .checked_mul(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(SCALE_FACTOR)
        .ok_or(StakeLendError::MathOverflow)?;
    
    Ok(apy_bps)
}

/// Calculate time until lock expiry
pub fn calculate_time_until_expiry(
    unlock_timestamp: u64,
) -> Result<u64, ProgramError> {
    let current_time = get_current_timestamp();
    
    if current_time >= unlock_timestamp {
        return Ok(0);
    }
    
    Ok(unlock_timestamp - current_time)
}

/// Calculate early unlock penalty
pub fn calculate_early_unlock_penalty(
    amount: u64,
    elapsed_time: u64,
    total_lock_duration: u64,
    max_penalty_bps: u16,
) -> Result<u64, ProgramError> {
    if total_lock_duration == 0 {
        return Ok(0);
    }
    
    // Percentage of time already served
    let time_served_pct = std::cmp::min(
        elapsed_time
            .checked_mul(BPS_DENOMINATOR)
            .ok_or(StakeLendError::MathOverflow)?
            .checked_div(total_lock_duration)
            .ok_or(StakeLendError::MathOverflow)?,
        BPS_DENOMINATOR
    );
    
    // Linear decrease of penalty based on time served
    let penalty_bps = max_penalty_bps as u64
        .checked_mul(BPS_DENOMINATOR - time_served_pct)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakeLendError::MathOverflow)?;
    
    calculate_fee(amount, penalty_bps as u16)
}