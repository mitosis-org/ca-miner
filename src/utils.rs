use alloy::primitives::{Address, B256};
use anyhow::Result;

pub fn parse_address(addr_str: &str) -> Result<Address> {
    addr_str
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))
}

pub fn parse_bytes32(hex_str: &str) -> Result<B256> {
    hex_str
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid bytes32: {}", e))
}

pub fn to_bytes32(val: u64) -> B256 {
    let mut bytes = [0u8; 32];
    bytes[24..32].copy_from_slice(&val.to_be_bytes());
    B256::from(bytes)
}

pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

// Legacy function for backward compatibility - now delegates to MinerConfig
pub fn compute_final_salt(config: &crate::config::MinerConfig, salt: &B256) -> B256 {
    config.compute_final_salt(salt)
}
