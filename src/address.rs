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
