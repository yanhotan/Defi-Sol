// smart-contracts/src/error.rs
use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, FromPrimitive, PartialEq)]
pub enum StakeLendError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    #[error("Not rent exempt")]
    NotRentExempt,
    
    #[error("Account not initialized")]
    UninitializedAccount,
    
    #[error("Program account not expected")]
    InvalidProgramAccount,
    
    #[error("Invalid owner")]
    InvalidOwner,
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Invalid deposit amount")]
    InvalidDepositAmount,
    
    #[error("Invalid withdrawal amount")]
    InvalidWithdrawalAmount,
    
    #[error("Invalid account data")]
    InvalidAccountData,
    
    #[error("Pool full")]
    PoolFull,
    
    #[error("Pool not accepting deposits")]
    PoolNotAcceptingDeposits,
    
    #[error("Pool not accepting borrows")]
    PoolNotAcceptingBorrows,
    
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[error("Insufficient collateral")]
    InsufficientCollateral,
    
    #[error("Unhealthy position")]
    UnhealthyPosition,
    
    #[error("Liquidation not allowed")]
    LiquidationNotAllowed,
    
    #[error("Lock period not ended")]
    LockPeriodNotEnded,
    
    #[error("Invalid oracle data")]
    InvalidOracleData,
    
    #[error("Invalid interest rate model")]
    InvalidInterestRateModel,
    
    #[error("Mathematical operation overflow")]
    MathOverflow,
    
    #[error("Invalid pool state")]
    InvalidPoolState,
    
    #[error("Invalid token account")]
    InvalidTokenAccount,
}

impl From<StakeLendError> for ProgramError {
    fn from(e: StakeLendError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl PrintProgramError for StakeLendError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for StakeLendError {
    fn type_of() -> &'static str {
        "StakeLendError"
    }
}