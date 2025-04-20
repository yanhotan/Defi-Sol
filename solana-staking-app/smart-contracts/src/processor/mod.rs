// smart-contracts/src/processor/mod.rs
pub mod stake;
pub mod lend;
pub mod borrow;
pub mod risk;
pub mod pools;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    error::StakeLendError,
    instructions::StakeLendInstruction,
    state::{ProtocolConfig, Pool, pda},
};

// Continuing smart-contracts/src/processor/mod.rs

/// Initialize the protocol configuration
pub fn initialize_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_account = next_account_info(accounts_iter)?;
    let config_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Only the admin can initialize the protocol
    if !admin_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract parameters
    let StakeLendInstruction::InitializeConfig {
        protocol_fee_bps,
        treasury_wallet,
    } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Verify the config account
    let (config_address, bump_seed) = pda::find_protocol_config_address(program_id);
    if config_address != *config_account.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Create the config account if it doesn't exist
    if config_account.data_is_empty() {
        let rent = Rent::get()?;
        let space = std::mem::size_of::<ProtocolConfig>();
        let rent_lamports = rent.minimum_balance(space);
        
        invoke_signed(
            &system_instruction::create_account(
                admin_account.key,
                &config_address,
                rent_lamports,
                space as u64,
                program_id,
            ),
            &[admin_account.clone(), config_account.clone(), system_program.clone()],
            &[&[pda::PROTOCOL_CONFIG_SEED.as_bytes(), &[bump_seed]]],
        )?;
    }

    // Initialize or update the config
    let mut config_data = ProtocolConfig {
        is_initialized: true,
        admin: *admin_account.key,
        protocol_fee_bps,
        treasury_wallet,
        pause_flags: 0,
        upgrade_authority: *admin_account.key,
    };

    // Serialize the config data
    borsh::to_writer(&mut config_account.data.borrow_mut()[..], &config_data)?;

    msg!("Protocol config initialized");
    Ok(())
}

/// Initialize a new pool
pub fn initialize_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_account = next_account_info(accounts_iter)?;
    let config_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let token_mint_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Only the admin can initialize pools
    if !admin_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load the config and verify admin
    let config = match ProtocolConfig::try_from_slice(&config_account.data.borrow()) {
        Ok(config) => config,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    if config.admin != *admin_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Extract pool parameters
    let StakeLendInstruction::InitializePool {
        pool_type,
        name,
        min_deposit,
        max_deposit,
        deposit_fee_bps,
        withdrawal_fee_bps,
        interest_rate_params,
        lock_duration_seconds,
    } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Verify pool authority PDA
    let (authority_address, _) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Create the pool data based on type
    let pool_data = match pool_type {
        0 => crate::state::PoolData::Basic(crate::state::BasicPoolData {
            instant_unstake_fee_bps: 50, // Default 0.5% fee for instant unstake
        }),
        1 => crate::state::PoolData::Lending(crate::state::LendingPoolData {
            interest_rate_params,
            current_borrow_rate: 0,
            current_supply_rate: 0,
            total_borrows: 0,
            total_reserves: 0,
            utilization_rate: 0,
            accumulated_interest_index: 1_000_000_000_000, // Start with 1.0 (scaled)
            last_interest_update_timestamp: crate::utils::math::get_current_timestamp(),
            liquidation_threshold: 8500, // 85% default
            liquidation_bonus: 500,      // 5% default
            max_ltv: 7500,               // 75% default
        }),
        2 => crate::state::PoolData::Lock(crate::state::LockPoolData {
            lock_duration: lock_duration_seconds.unwrap_or(0),
            yield_boost_bps: 500, // 5% additional yield by default
            early_unlock_penalty_bps: 1000, // 10% penalty for early unlock
        }),
        _ => return Err(StakeLendError::InvalidInstruction.into()),
    };

    // Initialize the pool
    let mut name_bytes = [0u8; 32];
    let name_len = std::cmp::min(name.len(), 32);
    name_bytes[..name_len].copy_from_slice(&name.as_bytes()[..name_len]);

    let pool = Pool {
        is_initialized: true,
        pool_type,
        name: name_bytes,
        pool_authority: *pool_authority.key,
        token_mint: *token_mint_account.key,
        lst_reserve: Pubkey::default(), // Will be set later during pool setup
        min_deposit,
        max_deposit,
        total_deposits: 0,
        deposit_fee_bps,
        withdrawal_fee_bps,
        total_shares: 0,
        last_update_timestamp: crate::utils::math::get_current_timestamp(),
        data: pool_data,
    };

    // Serialize the pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!("Pool initialized with type: {}", pool_type);
    Ok(())
}

/// Update pool state, interest rates, etc.
pub fn process_update_pool_state(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let oracle_account_opt = accounts_iter.next();

    // Verify admin signature
    if !admin_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Update timestamp
    let current_time = crate::utils::math::get_current_timestamp();
    pool.last_update_timestamp = current_time;

    // Update pool based on type
    match &mut pool.data {
        crate::state::PoolData::Basic(_) => {
            // Basic pool doesn't need much updating
        },
        crate::state::PoolData::Lending(lending_data) => {
            // Update interest rates and accumulated interest
            let time_elapsed = current_time.saturating_sub(lending_data.last_interest_update_timestamp);
            
            // Only update if some time has passed
            if time_elapsed > 0 {
                // Calculate utilization rate
                if pool.total_deposits > 0 {
                    lending_data.utilization_rate = crate::utils::math::calculate_utilization_rate(
                        lending_data.total_borrows,
                        pool.total_deposits,
                    )?;
                } else {
                    lending_data.utilization_rate = 0;
                }
                
                // Update interest rates based on utilization
                lending_data.current_borrow_rate = crate::utils::math::calculate_borrow_rate(
                    lending_data.utilization_rate,
                    &lending_data.interest_rate_params,
                )?;
                
                // Update supply rate (borrow rate * utilization rate * (1 - reserve factor))
                lending_data.current_supply_rate = crate::utils::math::calculate_supply_rate(
                    lending_data.current_borrow_rate,
                    lending_data.utilization_rate,
                    500, // 5% reserve factor
                )?;
                
                // Update interest index
                lending_data.accumulated_interest_index = crate::utils::math::update_interest_index(
                    lending_data.accumulated_interest_index,
                    lending_data.current_borrow_rate,
                    time_elapsed,
                )?;
                
                lending_data.last_interest_update_timestamp = current_time;
            }
        },
        crate::state::PoolData::Lock(_) => {
            // Lock pool doesn't need frequent updates
        },
    }

    // Save updated pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!("Pool state updated");
    Ok(())
}

/// Process claim rewards instruction
pub fn process_claim_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_pool_token_account = next_account_info(accounts_iter)?;
    let user_reward_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_reward_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load pool data
    let pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Calculate rewards based on user's share
    // This is a simplified version - a real implementation would track unclaimed rewards
    // and calculate based on staking duration, pool performance, etc.
    
    // For this example, we're just transferring a dummy amount
    let reward_amount = 1000; // Would be calculated based on user's shares and time staked
    
    // Transfer rewards
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &pool_reward_account.key,
        &user_reward_account.key,
        &pool_authority.key,
        &[],
        reward_amount,
    )?;
    
    invoke_signed(
        &transfer_ix,
        &[
            pool_reward_account.clone(),
            user_reward_account.clone(),
            pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    msg!("Rewards claimed: {}", reward_amount);
    Ok(())
}

/// Process withdraw from pool instruction
pub fn process_withdraw_from_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_pool_token_account = next_account_info(accounts_iter)?;
    let user_lst_token_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_lst_reserve = next_account_info(accounts_iter)?;
    let pool_token_mint = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract withdraw amount
    let StakeLendInstruction::WithdrawFromPool { amount } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Calculate LST token amount to withdraw based on pool shares
    let lst_amount = crate::utils::math::calculate_token_amount_from_shares(
        amount,
        pool.total_shares,
        pool.total_deposits,
    )?;

    // Apply withdrawal fee
    let fee_amount = crate::utils::math::calculate_fee(lst_amount, pool.withdrawal_fee_bps)?;
    let withdrawal_amount = lst_amount.saturating_sub(fee_amount);

    // Special handling for different pool types
    match &mut pool.data {
        crate::state::PoolData::Basic(_) => {
            // Basic pool - straightforward withdrawal
        },
        crate::state::PoolData::Lending(lending_data) => {
            // Check if there's enough liquidity (not all tokens may be lent out)
            let available_liquidity = pool.total_deposits.saturating_sub(lending_data.total_borrows);
            if withdrawal_amount > available_liquidity {
                return Err(StakeLendError::InsufficientLiquidity.into());
            }
        },
        crate::state::PoolData::Lock(lock_data) => {
            // For lock pools, we need to check if the lock period has ended
            // This is a simplified check - a real implementation would track individual positions
            let current_time = crate::utils::math::get_current_timestamp();
            
            // This is a placeholder - real implementation would check user's specific lock position
            // return Err(StakeLendError::LockPeriodNotEnded.into());
        },
    }

    // Burn pool tokens
    let burn_ix = spl_token::instruction::burn(
        &token_program.key,
        &user_pool_token_account.key,
        &pool_token_mint.key,
        &user_account.key,
        &[],
        amount,
    )?;
    
    invoke(
        &burn_ix,
        &[
            user_pool_token_account.clone(),
            pool_token_mint.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Transfer LST tokens to user
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &pool_lst_reserve.key,
        &user_lst_token_account.key,
        &pool_authority.key,
        &[],
        withdrawal_amount,
    )?;
    
    invoke_signed(
        &transfer_ix,
        &[
            pool_lst_reserve.clone(),
            user_lst_token_account.clone(),
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
    pool.total_deposits = pool.total_deposits.saturating_sub(lst_amount);
    pool.total_shares = pool.total_shares.saturating_sub(amount);
    
    // Save updated pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!("Withdrawn {} LST tokens by burning {} pool tokens", withdrawal_amount, amount);
    Ok(())
}