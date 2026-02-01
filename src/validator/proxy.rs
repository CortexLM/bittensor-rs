//! Proxy account operations for Bittensor
//! Allows delegating permissions to other accounts

use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::errors::{BittensorError, BittensorResult, ChainQueryError, ExtrinsicError};
use crate::utils::decoders::{decode_account_id32, decode_u128};
use parity_scale_codec::{Decode, Encode};
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const PROXY_MODULE: &str = "Proxy";

/// Proxy types for Bittensor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Default)]
#[repr(u8)]
pub enum ProxyType {
    /// Full permissions
    #[default]
    Any = 0,
    /// Non-transfer related permissions
    NonTransfer = 1,
    /// Governance related permissions
    Governance = 2,
    /// Staking related permissions
    Staking = 3,
    /// Registration related permissions
    Registration = 4,
    /// Transfer related permissions (like SudoUncheckedSetBalance)
    Transfer = 5,
    /// Subnet owner specific permissions
    Owner = 6,
    /// Non-critical validator permissions
    NonCritical = 7,
    /// Triumvirate/Senate permissions
    Triumvirate = 8,
    /// Subnet-related permissions
    Subnet = 9,
    /// Childkey permissions
    Childkey = 10,
    /// Senate permissions
    Senate = 11,
}

impl ProxyType {
    /// Convert proxy type to Value for extrinsic submission
    fn to_value(self) -> Value {
        let variant_name = match self {
            ProxyType::Any => "Any",
            ProxyType::NonTransfer => "NonTransfer",
            ProxyType::Governance => "Governance",
            ProxyType::Staking => "Staking",
            ProxyType::Registration => "Registration",
            ProxyType::Transfer => "Transfer",
            ProxyType::Owner => "Owner",
            ProxyType::NonCritical => "NonCritical",
            ProxyType::Triumvirate => "Triumvirate",
            ProxyType::Subnet => "Subnet",
            ProxyType::Childkey => "Childkey",
            ProxyType::Senate => "Senate",
        };
        Value::named_variant(variant_name, Vec::<(&str, Value)>::new())
    }

    /// Try to parse proxy type from a string representation
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Any" => Some(ProxyType::Any),
            "NonTransfer" => Some(ProxyType::NonTransfer),
            "Governance" => Some(ProxyType::Governance),
            "Staking" => Some(ProxyType::Staking),
            "Registration" => Some(ProxyType::Registration),
            "Transfer" => Some(ProxyType::Transfer),
            "Owner" => Some(ProxyType::Owner),
            "NonCritical" => Some(ProxyType::NonCritical),
            "Triumvirate" => Some(ProxyType::Triumvirate),
            "Subnet" => Some(ProxyType::Subnet),
            "Childkey" => Some(ProxyType::Childkey),
            "Senate" => Some(ProxyType::Senate),
            _ => None,
        }
    }
}

/// Proxy account information
#[derive(Debug, Clone)]
pub struct ProxyInfo {
    /// The delegate account that has been granted proxy permissions
    pub delegate: AccountId32,
    /// The type of proxy permissions granted
    pub proxy_type: ProxyType,
    /// Delay in blocks before the proxy can execute calls
    pub delay: u32,
}

// =============================================================================
// Proxy Management
// =============================================================================

/// Add a proxy account
///
/// Allows the signer to grant proxy permissions to the delegate account.
/// The delegate can then execute calls on behalf of the signer.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `signer` - The account granting proxy permissions
/// * `delegate` - The account receiving proxy permissions
/// * `proxy_type` - The type of proxy permissions to grant
/// * `delay` - Delay in blocks before the proxy can execute calls
/// * `wait_for` - How long to wait for the extrinsic
///
/// # Returns
/// The transaction hash on success
pub async fn add_proxy(
    client: &BittensorClient,
    signer: &BittensorSigner,
    delegate: &AccountId32,
    proxy_type: ProxyType,
    delay: u32,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    let args = vec![
        Value::from_bytes(delegate.encode()),
        proxy_type.to_value(),
        Value::u128(delay as u128),
    ];

    client
        .submit_extrinsic(PROXY_MODULE, "add_proxy", args, signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to add proxy: {}", e),
                PROXY_MODULE,
                "add_proxy",
            ))
        })
}

/// Remove a proxy account
///
/// Revokes proxy permissions from the delegate account.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `signer` - The account revoking proxy permissions
/// * `delegate` - The account losing proxy permissions
/// * `proxy_type` - The type of proxy permissions to revoke
/// * `delay` - The delay that was set when adding the proxy
/// * `wait_for` - How long to wait for the extrinsic
///
/// # Returns
/// The transaction hash on success
pub async fn remove_proxy(
    client: &BittensorClient,
    signer: &BittensorSigner,
    delegate: &AccountId32,
    proxy_type: ProxyType,
    delay: u32,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    let args = vec![
        Value::from_bytes(delegate.encode()),
        proxy_type.to_value(),
        Value::u128(delay as u128),
    ];

    client
        .submit_extrinsic(PROXY_MODULE, "remove_proxy", args, signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to remove proxy: {}", e),
                PROXY_MODULE,
                "remove_proxy",
            ))
        })
}

/// Remove all proxies
///
/// Revokes all proxy permissions granted by the signer.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `signer` - The account revoking all proxy permissions
/// * `wait_for` - How long to wait for the extrinsic
///
/// # Returns
/// The transaction hash on success
pub async fn remove_proxies(
    client: &BittensorClient,
    signer: &BittensorSigner,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    client
        .submit_extrinsic(PROXY_MODULE, "remove_proxies", Vec::new(), signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to remove all proxies: {}", e),
                PROXY_MODULE,
                "remove_proxies",
            ))
        })
}

/// Execute call as proxy
///
/// Allows a proxy account to execute a call on behalf of the real account.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `signer` - The proxy account executing the call
/// * `real` - The account on whose behalf the call is being made
/// * `force_proxy_type` - Optional: force a specific proxy type check
/// * `call` - The encoded call data to execute
/// * `wait_for` - How long to wait for the extrinsic
///
/// # Returns
/// The transaction hash on success
pub async fn proxy(
    client: &BittensorClient,
    signer: &BittensorSigner,
    real: &AccountId32,
    force_proxy_type: Option<ProxyType>,
    call: Vec<u8>,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    let force_proxy_type_value = match force_proxy_type {
        Some(pt) => Value::named_variant("Some", [("value", pt.to_value())]),
        None => Value::named_variant("None", Vec::<(&str, Value)>::new()),
    };

    let args = vec![
        Value::from_bytes(real.encode()),
        force_proxy_type_value,
        Value::from_bytes(&call),
    ];

    client
        .submit_extrinsic(PROXY_MODULE, "proxy", args, signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to execute proxy call: {}", e),
                PROXY_MODULE,
                "proxy",
            ))
        })
}

/// Create a pure (anonymous) proxy
///
/// Creates a new account that can only be controlled by the spawner via proxy calls.
/// This is useful for creating accounts that cannot directly sign transactions.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `signer` - The spawner account that will control the pure proxy
/// * `proxy_type` - The type of proxy permissions for the pure proxy
/// * `delay` - Delay in blocks before the proxy can execute calls
/// * `index` - A disambiguation index to allow creating multiple pure proxies with the same parameters
/// * `wait_for` - How long to wait for the extrinsic
///
/// # Returns
/// The transaction hash on success
pub async fn create_pure(
    client: &BittensorClient,
    signer: &BittensorSigner,
    proxy_type: ProxyType,
    delay: u32,
    index: u16,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    let args = vec![
        proxy_type.to_value(),
        Value::u128(delay as u128),
        Value::u128(index as u128),
    ];

    client
        .submit_extrinsic(PROXY_MODULE, "create_pure", args, signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to create pure proxy: {}", e),
                PROXY_MODULE,
                "create_pure",
            ))
        })
}

/// Kill a pure proxy
///
/// Removes a pure proxy account that was created by the spawner.
/// This can only be called by a proxy of the pure account.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `signer` - The proxy account that controls the pure proxy
/// * `spawner` - The account that originally created the pure proxy
/// * `proxy_type` - The proxy type used when creating the pure proxy
/// * `index` - The disambiguation index used when creating the pure proxy
/// * `height` - The block height at which the pure proxy was created
/// * `ext_index` - The extrinsic index in that block
/// * `wait_for` - How long to wait for the extrinsic
///
/// # Returns
/// The transaction hash on success
#[allow(clippy::too_many_arguments)]
pub async fn kill_pure(
    client: &BittensorClient,
    signer: &BittensorSigner,
    spawner: &AccountId32,
    proxy_type: ProxyType,
    index: u16,
    height: u32,
    ext_index: u32,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    let args = vec![
        Value::from_bytes(spawner.encode()),
        proxy_type.to_value(),
        Value::u128(index as u128),
        Value::u128(height as u128),
        Value::u128(ext_index as u128),
    ];

    client
        .submit_extrinsic(PROXY_MODULE, "kill_pure", args, signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to kill pure proxy: {}", e),
                PROXY_MODULE,
                "kill_pure",
            ))
        })
}

// =============================================================================
// Proxy Queries
// =============================================================================

/// Get all proxies for an account
///
/// Returns a list of all proxy definitions for the given account.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `account` - The account to query proxies for
///
/// # Returns
/// A list of ProxyInfo containing delegate, proxy type, and delay
pub async fn get_proxies(
    client: &BittensorClient,
    account: &AccountId32,
) -> BittensorResult<Vec<ProxyInfo>> {
    let result = client
        .storage_with_keys(
            PROXY_MODULE,
            "Proxies",
            vec![Value::from_bytes(account.encode())],
        )
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query proxies: {}", e),
                PROXY_MODULE,
                "Proxies",
            ))
        })?;

    match result {
        Some(value) => parse_proxies_storage(&value),
        None => Ok(Vec::new()),
    }
}

/// Check if an account is a proxy for another
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `real` - The account that may have granted proxy permissions
/// * `delegate` - The account to check if it has proxy permissions
/// * `proxy_type` - Optional: check for a specific proxy type
///
/// # Returns
/// True if the delegate is a proxy for the real account
pub async fn is_proxy(
    client: &BittensorClient,
    real: &AccountId32,
    delegate: &AccountId32,
    proxy_type: Option<ProxyType>,
) -> BittensorResult<bool> {
    let proxies = get_proxies(client, real).await?;

    for proxy_info in proxies {
        if proxy_info.delegate == *delegate {
            match proxy_type {
                Some(pt) => {
                    if proxy_info.proxy_type == pt {
                        return Ok(true);
                    }
                }
                None => return Ok(true),
            }
        }
    }

    Ok(false)
}

/// Parse the Proxies storage value into a list of ProxyInfo
///
/// The Proxies storage returns a tuple of (BoundedVec<ProxyDefinition>, Balance)
/// We parse this by examining the debug representation of the Value
fn parse_proxies_storage(value: &Value) -> BittensorResult<Vec<ProxyInfo>> {
    let mut proxies = Vec::new();
    let value_str = format!("{:?}", value);

    // Parse proxy definitions from the storage value
    // Each ProxyDefinition has format: { delegate: AccountId, proxy_type: ProxyType, delay: u32 }
    // The storage format is a tuple: ((proxy_list), deposit)

    // Extract account IDs from the value string
    let account_ids = extract_account_ids_from_debug(&value_str);

    // Extract proxy types from the value string
    let proxy_types = extract_proxy_types_from_debug(&value_str);

    // Extract delays from the value string
    let delays = extract_delays_from_debug(&value_str);

    // Match up the extracted values into ProxyInfo structs
    // The first account ID in the tuple is usually followed by proxy type and delay
    let num_proxies = account_ids
        .len()
        .min(proxy_types.len())
        .min(delays.len());

    for i in 0..num_proxies {
        proxies.push(ProxyInfo {
            delegate: account_ids[i].clone(),
            proxy_type: proxy_types[i],
            delay: delays[i],
        });
    }

    Ok(proxies)
}

/// Extract AccountId32 values from a debug string representation
fn extract_account_ids_from_debug(s: &str) -> Vec<AccountId32> {
    let mut accounts = Vec::new();

    // Look for 0x-prefixed hex strings that are 64 characters (32 bytes)
    let mut search_start = 0;
    while let Some(start) = s[search_start..].find("0x") {
        let abs_start = search_start + start;
        let hex_start = abs_start + 2;

        if hex_start < s.len() {
            let hex_chars: String = s[hex_start..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .take(64)
                .collect();

            if hex_chars.len() == 64 {
                if let Ok(bytes) = hex::decode(&hex_chars) {
                    if bytes.len() == 32 {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&bytes);
                        accounts.push(AccountId32::from(arr));
                    }
                }
            }

            search_start = hex_start + hex_chars.len();
        } else {
            break;
        }
    }

    accounts
}

/// Extract ProxyType values from a debug string representation
fn extract_proxy_types_from_debug(s: &str) -> Vec<ProxyType> {
    let mut types = Vec::new();

    // Look for variant names in the debug output
    let variant_names = [
        "Any",
        "NonTransfer",
        "Governance",
        "Staking",
        "Registration",
        "Transfer",
        "Owner",
        "NonCritical",
        "Triumvirate",
        "Subnet",
        "Childkey",
        "Senate",
    ];

    // Find all variant patterns like Variant("TypeName" or name: "TypeName"
    for variant_name in variant_names {
        let mut search_start = 0;
        while search_start < s.len() {
            // Check for Variant("TypeName" pattern
            let pattern1 = format!("Variant(\"{}", variant_name);
            let pattern2 = format!("\"{}\"", variant_name);

            if let Some(pos) = s[search_start..].find(&pattern1) {
                if let Some(pt) = ProxyType::from_str(variant_name) {
                    types.push(pt);
                }
                search_start = search_start + pos + pattern1.len();
            } else if let Some(pos) = s[search_start..].find(&pattern2) {
                // Only count if it looks like a variant context
                let context_start = pos.saturating_sub(10);
                let context = &s[search_start + context_start..search_start + pos + pattern2.len()];
                if context.contains("Variant") || context.contains("name:") {
                    if let Some(pt) = ProxyType::from_str(variant_name) {
                        types.push(pt);
                    }
                }
                search_start = search_start + pos + pattern2.len();
            } else {
                break;
            }
        }
    }

    // If no variants found, try numeric parsing
    if types.is_empty() {
        // Look for U8/U16/U32 patterns that could be proxy type indices
        let mut search_start = 0;
        while let Some(pos) = s[search_start..].find("U8(") {
            let abs_pos = search_start + pos;
            let num_start = abs_pos + 3;
            if let Some(end) = s[num_start..].find(')') {
                if let Ok(num) = s[num_start..num_start + end].trim().parse::<u8>() {
                    if num <= 11 {
                        if let Some(pt) = proxy_type_from_u8(num) {
                            types.push(pt);
                        }
                    }
                }
            }
            search_start = num_start;
        }
    }

    types
}

/// Extract delay values (u32) from a debug string representation
fn extract_delays_from_debug(s: &str) -> Vec<u32> {
    let mut delays = Vec::new();

    // Look for U32 patterns
    let mut search_start = 0;
    while let Some(pos) = s[search_start..].find("U32(") {
        let abs_pos = search_start + pos;
        let num_start = abs_pos + 4;
        if let Some(end) = s[num_start..].find(')') {
            if let Ok(num) = s[num_start..num_start + end].trim().parse::<u32>() {
                delays.push(num);
            }
        }
        search_start = num_start;
    }

    // Also try U64 in case delays are stored as larger integers
    if delays.is_empty() {
        search_start = 0;
        while let Some(pos) = s[search_start..].find("U64(") {
            let abs_pos = search_start + pos;
            let num_start = abs_pos + 4;
            if let Some(end) = s[num_start..].find(')') {
                if let Ok(num) = s[num_start..num_start + end].trim().parse::<u64>() {
                    delays.push(num as u32);
                }
            }
            search_start = num_start;
        }
    }

    delays
}

/// Convert a u8 value to ProxyType
fn proxy_type_from_u8(value: u8) -> Option<ProxyType> {
    match value {
        0 => Some(ProxyType::Any),
        1 => Some(ProxyType::NonTransfer),
        2 => Some(ProxyType::Governance),
        3 => Some(ProxyType::Staking),
        4 => Some(ProxyType::Registration),
        5 => Some(ProxyType::Transfer),
        6 => Some(ProxyType::Owner),
        7 => Some(ProxyType::NonCritical),
        8 => Some(ProxyType::Triumvirate),
        9 => Some(ProxyType::Subnet),
        10 => Some(ProxyType::Childkey),
        11 => Some(ProxyType::Senate),
        _ => None,
    }
}

/// Parse a ProxyType from a Value using debug string parsing
#[allow(dead_code)]
fn parse_proxy_type(value: &Value) -> Option<ProxyType> {
    let s = format!("{:?}", value);

    // Try variant pattern first
    for (variant_name, pt) in [
        ("Any", ProxyType::Any),
        ("NonTransfer", ProxyType::NonTransfer),
        ("Governance", ProxyType::Governance),
        ("Staking", ProxyType::Staking),
        ("Registration", ProxyType::Registration),
        ("Transfer", ProxyType::Transfer),
        ("Owner", ProxyType::Owner),
        ("NonCritical", ProxyType::NonCritical),
        ("Triumvirate", ProxyType::Triumvirate),
        ("Subnet", ProxyType::Subnet),
        ("Childkey", ProxyType::Childkey),
        ("Senate", ProxyType::Senate),
    ] {
        if s.contains(&format!("\"{}\"", variant_name)) {
            return Some(pt);
        }
    }

    // Try numeric parsing
    if let Ok(num) = decode_u128(value) {
        return proxy_type_from_u8(num as u8);
    }

    None
}

/// Parse a u32 from a Value
#[allow(dead_code)]
fn parse_u32(value: &Value) -> Option<u32> {
    decode_u128(value).ok().map(|v| v as u32)
}

/// Parse an AccountId32 from a Value
#[allow(dead_code)]
fn parse_account_id(value: &Value) -> Option<AccountId32> {
    decode_account_id32(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_type_encoding() {
        assert_eq!(ProxyType::Any as u8, 0);
        assert_eq!(ProxyType::Staking as u8, 3);
        assert_eq!(ProxyType::Senate as u8, 11);
    }

    #[test]
    fn test_proxy_type_default() {
        assert_eq!(ProxyType::default(), ProxyType::Any);
    }

    #[test]
    fn test_proxy_type_from_str() {
        assert_eq!(ProxyType::from_str("Any"), Some(ProxyType::Any));
        assert_eq!(ProxyType::from_str("Staking"), Some(ProxyType::Staking));
        assert_eq!(ProxyType::from_str("Senate"), Some(ProxyType::Senate));
        assert_eq!(ProxyType::from_str("Invalid"), None);
        assert_eq!(ProxyType::from_str(""), None);
    }

    #[test]
    fn test_proxy_type_to_value() {
        let value = ProxyType::Any.to_value();
        let debug_str = format!("{:?}", value);
        assert!(debug_str.contains("Any"));

        let value = ProxyType::Staking.to_value();
        let debug_str = format!("{:?}", value);
        assert!(debug_str.contains("Staking"));
    }

    #[test]
    fn test_proxy_info_debug() {
        let account_bytes = [1u8; 32];
        let info = ProxyInfo {
            delegate: AccountId32::from(account_bytes),
            proxy_type: ProxyType::Staking,
            delay: 100,
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("Staking"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_proxy_info_clone() {
        let account_bytes = [2u8; 32];
        let info = ProxyInfo {
            delegate: AccountId32::from(account_bytes),
            proxy_type: ProxyType::Governance,
            delay: 50,
        };

        let cloned = info.clone();
        assert_eq!(cloned.delegate, info.delegate);
        assert_eq!(cloned.proxy_type, info.proxy_type);
        assert_eq!(cloned.delay, info.delay);
    }

    #[test]
    fn test_proxy_type_from_u8() {
        assert_eq!(proxy_type_from_u8(0), Some(ProxyType::Any));
        assert_eq!(proxy_type_from_u8(3), Some(ProxyType::Staking));
        assert_eq!(proxy_type_from_u8(11), Some(ProxyType::Senate));
        assert_eq!(proxy_type_from_u8(255), None);
    }

    #[test]
    fn test_extract_account_ids_from_debug() {
        // Test with a known hex AccountId
        let hex_account = "0x0101010101010101010101010101010101010101010101010101010101010101";
        let test_str = format!("Some data {} more data", hex_account);
        let accounts = extract_account_ids_from_debug(&test_str);
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0], AccountId32::from([1u8; 32]));
    }

    #[test]
    fn test_extract_delays_from_debug() {
        let test_str = "Composite { values: [U32(100), U32(200)] }";
        let delays = extract_delays_from_debug(test_str);
        assert_eq!(delays.len(), 2);
        assert_eq!(delays[0], 100);
        assert_eq!(delays[1], 200);
    }

    #[test]
    fn test_proxy_type_equality() {
        assert_eq!(ProxyType::Any, ProxyType::Any);
        assert_ne!(ProxyType::Any, ProxyType::Staking);
        assert_eq!(ProxyType::Senate, ProxyType::Senate);
    }

    #[test]
    fn test_all_proxy_types_have_from_str() {
        let types = [
            ("Any", ProxyType::Any),
            ("NonTransfer", ProxyType::NonTransfer),
            ("Governance", ProxyType::Governance),
            ("Staking", ProxyType::Staking),
            ("Registration", ProxyType::Registration),
            ("Transfer", ProxyType::Transfer),
            ("Owner", ProxyType::Owner),
            ("NonCritical", ProxyType::NonCritical),
            ("Triumvirate", ProxyType::Triumvirate),
            ("Subnet", ProxyType::Subnet),
            ("Childkey", ProxyType::Childkey),
            ("Senate", ProxyType::Senate),
        ];

        for (name, expected) in types {
            assert_eq!(
                ProxyType::from_str(name),
                Some(expected),
                "Failed for {}",
                name
            );
        }
    }

    #[test]
    fn test_all_proxy_types_u8_values() {
        assert_eq!(ProxyType::Any as u8, 0);
        assert_eq!(ProxyType::NonTransfer as u8, 1);
        assert_eq!(ProxyType::Governance as u8, 2);
        assert_eq!(ProxyType::Staking as u8, 3);
        assert_eq!(ProxyType::Registration as u8, 4);
        assert_eq!(ProxyType::Transfer as u8, 5);
        assert_eq!(ProxyType::Owner as u8, 6);
        assert_eq!(ProxyType::NonCritical as u8, 7);
        assert_eq!(ProxyType::Triumvirate as u8, 8);
        assert_eq!(ProxyType::Subnet as u8, 9);
        assert_eq!(ProxyType::Childkey as u8, 10);
        assert_eq!(ProxyType::Senate as u8, 11);
    }
}
