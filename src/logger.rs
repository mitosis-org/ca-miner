use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Logger;

impl Logger {
    pub fn header(message: &str) {
        println!("{}", format!("🚀 {}", message).bright_cyan().bold());
    }

    pub fn success(message: &str) {
        println!("{}", format!("✅ {}", message).bright_green().bold());
    }

    pub fn info(label: &str, value: &str) {
        println!(
            "{}{}: {}",
            "  ".bright_black(),
            label.bright_white().bold(),
            value.bright_yellow()
        );
    }

    pub fn warning(message: &str) {
        println!("{}", format!("⚠️  {}", message).bright_yellow().bold());
    }

    pub fn error(message: &str) {
        println!("{}", format!("❌ {}", message).bright_red().bold());
    }

    pub fn mining_start() {
        println!(
            "{}",
            "🔥 Starting mining process...".bright_magenta().bold()
        );
        println!();
    }

    pub fn found_result(message: &str) {
        println!();
        println!("{}", "🎉 FOUND MATCH!".bright_green().bold().on_black());
        println!("{}", message.bright_green());
    }

    pub fn no_result() {
        println!();
        println!("{}", "💔 No matching salt found".bright_red().bold());
    }

    pub fn separator() {
        println!("{}", "─".repeat(60).bright_black());
    }

    pub fn create_progress_bar(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.cyan} {msg} | {elapsed} | {per_sec}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn print_metrics(checked: u64, rate: f64, elapsed: f64) {
        Self::separator();
        Self::info("Total Checked", &crate::format_number(checked));
        Self::info("Average Rate", &format!("{:.0} salts/sec", rate));
        Self::info("Time Elapsed", &format!("{:.2}s", elapsed));
        Self::separator();
    }
}
