use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
    msg,
};

use spl_token::state::{Account as TokenAccount, Mint};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    error::StakeLendError,
    state::{Pool, PoolData, UserPosition, pda},
};

/// Validate user accounts for standard operations
pub fn validate_user_accounts(
    user_account: &AccountInfo,
    user_position_account: Option<&AccountInfo>,
) -> ProgramResult {
    // Check user is a signer
    check_signer(user_account)?;
    
    // If user position account is provided, validate it
    if let Some(position_account) = user_position_account {
        let user_position = match UserPosition::try_from_slice(&position_account.data.borrow()) {
            Ok(position) => position,
            Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
        };
        
        // Verify user is the owner of the position
        if user_position.owner != *user_account.key {
            return Err(StakeLendError::InvalidOwner.into());
        }
    }
    
    Ok(())
}

/// Validate pool accounts for operations
pub fn validate_pool_accounts(
    program_id: &Pubkey,
    pool_account: &AccountInfo,
    pool_authority: &AccountInfo,
    pool_type: Option<u8>,
) -> ProgramResult {
    // Load pool data
    let pool = match Pool::try_from_slice(&pool_account.data.borrow()) {
        Ok(pool) => pool,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };
    
    // Verify pool authority PDA
    let (authority_address, _) = pda::find_pool_authority_address(program_id, pool_account.key);
    if authority_address != *pool_authority.key {
        return Err(StakeLendError::InvalidProgramAccount.into());
    }
    
    // If specific pool type is required, verify it
    if let Some(required_type) = pool_type {
        let actual_type = match pool.data {
            PoolData::Basic(_) => 0,
            PoolData::Lending(_) => 1,
            PoolData::Lock(_) => 2,
        };
        
        if actual_type != required_type {
            return Err(StakeLendError::InvalidPoolType.into());
        }
    }
    
    Ok(())
}

/// Validate token accounts for operations
pub fn validate_token_accounts(
    user: &Pubkey,
    mint: &Pubkey,
    user_token_account: &AccountInfo,
    verify_associated: bool,
) -> ProgramResult {
    // Check account ownership
    if *user_token_account.owner != spl_token::id() {
        return Err(StakeLendError::InvalidTokenAccount.into());
    }
    
    // Load token account data
    let token_account_data = TokenAccount::unpack(&user_token_account.data.borrow())?;
    
    // Verify mint
    if token_account_data.mint != *mint {
        return Err(StakeLendError::TokenMintMismatch.into());
    }
    
    // Verify owner
    if token_account_data.owner != *user {
        return Err(StakeLendError::InvalidTokenOwner.into());
    }
    
    // Optional: verify this is the associated token address
    if verify_associated {
        let expected_ata = get_associated_token_address(user, mint);
        if expected_ata != *user_token_account.key {
            return Err(StakeLendError::InvalidAssociatedTokenAccount.into());
        }
    }
    
    Ok(())
}

/// Check if the account is owned by the given owner
pub fn check_account_owner(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        msg!("Account owner mismatch. Expected: {}, Actual: {}", 
            owner, account.owner);
        return Err(StakeLendError::InvalidAccountOwner.into());
    }
    Ok(())
}

/// Check if the account is a signer
pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    if !account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// Check if the token account's mint matches the expected mint
pub fn check_token_mint(token_account: &AccountInfo, expected_mint: &Pubkey) -> ProgramResult {
    // Verify account is owned by the token program
    check_account_owner(token_account, &spl_token::id())?;
    
    // Unpack token account data
    let token_account_data = TokenAccount::unpack(&token_account.data.borrow())?;
    
    // Verify mint
    if token_account_data.mint != *expected_mint {
        return Err(StakeLendError::TokenMintMismatch.into());
    }
    
    Ok(())
}

/// Verify an account is initialized and has sufficient rent exemption
pub fn verify_initialized_account(account: &AccountInfo) -> ProgramResult {
    // Check account is not empty
    if account.data_is_empty() {
        return Err(StakeLendError::UninitializedAccount.into());
    }
    
    // Check account is rent exempt
    let rent = Rent::get()?;
    if !rent.is_exempt(account.lamports(), account.data_len()) {
        return Err(StakeLendError::NotRentExempt.into());
    }
    
    Ok(())
}

/// Initialize account with proper space and rent exemption
pub fn initialize_account<'a>(
    payer: &AccountInfo<'a>,
    new_account: &AccountInfo<'a>,
    owner: &Pubkey,
    space: usize,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    // Check if account is already initialized
    if !new_account.data_is_empty() {
        return Err(StakeLendError::AccountAlreadyInitialized.into());
    }
    
    // Calculate rent exempt amount
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(space);
    
    // Create account instruction
    let create_account_ix = system_instruction::create_account(
        payer.key,
        new_account.key,
        rent_lamports,
        space as u64,
        owner,
    );
    
    // Create the account
    invoke(
        &create_account_ix,
        &[
            payer.clone(),
            new_account.clone(),
            system_program.clone(),
        ],
    )?;
    
    Ok(())
}

/// Verify that a value is within acceptable range
pub fn verify_value_in_range<T: PartialOrd>(
    value: T,
    min: T,
    max: T,
    error: ProgramError,
) -> ProgramResult {
    if value < min || value > max {
        return Err(error);
    }
    Ok(())
}

/// Verify that a user has sufficient balance in a token account
pub fn verify_token_balance(
    token_account: &AccountInfo,
    required_amount: u64,
) -> ProgramResult {
    // Unpack token account data
    let token_account_data = TokenAccount::unpack(&token_account.data.borrow())?;
    
    // Check balance
    if token_account_data.amount < required_amount {
        return Err(StakeLendError::InsufficientFunds.into());
    }
    
    Ok(())
}

/// Verify that a mint account has the expected properties
pub fn verify_mint_account(
    mint_account: &AccountInfo,
    expected_decimals: Option<u8>,
    expected_mint_authority: Option<&Pubkey>,
) -> ProgramResult {
    // Verify account is owned by the token program
    check_account_owner(mint_account, &spl_token::id())?;
    
    // Unpack mint data
    let mint_data = Mint::unpack(&mint_account.data.borrow())?;
    
    // Verify decimals if provided
    if let Some(decimals) = expected_decimals {
        if mint_data.decimals != decimals {
            return Err(StakeLendError::InvalidMintDecimals.into());
        }
    }
    
    // Verify mint authority if provided
    if let Some(mint_authority) = expected_mint_authority {
        match mint_data.mint_authority {
            spl_token::state::Authority::Key(authority) => {
                if authority != *mint_authority {
                    return Err(StakeLendError::InvalidMintAuthority.into());
                }
            },
            _ => return Err(StakeLendError::InvalidMintAuthority.into()),
        }
    }
    
    Ok(())
}

/// Verify that the accounts needed for staking are valid
pub fn validate_staking_accounts(
    program_id: &Pubkey,
    user: &AccountInfo,
    user_token_account: &AccountInfo,
    pool_account: &AccountInfo,
    pool_authority: &AccountInfo,
    pool_token_mint: &AccountInfo,
    token_mint: &Pubkey,
) -> ProgramResult {
    // Check user is signer
    check_signer(user)?;
    
    // Validate user token account
    validate_token_accounts(user.key, token_mint, user_token_account, false)?;
    
    // Validate pool accounts
    validate_pool_accounts(program_id, pool_account, pool_authority, None)?;
    
    // Verify pool token mint
    check_account_owner(pool_token_mint, &spl_token::id())?;
    
    // Additional pool token mint validations could be added here
    
    Ok(())
}

/// Validate admin authority for privileged operations
pub fn validate_admin_authority(
    admin_account: &AccountInfo,
    config_account: &AccountInfo,
) -> ProgramResult {
    // Check admin is signer
    check_signer(admin_account)?;
    
    // Load protocol config
    let config = match crate::state::ProtocolConfig::try_from_slice(&config_account.data.borrow()) {
        Ok(config) => config,
        Err(_) => return Err(StakeLendError::InvalidAccountData.into()),
    };
    
    // Verify admin pubkey matches
    if config.admin != *admin_account.key {
        return Err(StakeLendError::InvalidAdminAuthority.into());
    }
    
    Ok(())
}