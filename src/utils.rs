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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, b256};

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(12345), "12,345");
        assert_eq!(format_number(123456), "123,456");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(12345678901), "12,345,678,901");
    }

    #[test]
    fn test_parse_address_valid() {
        let addr_str = "0x742d35cc6bf8632ebc4532fb6d8b2946fbbb85c8";
        let result = parse_address(addr_str);
        assert!(result.is_ok());

        let expected = address!("742d35cc6bf8632ebc4532fb6d8b2946fbbb85c8");
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_parse_address_invalid() {
        let invalid_addresses = [
            "0xinvalid",
            "not_an_address",
            "0x742d35cc6bf8632ebc4532fb6d8b2946fbbb85c", // too short
            "0x742d35cc6bf8632ebc4532fb6d8b2946fbbb85c899", // too long
        ];

        for addr in invalid_addresses {
            let result = parse_address(addr);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_bytes32_valid() {
        let hex_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = parse_bytes32(hex_str);
        assert!(result.is_ok());

        let expected = b256!("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_parse_bytes32_invalid() {
        let invalid_bytes = [
            "0xinvalid",
            "not_bytes32",
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd", // too short
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdefff", // too long
        ];

        for bytes in invalid_bytes {
            let result = parse_bytes32(bytes);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_to_bytes32() {
        // Test with zero
        let result = to_bytes32(0);
        let expected = B256::ZERO;
        assert_eq!(result, expected);

        // Test with small number
        let result = to_bytes32(123);
        let mut expected_bytes = [0u8; 32];
        expected_bytes[24..32].copy_from_slice(&123u64.to_be_bytes());
        let expected = B256::from(expected_bytes);
        assert_eq!(result, expected);

        // Test with max u64
        let result = to_bytes32(u64::MAX);
        let mut expected_bytes = [0u8; 32];
        expected_bytes[24..32].copy_from_slice(&u64::MAX.to_be_bytes());
        let expected = B256::from(expected_bytes);
        assert_eq!(result, expected);
    }
}
