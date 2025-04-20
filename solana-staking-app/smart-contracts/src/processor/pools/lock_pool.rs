    /// Process operations specific to the lock pool
    /// Handles user deposits with fixed-term lockup and high-yield rewards

    /// Process creating a new lock position
    /// - Validates user deposit amount and lock duration
    /// - Locks SOL for the specified term (e.g., 30, 90, 180 days)
    /// - Issues mSOL tokens with yield boost
    /// - Records user position with lockup details

    /// Process unlocking a position
    /// - Validates if lockup period has ended
    /// - Transfers SOL and accrued yield to user
    /// - Updates pool state and burns mSOL tokens
    /// - Applies penalties for early withdrawal (if applicable)

    /// Process calculating yield for locked positions
    /// - Calculates yield based on lock duration and pool performance
    /// - Uses math.rs for yield boost calculations
    /// - Updates user position with accrued yield
    /// - Integrates oracle.rs for market data

    // smart-contracts/src/processor/pools/lock_pool.rs
    
/// Process operations specific to the lock pool
/// Handles user deposits with fixed-term lockup and high-yield rewards
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
    state::{Pool, PoolData, LockPoolData, LockPosition, pda},
    utils::math,
};

/// Process operations specific to lock pool
pub fn process_lock_pool_operation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    match instruction {
        StakeLendInstruction::CreateLockPosition { amount, duration } => {
            process_create_lock_position(program_id, accounts, amount, duration)
        },
        StakeLendInstruction::ExtendLockDuration { position_id, new_duration } => {
            process_extend_lock_duration(program_id, accounts, position_id, new_duration)
        },
        StakeLendInstruction::ClaimLockRewards { position_id } => {
            process_claim_lock_rewards(program_id, accounts, position_id)
        },
        StakeLendInstruction::WithdrawMaturedLock { position_id } => {
            process_withdraw_matured_lock(program_id, accounts, position_id)
        },
        StakeLendInstruction::UpdateLockYieldBoost { duration, boost_bps } => {
            process_update_lock_yield_boost(program_id, accounts, duration, boost_bps)
        },
        StakeLendInstruction::DepositToPool { .. } |
        StakeLendInstruction::WithdrawFromPool { .. } => {
            // These operations are handled by the main processor
            // But we can add lock-pool specific logic here if needed
            Err(StakeLendError::UnsupportedInstruction.into())
        },
        _ => {
            // Other operations are not supported for lock pools
            Err(StakeLendError::UnsupportedInstruction.into())
        }
    }
}

/// Process creating a new lock position
pub fn process_create_lock_position(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    duration: u64,
) -> ProgramResult {
    // Account validation and extraction
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let lock_position_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_reserve_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lock pool
    let mut lock_data = match &pool.data {
        PoolData::Lock(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Verify lock duration is within allowed range
    if duration < lock_data.min_lock_duration || duration > lock_data.max_lock_duration {
        return Err(StakeLendError::InvalidLockDuration.into());
    }

    // Verify min deposit amount
    if amount < lock_data.min_lock_amount {
        return Err(StakeLendError::InsufficientAmount.into());
    }

    // Calculate yield boost based on duration
    let boost_bps = calculate_boost_multiplier(&lock_data, duration)?;

    // Transfer tokens from user to pool reserve
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &user_token_account.key,
        &pool_reserve_account.key,
        &user_account.key,
        &[],
        amount,
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

    // Calculate unlock timestamp
    let current_timestamp = math::get_current_timestamp();
    let unlock_timestamp = current_timestamp.checked_add(duration)
        .ok_or(StakeLendError::MathOverflow)?;

    // Initialize lock position state
    let lock_position = LockPosition {
        owner: *user_account.key,
        pool: *pool_account.key,
        position_id: lock_data.next_position_id,
        amount,
        start_timestamp: current_timestamp,
        unlock_timestamp,
        duration,
        boost_bps,
        claimed_rewards: 0,
        last_claim_timestamp: current_timestamp,
        is_active: true,
    };

    // Save lock position data
    borsh::to_writer(&mut lock_position_account.data.borrow_mut()[..], &lock_position)?;

    // Update pool state
    lock_data.total_locked = lock_data.total_locked.checked_add(amount)
        .ok_or(StakeLendError::MathOverflow)?;
    
    lock_data.positions_count = lock_data.positions_count.checked_add(1)
        .ok_or(StakeLendError::MathOverflow)?;
    
    lock_data.next_position_id = lock_data.next_position_id.checked_add(1)
        .ok_or(StakeLendError::MathOverflow)?;

    // Update pool data
    pool.total_deposits = pool.total_deposits.checked_add(amount)
        .ok_or(StakeLendError::MathOverflow)?;
    pool.data = PoolData::Lock(lock_data);
    
    // Update additional pool state
    super::update_pool_state(&mut pool)?;

    // Save updated pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!(
        "Lock position created: id={}, amount={}, duration={}, boost_bps={}, unlock_at={}",
        lock_position.position_id,
        amount,
        duration,
        boost_bps,
        unlock_timestamp
    );

    Ok(())
}

/// Process extending the duration of an existing lock
pub fn process_extend_lock_duration(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    position_id: u64,
    new_duration: u64,
) -> ProgramResult {
    // Account validation and extraction
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let lock_position_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load lock position data
    let mut lock_position = match LockPosition::try_from_slice(&lock_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify position ID
    if lock_position.position_id != position_id {
        return Err(StakeLendError::InvalidPositionId.into());
    }

    // Verify position ownership
    if lock_position.owner != *user_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Verify position is active
    if !lock_position.is_active {
        return Err(StakeLendError::PositionNotActive.into());
    }

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lock pool
    let mut lock_data = match &pool.data {
        PoolData::Lock(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify new duration is within allowed range
    if new_duration < lock_data.min_lock_duration || new_duration > lock_data.max_lock_duration {
        return Err(StakeLendError::InvalidLockDuration.into());
    }

    // Verify new duration is longer than remaining duration
    let current_timestamp = math::get_current_timestamp();
    let remaining_duration = if lock_position.unlock_timestamp > current_timestamp {
        lock_position.unlock_timestamp - current_timestamp
    } else {
        0
    };

    if new_duration <= remaining_duration {
        return Err(StakeLendError::InvalidLockDuration.into());
    }

    // Calculate new unlock timestamp
    let extension_amount = new_duration - remaining_duration;
    let new_unlock_timestamp = lock_position.unlock_timestamp.checked_add(extension_amount)
        .ok_or(StakeLendError::MathOverflow)?;

    // Calculate new boost based on total duration
    let total_duration = lock_position.duration.checked_add(extension_amount)
        .ok_or(StakeLendError::MathOverflow)?;
    let new_boost_bps = calculate_boost_multiplier(&lock_data, total_duration)?;

    // Update lock position
    lock_position.unlock_timestamp = new_unlock_timestamp;
    lock_position.duration = total_duration;
    lock_position.boost_bps = new_boost_bps;

    // Save updated lock position
    borsh::to_writer(&mut lock_position_account.data.borrow_mut()[..], &lock_position)?;

    // Update pool state
    super::update_pool_state(&mut pool)?;

    // Save updated pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!(
        "Lock duration extended: id={}, new_duration={}, new_unlock_at={}, new_boost_bps={}",
        position_id,
        total_duration,
        new_unlock_timestamp,
        new_boost_bps
    );

    Ok(())
}

/// Process claiming accrued rewards for a lock position
pub fn process_claim_lock_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    position_id: u64,
) -> ProgramResult {
    // Account validation and extraction
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_reward_token_account = next_account_info(accounts_iter)?;
    let lock_position_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_reward_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load lock position data
    let mut lock_position = match LockPosition::try_from_slice(&lock_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify position ID
    if lock_position.position_id != position_id {
        return Err(StakeLendError::InvalidPositionId.into());
    }

    // Verify position ownership
    if lock_position.owner != *user_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Verify position is active
    if !lock_position.is_active {
        return Err(StakeLendError::PositionNotActive.into());
    }

    // Load pool data
    let pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lock pool
    let lock_data = match &pool.data {
        PoolData::Lock(data) => data,
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Calculate accrued rewards
    let current_timestamp = math::get_current_timestamp();
    let time_elapsed = current_timestamp.saturating_sub(lock_position.last_claim_timestamp);
    
    // Apply boost to base reward rate
    let base_rate = lock_data.base_reward_rate;
    let boosted_rate = base_rate
        .checked_mul(10000 + lock_position.boost_bps as u64)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(10000)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // Calculate rewards over elapsed time
    const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;
    let accrued_rewards = lock_position.amount
        .checked_mul(boosted_rate)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_mul(time_elapsed)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(SECONDS_PER_YEAR)
        .ok_or(StakeLendError::MathOverflow)?;

    // Early return if no rewards to claim
    if accrued_rewards == 0 {
        return Ok(());
    }

    // Transfer rewards to user
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &pool_reward_account.key,
        &user_reward_token_account.key,
        &pool_authority.key,
        &[],
        accrued_rewards,
    )?;
    
    invoke_signed(
        &transfer_ix,
        &[
            pool_reward_account.clone(),
            user_reward_token_account.clone(),
            pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    // Update lock position
    lock_position.claimed_rewards = lock_position.claimed_rewards.checked_add(accrued_rewards)
        .ok_or(StakeLendError::MathOverflow)?;
    lock_position.last_claim_timestamp = current_timestamp;

    // Save updated lock position
    borsh::to_writer(&mut lock_position_account.data.borrow_mut()[..], &lock_position)?;

    msg!(
        "Rewards claimed: id={}, amount={}, boosted_rate={}, total_claimed={}",
        position_id,
        accrued_rewards,
        boosted_rate,
        lock_position.claimed_rewards
    );

    Ok(())
}

/// Process withdrawing funds from a matured lock position
pub fn process_withdraw_matured_lock(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    position_id: u64,
) -> ProgramResult {
    // Account validation and extraction
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let user_reward_token_account = next_account_info(accounts_iter)?;
    let lock_position_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_reserve_account = next_account_info(accounts_iter)?;
    let pool_reward_account = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load lock position data
    let mut lock_position = match LockPosition::try_from_slice(&lock_position_account.data.borrow()) {
        Ok(position) => position,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify position ID
    if lock_position.position_id != position_id {
        return Err(StakeLendError::InvalidPositionId.into());
    }

    // Verify position ownership
    if lock_position.owner != *user_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Verify position is active
    if !lock_position.is_active {
        return Err(StakeLendError::PositionNotActive.into());
    }

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lock pool
    let mut lock_data = match &pool.data {
        PoolData::Lock(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Verify lock has matured
    let current_timestamp = math::get_current_timestamp();
    let is_matured = current_timestamp >= lock_position.unlock_timestamp;

    let mut withdraw_amount = lock_position.amount;
    
    // Apply early withdrawal penalty if not matured
    if !is_matured {
        let penalty_bps = lock_data.early_unlock_penalty_bps;
        let penalty_amount = math::calculate_fee(withdraw_amount, penalty_bps)?;
        withdraw_amount = withdraw_amount.saturating_sub(penalty_amount);
        
        msg!(
            "Early withdrawal penalty applied: {}% ({})",
            penalty_bps as f64 / 100.0,
            penalty_amount
        );
    }

    // Calculate any remaining rewards
    let time_elapsed = current_timestamp.saturating_sub(lock_position.last_claim_timestamp);
    
    // Apply boost to base reward rate
    let base_rate = lock_data.base_reward_rate;
    let boosted_rate = base_rate
        .checked_mul(10000 + lock_position.boost_bps as u64)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(10000)
        .ok_or(StakeLendError::MathOverflow)?;
    
    // Calculate rewards over elapsed time
    const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;
    let final_rewards = lock_position.amount
        .checked_mul(boosted_rate)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_mul(time_elapsed)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(SECONDS_PER_YEAR)
        .ok_or(StakeLendError::MathOverflow)?;

    // Transfer principal back to user
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &pool_reserve_account.key,
        &user_token_account.key,
        &pool_authority.key,
        &[],
        withdraw_amount,
    )?;
    
    invoke_signed(
        &transfer_ix,
        &[
            pool_reserve_account.clone(),
            user_token_account.clone(),
            pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    // Transfer final rewards if any
    if final_rewards > 0 {
        let reward_transfer_ix = spl_token::instruction::transfer(
            &token_program.key,
            &pool_reward_account.key,
            &user_reward_token_account.key,
            &pool_authority.key,
            &[],
            final_rewards,
        )?;
        
        invoke_signed(
            &reward_transfer_ix,
            &[
                pool_reward_account.clone(),
                user_reward_token_account.clone(),
                pool_authority.clone(),
                token_program.clone(),
            ],
            &[&[
                pda::POOL_AUTHORITY_SEED.as_bytes(),
                pool_account.key.as_ref(),
                &[authority_bump],
            ]],
        )?;
    }

    // Update lock position to inactive
    lock_position.is_active = false;
    lock_position.claimed_rewards = lock_position.claimed_rewards.checked_add(final_rewards)
        .ok_or(StakeLendError::MathOverflow)?;

    // Update pool state
    lock_data.total_locked = lock_data.total_locked.saturating_sub(lock_position.amount);
    pool.total_deposits = pool.total_deposits.saturating_sub(lock_position.amount);
    pool.data = PoolData::Lock(lock_data);
    
    super::update_pool_state(&mut pool)?;

    // Save updated position and pool data
    borsh::to_writer(&mut lock_position_account.data.borrow_mut()[..], &lock_position)?;
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!(
        "Lock position withdrawn: id={}, amount={}, final_rewards={}, early={}",
        position_id,
        withdraw_amount,
        final_rewards,
        !is_matured
    );

    Ok(())
}

/// Process updating the yield boost parameters for lock durations
pub fn process_update_lock_yield_boost(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    duration: u64,
    boost_bps: u16,
) -> ProgramResult {
    // Account validation and extraction
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

    // Verify pool is a lock pool
    let mut lock_data = match &pool.data {
        PoolData::Lock(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Validate parameters
    if boost_bps > 10000 { // Max 100% boost
        return Err(StakeLendError::InvalidParameter.into());
    }

    if duration < lock_data.min_lock_duration || duration > lock_data.max_lock_duration {
        return Err(StakeLendError::InvalidLockDuration.into());
    }

    // Update boost configuration
    if duration == lock_data.min_lock_duration {
        lock_data.min_duration_boost_bps = boost_bps;
    } else if duration == lock_data.max_lock_duration {
        lock_data.max_boost_bps = boost_bps;
    } else {
        // For intermediate durations, we'd typically have a mapping
        // but for simplicity in this example, we just update the max
        lock_data.max_boost_bps = boost_bps;
    }

    // Update pool data
    pool.data = PoolData::Lock(lock_data);
    
    // Update pool state
    super::update_pool_state(&mut pool)?;
    
    // Save updated pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!(
        "Lock yield boost updated: duration={}, boost_bps={}",
        duration,
        boost_bps
    );

    Ok(())
}

/// Calculate boost multiplier based on lock duration
fn calculate_boost_multiplier(lock_data: &LockPoolData, duration: u64) -> Result<u16, ProgramError> {
    // No boost for minimum duration
    if duration <= lock_data.min_lock_duration {
        return Ok(lock_data.min_duration_boost_bps);
    }
    
    // Full boost for maximum duration
    if duration >= lock_data.max_lock_duration {
        return Ok(lock_data.max_boost_bps);
    }
    
    // Linear interpolation for durations in between
    let duration_range = lock_data.max_lock_duration.saturating_sub(lock_data.min_lock_duration);
    let duration_above_min = duration.saturating_sub(lock_data.min_lock_duration);
    let boost_range = lock_data.max_boost_bps.saturating_sub(lock_data.min_duration_boost_bps);
    
    let boost_increment = boost_range as u64
        .checked_mul(duration_above_min)
        .ok_or(StakeLendError::MathOverflow)?
        .checked_div(duration_range)
        .ok_or(StakeLendError::MathOverflow)?;
    
    let boost = lock_data.min_duration_boost_bps.saturating_add(boost_increment as u16);
    
    Ok(boost)
}