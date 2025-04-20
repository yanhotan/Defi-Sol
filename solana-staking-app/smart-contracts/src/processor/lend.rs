// smart-contracts/src/processor/lend.rs
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

/// Process borrow instruction
pub fn process_borrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_collateral_account = next_account_info(accounts_iter)?;
    let user_lst_token_account = next_account_info(accounts_iter)?;
    let user_obligation_account = next_account_info(accounts_iter)?;
    let lending_pool_account = next_account_info(accounts_iter)?;
    let lending_pool_reserve = next_account_info(accounts_iter)?;
    let lending_pool_authority = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract borrow parameters
    let StakeLendInstruction::Borrow { amount, collateral_token } = instruction else {
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
        _ => return Err(StakeLendError::InvalidPoolState.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(
        program_id,
        lending_pool_account.key,
    );
    if authority_address != *lending_pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Create or load obligation account
    let mut obligation = if user_obligation_account.data_is_empty() {
        // Create new obligation
        let obligation = Obligation {
            is_initialized: true,
            owner: *user_account.key,
            lending_pool: *lending_pool_account.key,
            collateral_amount: 0,
            collateral_mint: collateral_token,
            borrowed_amount: 0,
            borrowed_amount_with_interest: 0,
            cumulative_borrow_rate_snapshot: lending_data.accumulated_interest_index,
            loan_origination_timestamp: crate::utils::math::get_current_timestamp(),
            last_update_timestamp: crate::utils::math::get_current_timestamp(),
        };
        obligation
    } else {
        // Load existing obligation
        let mut obligation = match Obligation::try_from_slice(&user_obligation_account.data.borrow()) {
            Ok(obligation) => obligation,
            Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
        };

        // Update obligation with accrued interest
        obligation.borrowed_amount_with_interest = crate::utils::math::calculate_accrued_interest(
            obligation.borrowed_amount_with_interest,
            obligation.cumulative_borrow_rate_snapshot,
            lending_data.accumulated_interest_index,
        )?;
        
        obligation.cumulative_borrow_rate_snapshot = lending_data.accumulated_interest_index;
        obligation.last_update_timestamp = crate::utils::math::get_current_timestamp();
        
        obligation
    };

    // Check available liquidity
    let available_liquidity = pool.total_deposits.saturating_sub(lending_data.total_borrows);
    if amount > available_liquidity {
        return Err(StakeLendError::InsufficientLiquidity.into());
    }

    // Transfer collateral from user if needed
    // This is simplified - in a real implementation, you would check existing collateral
    // and add to it, or accept different types of collateral
    
    // Get collateral value from oracle
    // This is simplified - in reality, you'd query a price oracle
    let collateral_price = 1_000_000; // Placeholder price with 6 decimals
    
    // Calculate required collateral based on LTV
    let required_collateral = crate::utils::math::calculate_required_collateral(
        amount,
        collateral_price,
        lending_data.max_ltv,
    )?;
    
    // Check if user has enough collateral
    // This is simplified - in reality, you'd actually check the token balance
    

    let transfer_collateral_ix = spl_token::instruction::transfer(
        &token_program.key,
        &user_collateral_account.key,
        &lending_pool_reserve.key, // This should be a separate collateral reserve in practice
        &user_account.key,
        &[],
        required_collateral,
    )?;

    invoke(
        &transfer_collateral_ix,
        &[
            user_collateral_account.clone(),
            lending_pool_reserve.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Update the obligation with the collateral
    obligation.collateral_amount = obligation.collateral_amount.saturating_add(required_collateral);

    // Calculate maximum borrow amount based on collateral value
    let max_borrow_amount = crate::utils::math::calculate_max_borrow_amount(
        obligation.collateral_amount,
        collateral_price,
        lending_data.max_ltv,
    )?;

    // Verify user isn't trying to borrow more than allowed by their collateral
    let total_borrowed_with_new = obligation.borrowed_amount_with_interest.saturating_add(amount);
    if total_borrowed_with_new > max_borrow_amount {
        return Err(StakeLendError::InsufficientCollateral.into());
    }

    // Transfer LST tokens to the user (the actual borrowed amount)
    let transfer_lst_ix = spl_token::instruction::transfer(
        &token_program.key,
        &lending_pool_reserve.key,
        &user_lst_token_account.key,
        &lending_pool_authority.key,
        &[],
        amount,
    )?;

    invoke_signed(
        &transfer_lst_ix,
        &[
            lending_pool_reserve.clone(),
            user_lst_token_account.clone(),
            lending_pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            lending_pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    // Update obligation with new borrowed amount 
    obligation.borrowed_amount = obligation.borrowed_amount.saturating_add(amount);
    obligation.borrowed_amount_with_interest = obligation.borrowed_amount_with_interest.saturating_add(amount);

    // Update pool state with new borrows
    lending_data.total_borrows = lending_data.total_borrows.saturating_add(amount);

    // Update lending pool utilization rate and interest rates
    if pool.total_deposits > 0 {
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

    // Save updated pool data
    borsh::to_writer(&mut lending_pool_account.data.borrow_mut()[..], &pool)?;

    // Save updated obligation data
    borsh::to_writer(&mut user_obligation_account.data.borrow_mut()[..], &obligation)?;

    msg!("Borrowed {} tokens with {} collateral", amount, required_collateral);
    Ok(())
    }

    /// Process repay instruction
    pub fn process_repay(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
    ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_lst_token_account = next_account_info(accounts_iter)?;
    let user_collateral_account = next_account_info(accounts_iter)?;
    let user_obligation_account = next_account_info(accounts_iter)?;
    let lending_pool_account = next_account_info(accounts_iter)?;
    let lending_pool_reserve = next_account_info(accounts_iter)?;
    let lending_pool_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract repay parameters
    let StakeLendInstruction::Repay { amount, withdraw_collateral } = instruction else {
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
        _ => return Err(StakeLendError::InvalidPoolState.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(
        program_id,
        lending_pool_account.key,
    );
    if authority_address != *lending_pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Load obligation
    let mut obligation = match Obligation::try_from_slice(&user_obligation_account.data.borrow()) {
        Ok(obligation) => obligation,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Verify obligation owner
    if obligation.owner != *user_account.key {
        return Err(StakeLendError::InvalidOwner.into());
    }

    // Update obligation with accrued interest
    obligation.borrowed_amount_with_interest = crate::utils::math::calculate_accrued_interest(
        obligation.borrowed_amount_with_interest,
        obligation.cumulative_borrow_rate_snapshot,
        lending_data.accumulated_interest_index,
    )?;

    obligation.cumulative_borrow_rate_snapshot = lending_data.accumulated_interest_index;
    obligation.last_update_timestamp = crate::utils::math::get_current_timestamp();

    // Calculate repay amount (either user specified amount or full debt)
    let repay_amount = if amount == 0 {
        obligation.borrowed_amount_with_interest
    } else {
        std::cmp::min(amount, obligation.borrowed_amount_with_interest)
    };

    // Transfer tokens from user to pool reserve
    let transfer_repay_ix = spl_token::instruction::transfer(
        &token_program.key,
        &user_lst_token_account.key,
        &lending_pool_reserve.key,
        &user_account.key,
        &[],
        repay_amount,
    )?;

    invoke(
        &transfer_repay_ix,
        &[
            user_lst_token_account.clone(),
            lending_pool_reserve.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Update obligation with reduced debt
    obligation.borrowed_amount_with_interest = obligation.borrowed_amount_with_interest.saturating_sub(repay_amount);

    // If principal is being repaid (not just interest)
    let principal_repayment = std::cmp::min(repay_amount, obligation.borrowed_amount);
    obligation.borrowed_amount = obligation.borrowed_amount.saturating_sub(principal_repayment);

    // Update pool state
    lending_data.total_borrows = lending_data.total_borrows.saturating_sub(principal_repayment);

    // Handle collateral withdrawal if requested
    if withdraw_collateral > 0 && obligation.borrowed_amount_with_interest < obligation.collateral_amount {
        // Calculate how much collateral can be safely withdrawn
        let collateral_price = 1_000_000; // Placeholder price with 6 decimals
        
        let required_collateral = if obligation.borrowed_amount_with_interest > 0 {
            crate::utils::math::calculate_required_collateral(
                obligation.borrowed_amount_with_interest,
                collateral_price,
                lending_data.max_ltv,
            )?
        } else {
            0
        };
        
        // Calculate max withdrawable collateral
        let max_withdrawable = obligation.collateral_amount.saturating_sub(required_collateral);
        let collateral_to_withdraw = std::cmp::min(withdraw_collateral, max_withdrawable);
        
        if collateral_to_withdraw > 0 {
            // Transfer collateral back to user
            let transfer_collateral_ix = spl_token::instruction::transfer(
                &token_program.key,
                &lending_pool_reserve.key, // Should be a separate collateral reserve in practice
                &user_collateral_account.key,
                &lending_pool_authority.key,
                &[],
                collateral_to_withdraw,
            )?;
            
            invoke_signed(
                &transfer_collateral_ix,
                &[
                    lending_pool_reserve.clone(),
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
            obligation.collateral_amount = obligation.collateral_amount.saturating_sub(collateral_to_withdraw);
        }
    }

    // Update pool utilization rate and interest rates
    if pool.total_deposits > 0 {
        lending_data.utilization_rate = crate::utils::math::calculate_utilization_rate(
            lending_data.total_borrows,
            pool.total_deposits,
        )?;
        
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

    // If debt is completely repaid and no collateral remains, close obligation account
    if obligation.borrowed_amount_with_interest == 0 && obligation.collateral_amount == 0 {
        // In a real implementation, you might want to close the account and reclaim rent
        // This would involve transferring lamports to the user and zeroing out data
    }

    // Save updated pool data
    borsh::to_writer(&mut lending_pool_account.data.borrow_mut()[..], &pool)?;

    // Save updated obligation data
    borsh::to_writer(&mut user_obligation_account.data.borrow_mut()[..], &obligation)?;

    msg!("Repaid {} tokens and withdrew {} collateral", repay_amount, withdraw_collateral);
    Ok(())
    }

    /// Process liquidation instruction
    pub fn process_liquidate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
    ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let liquidator_account = next_account_info(accounts_iter)?;
    let liquidator_lst_token_account = next_account_info(accounts_iter)?;
    let liquidator_collateral_account = next_account_info(accounts_iter)?;
    let borrower_obligation_account = next_account_info(accounts_iter)?;
    let lending_pool_account = next_account_info(accounts_iter)?;
    let lending_pool_reserve = next_account_info(accounts_iter)?;
    let lending_pool_authority = next_account_info(accounts_iter)?;
    let oracle_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify liquidator signature
    if !liquidator_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract liquidation parameters
    let StakeLendInstruction::Liquidate { repay_amount } = instruction else {
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
        _ => return Err(StakeLendError::InvalidPoolState.into()),
    };

    // Verify pool authority
    let (authority_address, authority_bump) = pda::find_pool_authority_address(
        program_id,
        lending_pool_account.key,
    );
    if authority_address != *lending_pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }

    // Load borrower's obligation
    let mut obligation = match Obligation::try_from_slice(&borrower_obligation_account.data.borrow()) {
        Ok(obligation) => obligation,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };

    // Update obligation with accrued interest
    obligation.borrowed_amount_with_interest = crate::utils::math::calculate_accrued_interest(
        obligation.borrowed_amount_with_interest,
        obligation.cumulative_borrow_rate_snapshot,
        lending_data.accumulated_interest_index,
    )?;

    obligation.cumulative_borrow_rate_snapshot = lending_data.accumulated_interest_index;
    obligation.last_update_timestamp = crate::utils::math::get_current_timestamp();

    // Get collateral price from oracle
    let collateral_price = 1_000_000; // Placeholder price with 6 decimals

    // Calculate health factor
    let health_factor = crate::utils::math::calculate_health_factor(
        obligation.collateral_amount,
        collateral_price,
        obligation.borrowed_amount_with_interest,
        lending_data.liquidation_threshold,
    )?;

    // Check if liquidation is allowed (health factor < 1.0)
    if health_factor >= 10_000 { // 1.0 scaled to basis points
        return Err(StakeLendError::CannotLiquidateHealthyPosition.into());
    }

    // Calculate maximum repayable amount (typically up to 50% of the debt)
    let max_liquidation_amount = obligation.borrowed_amount_with_interest / 2;
    let liquidation_amount = std::cmp::min(repay_amount, max_liquidation_amount);

    // Calculate collateral to receive (with bonus)
    let collateral_value_required = (liquidation_amount as u128)
        .saturating_mul(10_000 + lending_data.liquidation_bonus as u128)
        .saturating_div(10_000) as u64;

    let collateral_to_liquidate = crate::utils::math::calculate_collateral_amount(
        collateral_value_required,
        collateral_price,
    )?;

    // Ensure there's enough collateral
    if collateral_to_liquidate > obligation.collateral_amount {
        return Err(StakeLendError::InsufficientCollateral.into());
    }

    // Repay debt from liquidator to pool
    let transfer_repay_ix = spl_token::instruction::transfer(
        &token_program.key,
        &liquidator_lst_token_account.key,
        &lending_pool_reserve.key,
        &liquidator_account.key,
        &[],
        liquidation_amount,
    )?;

    invoke(
        &transfer_repay_ix,
        &[
            liquidator_lst_token_account.clone(),
            lending_pool_reserve.clone(),
            liquidator_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Transfer collateral to liquidator
    let transfer_collateral_ix = spl_token::instruction::transfer(
        &token_program.key,
        &lending_pool_reserve.key, // Should be a separate collateral reserve in practice
        &liquidator_collateral_account.key,
        &lending_pool_authority.key,
        &[],
        collateral_to_liquidate,
    )?;

    invoke_signed(
        &transfer_collateral_ix,
        &[
            lending_pool_reserve.clone(),
            liquidator_collateral_account.clone(),
            lending_pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            lending_pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    // Update obligation
    obligation.borrowed_amount_with_interest = obligation.borrowed_amount_with_interest.saturating_sub(liquidation_amount);
    obligation.borrowed_amount = obligation.borrowed_amount.saturating_sub(
        std::cmp::min(liquidation_amount, obligation.borrowed_amount)
    );
    obligation.collateral_amount = obligation.collateral_amount.saturating_sub(collateral_to_liquidate);

    // Update pool state
    lending_data.total_borrows = lending_data.total_borrows.saturating_sub(
        std::cmp::min(liquidation_amount, obligation.borrowed_amount)
    );

    // Save updated pool data
    borsh::to_writer(&mut lending_pool_account.data.borrow_mut()[..], &pool)?;

    // Save updated obligation data
    borsh::to_writer(&mut borrower_obligation_account.data.borrow_mut()[..], &obligation)?;

    msg!(
        "Liquidated {} debt with {} collateral (including bonus)",
        liquidation_amount,
        collateral_to_liquidate
    );
    Ok(())
    }