/// Process operations specific to the lending pool
/// Handles user deposits, borrowing, interest accrual, and repayments

/// Process creating a new lending position
/// - Validates user deposit amount
/// - Updates pool state with new deposit
/// - Issues mSOL tokens to user
/// - Records user position for interest tracking

/// Process borrowing from the lending pool
/// - Validates borrower eligibility and collateral
/// - Checks pool liquidity
/// - Transfers SOL to borrower
/// - Updates pool state and borrower debt

/// Process repaying a loan
/// - Validates repayment amount
/// - Updates borrower debt and pool state
/// - Applies interest to lenders’ positions
/// - Handles partial or full repayments

/// Process accruing interest for lenders
/// - Calculates interest based on pool utilization and time
/// - Updates user positions with accrued interest
/// - Uses math.rs for interest rate models
/// - Integrates oracle.rs for external price feeds


// smart-contracts/src/processor/pools/lending_pool.rs
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
    state::{Pool, PoolData, LendingPoolData, UserPosition, pda, HealthStatus},
    utils::math,
};

/// Process operations specific to lending pool
pub fn process_lending_pool_operation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    match instruction {
        StakeLendInstruction::BorrowFromPool { amount } => {
            process_borrow(program_id, accounts, amount)
        },
        StakeLendInstruction::RepayToPool { amount } => {
            process_repay(program_id, accounts, amount)
        },
        StakeLendInstruction::AddCollateral { asset_mint, amount } => {
            process_add_collateral(program_id, accounts, asset_mint, amount)
        },
        StakeLendInstruction::RemoveCollateral { asset_mint, amount } => {
            process_remove_collateral(program_id, accounts, asset_mint, amount)
        },
        StakeLendInstruction::UpdateInterestRates { param_type, new_value } => {
            process_update_interest_rates(program_id, accounts, param_type, new_value)
        },
        StakeLendInstruction::LiquidateBorrower { repay_amount } => {
            process_liquidate_borrower(program_id, accounts, repay_amount)
        },
        StakeLendInstruction::DepositToPool { .. } |
        StakeLendInstruction::WithdrawFromPool { .. } => {
            // These operations are handled by the main processor
            // But we can add lending-pool specific logic here if needed
            Err(StakeLendError::UnsupportedInstruction.into())
        },
        _ => {
            // Other operations are not supported for lending pools
            Err(StakeLendError::UnsupportedInstruction.into())
        }
    }
}

/// Process borrowing from the lending pool
pub fn process_borrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let borrower_account = next_account_info(accounts_iter)?;
    let borrower_position_account = next_account_info(accounts_iter)?;
    let borrower_token_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_reserve_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify borrower signature
    if !borrower_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load borrower position
    let mut borrower_position = match UserPosition::try_from_slice(&borrower_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify position ownership
    if borrower_position.owner != *borrower_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    let mut lending_data = match &pool.data {
        PoolData::Lending(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Check pool has sufficient liquidity
    if lending_data.available_liquidity < amount {
        return Err(StakeLendError::InsufficientPoolFunds.into());
    }

    // Get price data for health check
    let prices = crate::utils::oracle::get_asset_prices(oracle_account)?;

    // Add the new borrow to user position temporarily to simulate health factor
    let borrow_asset_mint = *pool_reserve_account.key;
    
    // Add or update debt asset in borrower position
    if let Some(existing_debt) = borrower_position.debt_assets.get_mut(&borrow_asset_mint) {
        existing_debt.amount = existing_debt.amount.checked_add(amount)
            .ok_or(StakeLendError::MathOverflow)?;
    } else {
        borrower_position.debt_assets.insert(
            borrow_asset_mint,
            crate::state::AssetInfo {
                amount,
                collateral_factor: 0, // Debt assets don't have collateral factor
                last_updated_slot: solana_program::clock::Clock::get()?.slot,
                interest_index: lending_data.accumulated_interest_index,
            }
        );
    }

    // Calculate simulated health factor
    let simulated_health_factor = math::calculate_health_factor(
        &borrower_position.collateral_assets,
        &borrower_position.debt_assets,
        &prices,
    )?;

    // Check health factor meets minimum requirement
    if simulated_health_factor < crate::processor::risk::MIN_INITIAL_HEALTH_FACTOR {
        return Err(StakeLendError::InsufficientCollateral.into());
    }

    // Update interest accumulation before modifying state
    let current_timestamp = math::get_current_timestamp();
    let time_elapsed = current_timestamp.saturating_sub(lending_data.last_interest_update_timestamp);
    
    if time_elapsed > 0 {
        // Update interest accumulators
        lending_data.accumulated_interest_index = math::update_interest_index(
            lending_data.accumulated_interest_index,
            lending_data.current_borrow_rate,
            time_elapsed,
        )?;
        lending_data.last_interest_update_timestamp = current_timestamp;
    }

    // Transfer tokens to borrower
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &pool_reserve_account.key,
        &borrower_token_account.key,
        &pool_authority.key,
        &[],
        amount,
    )?;
    
    invoke_signed(
        &transfer_ix,
        &[
            pool_reserve_account.clone(),
            borrower_token_account.clone(),
            pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    // Update pool state
    lending_data.available_liquidity = lending_data.available_liquidity
        .checked_sub(amount)
        .ok_or(StakeLendError::MathOverflow)?;
    
    lending_data.total_borrows = lending_data.total_borrows
        .checked_add(amount)
        .ok_or(StakeLendError::MathOverflow)?;
    
    lending_data.utilization_rate = math::calculate_utilization_rate(
        lending_data.total_borrows,
        pool.total_deposits,
    )?;

    // Update borrower position
    borrower_position.health_factor = simulated_health_factor;
    borrower_position.health_status = if simulated_health_factor >= crate::processor::risk::HEALTHY_FACTOR_THRESHOLD {
        HealthStatus::Healthy
    } else if simulated_health_factor >= crate::processor::risk::LIQUIDATION_THRESHOLD {
        HealthStatus::Warning
    } else {
        HealthStatus::Liquidatable
    };
    borrower_position.last_updated_timestamp = current_timestamp;

    // Save updated pool data
    pool.data = PoolData::Lending(lending_data);
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    // Save updated borrower position
    borsh::to_writer(&mut borrower_position_account.data.borrow_mut()[..], &borrower_position)?;

    msg!(
        "Borrow successful: amount={}, new_health_factor={}, utilization={}",
        amount,
        simulated_health_factor,
        lending_data.utilization_rate
    );
    
    Ok(())
}

/// Process repayment to the lending pool
pub fn process_repay(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let borrower_position_account = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_reserve_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load borrower position
    let mut borrower_position = match UserPosition::try_from_slice(&borrower_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    let mut lending_data = match &pool.data {
        PoolData::Lending(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Get the debt asset
    let debt_mint = *pool_reserve_account.key;
    let debt_asset = borrower_position.debt_assets.get_mut(&debt_mint)
        .ok_or(StakeLendError::NoDebtFound)?;

    // Update interest accumulation before modifying state
    let current_timestamp = math::get_current_timestamp();
    let time_elapsed = current_timestamp.saturating_sub(lending_data.last_interest_update_timestamp);
    
    if time_elapsed > 0 {
        // Update interest accumulators
        lending_data.accumulated_interest_index = math::update_interest_index(
            lending_data.accumulated_interest_index,
            lending_data.current_borrow_rate,
            time_elapsed,
        )?;
        lending_data.last_interest_update_timestamp = current_timestamp;
    }
    
    // Calculate accrued interest
    let interest_accrued = math::calculate_accumulated_interest(
        debt_asset.amount,
        debt_asset.interest_index,
        lending_data.accumulated_interest_index,
    )?;
    
    // Update debt with accrued interest
    let total_debt = debt_asset.amount
        .checked_add(interest_accrued)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // Determine actual repayment amount (can't repay more than owed)
    let actual_repay_amount = std::cmp::min(amount, total_debt);
    
    // Transfer tokens from user to pool reserve
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &user_token_account.key,
        &pool_reserve_account.key,
        &user_account.key,
        &[],
        actual_repay_amount,
    )?;
    
    invoke(
        &transfer_ix,
        &[
            user_token_account.clone(),
            pool_reserve_account.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Update pool state
    lending_data.available_liquidity = lending_data.available_liquidity
        .checked_add(actual_repay_amount)
        .ok_or(StakeLendError::MathOverflow)?;
    
    lending_data.total_borrows = lending_data.total_borrows
        .checked_sub(actual_repay_amount)
        .ok_or(StakeLendError::MathOverflow)?;
    
    lending_data.utilization_rate = math::calculate_utilization_rate(
        lending_data.total_borrows,
        pool.total_deposits,
    )?;

    // Update borrower position
    if actual_repay_amount >= total_debt {
        // Fully repaid, remove debt asset
        borrower_position.debt_assets.remove(&debt_mint);
    } else {
        // Partially repaid, update debt amount and interest index
        debt_asset.amount = total_debt - actual_repay_amount;
        debt_asset.interest_index = lending_data.accumulated_interest_index;
        debt_asset.last_updated_slot = solana_program::clock::Clock::get()?.slot;
    }

    // Recalculate health factor if there are still debts
    if !borrower_position.debt_assets.is_empty() {
        let prices = crate::utils::oracle::get_asset_prices(next_account_info(accounts_iter)?)?;
        
        let new_health_factor = math::calculate_health_factor(
            &borrower_position.collateral_assets,
            &borrower_position.debt_assets,
            &prices,
        )?;
        
        borrower_position.health_factor = new_health_factor;
        borrower_position.health_status = if new_health_factor >= crate::processor::risk::HEALTHY_FACTOR_THRESHOLD {
            HealthStatus::Healthy
        } else if new_health_factor >= crate::processor::risk::LIQUIDATION_THRESHOLD {
            HealthStatus::Warning
        } else {
            HealthStatus::Liquidatable
        };
    } else {
        borrower_position.health_factor = u64::MAX;
        borrower_position.health_status = HealthStatus::Healthy;
    }
    
    borrower_position.last_updated_timestamp = current_timestamp;

    // Save updated pool data
    pool.data = PoolData::Lending(lending_data);
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    // Save updated borrower position
    borsh::to_writer(&mut borrower_position_account.data.borrow_mut()[..], &borrower_position)?;

    msg!(
        "Repayment successful: amount={}, interest_paid={}, remaining_debt={}",
        actual_repay_amount,
        interest_accrued,
        if borrower_position.debt_assets.contains_key(&debt_mint) { 
            borrower_position.debt_assets[&debt_mint].amount 
        } else { 
            0 
        }
    );
    
    Ok(())
}

/// Process adding collateral to user position
pub fn process_add_collateral(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    collateral_mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_position_account = next_account_info(accounts_iter)?;
    let user_collateral_token_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_collateral_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify collateral token account
    crate::utils::validation::validate_token_accounts(
        user_account.key,
        &collateral_mint,
        user_collateral_token_account,
        false,
    )?;

    // Load user position
    let mut user_position = match UserPosition::try_from_slice(&user_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify position ownership
    if user_position.owner != *user_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Load pool data
    let pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    let lending_data = match &pool.data {
        PoolData::Lending(data) => data,
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Check if collateral is supported and get collateral factor
    let collateral_factor = lending_data.supported_collaterals
        .get(&collateral_mint)
        .ok_or(StakeLendError::UnsupportedCollateral)?
        .collateral_factor;

    // Transfer tokens from user to pool collateral account
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &user_collateral_token_account.key,
        &pool_collateral_account.key,
        &user_account.key,
        &[],
        amount,
    )?;
    
    invoke(
        &transfer_ix,
        &[
            user_collateral_token_account.clone(),
            pool_collateral_account.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Add or update collateral asset in user position
    if let Some(collateral_asset) = user_position.collateral_assets.get_mut(&collateral_mint) {
        collateral_asset.amount = collateral_asset.amount.checked_add(amount)
            .ok_or(StakeLendError::MathOverflow)?;
    } else {
        user_position.collateral_assets.insert(
            collateral_mint,
            crate::state::AssetInfo {
                amount,
                collateral_factor,
                last_updated_slot: solana_program::clock::Clock::get()?.slot,
                interest_index: 0, // Not used for collateral
            }
        );
    }

    // If user has debt, recalculate health factor
    if !user_position.debt_assets.is_empty() {
        let prices = crate::utils::oracle::get_asset_prices(oracle_account)?;
        
        let new_health_factor = math::calculate_health_factor(
            &user_position.collateral_assets,
            &user_position.debt_assets,
            &prices,
        )?;
        
        user_position.health_factor = new_health_factor;
        user_position.health_status = if new_health_factor >= crate::processor::risk::HEALTHY_FACTOR_THRESHOLD {
            HealthStatus::Healthy
        } else if new_health_factor >= crate::processor::risk::LIQUIDATION_THRESHOLD {
            HealthStatus::Warning
        } else {
            HealthStatus::Liquidatable
        };
    }
    
    user_position.last_updated_timestamp = math::get_current_timestamp();

    // Save updated user position
    borsh::to_writer(&mut user_position_account.data.borrow_mut()[..], &user_position)?;

    msg!(
        "Collateral added: asset={}, amount={}, health_factor={}",
        collateral_mint,
        amount,
        user_position.health_factor
    );
    
    Ok(())
}

/// Process removing collateral from user position
pub fn process_remove_collateral(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    collateral_mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_position_account = next_account_info(accounts_iter)?;
    let user_collateral_token_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_collateral_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load user position
    let mut user_position = match UserPosition::try_from_slice(&user_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify position ownership
    if user_position.owner != *user_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Verify collateral exists in user position
    let collateral_asset = user_position.collateral_assets.get_mut(&collateral_mint)
        .ok_or(StakeLendError::NoCollateralFound)?;

    // Check if user has sufficient collateral
    if collateral_asset.amount < amount {
        return Err(StakeLendError::InsufficientCollateral.into());
    }

    // Load pool data
    let pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    let _ = match &pool.data {
        PoolData::Lending(_) => (),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // If user has debt, simulate health factor after removal
    let simulated_health_factor = if !user_position.debt_assets.is_empty() {
        // Temporarily update collateral amount
        collateral_asset.amount = collateral_asset.amount.checked_sub(amount)
            .ok_or(StakeLendError::MathOverflow)?;
        
        // Get prices and calculate new health factor
        let prices = crate::utils::oracle::get_asset_prices(oracle_account)?;
        let health_factor = math::calculate_health_factor(
            &user_position.collateral_assets,
            &user_position.debt_assets,
            &prices,
        )?;
        
        // Restore original amount (will be updated after transfer)
        collateral_asset.amount = collateral_asset.amount.checked_add(amount)
            .ok_or(StakeLendError::MathOverflow)?;
        
        // Check if removal would make position unhealthy
        if health_factor < crate::processor::risk::MIN_INITIAL_HEALTH_FACTOR {
            return Err(StakeLendError::CollateralRemovalWouldBecomeUnsafe.into());
        }
        
        health_factor
    } else {
        u64::MAX // No debt, health factor is maximum
    };

    // Update collateral asset
    collateral_asset.amount = collateral_asset.amount.checked_sub(amount)
        .ok_or(StakeLendError::MathOverflow)?;

    // Transfer collateral from pool to user
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &pool_collateral_account.key,
        &user_collateral_token_account.key,
        &pool_authority.key,
        &[],
        amount,
    )?;
    
    invoke_signed(
        &transfer_ix,
        &[
            pool_collateral_account.clone(),
            user_collateral_token_account.clone(),
            pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    // If collateral asset is now 0, remove it from user position
    if collateral_asset.amount == 0 {
        user_position.collateral_assets.remove(&collateral_mint);
    }

    // Update health data
    user_position.health_factor = simulated_health_factor;
    user_position.health_status = if simulated_health_factor >= crate::processor::risk::HEALTHY_FACTOR_THRESHOLD {
        HealthStatus::Healthy
    } else if simulated_health_factor >= crate::processor::risk::LIQUIDATION_THRESHOLD {
        HealthStatus::Warning
    } else {
        HealthStatus::Liquidatable
    };

    // Save updated user position
    borsh::to_writer(&mut user_position_account.data.borrow_mut()[..], &user_position)?;

    msg!(
        "Collateral removed: amount={}, new_health_factor={}", 
        amount,
        simulated_health_factor
    );
    
    Ok(())
}

/// Process interest rate updates
pub fn process_update_interest_rates(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    param_type: u8,
    new_value: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_account = next_account_info(accounts_iter)?;
    let config_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;

    // Validate admin authority
    crate::utils::validation::validate_admin_authority(admin_account, config_account)?;

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    let mut lending_data = match &pool.data {
        PoolData::Lending(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Update appropriate interest rate parameter
    match param_type {
        0 => {
            // Base borrow rate (in bps)
            if new_value > math::MAX_INTEREST_RATE_BPS as u64 {
                return Err(StakeLendError::InvalidParameter.into());
            }
            lending_data.interest_rate_params.base_borrow_rate = new_value as u16;
            msg!("Updated base borrow rate to {} bps", new_value);
        },
        1 => {
            // Optimal borrow rate (in bps)
            if new_value > math::MAX_INTEREST_RATE_BPS as u64 {
                return Err(StakeLendError::InvalidParameter.into());
            }
            lending_data.interest_rate_params.optimal_borrow_rate = new_value as u16;
            msg!("Updated optimal borrow rate to {} bps", new_value);
        },
        2 => {
            // Max borrow rate (in bps)
            if new_value > math::MAX_INTEREST_RATE_BPS as u64 {
                return Err(StakeLendError::InvalidParameter.into());
            }
            lending_data.interest_rate_params.max_borrow_rate = new_value as u16;
            msg!("Updated max borrow rate to {} bps", new_value);
        },
        3 => {
            // Optimal utilization rate (in bps)
            if new_value > 10000 {
                return Err(StakeLendError::InvalidParameter.into());
            }
            lending_data.interest_rate_params.optimal_utilization_rate = new_value as u16;
            msg!("Updated optimal utilization rate to {} bps", new_value);
        },
        4 => {
            // Reserve factor (in bps)
            if new_value > 5000 { // Max 50% reserve factor
                return Err(StakeLendError::InvalidParameter.into());
            }
            lending_data.reserve_factor_bps = new_value as u16;
            msg!("Updated reserve factor to {} bps", new_value);
        },
        _ => return Err(StakeLendError::InvalidParameter.into()),
    }
    
    // Recalculate interest rates based on current utilization
    lending_data.current_borrow_rate = math::calculate_borrow_rate(
        lending_data.utilization_rate,
        &lending_data.interest_rate_params,
    )?;
    
    lending_data.current_supply_rate = math::calculate_supply_rate(
        lending_data.current_borrow_rate,
        lending_data.utilization_rate,
        lending_data.reserve_factor_bps,
    )?;

    // Update pool data
    pool.data = PoolData::Lending(lending_data);
    
    // Save updated pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!(
        "Interest rates updated: borrow_rate={} bps, supply_rate={} bps",
        lending_data.current_borrow_rate,
        lending_data.current_supply_rate
    );
    
    Ok(())
}

/// Process liquidation of unhealthy borrower positions
pub fn process_liquidate_borrower(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    repay_amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let liquidator_account = next_account_info(accounts_iter)?;
    let borrower_position_account = next_account_info(accounts_iter)?;
    let liquidator_repay_token_account = next_account_info(accounts_iter)?;
    let liquidator_receive_token_account = next_account_info(accounts_iter)?;
    let debt_mint_account = next_account_info(accounts_iter)?;
    let collateral_mint_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_debt_reserve = next_account_info(accounts_iter)?;
    let pool_collateral_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify liquidator signature
    if !liquidator_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load borrower position
    let mut borrower_position = match UserPosition::try_from_slice(&borrower_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    let mut lending_data = match &pool.data {
        PoolData::Lending(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Get debt and collateral mints
    let debt_mint = *debt_mint_account.key;
    let collateral_mint = *collateral_mint_account.key;

    // Verify borrower has the specified debt and collateral
    let debt_asset = borrower_position.debt_assets.get_mut(&debt_mint)
        .ok_or(StakeLendError::NoDebtFound)?;
    
    let collateral_asset = borrower_position.collateral_assets.get_mut(&collateral_mint)
        .ok_or(StakeLendError::NoCollateralFound)?;

    // Update interest accumulation
    let current_timestamp = math::get_current_timestamp();
    let time_elapsed = current_timestamp.saturating_sub(lending_data.last_interest_update_timestamp);
    
    if time_elapsed > 0 {
        // Update interest accumulators
        lending_data.accumulated_interest_index = math::update_interest_index(
            lending_data.accumulated_interest_index,
            lending_data.current_borrow_rate,
            time_elapsed,
        )?;
        lending_data.last_interest_update_timestamp = current_timestamp;
    }
    
    // Calculate accrued interest
    let interest_accrued = math::calculate_accumulated_interest(
        debt_asset.amount,
        debt_asset.interest_index,
        lending_data.accumulated_interest_index,
    )?;
    
    // Update debt with accrued interest
    let total_debt = debt_asset.amount
        .checked_add(interest_accrued)
        .ok_or(StakeLendError::MathOverflow)?;

    // Check borrower health to verify liquidation is allowed
    let prices = crate::utils::oracle::get_asset_prices(oracle_account)?;
    let health_factor = math::calculate_health_factor(
        &borrower_position.collateral_assets,
        &borrower_position.debt_assets,
        &prices,
    )?;
    
    if health_factor >= crate::processor::risk::LIQUIDATION_THRESHOLD {
        return Err(StakeLendError::PositionNotLiquidatable.into());
    }
    
    // Calculate liquidation amount (max 50% of debt)
    let max_liquidation_amount = total_debt
        .checked_mul(5000) // 50% in basis points
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(10000)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let liquidation_amount = std::cmp::min(repay_amount, max_liquidation_amount);
    
    // Get price data for collateral and debt
    let debt_price = *prices.get(&debt_mint).ok_or(StakeLendError::PriceNotFound)?;
    let collateral_price = *prices.get(&collateral_mint).ok_or(StakeLendError::PriceNotFound)?;
    
    // Calculate collateral to be liquidated (including bonus)
    let debt_value = liquidation_amount
        .checked_mul(debt_price)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let bonus_bps = lending_data.liquidation_bonus_bps;
    let bonus_factor = 10000 + bonus_bps as u64;
    
    let collateral_value_with_bonus = debt_value
        .checked_mul(bonus_factor)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(10000)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let collateral_amount = collateral_value_with_bonus
        .checked_div(collateral_price)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // Ensure we don't liquidate more collateral than available
    let collateral_amount = std::cmp::min(collateral_amount, collateral_asset.amount);

    // Transfer repayment from liquidator to pool reserve
    let repay_ix = spl_token::instruction::transfer(
        &token_program.key,
        &liquidator_repay_token_account.key,
        &pool_debt_reserve.key,
        &liquidator_account.key,
        &[],
        liquidation_amount,
    )?;
    
    invoke(
        &repay_ix,
        &[
            liquidator_repay_token_account.clone(),
            pool_debt_reserve.clone(),
            liquidator_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Transfer collateral from pool to liquidator
    let transfer_collateral_ix = spl_token::instruction::transfer(
        &token_program.key,
        &pool_collateral_account.key,
        &liquidator_receive_token_account.key,
        &pool_authority.key,
        &[],
        collateral_amount,
    )?;
    
    invoke_signed(
        &transfer_collateral_ix,
        &[
            pool_collateral_account.clone(),
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
    debt_asset.amount = total_debt - liquidation_amount;
    debt_asset.interest_index = lending_data.accumulated_interest_index;
    debt_asset.last_updated_slot = solana_program::clock::Clock::get()?.slot;
    
    collateral_asset.amount -= collateral_amount;
    
    // Remove assets if zero balance
    if debt_asset.amount == 0 {
        borrower_position.debt_assets.remove(&debt_mint);
    }
    
    if collateral_asset.amount == 0 {
        borrower_position.collateral_assets.remove(&collateral_mint);
    }
    
    // Recalculate health factor
    let new_health_factor = if borrower_position.debt_assets.is_empty() {
        u64::MAX // No debt means maximum health
    } else {
        math::calculate_health_factor(
            &borrower_position.collateral_assets,
            &borrower_position.debt_assets,
            &prices,
        )?
    };
    
    borrower_position.health_factor = new_health_factor;
    borrower_position.health_status = if new_health_factor >= crate::processor::risk::HEALTHY_FACTOR_THRESHOLD {
        HealthStatus::Healthy
    } else if new_health_factor >= crate::processor::risk::LIQUIDATION_THRESHOLD {
        HealthStatus::Warning
    } else {
        HealthStatus::Liquidatable
    };
    borrower_position.last_updated_timestamp = current_timestamp;

    // Update pool state
    lending_data.available_liquidity = lending_data.available_liquidity
        .checked_add(liquidation_amount)
        .ok_or(StakeLendError::MathOverflow)?;
    
    lending_data.total_borrows = lending_data.total_borrows
        .checked_sub(liquidation_amount)
        .ok_or(StakeLendError::MathOverflow)?;
    
    lending_data.utilization_rate = math::calculate_utilization_rate(
        lending_data.total_borrows,
        pool.total_deposits,
    )?;

    // Save updated pool data
    pool.data = PoolData::Lending(lending_data);
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    // Save updated borrower position
    borsh::to_writer(&mut borrower_position_account.data.borrow_mut()[..], &borrower_position)?;

    // Calculate bonus value
    let base_value = liquidation_amount
        .checked_mul(debt_price)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let collateral_value = collateral_amount
        .checked_mul(collateral_price)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let bonus_value = collateral_value.saturating_sub(base_value);

    msg!(
        "Liquidation successful: debt_repaid={}, collateral_liquidated={}, bonus_value={}, new_health_factor={}",
        liquidation_amount,
        collateral_amount,
        bonus_value,
        new_health_factor
    );
    
    Ok(())
}