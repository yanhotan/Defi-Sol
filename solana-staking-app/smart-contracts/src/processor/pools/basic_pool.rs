// smart-contracts/src/processor/pools/basic_pool.rs
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
    state::{Pool, PoolData, BasicPoolData, pda},
};

/// Process operation specific to basic staking pool
pub fn process_basic_pool_operation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    match instruction {
        StakeLendInstruction::InstantUnstake { amount } => {
            process_instant_unstake(program_id, accounts, amount)
        },
        StakeLendInstruction::UpdatePoolConfig { param_type, new_value } => {
            process_update_pool_config(program_id, accounts, param_type, new_value)
        },
        StakeLendInstruction::DepositToPool { .. } |
        StakeLendInstruction::WithdrawFromPool { .. } => {
            // These operations are handled by the main processor
            // But we can add any basic-pool specific logic here if needed
            Err(StakeLendError::UnsupportedInstruction.into())
        },
        _ => {
            // Other operations are not supported for basic pools
            Err(StakeLendError::UnsupportedInstruction.into())
        }
    }
}

/// Process instant unstake for basic pool
pub fn process_instant_unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
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

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a basic pool
    let basic_data = match &pool.data {
        PoolData::Basic(data) => data,
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Calculate LST token amount based on pool shares
    let lst_amount = crate::utils::math::calculate_token_amount_from_shares(
        amount,
        pool.total_shares,
        pool.total_deposits,
    )?;

    // Apply withdrawal fee and instant unstake fee
    let withdrawal_fee = crate::utils::math::calculate_fee(lst_amount, pool.withdrawal_fee_bps)?;
    let instant_unstake_fee = crate::utils::math::calculate_fee(
        lst_amount.saturating_sub(withdrawal_fee),
        basic_data.instant_unstake_fee_bps,
    )?;
    
    let total_fee = withdrawal_fee.saturating_add(instant_unstake_fee);
    let withdrawal_amount = lst_amount.saturating_sub(total_fee);

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

    msg!(
        "Instant unstake: {} pool tokens for {} LST tokens (after fees: {})",
        amount,
        lst_amount,
        withdrawal_amount
    );
    Ok(())
}

/// Update basic pool configuration parameters
pub fn process_update_pool_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    param_type: u8,
    new_value: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_account = next_account_info(accounts_iter)?;
    let config_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;

    // Verify admin signature
    if !admin_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load protocol config to verify admin
    let config = match crate::state::ProtocolConfig::try_from_slice(&config_account.data.borrow()) {
        Ok(config) => config,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    if config.admin != *admin_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a basic pool
    let mut basic_data = match &mut pool.data {
        PoolData::Basic(data) => data.clone(),
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Update the specified parameter
    match param_type {
        // Basic pool parameters
        0 => {
            // Instant unstake fee (in bps)
            if new_value > 10000 {
                return Err(StakeLendError::InvalidParameter.into());
            }
            basic_data.instant_unstake_fee_bps = new_value as u16;
            msg!("Updated instant unstake fee to {} bps", new_value);
        },
        1 => {
            // Liquidity target percentage (in bps)
            if new_value > 10000 {
                return Err(StakeLendError::InvalidParameter.into());
            }
            basic_data.liquidity_target_bps = new_value as u16;
            msg!("Updated liquidity target to {} bps", new_value);
        },
        2 => {
            // Max instant unstake amount
            basic_data.max_instant_unstake_amount = new_value;
            msg!("Updated max instant unstake amount to {}", new_value);
        },
        // General pool parameters
        10 => {
            // Withdrawal fee (in bps)
            if new_value > 10000 {
                return Err(StakeLendError::InvalidParameter.into());
            }
            pool.withdrawal_fee_bps = new_value as u16;
            msg!("Updated withdrawal fee to {} bps", new_value);
        },
        11 => {
            // Deposit fee (in bps)
            if new_value > 10000 {
                return Err(StakeLendError::InvalidParameter.into());
            }
            pool.deposit_fee_bps = new_value as u16;
            msg!("Updated deposit fee to {} bps", new_value);
        },
        _ => return Err(StakeLendError::InvalidParameter.into()),
    }

    // Update pool data with the modified basic data
    pool.data = PoolData::Basic(basic_data);
    
    // Update additional pool state information
    super::update_pool_state(&mut pool)?;
    
    // Save updated pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!("Pool configuration updated successfully");
    Ok(())
}