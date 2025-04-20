// smart-contracts/src/processor/pools/mod.rs
pub mod basic_pool;
pub mod lending_pool;
pub mod lock_pool;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

use crate::{
    error::StakeLendError,
    instructions::StakeLendInstruction,
    state::{Pool, PoolData},
};

/// Dispatch pool operations to the appropriate pool implementation
pub fn process_pool_operation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    // Extract the pool account to determine its type
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    
    // Load pool data
    let pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };
    
    // Dispatch based on pool type
    match pool.data {
        PoolData::Basic(_) => {
            basic_pool::process_basic_pool_operation(program_id, accounts, instruction)
        },
        PoolData::Lending(_) => {
            lending_pool::process_lending_pool_operation(program_id, accounts, instruction)
        },
        PoolData::Lock(_) => {
            lock_pool::process_lock_pool_operation(program_id, accounts, instruction)
        },
    }
}

/// Common utility to update pool state data
pub fn update_pool_state(pool: &mut Pool) -> ProgramResult {
    // Update timestamp
    pool.last_update_timestamp = crate::utils::math::get_current_timestamp();
    
    // Perform type-specific updates
    match &mut pool.data {
        PoolData::Basic(_) => {
            // Basic pool state doesn't need complex updates
        },
        PoolData::Lending(lending_data) => {
            // Update interest accumulator if some time has passed
            let time_elapsed = pool.last_update_timestamp.saturating_sub(lending_data.last_interest_update_timestamp);
            
            if time_elapsed > 0 && pool.total_deposits > 0 {
                // Calculate utilization rate
                lending_data.utilization_rate = crate::utils::math::calculate_utilization_rate(
                    lending_data.total_borrows,
                    pool.total_deposits,
                )?;
                
                // Update interest rates
                lending_data.current_borrow_rate = crate::utils::math::calculate_borrow_rate(
                    lending_data.utilization_rate,
                    &lending_data.interest_rate_params,
                )?;
                
                lending_data.current_supply_rate = crate::utils::math::calculate_supply_rate(
                    lending_data.current_borrow_rate,
                    lending_data.utilization_rate, 
                    500, // 5% reserve factor
                )?;
                
                // Accumulate interest
                lending_data.accumulated_interest_index = crate::utils::math::update_interest_index(
                    lending_data.accumulated_interest_index,
                    lending_data.current_borrow_rate,
                    time_elapsed,
                )?;
                
                lending_data.last_interest_update_timestamp = pool.last_update_timestamp;
            }
        },
        PoolData::Lock(_) => {
            // Lock pool state updates are typically handled during user operations
        },
    }
    
    Ok(())
}