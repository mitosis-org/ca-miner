use alloy::primitives::{Address, B256};
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::address::{check_address_match, get_deployed_address};
use crate::config::MinerConfig;
use crate::utils::to_bytes32;

#[derive(Debug)]
pub struct MinerResult {
    pub found: bool,
    pub raw_salt: Option<u64>,
    pub final_salt: Option<B256>,
    pub address: Option<Address>,
    pub checked: u64,
}

pub fn process_batch(
    config: &MinerConfig,
    start_salt: u64,
    batch_size: u64,
    found: &AtomicBool,
    use_random: bool,
) -> MinerResult {
    let mut rng = if use_random { Some(rand::rng()) } else { None };

    for i in 0..batch_size {
        if found.load(Ordering::Relaxed) {
            return MinerResult {
                found: false,
                raw_salt: None,
                final_salt: None,
                address: None,
                checked: i,
            };
        }

        let current_salt = if use_random {
            rng.as_mut().unwrap().random::<u64>()
        } else {
            start_salt + i
        };

        let raw_salt_bytes = to_bytes32(current_salt);
        let final_salt = config.compute_final_salt(&raw_salt_bytes);
        let deployed_addr = get_deployed_address(config, &final_salt);

        if check_address_match(&deployed_addr, config) {
            found.store(true, Ordering::Relaxed);
            return MinerResult {
                found: true,
                raw_salt: Some(current_salt),
                final_salt: Some(final_salt),
                address: Some(deployed_addr),
                checked: i + 1,
            };
        }
    }

    MinerResult {
        found: false,
        raw_salt: None,
        final_salt: None,
        address: None,
        checked: batch_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MinerConfig, MiningMode};
    use alloy::primitives::{address, b256};

    fn create_test_config() -> MinerConfig {
        MinerConfig {
            factory_address: address!("742d35cc6bf8632ebc4532fb6d8b2946fbbb85c8"),
            url_or_bytecode_bytes: b256!(
                "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            )
            .to_vec(),
            prefix_bytes: b"00".to_vec(), // Very permissive pattern for testing
            prefix_len: 2,
            postfix_bytes: Vec::new(),
            postfix_len: 0,
            mode: MiningMode::Create2,
            case_sensitive: false,
            postfix_only: false,
            dual_matching: false,
        }
    }

    fn create_impossible_config() -> MinerConfig {
        MinerConfig {
            factory_address: address!("742d35cc6bf8632ebc4532fb6d8b2946fbbb85c8"),
            url_or_bytecode_bytes: b256!(
                "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            )
            .to_vec(),
            prefix_bytes: b"ffffffffffffffffffffffffffffffffffffffff".to_vec(), // Impossible pattern
            prefix_len: 40, // Full address match - extremely unlikely
            postfix_bytes: Vec::new(),
            postfix_len: 0,
            mode: MiningMode::Create2,
            case_sensitive: false,
            postfix_only: false,
            dual_matching: false,
        }
    }

    #[test]
    fn test_process_batch_sequential() {
        let config = create_test_config();
        let found = AtomicBool::new(false);

        let result = process_batch(&config, 0, 10, &found, false);

        // Should check all 10 salts if no match found
        assert_eq!(result.checked, 10);

        // Result should be deterministic - check consistency of result fields
        if result.found {
            assert!(result.raw_salt.is_some());
            assert!(result.final_salt.is_some());
            assert!(result.address.is_some());
        } else {
            assert!(result.raw_salt.is_none());
            assert!(result.final_salt.is_none());
            assert!(result.address.is_none());
        }
    }

    #[test]
    fn test_process_batch_random() {
        let config = create_test_config();
        let found = AtomicBool::new(false);

        let result = process_batch(&config, 0, 10, &found, true);

        // Should check all 10 salts if no match found
        assert_eq!(result.checked, 10);

        // For random mode, we can't predict the salts used
        if result.found {
            assert!(result.raw_salt.is_some());
            assert!(result.final_salt.is_some());
            assert!(result.address.is_some());
        }
    }

    #[test]
    fn test_process_batch_early_termination() {
        let config = create_impossible_config(); // Use impossible config to avoid accidental matches
        let found = AtomicBool::new(true); // Start with found = true to test early termination

        let result = process_batch(&config, 0, 1000, &found, false);

        // Should terminate immediately when found is already true
        assert!(!result.found); // The result itself should be false since we didn't find a match
        assert_eq!(result.checked, 0); // Should check 0 items since found was already true
        assert!(result.raw_salt.is_none());
        assert!(result.final_salt.is_none());
        assert!(result.address.is_none());
    }

    #[test]
    fn test_process_batch_impossible_pattern() {
        let config = create_impossible_config();
        let found = AtomicBool::new(false);

        let result = process_batch(&config, 0, 5, &found, false);

        // Should not find anything with impossible pattern
        assert!(!result.found);
        assert_eq!(result.checked, 5);
        assert!(result.raw_salt.is_none());
        assert!(result.final_salt.is_none());
        assert!(result.address.is_none());
    }

    #[test]
    fn test_miner_result_debug() {
        let result = MinerResult {
            found: true,
            raw_salt: Some(123),
            final_salt: Some(B256::ZERO),
            address: Some(Address::ZERO),
            checked: 1,
        };

        // Test that MinerResult implements Debug
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("found: true"));
        assert!(debug_str.contains("checked: 1"));
    }

    #[test]
    fn test_process_batch_deterministic_sequential() {
        let config = create_test_config();
        let found1 = AtomicBool::new(false);
        let found2 = AtomicBool::new(false);

        // Run the same batch twice
        let result1 = process_batch(&config, 100, 10, &found1, false);
        let result2 = process_batch(&config, 100, 10, &found2, false);

        // Results should be identical for sequential mode with same parameters
        assert_eq!(result1.found, result2.found);
        assert_eq!(result1.checked, result2.checked);
        if result1.found && result2.found {
            assert_eq!(result1.raw_salt, result2.raw_salt);
            assert_eq!(result1.final_salt, result2.final_salt);
            assert_eq!(result1.address, result2.address);
        }
    }
}
