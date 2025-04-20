// smart-contracts/src/state.rs
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::convert::TryFrom;

use crate::error::StakeLendError;
use crate::instructions::InterestRateParams;

/// Protocol config
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct ProtocolConfig {
    pub is_initialized: bool,
    pub admin: Pubkey,
    pub protocol_fee_bps: u16,    // Protocol fee in basis points (e.g., 50 = 0.5%)
    pub treasury_wallet: Pubkey,  // Where protocol fees are sent
    pub pause_flags: u16,         // Bit flags for pausing different functions
    pub upgrade_authority: Pubkey,
}

impl IsInitialized for ProtocolConfig {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

/// Pool types
pub enum PoolType {
    Basic = 0,
    Lending = 1,
    Lock = 2,
}

impl TryFrom<u8> for PoolType {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PoolType::Basic),
            1 => Ok(PoolType::Lending),
            2 => Ok(PoolType::Lock),
            _ => Err(StakeLendError::InvalidInstruction.into()),
        }
    }
}

/// Common pool data shared across all pool types
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct Pool {
    pub is_initialized: bool,
    pub pool_type: u8,
    pub name: [u8; 32],  // Fixed size array for name
    pub pool_authority: Pubkey,
    pub token_mint: Pubkey,
    pub lst_reserve: Pubkey,
    pub min_deposit: u64,
    pub max_deposit: u64,
    pub total_deposits: u64,
    pub deposit_fee_bps: u16,
    pub withdrawal_fee_bps: u16,
    pub total_shares: u64,
    pub last_update_timestamp: u64,
    // Pool specific data based on type
    pub data: PoolData,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum PoolData {
    Basic(BasicPoolData),
    Lending(LendingPoolData),
    Lock(LockPoolData),
}

impl Default for PoolData {
    fn default() -> Self {
        PoolData::Basic(BasicPoolData::default())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct BasicPoolData {
    pub instant_unstake_fee_bps: u16,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct LendingPoolData {
    pub interest_rate_params: InterestRateParams,
    pub current_borrow_rate: u64,
    pub current_supply_rate: u64,
    pub total_borrows: u64,
    pub total_reserves: u64,
    pub utilization_rate: u64,
    pub accumulated_interest_index: u128,
    pub last_interest_update_timestamp: u64,
    pub liquidation_threshold: u16, // In bps (e.g. 8500 = 85%)
    pub liquidation_bonus: u16,     // In bps (e.g. 500 = 5%)
    pub max_ltv: u16,               // Maximum loan-to-value ratio in bps
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct LockPoolData {
    pub lock_duration: u64,      // Lock duration in seconds
    pub yield_boost_bps: u16,    // Additional yield boost in basis points
    pub early_unlock_penalty_bps: u16, // Penalty for early unlock
}

impl IsInitialized for Pool {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

/// Obligation represents a user's borrow position
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct Obligation {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub lending_pool: Pubkey,
    pub collateral_amount: u64,
    pub collateral_mint: Pubkey,
    pub borrowed_amount: u64,
    pub borrowed_amount_with_interest: u64,
    pub cumulative_borrow_rate_snapshot: u128,
    pub loan_origination_timestamp: u64,
    pub last_update_timestamp: u64,
}

impl IsInitialized for Obligation {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

/// User deposit in a lock pool
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct LockPosition {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub deposited_amount: u64,
    pub shares: u64,
    pub lock_start_timestamp: u64,
    pub lock_end_timestamp: u64,
}

impl IsInitialized for LockPosition {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

/// Oracle price data
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct OraclePrice {
    pub price: u64,           // Price with decimals
    pub decimals: u8,         // Number of decimal places in price
    pub confidence: u64,      // Price confidence interval
    pub last_update_timestamp: u64,
}

// PDA account seed constants
pub const PROTOCOL_CONFIG_SEED: &str = "config";
pub const POOL_AUTHORITY_SEED: &str = "pool_authority";
pub const OBLIGATION_SEED: &str = "obligation";
pub const LOCK_POSITION_SEED: &str = "lock_position";

// Utility functions for creating PDAs
pub mod pda {
    use super::*;
    use solana_program::program_error::ProgramError;

    pub fn find_protocol_config_address(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[PROTOCOL_CONFIG_SEED.as_bytes()], program_id)
    }

    pub fn find_pool_authority_address(program_id: &Pubkey, pool_address: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[POOL_AUTHORITY_SEED.as_bytes(), pool_address.as_ref()],
            program_id,
        )
    }

    pub fn find_obligation_address(
        program_id: &Pubkey,
        owner: &Pubkey,
        lending_pool: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                OBLIGATION_SEED.as_bytes(),
                owner.as_ref(),
                lending_pool.as_ref(),
            ],
            program_id,
        )
    }

    pub fn find_lock_position_address(
        program_id: &Pubkey,
        owner: &Pubkey,
        pool: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                LOCK_POSITION_SEED.as_bytes(),
                owner.as_ref(),
                pool.as_ref(),
            ],
            program_id,
        )
    }
}