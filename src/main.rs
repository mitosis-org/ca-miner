use anyhow::Result;
use clap::Parser;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use ca_miner::{
    format_number, parse_address, parse_bytes32, process_batch, Args, Commands, CommonArgs,
    Create2Args, Create3Args, Logger, MinerConfig, MinerResult, MiningMode,
};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Create2(create2_args) => {
            let config = build_create2_config(&create2_args)?;
            print_startup_info_create2(&create2_args, &config);
            run_mining(&create2_args, config)
        }
        Commands::Create3(create3_args) => {
            let config = build_create3_config(&create3_args)?;
            print_startup_info_create3(&create3_args, &config);
            run_mining(&create3_args, config)
        }
    }
}

fn build_create2_config(args: &Create2Args) -> Result<MinerConfig> {
    let factory_address = parse_address(args.factory())?;

    // For CREATE2, expect bytecode hash (32 bytes)
    let url_or_bytecode_bytes = if args.bytecode_hash.starts_with("0x") {
        parse_bytes32(&args.bytecode_hash)?.to_vec()
    } else {
        anyhow::bail!("For CREATE2 mode, provide bytecode hash as hex (0x...)")
    };

    // Validate argument combinations
    if args.postfix() && args.postfix_pattern().is_some() {
        anyhow::bail!(
            "Cannot use both --postfix and --postfix-pattern flags. Use --postfix-pattern for dual matching."
        );
    }

    let (prefix_bytes, prefix_len) = process_pattern_bytes(args.prefix(), args.case_sensitive());
    let (postfix_bytes, postfix_len) = if let Some(postfix_pattern) = args.postfix_pattern() {
        process_pattern_bytes(postfix_pattern, args.case_sensitive())
    } else {
        (Vec::new(), 0)
    };

    let postfix_only = args.postfix() && args.postfix_pattern().is_none();
    let dual_matching = args.postfix_pattern().is_some();

    Ok(MinerConfig {
        factory_address,
        url_or_bytecode_bytes,
        prefix_bytes,
        prefix_len,
        postfix_bytes,
        postfix_len,
        mode: MiningMode::Create2,
        case_sensitive: args.case_sensitive(),
        postfix_only,
        dual_matching,
    })
}

fn build_create3_config(args: &Create3Args) -> Result<MinerConfig> {
    let factory_address = parse_address(args.factory())?;

    // For CREATE3, use URL as bytes
    let url_or_bytecode_bytes = args.url.as_bytes().to_vec();

    // Validate argument combinations
    if args.postfix() && args.postfix_pattern().is_some() {
        anyhow::bail!(
            "Cannot use both --postfix and --postfix-pattern flags. Use --postfix-pattern for dual matching."
        );
    }

    let (prefix_bytes, prefix_len) = process_pattern_bytes(args.prefix(), args.case_sensitive());
    let (postfix_bytes, postfix_len) = if let Some(postfix_pattern) = args.postfix_pattern() {
        process_pattern_bytes(postfix_pattern, args.case_sensitive())
    } else {
        (Vec::new(), 0)
    };

    let postfix_only = args.postfix() && args.postfix_pattern().is_none();
    let dual_matching = args.postfix_pattern().is_some();

    Ok(MinerConfig {
        factory_address,
        url_or_bytecode_bytes,
        prefix_bytes,
        prefix_len,
        postfix_bytes,
        postfix_len,
        mode: MiningMode::Create3,
        case_sensitive: args.case_sensitive(),
        postfix_only,
        dual_matching,
    })
}

fn process_pattern_bytes(pattern: &str, case_sensitive: bool) -> (Vec<u8>, usize) {
    if case_sensitive {
        // For case-sensitive mode, preserve original case
        let pattern = pattern.strip_prefix("0x").unwrap_or(pattern);
        (pattern.as_bytes().to_vec(), pattern.len())
    } else {
        // For case-insensitive mode, convert to lowercase
        let pattern = pattern.strip_prefix("0x").unwrap_or(pattern).to_lowercase();
        (pattern.as_bytes().to_vec(), pattern.len())
    }
}

fn print_startup_info_create2(args: &Create2Args, config: &MinerConfig) {
    Logger::header("High-Performance CREATE2 Salt Miner");
    Logger::info("Mode", "CREATE2");
    Logger::info("Factory", args.factory());
    Logger::info("Bytecode Hash", &args.bytecode_hash);
    print_common_startup_info(args, config);
}

fn print_startup_info_create3(args: &Create3Args, config: &MinerConfig) {
    Logger::header("High-Performance CREATE3 Salt Miner");
    Logger::info("Mode", "CREATE3");
    Logger::info("Factory", args.factory());
    Logger::info("URL", &args.url);
    print_common_startup_info(args, config);
}

fn print_common_startup_info<T: CommonArgs>(args: &T, config: &MinerConfig) {
    // Display pattern information
    let case_mode = if args.case_sensitive() {
        "case-sensitive (EIP-55)"
    } else {
        "case-insensitive"
    };

    let display_prefix = get_display_pattern(args.prefix(), args.case_sensitive());
    let display_postfix = args
        .postfix_pattern()
        .as_ref()
        .map(|p| get_display_pattern(p, args.case_sensitive()));

    if config.dual_matching {
        Logger::info("Prefix", &format!("0x{} ({})", display_prefix, case_mode));
        Logger::info(
            "Postfix",
            &format!("0x{} ({})", display_postfix.unwrap(), case_mode),
        );
        Logger::info("Mode", "Dual matching (both prefix AND postfix must match)");
    } else if config.postfix_only {
        Logger::info("Postfix", &format!("0x{} ({})", display_prefix, case_mode));
    } else {
        Logger::info("Prefix", &format!("0x{} ({})", display_prefix, case_mode));
    }

    if args.random() {
        Logger::info("Salt Mode", "Random generation");
    } else {
        Logger::info("Starting Salt", &args.start_salt().to_string());
    }
    Logger::info("Max Iterations", &format_number(args.max_iterations()));
    Logger::info("Batch Size", &format_number(args.batch_size()));
    Logger::info("CPU Cores", &rayon::current_num_threads().to_string());

    Logger::separator();
}

fn get_display_pattern(pattern: &str, case_sensitive: bool) -> String {
    let clean_pattern = pattern.strip_prefix("0x").unwrap_or(pattern);
    if case_sensitive {
        clean_pattern.to_string()
    } else {
        clean_pattern.to_lowercase()
    }
}

fn run_mining<T: CommonArgs>(args: &T, config: MinerConfig) -> Result<()> {
    let start_time = Instant::now();
    let found = Arc::new(AtomicBool::new(false));
    let total_checked = Arc::new(AtomicU64::new(0));

    // Create batches for parallel processing
    let batches: Vec<u64> = (args.start_salt()..args.start_salt() + args.max_iterations())
        .step_by(args.batch_size() as usize)
        .collect();

    Logger::info("Processing Batches", &format_number(batches.len() as u64));
    Logger::mining_start();

    // Create progress bar
    let progress_bar = Logger::create_progress_bar("Mining salts...");

    // Start live status reporter thread
    let found_clone = Arc::clone(&found);
    let total_checked_clone = Arc::clone(&total_checked);
    let pb_clone = progress_bar.clone();
    let status_handle = thread::spawn(move || {
        run_status_reporter(found_clone, total_checked_clone, start_time, pb_clone);
    });

    // Process batches in parallel using Rayon
    let result = batches
        .par_iter()
        .map(|&batch_start| {
            let batch_size = std::cmp::min(
                args.batch_size(),
                args.start_salt() + args.max_iterations() - batch_start,
            );
            let result = process_batch(&config, batch_start, batch_size, &found, args.random());
            total_checked.fetch_add(result.checked, Ordering::Relaxed);
            result
        })
        .find_any(|result| result.found);

    let elapsed = start_time.elapsed();

    // Signal the status thread to stop
    found.store(true, Ordering::Relaxed);

    // Stop the status reporter
    status_handle.join().unwrap();
    progress_bar.finish_and_clear();

    display_results(result, elapsed, &total_checked, &config);

    Ok(())
}

fn run_status_reporter(
    found: Arc<AtomicBool>,
    total_checked: Arc<AtomicU64>,
    start_time: Instant,
    pb: ProgressBar,
) {
    let mut last_checked = 0u64;
    let mut last_time = Instant::now();

    while !found.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));

        let current_checked = total_checked.load(Ordering::Relaxed);
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(last_time).as_secs_f64();

        if elapsed > 0.0 {
            let delta = current_checked - last_checked;
            let rate = delta as f64 / elapsed;
            let total_elapsed = current_time.duration_since(start_time).as_secs_f64();
            let avg_rate = current_checked as f64 / total_elapsed;

            pb.set_message(format!(
                "Checked: {} | Current: {:.0} s/sec | Avg: {:.0} s/sec",
                format_number(current_checked),
                rate,
                avg_rate
            ));

            last_checked = current_checked;
            last_time = current_time;
        }
    }
}

fn display_results(
    result: Option<MinerResult>,
    elapsed: Duration,
    total_checked: &Arc<AtomicU64>,
    config: &MinerConfig,
) {
    match result {
        Some(result) if result.found => {
            let raw_salt = result.raw_salt.unwrap();
            let final_salt = result.final_salt.unwrap();
            let address = result.address.unwrap();

            Logger::found_result("Match discovered!");
            Logger::separator();
            Logger::info("Raw Salt", &format!("0x{:016x}", raw_salt));
            Logger::info("Final Salt", &final_salt.to_string());

            // Display address in appropriate format
            if config.case_sensitive {
                Logger::info(
                    "Address",
                    &format!("{} (EIP-55 checksum)", address.to_checksum(None)),
                );
            } else {
                Logger::info("Address", &address.to_string());
            }

            // Calculate performance metrics
            let checked = total_checked.load(Ordering::Relaxed);
            let rate = checked as f64 / elapsed.as_secs_f64();
            Logger::print_metrics(checked, rate, elapsed.as_secs_f64());
        }
        _ => {
            Logger::no_result();
            let checked = total_checked.load(Ordering::Relaxed);
            let rate = checked as f64 / elapsed.as_secs_f64();
            Logger::print_metrics(checked, rate, elapsed.as_secs_f64());
        }
    }
}
