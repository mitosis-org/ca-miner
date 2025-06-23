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
