// smart-contracts/src/processor/borrow.rs
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
    state::{Pool, Obligation, PoolData, LendingPoolData, pda},
};

/// Process deposit to pool instruction
pub fn process_deposit_to_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_lst_token_account = next_account_info(accounts_iter)?;
    let user_pool_token_account = next_account_info(accounts_iter)?;
    let pool_account = next_account_info(accounts_iter)?;
    let pool_lst_reserve = next_account_info(accounts_iter)?;
    let pool_token_mint = next_account_info(accounts_iter)?;
    let pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract deposit amount
    let StakeLendInstruction::DepositToPool { amount } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Load pool data
    let mut pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify deposit amount within pool limits
    if amount < pool.min_deposit || (pool.max_deposit > 0 && amount > pool.max_deposit) {
        return Err(StakeLendError::InvalidDepositAmount.into());
    }

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Calculate deposit fee
    let fee_amount = crate::utils::math::calculate_fee(amount, pool.deposit_fee_bps)?;
    let deposit_amount_after_fee = amount.saturating_sub(fee_amount);

    // Calculate pool tokens to mint based on current exchange rate
    let pool_tokens_to_mint = if pool.total_shares == 0 || pool.total_deposits == 0 {
        // Initial deposit, 1:1 ratio
        deposit_amount_after_fee
    } else {
        // Calculate based on current ratio
        crate::utils::math::calculate_pool_tokens_for_deposit(
            deposit_amount_after_fee,
            pool.total_deposits,
            pool.total_shares,
        )?
    };

    // Transfer LST tokens from user to pool
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &user_lst_token_account.key,
        &pool_lst_reserve.key,
        &user_account.key,
        &[],
        amount,
    )?;
    
    invoke(
        &transfer_ix,
        &[
            user_lst_token_account.clone(),
            pool_lst_reserve.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Mint pool tokens to user
    let mint_to_ix = spl_token::instruction::mint_to(
        &token_program.key,
        &pool_token_mint.key,
        &user_pool_token_account.key,
        &pool_authority.key,
        &[],
        pool_tokens_to_mint,
    )?;
    
    invoke_signed(
        &mint_to_ix,
        &[
            pool_token_mint.clone(),
            user_pool_token_account.clone(),
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
    pool.total_deposits = pool.total_deposits.saturating_add(deposit_amount_after_fee);
    pool.total_shares = pool.total_shares.saturating_add(pool_tokens_to_mint);
    
    // Special handling for different pool types
    match &mut pool.data {
        PoolData::Basic(_) => {
            // Basic pool doesn't need additional updates
        },
        PoolData::Lending(lending_data) => {
            // Update utilization rate if there are borrows
            if lending_data.total_borrows > 0 {
                lending_data.utilization_rate = crate::utils::math::calculate_utilization_rate(
                    lending_data.total_borrows,
                    pool.total_deposits,
                )?;
                
                // Update interest rates based on new utilization
                lending_data.current_borrow_rate = crate::utils::math::calculate_borrow_rate(
                    lending_data.utilization_rate,
                    &lending_data.interest_rate_params,
                )?;
                
                lending_data.current_supply_rate = crate::utils::math::calculate_supply_rate(
                    lending_data.current_borrow_rate,
                    lending_data.utilization_rate,
                    500, // 5% reserve factor
                )?;
            }
        },
        PoolData::Lock(_) => {
            // Lock pool doesn't need additional updates on deposit
        },
    }
    
    // Save updated pool data
    borsh::to_writer(&mut pool_account.data.borrow_mut()[..], &pool)?;

    msg!("Deposited {} LST tokens for {} pool tokens", amount, pool_tokens_to_mint);
    Ok(())
}

/// Process create obligation instruction
pub fn process_create_obligation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let obligation_account = next_account_info(accounts_iter)?;
    let lending_pool_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract obligation parameters
    let StakeLendInstruction::CreateObligation { collateral_mint } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Load lending pool
    let pool = match Pool::try_from_slice(&lending_pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    if let PoolData::Lending(lending_data) = pool.data {
        // Pool is a lending pool, continue
    } else {
        return Err(StakeLendError::InvalidPoolType.into());
    }

    // Create the obligation account
    let rent = solana_program::rent::Rent::get()?;
    let space = std::mem::size_of::<Obligation>();
    let rent_lamports = rent.minimum_balance(space);
    
    invoke(
        &solana_program::system_instruction::create_account(
            user_account.key,
            obligation_account.key,
            rent_lamports,
            space as u64,
            program_id,
        ),
        &[user_account.clone(), obligation_account.clone(), system_program.clone()],
    )?;

    // Initialize the obligation
    let obligation = Obligation {
        is_initialized: true,
        owner: *user_account.key,
        lending_pool: *lending_pool_account.key,
        collateral_amount: 0,
        collateral_mint: collateral_mint,
        borrowed_amount: 0,
        borrowed_amount_with_interest: 0,
        cumulative_borrow_rate_snapshot: 1_000_000_000_000, // Start with 1.0 (scaled)
        loan_origination_timestamp: crate::utils::math::get_current_timestamp(),
        last_update_timestamp: crate::utils::math::get_current_timestamp(),
    };

    // Serialize the obligation data
    borsh::to_writer(&mut obligation_account.data.borrow_mut()[..], &obligation)?;

    msg!("Obligation account created");
    Ok(())
}

/// Process modify collateral instruction
pub fn process_modify_collateral(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_collateral_account = next_account_info(accounts_iter)?;
    let user_obligation_account = next_account_info(accounts_iter)?;
    let lending_pool_account = next_account_info(accounts_iter)?;
    let collateral_reserve = next_account_info(accounts_iter)?;
    let lending_pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract collateral parameters
    let StakeLendInstruction::ModifyCollateral { amount, add } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Load obligation
    let mut obligation = match Obligation::try_from_slice(&user_obligation_account.data.borrow()) {
        Ok(obligation) => obligation,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify obligation owner
    if obligation.owner != *user_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Load lending pool
    let pool = match Pool::try_from_slice(&lending_pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    let lending_data = match pool.data {
        PoolData::Lending(data) => data,
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, lending_pool_account.key);
    if authority_address != *lending_pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Update the obligation with accrued interest
    obligation.borrowed_amount_with_interest = crate::utils::math::calculate_accrued_interest(
        obligation.borrowed_amount_with_interest,
        obligation.cumulative_borrow_rate_snapshot,
        lending_data.accumulated_interest_index,
    )?;
    
    obligation.cumulative_borrow_rate_snapshot = lending_data.accumulated_interest_index;
    obligation.last_update_timestamp = crate::utils::math::get_current_timestamp();

    if add {
        // Add collateral

        // Transfer collateral from user to protocol
        let transfer_ix = spl_token::instruction::transfer(
            &token_program.key,
            &user_collateral_account.key,
            &collateral_reserve.key,
            &user_account.key,
            &[],
            amount,
        )?;
        
        invoke(
            &transfer_ix,
            &[
                user_collateral_account.clone(),
                collateral_reserve.clone(),
                user_account.clone(),
                token_program.clone(),
            ],
        )?;

        // Update obligation collateral
        obligation.collateral_amount = obligation.collateral_amount.saturating_add(amount);
    } else {
        // Remove collateral
        
        // Check if removing would put the loan below the required collateral ratio
        if obligation.borrowed_amount_with_interest > 0 {
            // Get collateral price (simplified)
            let collateral_price = 1_000_000; // Placeholder price with 6 decimals
            
            // Calculate the remaining collateral after withdrawal
            let remaining_collateral = obligation.collateral_amount.saturating_sub(amount);
            
            // Calculate required collateral based on LTV
            let required_collateral = crate::utils::math::calculate_required_collateral(
                obligation.borrowed_amount_with_interest,
                collateral_price,
                lending_data.max_ltv,
            )?;
            
            // Ensure remaining collateral is sufficient
            if remaining_collateral < required_collateral {
                return Err(StakeLendError::InsufficientCollateral.into());
            }
        }

        // Transfer collateral back to user
        let transfer_ix = spl_token::instruction::transfer(
            &token_program.key,
            &collateral_reserve.key,
            &user_collateral_account.key,
            &lending_pool_authority.key,
            &[],
            amount,
        )?;
        
        invoke_signed(
            &transfer_ix,
            &[
                collateral_reserve.clone(),
                user_collateral_account.clone(),
                lending_pool_authority.clone(),
                token_program.clone(),
            ],
            &[&[
                pda::POOL_AUTHORITY_SEED.as_bytes(),
                lending_pool_account.key.as_ref(),
                &[authority_bump],
            ]],
        )?;

        // Update obligation collateral
        obligation.collateral_amount = obligation.collateral_amount.saturating_sub(amount);
    }

    // Save updated obligation data
    borsh::to_writer(&mut user_obligation_account.data.borrow_mut()[..], &obligation)?;

    msg!(
        "{} {} collateral, new amount: {}",
        if add { "Added" } else { "Removed" },
        amount,
        obligation.collateral_amount
    );
    Ok(())
}

/// Process flash loan instruction
pub fn process_flash_loan(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let lending_pool_account = next_account_info(accounts_iter)?;
    let lending_pool_reserve = next_account_info(accounts_iter)?;
    let lending_pool_authority = next_account_info(accounts_iter)?;
    let flash_loan_fee_receiver = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    
    // The remaining accounts are for the callback instruction

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract flash loan parameters
    let StakeLendInstruction::FlashLoan { amount, callback_instruction } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Load lending pool
    let mut pool = match Pool::try_from_slice(&lending_pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify pool is a lending pool
    let lending_data = match &mut pool.data {
        PoolData::Lending(data) => data,
        _ => return Err(StakeLendError::InvalidPoolType.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(program_id, lending_pool_account.key);
    if authority_address != *lending_pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Check available liquidity
    let available_liquidity = pool.total_deposits.saturating_sub(lending_data.total_borrows);
    if amount > available_liquidity {
        return Err(StakeLendError::InsufficientLiquidity.into());
    }

    // Calculate flash loan fee
    let flash_loan_fee_bps = 10; // 0.1% fee
    let flash_loan_fee = crate::utils::math::calculate_fee(amount, flash_loan_fee_bps)?;
    let repay_amount = amount.saturating_add(flash_loan_fee);

    // Record initial token balances to verify repayment
    let initial_reserve_balance = crate::utils::validation::get_token_balance(lending_pool_reserve)?;

    // Transfer loan amount to user
    let transfer_loan_ix = spl_token::instruction::transfer(
        &token_program.key,
        &lending_pool_reserve.key,
        &user_token_account.key,
        &lending_pool_authority.key,
        &[],
        amount,
    )?;
    
    invoke_signed(
        &transfer_loan_ix,
        &[
            lending_pool_reserve.clone(),
            user_token_account.clone(),
            lending_pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            lending_pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    // Execute callback instruction (user-defined transaction)
    // This would execute whatever transaction the user wants to perform with the borrowed funds
    // In a real implementation, this would be more complex with proper security checks
    
    // Check that loan was repaid with fee
    let final_reserve_balance = crate::utils::validation::get_token_balance(lending_pool_reserve)?;
    if final_reserve_balance < initial_reserve_balance.saturating_add(flash_loan_fee) {
        return Err(StakeLendError::FlashLoanNotRepaid.into());
    }

    // Transfer fee to fee receiver
    let transfer_fee_ix = spl_token::instruction::transfer(
        &token_program.key,
        &lending_pool_reserve.key,
        &flash_loan_fee_receiver.key,
        &lending_pool_authority.key,
        &[],
        flash_loan_fee,
    )?;
    
    invoke_signed(
        &transfer_fee_ix,
        &[
            lending_pool_reserve.clone(),
            flash_loan_fee_receiver.clone(),
            lending_pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            lending_pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    msg!("Flash loan of {} tokens processed successfully with {} fee", amount, flash_loan_fee);
    Ok(())
}