//! CLI utility functions for terminal interaction and formatting.

use crate::core::constants::RAOPERTAO;
use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};
use console::{style, Term};
use dialoguer::{Confirm, Input, Password};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Prompt for confirmation with default behavior based on `no_prompt` flag.
/// If `no_prompt` is true, returns true without prompting.
pub fn confirm(message: &str, no_prompt: bool) -> bool {
    if no_prompt {
        return true;
    }

    Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap_or(false)
}

/// Prompt for password input (hidden characters).
pub fn prompt_password(message: &str) -> String {
    Password::new()
        .with_prompt(message)
        .interact()
        .unwrap_or_default()
}

/// Prompt for optional password input. Returns None if empty.
pub fn prompt_password_optional(message: &str) -> Option<String> {
    let password = Password::new()
        .with_prompt(message)
        .allow_empty_password(true)
        .interact()
        .unwrap_or_default();

    if password.is_empty() {
        None
    } else {
        Some(password)
    }
}

/// Prompt for text input with a default value.
pub fn prompt_input(message: &str) -> String {
    Input::new()
        .with_prompt(message)
        .interact_text()
        .unwrap_or_default()
}

/// Prompt for text input with a default value.
pub fn prompt_input_with_default(message: &str, default: &str) -> String {
    Input::new()
        .with_prompt(message)
        .default(default.to_string())
        .interact_text()
        .unwrap_or_else(|_| default.to_string())
}

/// Create a spinner progress bar with message.
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.blue} {msg}")
            .expect("valid template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Print success message in green.
pub fn print_success(message: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", style("✓").green().bold(), message));
}

/// Print error message in red.
pub fn print_error(message: &str) {
    let term = Term::stderr();
    let _ = term.write_line(&format!("{} {}", style("✗").red().bold(), message));
}

/// Print info message in blue.
pub fn print_info(message: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", style("ℹ").blue().bold(), message));
}

/// Print warning message in yellow.
pub fn print_warning(message: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", style("⚠").yellow().bold(), message));
}

/// Format RAO balance as TAO (1 TAO = RAOPERTAO RAO).
pub fn format_tao(rao: u128) -> String {
    let whole = rao / RAOPERTAO;
    let fraction = rao % RAOPERTAO;
    format!("{}.{:09} τ", whole, fraction)
}

/// Format TAO as RAO for display (preserves decimal precision).
pub fn tao_to_rao(tao: f64) -> u128 {
    crate::utils::balance_newtypes::tao_to_rao(tao)
}

/// Format SS58 address (truncated for display).
/// Shows first 8 and last 8 characters with "..." in between.
pub fn format_address(address: &str) -> String {
    if address.len() <= 18 {
        return address.to_string();
    }
    format!("{}...{}", &address[..8], &address[address.len() - 8..])
}

/// Format SS58 address for full display.
pub fn format_address_full(address: &str) -> String {
    address.to_string()
}

/// Create a styled table for CLI output.
pub fn create_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

/// Create a table with custom headers.
pub fn create_table_with_headers(headers: &[&str]) -> Table {
    let mut table = create_table();
    table.set_header(headers.iter().map(|h| style(*h).bold().to_string()));
    table
}

/// Parse comma-separated list of u16 values.
pub fn parse_u16_list(input: &str) -> anyhow::Result<Vec<u16>> {
    input
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<u16>()
                .map_err(|e| anyhow::anyhow!("Invalid u16 value '{}': {}", s.trim(), e))
        })
        .collect()
}

/// Parse comma-separated list of f64 values.
pub fn parse_f64_list(input: &str) -> anyhow::Result<Vec<f64>> {
    input
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<f64>()
                .map_err(|e| anyhow::anyhow!("Invalid f64 value '{}': {}", s.trim(), e))
        })
        .collect()
}

/// Get network endpoint from network name or custom endpoint.
pub fn resolve_endpoint(network: &str, custom_endpoint: Option<&str>) -> String {
    if let Some(endpoint) = custom_endpoint {
        return endpoint.to_string();
    }

    if let Ok(endpoint) = std::env::var("BITTENSOR_RPC") {
        return endpoint;
    }

    match network.to_lowercase().as_str() {
        "finney" => "wss://entrypoint-finney.opentensor.ai:443".to_string(),
        "test" | "testnet" => "wss://test.finney.opentensor.ai:443".to_string(),
        "local" | "localhost" => "ws://127.0.0.1:9944".to_string(),
        "archive" => "wss://archive.chain.opentensor.ai:443".to_string(),
        _ => network.to_string(),
    }
}

/// Format duration for display.
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }
}

/// Validate SS58 address format.
pub fn is_valid_ss58(address: &str) -> bool {
    if address.len() < 46 || address.len() > 48 {
        return false;
    }
    address.chars().all(|c| c.is_alphanumeric())
}

/// Create a BittensorSigner from a wallet Keypair
pub fn keypair_to_signer(keypair: &crate::wallet::Keypair) -> crate::chain::BittensorSigner {
    crate::chain::create_signer(keypair.pair().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tao() {
        assert_eq!(format_tao(0), "0.000000000 τ");
        assert_eq!(format_tao(1_000_000_000), "1.000000000 τ");
        assert_eq!(format_tao(1_500_000_000), "1.500000000 τ");
        assert_eq!(format_tao(123_456_789_012), "123.456789012 τ");
    }

    #[test]
    fn test_format_address() {
        let addr = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
        assert_eq!(format_address(addr), "5GrwvaEF...oHGKutQY");

        let short = "5GrwvaEF";
        assert_eq!(format_address(short), "5GrwvaEF");
    }

    #[test]
    fn test_parse_u16_list() {
        assert_eq!(parse_u16_list("1,2,3").unwrap(), vec![1, 2, 3]);
        assert_eq!(parse_u16_list("1, 2, 3").unwrap(), vec![1, 2, 3]);
        assert!(parse_u16_list("1,invalid").is_err());
    }

    #[test]
    fn test_resolve_endpoint() {
        let previous = std::env::var("BITTENSOR_RPC").ok();
        std::env::remove_var("BITTENSOR_RPC");
        assert_eq!(
            resolve_endpoint("finney", None),
            "wss://entrypoint-finney.opentensor.ai:443"
        );
        assert_eq!(resolve_endpoint("local", None), "ws://127.0.0.1:9944");
        assert_eq!(
            resolve_endpoint("finney", Some("ws://custom:9944")),
            "ws://custom:9944"
        );
        if let Some(value) = previous {
            std::env::set_var("BITTENSOR_RPC", value);
        } else {
            std::env::remove_var("BITTENSOR_RPC");
        }
    }
}
