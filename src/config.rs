use alloy::primitives::{keccak256, Address, B256};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "miner")]
#[command(about = "High-performance CREATE2/CREATE3 salt miner")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Mine CREATE2 addresses using bytecode hash
    Create2(Create2Args),
    /// Mine CREATE3 addresses using URL string
    Create3(Create3Args),
}

#[derive(Parser)]
pub struct Create2Args {
    /// Factory contract address
    pub factory: String,

    /// Bytecode hash (32 bytes hex, starting with 0x)
    pub bytecode_hash: String,

    /// Desired address prefix (hex)
    pub prefix: String,

    /// Starting salt value
    #[arg(long, default_value = "0")]
    pub start_salt: u64,

    /// Maximum iterations
    #[arg(long, default_value = "10000000000")]
    pub max_iterations: u64,

    /// Batch size for processing
    #[arg(long, default_value = "100000")]
    pub batch_size: u64,

    /// Use random salts instead of sequential
    #[arg(long)]
    pub random: bool,

    /// Use case-sensitive matching with Ethereum checksum addresses (EIP-55)
    #[arg(long)]
    pub case_sensitive: bool,

    /// Match postfix/suffix instead of prefix
    #[arg(long)]
    pub postfix: bool,

    /// Postfix pattern for dual prefix+postfix matching (hex)
    #[arg(long)]
    pub postfix_pattern: Option<String>,
}

#[derive(Parser)]
pub struct Create3Args {
    /// Factory contract address
    pub factory: String,

    /// URL string for salt computation
    pub url: String,

    /// Desired address prefix (hex)
    pub prefix: String,

    /// Starting salt value
    #[arg(long, default_value = "0")]
    pub start_salt: u64,

    /// Maximum iterations
    #[arg(long, default_value = "10000000000")]
    pub max_iterations: u64,

    /// Batch size for processing
    #[arg(long, default_value = "100000")]
    pub batch_size: u64,

    /// Use random salts instead of sequential
    #[arg(long)]
    pub random: bool,

    /// Use case-sensitive matching with Ethereum checksum addresses (EIP-55)
    #[arg(long)]
    pub case_sensitive: bool,

    /// Match postfix/suffix instead of prefix
    #[arg(long)]
    pub postfix: bool,

    /// Postfix pattern for dual prefix+postfix matching (hex)
    #[arg(long)]
    pub postfix_pattern: Option<String>,
}

// Common arguments extraction trait
pub trait CommonArgs: Sync {
    fn factory(&self) -> &str;
    fn prefix(&self) -> &str;
    fn start_salt(&self) -> u64;
    fn max_iterations(&self) -> u64;
    fn batch_size(&self) -> u64;
    fn random(&self) -> bool;
    fn case_sensitive(&self) -> bool;
    fn postfix(&self) -> bool;
    fn postfix_pattern(&self) -> &Option<String>;
}

impl CommonArgs for Create2Args {
    fn factory(&self) -> &str {
        &self.factory
    }
    fn prefix(&self) -> &str {
        &self.prefix
    }
    fn start_salt(&self) -> u64 {
        self.start_salt
    }
    fn max_iterations(&self) -> u64 {
        self.max_iterations
    }
    fn batch_size(&self) -> u64 {
        self.batch_size
    }
    fn random(&self) -> bool {
        self.random
    }
    fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }
    fn postfix(&self) -> bool {
        self.postfix
    }
    fn postfix_pattern(&self) -> &Option<String> {
        &self.postfix_pattern
    }
}

impl CommonArgs for Create3Args {
    fn factory(&self) -> &str {
        &self.factory
    }
    fn prefix(&self) -> &str {
        &self.prefix
    }
    fn start_salt(&self) -> u64 {
        self.start_salt
    }
    fn max_iterations(&self) -> u64 {
        self.max_iterations
    }
    fn batch_size(&self) -> u64 {
        self.batch_size
    }
    fn random(&self) -> bool {
        self.random
    }
    fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }
    fn postfix(&self) -> bool {
        self.postfix
    }
    fn postfix_pattern(&self) -> &Option<String> {
        &self.postfix_pattern
    }
}

#[derive(Clone)]
pub struct MinerConfig {
    pub factory_address: Address,
    pub url_or_bytecode_bytes: Vec<u8>,
    pub prefix_bytes: Vec<u8>,
    pub prefix_len: usize,
    pub postfix_bytes: Vec<u8>,
    pub postfix_len: usize,
    pub mode: MiningMode,
    pub case_sensitive: bool,
    pub postfix_only: bool, // true when using --postfix flag (legacy behavior)
    pub dual_matching: bool, // true when using both prefix and postfix patterns
}

#[derive(Clone, Debug)]
pub enum MiningMode {
    Create2,
    Create3,
}

impl MinerConfig {
    pub fn compute_final_salt(&self, salt: &B256) -> B256 {
        match self.mode {
            MiningMode::Create2 => {
                // For CREATE2, salt is used directly
                *salt
            }
            MiningMode::Create3 => {
                // For CREATE3, salt is combined with URL
                let mut packed = Vec::with_capacity(self.url_or_bytecode_bytes.len() + 32);
                packed.extend_from_slice(&self.url_or_bytecode_bytes);
                packed.extend_from_slice(salt.as_slice());
                keccak256(&packed)
            }
        }
    }
}
