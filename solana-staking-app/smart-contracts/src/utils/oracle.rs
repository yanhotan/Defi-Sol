use std::collections::HashMap;
use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
};

use crate::error::StakeLendError;

/// Market risk metrics from oracle data
pub struct MarketRiskMetrics {
    pub volatility_index: u64,         // Volatility index (higher = more volatile)
    pub market_trend: i8,              // Market trend (-100 to 100, negative = bearish)
    pub liquidity_score: u16,          // Liquidity score (0-10000, higher = more liquid)
    pub risk_level: RiskLevel,         // Overall risk assessment
    pub timestamp: u64,                // Timestamp of these metrics
}

/// Risk level classification
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

/// Supported oracle providers
pub enum OracleProvider {
    Pyth,
    Switchboard,
    ChainLink,
    Custom,
}

/// Asset price with additional metadata
pub struct PriceData {
    pub price: u64,               // Price in quote currency (normalized to SCALE_FACTOR)
    pub confidence: u64,          // Confidence interval
    pub timestamp: u64,           // Last update timestamp
    pub source: OracleProvider,   // Source of this price data
}

/// Oracle account layout for Pyth price feeds
#[allow(dead_code)]
mod pyth_layout {
    use std::mem::size_of;

    // Price component from Pyth oracle
    pub struct Price {
        pub price: i64,
        pub conf: u64,
        pub expo: i32,
        pub publish_time: i64,
    }

    // Offset of price in Pyth account data
    pub const PRICE_OFFSET: usize = size_of::<u32>() + // magic
                                    size_of::<u32>() + // version
                                    size_of::<u32>() + // type
                                    size_of::<u32>(); // size

    // Get price component from account data
    pub fn get_price(data: &[u8]) -> Option<Price> {
        if data.len() < PRICE_OFFSET + size_of::<Price>() {
            return None;
        }

        let price_bytes = &data[PRICE_OFFSET..PRICE_OFFSET + size_of::<Price>()];
        
        let price = i64::from_le_bytes(price_bytes[0..8].try_into().ok()?);
        let conf = u64::from_le_bytes(price_bytes[8..16].try_into().ok()?);
        let expo = i32::from_le_bytes(price_bytes[16..20].try_into().ok()?);
        let publish_time = i64::from_le_bytes(price_bytes[20..28].try_into().ok()?);
        
        Some(Price {
            price,
            conf,
            expo,
            publish_time,
        })
    }
}

// Scale factor for price normalization (6 decimals)
const PRICE_SCALE_FACTOR: u64 = 1_000_000;

// Asset price mapping
type AssetPriceMap = HashMap<Pubkey, u64>;

/// Get asset prices from oracle account
/// Supports Pyth, Switchboard, and custom oracles
pub fn get_asset_prices(oracle_account: &AccountInfo) -> Result<AssetPriceMap, ProgramError> {
    let mut prices = HashMap::new();
    
    // Detect the oracle provider by checking account owner
    // (In a real implementation, we would have a list of allowed oracle program IDs)
    
    // This is a simplified implementation - in a real protocol,
    // we would have multiple accounts with price feeds for different assets
    
    // For demonstration purposes only - actual implementation would:
    // 1. Parse the oracle data format properly
    // 2. Support multiple price feeds for different assets
    // 3. Validate the recency and confidence of the price data
    
    if is_pyth_oracle_account(oracle_account) {
        parse_pyth_oracle_data(oracle_account, &mut prices)?;
    } else if is_switchboard_oracle_account(oracle_account) {
        parse_switchboard_oracle_data(oracle_account, &mut prices)?;
    } else if is_chainlink_oracle_account(oracle_account) {
        parse_chainlink_oracle_data(oracle_account, &mut prices)?;
    } else {
        // Fallback to custom oracle data format
        parse_custom_oracle_data(oracle_account, &mut prices)?;
    }
    
    // Ensure we have at least some prices
    if prices.is_empty() {
        return Err(StakeLendError::OracleDataNotFound.into());
    }
    
    Ok(prices)
}

/// Get market risk metrics from oracle data
pub fn get_market_risk_metrics(
    market_risk_account: &AccountInfo
) -> Result<MarketRiskMetrics, ProgramError> {
    // In a real implementation, we would parse the market risk data
    // from a specialized oracle or risk assessment service
    
    // For this example, we'll use a simplified approach
    
    let data = market_risk_account.data.borrow();
    if data.len() < 24 {
        return Err(StakeLendError::InvalidAccountData.into());
    }
    
    // Parse volatility index (higher = more volatile)
    let volatility_index = u64::from_le_bytes(data[0..8].try_into().unwrap());
    
    // Parse market trend (-100 to 100, negative = bearish)
    let market_trend = i8::from_le_bytes(data[8..9].try_into().unwrap());
    
    // Parse liquidity score (0-10000, higher = more liquid)
    let liquidity_score = u16::from_le_bytes(data[9..11].try_into().unwrap());
    
    // Parse timestamp
    let timestamp = u64::from_le_bytes(data[11..19].try_into().unwrap());
    
    // Calculate risk level based on metrics
    let risk_level = if volatility_index > 8000 {
        RiskLevel::Extreme
    } else if volatility_index > 5000 {
        RiskLevel::High
    } else if volatility_index > 2000 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };
    
    let metrics = MarketRiskMetrics {
        volatility_index,
        market_trend,
        liquidity_score,
        risk_level,
        timestamp,
    };
    
    msg!("Market risk metrics: volatility={}, trend={}, risk_level={:?}", 
        volatility_index, market_trend, risk_level);
    
    Ok(metrics)
}

/// Get price of a specific asset by mint
pub fn get_asset_price(
    mint: &Pubkey,
    oracle_account: &AccountInfo,
) -> Result<PriceData, ProgramError> {
    // In a production environment, we would have a mapping of mints to 
    // specific oracle accounts, and would query the correct oracle for each asset
    
    let prices = get_asset_prices(oracle_account)?;
    let price = prices.get(mint).ok_or(StakeLendError::PriceNotFound)?;
    
    // For this simplified implementation, we're returning minimal metadata
    // In a real implementation, we would include confidence and timestamps
    let price_data = PriceData {
        price: *price,
        confidence: 0, // Would come from actual oracle data
        timestamp: crate::utils::math::get_current_timestamp(),
        source: OracleProvider::Pyth, // Default to Pyth in this example
    };
    
    Ok(price_data)
}

/// Verify that a price is within acceptable bounds
pub fn verify_price_validity(
    price_data: &PriceData,
    min_acceptable_confidence: u64,
    max_price_age_seconds: u64,
) -> Result<(), ProgramError> {
    let current_timestamp = crate::utils::math::get_current_timestamp();
    
    // Check price age
    if current_timestamp.saturating_sub(price_data.timestamp) > max_price_age_seconds {
        return Err(StakeLendError::StaleOracleData.into());
    }
    
    // Check confidence interval
    if price_data.confidence > min_acceptable_confidence {
        return Err(StakeLendError::LowPriceConfidence.into());
    }
    
    // Could add additional checks here like comparing to TWAPs
    // or checking against valid price ranges
    
    Ok(())
}

/// Helper function to check if an account is a Pyth oracle
fn is_pyth_oracle_account(account: &AccountInfo) -> bool {
    // In a real implementation, we would check against the Pyth program ID
    // and validate the account structure
    
    // This is just a placeholder implementation
    let data = account.data.borrow();
    data.len() >= 4 && data[0..4] == [0x50, 0x79, 0x74, 0x68] // "Pyth" in ASCII
}

/// Helper function to check if an account is a Switchboard oracle
fn is_switchboard_oracle_account(account: &AccountInfo) -> bool {
    // In a real implementation, we would check against the Switchboard program ID
    // and validate the account structure
    
    // This is just a placeholder implementation
    let data = account.data.borrow();
    data.len() >= 4 && data[0..4] == [0x53, 0x42, 0x44, 0x41] // "SBDA" in ASCII
}

/// Helper function to check if an account is a Chainlink oracle
fn is_chainlink_oracle_account(account: &AccountInfo) -> bool {
    // In a real implementation, we would check against the Chainlink program ID
    // and validate the account structure
    
    // This is just a placeholder implementation
    let data = account.data.borrow();
    data.len() >= 4 && data[0..4] == [0x43, 0x4C, 0x4E, 0x4B] // "CLNK" in ASCII
}

/// Parse Pyth oracle data to extract prices
fn parse_pyth_oracle_data(
    oracle_account: &AccountInfo,
    prices: &mut AssetPriceMap,
) -> Result<(), ProgramError> {
    let data = oracle_account.data.borrow();
    
    // In a real implementation, we would properly parse the Pyth price accounts
    // and extract the price, confidence, and publish time
    
    if let Some(price_data) = pyth_layout::get_price(&data) {
        // Normalize price based on exponent
        let normalized_price = normalize_pyth_price(price_data.price, price_data.expo)?;
        
        // In a real implementation, we would have a mapping of oracles to asset mints
        // For this example, we'll use a hardcoded mint for SOL
        let sol_mint = Pubkey::new_unique(); // Replace with actual SOL mint
        prices.insert(sol_mint, normalized_price);
        
        msg!("Parsed Pyth price: {}", normalized_price);
    } else {
        return Err(StakeLendError::OracleDataNotFound.into());
    }
    
    Ok(())
}

/// Parse Switchboard oracle data to extract prices
fn parse_switchboard_oracle_data(
    oracle_account: &AccountInfo,
    prices: &mut AssetPriceMap,
) -> Result<(), ProgramError> {
    // Simplified implementation - in reality, would parse Switchboard's
    // aggregator account structure
    
    // For this example, we'll just use mock data
    let sol_mint = Pubkey::new_unique(); // Replace with actual SOL mint
    let msol_mint = Pubkey::new_unique(); // Replace with actual mSOL mint
    
    // Mock prices (in reality, would be parsed from the oracle data)
    prices.insert(sol_mint, 100 * PRICE_SCALE_FACTOR); // $100 per SOL
    prices.insert(msol_mint, 105 * PRICE_SCALE_FACTOR); // $105 per mSOL
    
    msg!("Parsed Switchboard prices: SOL = {}, mSOL = {}", 
        prices[&sol_mint], prices[&msol_mint]);
    
    Ok(())
}

/// Parse Chainlink oracle data to extract prices
fn parse_chainlink_oracle_data(
    oracle_account: &AccountInfo,
    prices: &mut AssetPriceMap,
) -> Result<(), ProgramError> {
    // Simplified implementation - in reality, would parse Chainlink's
    // price feed account structure
    
    // For this example, we'll just use mock data
    let sol_mint = Pubkey::new_unique(); // Replace with actual SOL mint
    let jito_sol_mint = Pubkey::new_unique(); // Replace with actual jitoSOL mint
    
    // Mock prices (in reality, would be parsed from the oracle data)
    prices.insert(sol_mint, 100 * PRICE_SCALE_FACTOR); // $100 per SOL
    prices.insert(jito_sol_mint, 103 * PRICE_SCALE_FACTOR); // $103 per jitoSOL
    
    msg!("Parsed Chainlink prices: SOL = {}, jitoSOL = {}", 
        prices[&sol_mint], prices[&jito_sol_mint]);
    
    Ok(())
}

/// Parse custom oracle data to extract prices
fn parse_custom_oracle_data(
    oracle_account: &AccountInfo,
    prices: &mut AssetPriceMap,
) -> Result<(), ProgramError> {
    let data = oracle_account.data.borrow();
    
    // Expect at least header (8 bytes) + one price entry (32 + 8 bytes)
    if data.len() < 48 {
        return Err(StakeLendError::InvalidAccountData.into());
    }
    
    // Parse number of price entries
    let num_entries = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
    
    // Validate data length
    if data.len() < 8 + num_entries * 40 {
        return Err(StakeLendError::InvalidAccountData.into());
    }
    
    // Parse price entries
    let mut offset = 8;
    for _ in 0..num_entries {
        let mint_bytes: [u8; 32] = data[offset..offset+32].try_into().unwrap();
        let mint = Pubkey::new_from_array(mint_bytes);
        offset += 32;
        
        let price = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        prices.insert(mint, price);
    }
    
    msg!("Parsed {} custom price entries", num_entries);
    
    Ok(())
}

/// Normalize Pyth price based on exponent
fn normalize_pyth_price(price: i64, exponent: i32) -> Result<u64, ProgramError> {
    if price < 0 {
        return Err(StakeLendError::InvalidOraclePrice.into());
    }
    
    // Convert to our standard scale factor (6 decimals)
    let price_u64 = price as u64;
    
    if exponent < 0 {
        // Price has more decimals than our scale factor
        let scale_down = 10_u64.checked_pow(-exponent as u32)
            .ok_or(StakeLendError::MathOverflow)?;
        price_u64
            .checked_div(scale_down)
            .ok_or(StakeLendError::MathOverflow.into())
    } else {
        // Price has fewer decimals than our scale factor
        let scale_up = 10_u64.checked_pow(exponent as u32)
            .ok_or(StakeLendError::MathOverflow)?;
        price_u64
            .checked_mul(scale_up)
            .ok_or(StakeLendError::MathOverflow.into())
    }
}