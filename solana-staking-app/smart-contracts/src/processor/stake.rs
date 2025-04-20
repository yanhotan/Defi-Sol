// smart-contracts/src/processor/stake.rs
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

use crate::{
    error::StakeLendError,
    instructions::StakeLendInstruction,
    state::{Pool, pda},
};

/// Process stake SOL instruction
pub fn process_stake_sol(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_sol_account = next_account_info(accounts_iter)?;
    let user_lst_token_account = next_account_info(accounts_iter)?;
    let staking_pool_account = next_account_info(accounts_iter)?;
    let staking_pool_token_mint = next_account_info(accounts_iter)?;
    let staking_pool_reserve = next_account_info(accounts_iter)?;
    let staking_pool_authority = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract stake parameters
    let StakeLendInstruction::StakeSOL { amount, lst_type } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Verify stake amount
    if amount == 0 {
        return Err(StakeLendError::InvalidDepositAmount.into());
    }

    // Load staking pool data (would be a lookup to marinade or jito in a real implementation)
    // This is simplified - in reality, you'd interact with the actual staking program
    
    // Transfer SOL from user to pool reserve
    invoke(
        &system_instruction::transfer(
            user_account.key,
            staking_pool_reserve.key,
            amount,
        ),
        &[
            user_account.clone(),
            staking_pool_reserve.clone(),
            system_program.clone(),
        ],
    )?;

    // Calculate LST amount (typically 1:1 minus fees in most liquid staking protocols)
    // In a real implementation, this would use the staking protocol's exchange rate
    let lst_fee = amount / 100; // Simplified 1% fee
    let lst_amount = amount - lst_fee;

    // Mint LST tokens to user
    // This is simplified - in reality, you'd receive tokens from the LST protocol
    let mint_to_ix = spl_token::instruction::mint_to(
        &token_program.key,
        &staking_pool_token_mint.key,
        &user_lst_token_account.key,
        &staking_pool_authority.key,
        &[],
        lst_amount,
    )?;
    
    let (authority_address, authority_bump) = pda::find_pool_authority_address(
        program_id,
        staking_pool_account.key,
    );
    
    invoke_signed(
        &mint_to_ix,
        &[
            staking_pool_token_mint.clone(),
            user_lst_token_account.clone(),
            staking_pool_authority.clone(),
            token_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            staking_pool_account.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    msg!("Staked {} SOL for {} LST tokens", amount, lst_amount);
    Ok(())
}

/// Process unstake SOL instruction
pub fn process_unstake_sol(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StakeLendInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let user_lst_token_account = next_account_info(accounts_iter)?;
    let user_sol_account = next_account_info(accounts_iter)?;
    let program_lst_reserve = next_account_info(accounts_iter)?;
    let program_sol_reserve = next_account_info(accounts_iter)?;
    let program_authority = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Verify user signature
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Extract unstake parameters
    let StakeLendInstruction::UnstakeSOL { amount, instant } = instruction else {
        return Err(StakeLendError::InvalidInstruction.into());
    };

    // Verify unstake amount
    if amount == 0 {
        return Err(StakeLendError::InvalidWithdrawalAmount.into());
    }

    // Transfer LST tokens from user to program reserve
    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &user_lst_token_account.key,
        &program_lst_reserve.key,
        &user_account.key,
        &[],
        amount,
    )?;
    
    invoke(
        &transfer_ix,
        &[
            user_lst_token_account.clone(),
            program_lst_reserve.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Calculate SOL amount to return
    // In a real implementation, this would use the staking protocol's exchange rate
    let mut sol_amount = amount;
    
    // Apply fee if instant unstake
    if instant {
        let instant_fee = sol_amount / 20; // 5% fee for instant unstake
        sol_amount -= instant_fee;
    } else {
        // For delayed unstake, you would add user to a queue
        // and they would claim their SOL after a delay period
        // This is simplified for the example
    }

    // Transfer SOL to user
    let (authority_address, authority_bump) = pda::find_pool_authority_address(
        program_id,
        program_sol_reserve.key,
    );
    
    let transfer_sol_ix = system_instruction::transfer(
        program_sol_reserve.key,
        user_sol_account.key,
        sol_amount,
    );
    
    invoke_signed(
        &transfer_sol_ix,
        &[
            program_sol_reserve.clone(),
            user_sol_account.clone(),
            system_program.clone(),
        ],
        &[&[
            pda::POOL_AUTHORITY_SEED.as_bytes(),
            program_sol_reserve.key.as_ref(),
            &[authority_bump],
        ]],
    )?;

    msg!("Unstaked {} LST tokens for {} SOL", amount, sol_amount);
    Ok(())
}