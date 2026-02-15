use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use subxt::dynamic::Value;

const ADMIN_UTILS: &str = "AdminUtils";

/// Generic hyperparameter setter using 'AdminUtils::sudo_set_*' pattern.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (must be subnet owner or root).
/// * `netuid` — The subnet ID.
/// * `param_name` — The parameter name (maps to `sudo_set_{param_name}`).
/// * `value` — The value to set (as u64, cast appropriately on-chain).
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn set_hyperparameter(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    param_name: &str,
    value: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let function = format!("sudo_set_{}", param_name);
    let args = vec![Value::from(netuid), Value::from(value)];

    client
        .submit_extrinsic(ADMIN_UTILS, &function, args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set hyperparameter '{}': {}", param_name, e))
}

/// AdminUtils pallet dispatch: `sudo_set_tempo(netuid, tempo)`
pub async fn sudo_set_tempo(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    tempo: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(tempo)];

    client
        .submit_extrinsic(ADMIN_UTILS, "sudo_set_tempo", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set tempo: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_weights_rate_limit(netuid, rate_limit)`
pub async fn sudo_set_weights_rate_limit(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    rate_limit: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(rate_limit)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_weights_rate_limit",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set weights rate limit: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_max_allowed_validators(netuid, max_validators)`
pub async fn sudo_set_max_allowed_validators(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    max_validators: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(max_validators)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_max_allowed_validators",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set max allowed validators: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_min_allowed_weights(netuid, min_weights)`
pub async fn sudo_set_min_allowed_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    min_weights: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(min_weights)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_min_allowed_weights",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set min allowed weights: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_max_weights_limit(netuid, max_weights)`
pub async fn sudo_set_max_weights_limit(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    max_weights: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(max_weights)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_max_weights_limit",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set max weights limit: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_immunity_period(netuid, immunity_period)`
pub async fn sudo_set_immunity_period(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    immunity_period: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(immunity_period)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_immunity_period",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set immunity period: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_activity_cutoff(netuid, activity_cutoff)`
pub async fn sudo_set_activity_cutoff(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    activity_cutoff: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(activity_cutoff)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_activity_cutoff",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set activity cutoff: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_registration_allowed(netuid, allowed)`
pub async fn sudo_set_registration_allowed(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    allowed: bool,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::bool(allowed)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_registration_allowed",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set registration allowed: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_target_registrations_per_interval(netuid, target)`
pub async fn sudo_set_target_registrations_per_interval(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    target: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(target)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_target_registrations_per_interval",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set target registrations per interval: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_difficulty(netuid, difficulty)`
pub async fn sudo_set_difficulty(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    difficulty: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(difficulty)];

    client
        .submit_extrinsic(ADMIN_UTILS, "sudo_set_difficulty", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set difficulty: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_max_registrations_per_block(netuid, max_registrations)`
pub async fn sudo_set_max_registrations_per_block(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    max_registrations: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(max_registrations)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_max_registrations_per_block",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set max registrations per block: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_serving_rate_limit(netuid, rate_limit)`
pub async fn sudo_set_serving_rate_limit(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    rate_limit: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from(rate_limit)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_serving_rate_limit",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set serving rate limit: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_commit_reveal_weights_enabled(netuid, enabled)`
pub async fn sudo_set_commit_reveal_weights_enabled(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    enabled: bool,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::bool(enabled)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_commit_reveal_weights_enabled",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set commit reveal weights enabled: {}", e))
}

/// AdminUtils pallet dispatch: `sudo_set_liquid_alpha_enabled(netuid, enabled)`
pub async fn sudo_set_liquid_alpha_enabled(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    enabled: bool,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::bool(enabled)];

    client
        .submit_extrinsic(
            ADMIN_UTILS,
            "sudo_set_liquid_alpha_enabled",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set liquid alpha enabled: {}", e))
}
