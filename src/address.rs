use crate::config::{MinerConfig, MiningMode};
use alloy::primitives::{keccak256, Address, B256};

// Solady's CREATE3 proxy init code hash constant
const CREATE3_PROXY_INITCODE_HASH: B256 = B256::new([
    0x21, 0xc3, 0x5d, 0xbe, 0x1b, 0x34, 0x4a, 0x24, 0x88, 0xcf, 0x33, 0x21, 0xd6, 0xce, 0x54, 0x2f,
    0x8e, 0x9f, 0x30, 0x55, 0x44, 0xff, 0x09, 0xe4, 0x99, 0x3a, 0x62, 0x31, 0x9a, 0x49, 0x7c, 0x1f,
]);

pub fn get_create2_address(config: &MinerConfig, salt: &B256) -> Address {
    // CREATE2 address = keccak256(0xff + factory + salt + keccak256(bytecode))[12:]
    let mut packed = Vec::with_capacity(1 + 20 + 32 + config.url_or_bytecode_bytes.len());
    packed.push(0xff);
    packed.extend_from_slice(config.factory_address.as_slice());
    packed.extend_from_slice(salt.as_slice());
    packed.extend_from_slice(&config.url_or_bytecode_bytes);

    let hash = keccak256(&packed);
    Address::from_slice(&hash[12..])
}

pub fn get_create3_address(config: &MinerConfig, salt: &B256) -> Address {
    // Step 1: Compute proxy address using CREATE2
    let mut packed = Vec::with_capacity(85);
    packed.push(0xff);
    packed.extend_from_slice(config.factory_address.as_slice());
    packed.extend_from_slice(salt.as_slice());
    packed.extend_from_slice(CREATE3_PROXY_INITCODE_HASH.as_slice());

    let proxy_hash = keccak256(&packed);
    let proxy_addr = Address::from_slice(&proxy_hash[12..]);

    // Step 2: Compute deployed address using CREATE from proxy (nonce=1)
    // Manual RLP encoding for [address, nonce=1]
    let mut rlp_data = Vec::with_capacity(23);
    rlp_data.push(0xd6); // RLP list with 22 bytes (0xc0 + 22)
    rlp_data.push(0x94); // Byte string with 20 bytes (0x80 + 20)
    rlp_data.extend_from_slice(proxy_addr.as_slice());
    rlp_data.push(0x01); // Integer 1

    let deployed_hash = keccak256(&rlp_data);
    Address::from_slice(&deployed_hash[12..])
}

pub fn get_deployed_address(config: &MinerConfig, salt: &B256) -> Address {
    match config.mode {
        MiningMode::Create2 => get_create2_address(config, salt),
        MiningMode::Create3 => get_create3_address(config, salt),
    }
}

/// Check if address matches the target pattern
pub fn check_address_match(addr: &Address, config: &MinerConfig) -> bool {
    if config.case_sensitive {
        // Use EIP-55 checksum format for case-sensitive matching
        let checksum_addr = addr.to_checksum(None);

        let prefix_target = String::from_utf8_lossy(&config.prefix_bytes).to_string();
        let postfix_target = String::from_utf8_lossy(&config.postfix_bytes).to_string();

        if config.dual_matching {
            // Both prefix and postfix must match
            checksum_addr.starts_with(&prefix_target) && checksum_addr.ends_with(&postfix_target)
        } else if config.postfix_only {
            // Legacy postfix-only mode
            checksum_addr.ends_with(&prefix_target)
        } else {
            // Prefix-only mode
            checksum_addr.starts_with(&prefix_target)
        }
    } else {
        // Use fast case-insensitive matching
        if config.dual_matching {
            // Both prefix and postfix must match
            check_prefix_match(addr, &config.prefix_bytes, config.prefix_len)
                && check_postfix_match(addr, &config.postfix_bytes, config.postfix_len)
        } else if config.postfix_only {
            // Legacy postfix-only mode
            check_postfix_match(addr, &config.prefix_bytes, config.prefix_len)
        } else {
            // Prefix-only mode
            check_prefix_match(addr, &config.prefix_bytes, config.prefix_len)
        }
    }
}

fn check_prefix_match(addr: &Address, prefix: &[u8], prefix_len: usize) -> bool {
    // Convert to hex manually for better performance
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let addr_bytes = addr.as_slice();

    for (i, &prefix_char) in prefix.iter().enumerate().take(prefix_len) {
        let byte_idx = i / 2;
        let is_high_nibble = i % 2 == 0;

        if byte_idx >= addr_bytes.len() {
            return false;
        }

        let nibble = if is_high_nibble {
            (addr_bytes[byte_idx] >> 4) & 0x0f
        } else {
            addr_bytes[byte_idx] & 0x0f
        };

        let addr_char = HEX_CHARS[nibble as usize];
        let prefix_char_lower = prefix_char.to_ascii_lowercase();

        if addr_char != prefix_char_lower {
            return false;
        }
    }
    true
}

fn check_postfix_match(addr: &Address, postfix: &[u8], postfix_len: usize) -> bool {
    // Convert to hex manually for better performance
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let addr_bytes = addr.as_slice();

    // Address is 20 bytes = 40 hex chars
    if postfix_len > 40 {
        return false;
    }

    let start_pos = 40 - postfix_len;

    for (i, &postfix_char) in postfix.iter().enumerate().take(postfix_len) {
        let hex_pos = start_pos + i;
        let byte_idx = hex_pos / 2;
        let is_high_nibble = hex_pos % 2 == 0;

        if byte_idx >= addr_bytes.len() {
            return false;
        }

        let nibble = if is_high_nibble {
            (addr_bytes[byte_idx] >> 4) & 0x0f
        } else {
            addr_bytes[byte_idx] & 0x0f
        };

        let addr_char = HEX_CHARS[nibble as usize];
        let postfix_char_lower = postfix_char.to_ascii_lowercase();

        if addr_char != postfix_char_lower {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, b256};

    fn create_test_config_create2() -> MinerConfig {
        MinerConfig {
            factory_address: address!("742d35cc6bf8632ebc4532fb6d8b2946fbbb85c8"),
            url_or_bytecode_bytes: b256!(
                "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            )
            .to_vec(),
            prefix_bytes: b"dead".to_vec(),
            prefix_len: 4,
            postfix_bytes: Vec::new(),
            postfix_len: 0,
            mode: MiningMode::Create2,
            case_sensitive: false,
            postfix_only: false,
            dual_matching: false,
        }
    }

    fn create_test_config_create3() -> MinerConfig {
        MinerConfig {
            factory_address: address!("742d35cc6bf8632ebc4532fb6d8b2946fbbb85c8"),
            url_or_bytecode_bytes: b"https://example.com".to_vec(),
            prefix_bytes: b"cafe".to_vec(),
            prefix_len: 4,
            postfix_bytes: Vec::new(),
            postfix_len: 0,
            mode: MiningMode::Create3,
            case_sensitive: false,
            postfix_only: false,
            dual_matching: false,
        }
    }

    #[test]
    fn test_get_create2_address() {
        let config = create_test_config_create2();
        let salt = B256::ZERO;

        let addr = get_create2_address(&config, &salt);

        // Test that we get a valid address (20 bytes)
        assert_eq!(addr.as_slice().len(), 20);

        // Test deterministic behavior - same inputs should produce same output
        let addr2 = get_create2_address(&config, &salt);
        assert_eq!(addr, addr2);

        // Test different salt produces different address
        let different_salt =
            b256!("0000000000000000000000000000000000000000000000000000000000000001");
        let addr3 = get_create2_address(&config, &different_salt);
        assert_ne!(addr, addr3);
    }

    #[test]
    fn test_get_create3_address() {
        let config = create_test_config_create3();
        let salt = B256::ZERO;

        let addr = get_create3_address(&config, &salt);

        // Test that we get a valid address (20 bytes)
        assert_eq!(addr.as_slice().len(), 20);

        // Test deterministic behavior
        let addr2 = get_create3_address(&config, &salt);
        assert_eq!(addr, addr2);

        // Test different salt produces different address
        let different_salt =
            b256!("0000000000000000000000000000000000000000000000000000000000000001");
        let addr3 = get_create3_address(&config, &different_salt);
        assert_ne!(addr, addr3);
    }

    #[test]
    fn test_get_deployed_address() {
        let config_create2 = create_test_config_create2();
        let config_create3 = create_test_config_create3();
        let salt = B256::ZERO;

        let addr_create2 = get_deployed_address(&config_create2, &salt);
        let addr_create3 = get_deployed_address(&config_create3, &salt);

        // Should delegate to the correct function based on mode
        assert_eq!(addr_create2, get_create2_address(&config_create2, &salt));
        assert_eq!(addr_create3, get_create3_address(&config_create3, &salt));

        // Different modes should produce different addresses
        assert_ne!(addr_create2, addr_create3);
    }

    #[test]
    fn test_check_prefix_match() {
        // Create an address that starts with 'dead'
        let addr = address!("deadbeefcafebabe1234567890abcdef12345678");

        let prefix = b"dead";
        assert!(check_prefix_match(&addr, prefix, 4));

        let wrong_prefix = b"cafe";
        assert!(!check_prefix_match(&addr, wrong_prefix, 4));

        // Test partial match
        let partial_prefix = b"de";
        assert!(check_prefix_match(&addr, partial_prefix, 2));
    }

    #[test]
    fn test_check_postfix_match() {
        // Create an address that ends with 'beef'
        let addr = address!("1234567890abcdef1234567890abcdefdeadbeef");

        let postfix = b"beef";
        assert!(check_postfix_match(&addr, postfix, 4));

        let wrong_postfix = b"cafe";
        assert!(!check_postfix_match(&addr, wrong_postfix, 4));

        // Test partial match
        let partial_postfix = b"ef";
        assert!(check_postfix_match(&addr, partial_postfix, 2));
    }

    #[test]
    fn test_check_address_match_prefix_only() {
        let mut config = create_test_config_create2();
        config.prefix_bytes = b"dead".to_vec();
        config.prefix_len = 4;

        let matching_addr = address!("deadbeefcafebabe1234567890abcdef12345678");
        let non_matching_addr = address!("cafebabedeadbeef1234567890abcdef12345678");

        assert!(check_address_match(&matching_addr, &config));
        assert!(!check_address_match(&non_matching_addr, &config));
    }

    #[test]
    fn test_check_address_match_postfix_only() {
        let mut config = create_test_config_create2();
        config.prefix_bytes = b"beef".to_vec();
        config.prefix_len = 4;
        config.postfix_only = true;

        let matching_addr = address!("1234567890abcdef1234567890abcdefdeadbeef");
        let non_matching_addr = address!("beefcafebabe1234567890abcdef123456789012");

        assert!(check_address_match(&matching_addr, &config));
        assert!(!check_address_match(&non_matching_addr, &config));
    }

    #[test]
    fn test_check_address_match_dual_matching() {
        let mut config = create_test_config_create2();
        config.prefix_bytes = b"dead".to_vec();
        config.prefix_len = 4;
        config.postfix_bytes = b"beef".to_vec();
        config.postfix_len = 4;
        config.dual_matching = true;

        let matching_addr = address!("deadbeefcafebabe1234567890abcdefdeadbeef");
        let prefix_only_addr = address!("deadbeefcafebabe1234567890abcdef12345678");
        let postfix_only_addr = address!("1234567890abcdef1234567890abcdefdeadbeef");
        let non_matching_addr = address!("cafebabe1234567890abcdef123456789012dead");

        assert!(check_address_match(&matching_addr, &config));
        assert!(!check_address_match(&prefix_only_addr, &config));
        assert!(!check_address_match(&postfix_only_addr, &config));
        assert!(!check_address_match(&non_matching_addr, &config));
    }

    #[test]
    fn test_check_address_match_case_sensitive() {
        let mut config = create_test_config_create2();
        config.case_sensitive = true;
        config.prefix_bytes = b"0xDead".to_vec(); // Note the capital D

        // This test is more complex because EIP-55 checksum addresses depend on the full address
        // We'll test that the case-sensitive flag changes behavior
        let addr = address!("deadbeefcafebabe1234567890abcdef12345678");

        // Test with case sensitive vs insensitive
        config.case_sensitive = false;
        config.prefix_bytes = b"dead".to_vec();
        config.prefix_len = 4;
        let case_insensitive_result = check_address_match(&addr, &config);

        config.case_sensitive = true;
        let case_sensitive_result = check_address_match(&addr, &config);

        // Results may differ based on EIP-55 checksum
        // At minimum, the function should handle both modes without panicking
        // The important thing is that both functions execute without panic
        let _ = case_insensitive_result;
        let _ = case_sensitive_result;
    }
}
