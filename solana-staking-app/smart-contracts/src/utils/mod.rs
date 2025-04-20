pub mod math;
pub mod oracle;
pub mod validation;

// Re-export commonly used utility functions for easier access
pub use math::{
    calculate_fee,
    calculate_health_factor,
    calculate_token_amount_from_shares,
    calculate_shares_from_token_amount,
    calculate_utilization_rate,
    calculate_borrow_rate,
    calculate_supply_rate,
    get_current_timestamp,
};

pub use oracle::{
    get_asset_prices,
    get_market_risk_metrics,
};

pub use validation::{
    validate_user_accounts,
    validate_pool_accounts,
    validate_token_accounts,
    check_account_owner,
    check_signer,
    check_token_mint,
};

/// Utility function to log transaction details
pub fn log_tx_info(
    operation: &str,
    user: &str,
    amount: Option<u64>,
    pool_id: Option<&str>,
) {
    use solana_program::msg;
    
    let amount_str = amount.map_or("N/A".to_string(), |a| a.to_string());
    let pool_str = pool_id.unwrap_or("N/A");
    
    msg!(
        "Operation: {}, User: {}, Amount: {}, Pool: {}",
        operation,
        user,
        amount_str,
        pool_str
    );
}

/// Convert a string to a fixed-length byte array
pub fn string_to_bytes<const N: usize>(s: &str) -> [u8; N] {
    let mut buffer = [0u8; N];
    let bytes = s.as_bytes();
    let len = std::cmp::min(bytes.len(), N);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer
}

/// Calculate APY from base reward rate
pub fn calculate_display_apy(reward_rate: u64, boost_bps: u16) -> f64 {
    // Convert from internal representation to percentage
    let base_rate = reward_rate as f64 / 10000.0;  // Assuming reward rate is in basis points
    let boost_factor = 1.0 + (boost_bps as f64 / 10000.0);
    
    (base_rate * boost_factor) * 100.0  // Convert to percentage
}

/// Format a public key as a shortened string for display
pub fn format_pubkey_short(pubkey: &solana_program::pubkey::Pubkey) -> String {
    let pk_str = pubkey.to_string();
    format!("{}...{}", &pk_str[0..4], &pk_str[pk_str.len() - 4..])
}