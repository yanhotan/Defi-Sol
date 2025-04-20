// smart-contracts/src/instructions.rs
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey,
};

use crate::{error::StakeLendError, processor};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum StakeLendInstruction {
    /// Initialize program config with admin settings
    /// Accounts:
    /// 0. `[signer]` Program admin
    /// 1. `[writable]` Config account
    /// 2. `[]` System program
    InitializeConfig {
        protocol_fee_bps: u16,
        treasury_wallet: Pubkey,
    },

    /// Initialize pool (admin only)
    /// Accounts:
    /// 0. `[signer]` Program admin
    /// 1. `[]` Config account
    /// 2. `[writable]` Pool account
    /// 3. `[writable]` Pool token mint account
    /// 4. `[]` Pool authority (PDA)
    /// 5. `[]` System program
    /// 6. `[]` Token program
    InitializePool {
        pool_type: u8, // 0: Basic, 1: Lending, 2: Lock
        name: String,  // Name identifier
        min_deposit: u64,
        max_deposit: u64,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        interest_rate_params: InterestRateParams,
        lock_duration_seconds: Option<u64>, // For Lock pools
    },

    /// Stake SOL and receive liquid staking tokens
    /// Accounts:
    /// 0. `[signer]` User account
    /// 1. `[writable]` User SOL account (must be a system account)
    /// 2. `[writable]` User LST token account
    /// 3. `[writable]` Staking pool account
    /// 4. `[writable]` Staking pool token mint
    /// 5. `[writable]` Staking pool reserve account
    /// 6. `[]` Staking pool authority
    /// 7. `[]` System program
    /// 8. `[]` Token program
    StakeSOL {
        amount: u64,          // Amount of SOL to stake
        lst_type: u8,         // 0: mSOL, 1: jitoSOL
    },

    /// Deposit liquid staking tokens into a pool
    /// Accounts:
    /// 0. `[signer]` User account
    /// 1. `[writable]` User LST token account
    /// 2. `[writable]` User pool token account
    /// 3. `[writable]` Pool account
    /// 4. `[writable]` Pool LST reserve account
    /// 5. `[writable]` Pool token mint
    /// 6. `[]` Pool authority
    /// 7. `[]` Token program
    DepositToPool {
        amount: u64,          // Amount of LST to deposit
        pool_type: u8,        // 0: Basic, 1: Lending, 2: Lock
        lock_duration: Option<u64>, // For Lock pools
    },

    /// Withdraw assets from a pool
    /// Accounts:
    /// 0. `[signer]` User account
    /// 1. `[writable]` User pool token account
    /// 2. `[writable]` User LST token account
    /// 3. `[writable]` Pool account
    /// 4. `[writable]` Pool LST reserve account 
    /// 5. `[writable]` Pool token mint
    /// 6. `[]` Pool authority
    /// 7. `[]` Token program
    WithdrawFromPool {
        amount: u64,          // Amount of pool tokens to burn for withdrawal
    },

    /// Unstake LST back to SOL
    /// Accounts:
    /// 0. `[signer]` User account
    /// 1. `[writable]` User LST token account
    /// 2. `[writable]` User SOL account
    /// 3. `[writable]` Program LST reserve account
    /// 4. `[writable]` Program SOL reserve account
    /// 5. `[]` Program authority
    /// 6. `[]` System program
    /// 7. `[]` Token program 
    UnstakeSOL {
        amount: u64,          // Amount of LST to unstake
        instant: bool,        // Instant unstake (with fee) or delayed
    },

    /// Borrow assets from the lending pool
    /// Accounts:
    /// 0. `[signer]` User account
    /// 1. `[writable]` User token account (deposited collateral)
    /// 2. `[writable]` User LST token account to receive borrowed amount
    /// 3. `[writable]` User obligation account
    /// 4. `[writable]` Lending pool account
    /// 5. `[writable]` Lending pool reserve account
    /// 6. `[]` Lending pool authority
    /// 7. `[]` Oracle account
    /// 8. `[]` Token program
    Borrow {
        amount: u64,          // Amount to borrow
        collateral_token: Pubkey, // Mint address of collateral token
    },

    /// Repay borrowed assets
    /// Accounts:
    /// 0. `[signer]` User account
    /// 1. `[writable]` User LST token account to repay from
    /// 2. `[writable]` User obligation account
    /// 3. `[writable]` Lending pool account
    /// 4. `[writable]` Lending pool reserve account
    /// 5. `[]` Lending pool authority
    /// 6. `[]` Token program
    Repay {
        amount: u64,          // Amount to repay
    },

    /// Liquidate unhealthy position
    /// Accounts:
    /// 0. `[signer]` Liquidator account
    /// 1. `[writable]` Liquidator LST token account
    /// 2. `[writable]` Liquidator collateral token account
    /// 3. `[writable]` Borrower obligation account
    /// 4. `[writable]` Lending pool account
    /// 5. `[writable]` Lending pool reserve account
    /// 6. `[]` Lending pool authority
    /// 7. `[]` Oracle account
    /// 8. `[]` Token program
    Liquidate {
        repay_amount: u64,    // Amount of debt to repay
    },

    /// Update interest rates and pool states
    /// Accounts:
    /// 0. `[signer]` Admin or authorized account
    /// 1. `[writable]` Pool account
    /// 2. `[]` Oracle account (optional)
    UpdatePoolState {},

    /// Claim rewards from different pools
    /// Accounts:
    /// 0. `[signer]` User account
    /// 1. `[writable]` User pool token account
    /// 2. `[writable]` User reward token account
    /// 3. `[writable]` Pool account
    /// 4. `[writable]` Pool reward account
    /// 5. `[]` Pool authority
    /// 6. `[]` Token program
    ClaimRewards {},
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct InterestRateParams {
    pub base_rate: u64,              // Base rate in bps (e.g. 200 = 2%)
    pub slope1: u64,                 // Slope for utilization < optimal
    pub slope2: u64,                 // Slope for utilization > optimal
    pub optimal_utilization: u64,    // Optimal utilization point in bps (e.g. 8000 = 80%)
}

impl StakeLendInstruction {
    /// Unpacks a byte buffer into a StakeLendInstruction
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(StakeLendError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (protocol_fee_bps, rest) = Self::unpack_u16(rest)?;
                let treasury_wallet = Self::unpack_pubkey(rest)?;
                Self::InitializeConfig {
                    protocol_fee_bps,
                    treasury_wallet,
                }
            }
            1 => {
                let (pool_type, rest) = Self::unpack_u8(rest)?;
                let (name_len, rest) = Self::unpack_u8(rest)?;
                let (name_bytes, rest) = rest.split_at(name_len as usize);
                let name = String::from_utf8(name_bytes.to_vec())
                    .map_err(|_| StakeLendError::InvalidInstruction)?;
                
                let (min_deposit, rest) = Self::unpack_u64(rest)?;
                let (max_deposit, rest) = Self::unpack_u64(rest)?;
                let (deposit_fee_bps, rest) = Self::unpack_u16(rest)?;
                let (withdrawal_fee_bps, rest) = Self::unpack_u16(rest)?;
                
                let (base_rate, rest) = Self::unpack_u64(rest)?;
                let (slope1, rest) = Self::unpack_u64(rest)?;
                let (slope2, rest) = Self::unpack_u64(rest)?;
                let (optimal_utilization, rest) = Self::unpack_u64(rest)?;
                
                let interest_rate_params = InterestRateParams {
                    base_rate,
                    slope1,
                    slope2,
                    optimal_utilization,
                };
                
                let (has_lock_duration, rest) = Self::unpack_u8(rest)?;
                let lock_duration_seconds = if has_lock_duration == 1 {
                    let (duration, _) = Self::unpack_u64(rest)?;
                    Some(duration)
                } else {
                    None
                };
                
                Self::InitializePool {
                    pool_type,
                    name,
                    min_deposit,
                    max_deposit,
                    deposit_fee_bps,
                    withdrawal_fee_bps,
                    interest_rate_params,
                    lock_duration_seconds,
                }
            }
            2 => {
                let (amount, rest) = Self::unpack_u64(rest)?;
                let (lst_type, _) = Self::unpack_u8(rest)?;
                Self::StakeSOL { amount, lst_type }
            }
            3 => {
                let (amount, rest) = Self::unpack_u64(rest)?;
                let (pool_type, rest) = Self::unpack_u8(rest)?;
                
                let (has_lock_duration, rest) = Self::unpack_u8(rest)?;
                let lock_duration = if has_lock_duration == 1 {
                    let (duration, _) = Self::unpack_u64(rest)?;
                    Some(duration)
                } else {
                    None
                };
                
                Self::DepositToPool {
                    amount,
                    pool_type,
                    lock_duration,
                }
            }
            4 => {
                let (amount, _) = Self::unpack_u64(rest)?;
                Self::WithdrawFromPool { amount }
            }
            5 => {
                let (amount, rest) = Self::unpack_u64(rest)?;
                let (instant, _) = Self::unpack_u8(rest)?;
                Self::UnstakeSOL {
                    amount,
                    instant: instant != 0,
                }
            }
            6 => {
                let (amount, rest) = Self::unpack_u64(rest)?;
                let collateral_token = Self::unpack_pubkey(rest)?;
                Self::Borrow {
                    amount,
                    collateral_token,
                }
            }
            7 => {
                let (amount, _) = Self::unpack_u64(rest)?;
                Self::Repay { amount }
            }
            8 => {
                let (repay_amount, _) = Self::unpack_u64(rest)?;
                Self::Liquidate { repay_amount }
            }
            9 => Self::UpdatePoolState {},
            10 => Self::ClaimRewards {},
            _ => return Err(StakeLendError::InvalidInstruction.into()),
        })
    }

    fn unpack_u8(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
        if input.len() >= 1 {
            let (value, rest) = input.split_at(1);
            let value = value[0];
            Ok((value, rest))
        } else {
            Err(StakeLendError::InvalidInstruction.into())
        }
    }

    fn unpack_u16(input: &[u8]) -> Result<(u16, &[u8]), ProgramError> {
        if input.len() >= 2 {
            let (value, rest) = input.split_at(2);
            let value = u16::from_le_bytes([value[0], value[1]]);
            Ok((value, rest))
        } else {
            Err(StakeLendError::InvalidInstruction.into())
        }
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() >= 8 {
            let (value, rest) = input.split_at(8);
            let value = u64::from_le_bytes(
                value.try_into().map_err(|_| StakeLendError::InvalidInstruction)?,
            );
            Ok((value, rest))
        } else {
            Err(StakeLendError::InvalidInstruction.into())
        }
    }

    fn unpack_pubkey(input: &[u8]) -> Result<Pubkey, ProgramError> {
        if input.len() >= 32 {
            let (key, _) = input.split_at(32);
            Ok(Pubkey::new_from_array(
                key.try_into().map_err(|_| StakeLendError::InvalidInstruction)?,
            ))
        } else {
            Err(StakeLendError::InvalidInstruction.into())
        }
    }
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StakeLendInstruction::unpack(instruction_data)?;

    match instruction {
        StakeLendInstruction::InitializeConfig { .. } => {
            processor::initialize_config(program_id, accounts, instruction)
        }
        StakeLendInstruction::InitializePool { .. } => {
            processor::initialize_pool(program_id, accounts, instruction)
        }
        StakeLendInstruction::StakeSOL { .. } => processor::stake::process_stake_sol(program_id, accounts, instruction),
        StakeLendInstruction::DepositToPool { pool_type, .. } => match pool_type {
            0 => processor::pools::basic_pool::process_deposit(program_id, accounts, instruction),
            1 => processor::pools::lending_pool::process_deposit(program_id, accounts, instruction),
            2 => processor::pools::lock_pool::process_deposit(program_id, accounts, instruction),
            _ => Err(StakeLendError::InvalidInstruction.into()),
        },
        StakeLendInstruction::WithdrawFromPool { .. } => {
            processor::process_withdraw_from_pool(program_id, accounts, instruction)
        }
        StakeLendInstruction::UnstakeSOL { .. } => {
            processor::stake::process_unstake_sol(program_id, accounts, instruction)
        }
        StakeLendInstruction::Borrow { .. } => processor::lend::process_borrow(program_id, accounts, instruction),
        StakeLendInstruction::Repay { .. } => processor::lend::process_repay(program_id, accounts, instruction),
        StakeLendInstruction::Liquidate { .. } => processor::risk::process_liquidate(program_id, accounts, instruction),
        StakeLendInstruction::UpdatePoolState { .. } => {
            processor::process_update_pool_state(program_id, accounts, instruction)
        }
        StakeLendInstruction::ClaimRewards { .. } => {
            processor::process_claim_rewards(program_id, accounts, instruction)
        }
    }
}