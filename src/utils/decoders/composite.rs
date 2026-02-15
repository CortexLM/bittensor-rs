use super::utils;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::OnceLock;
use subxt::dynamic::{At, Value};
use subxt::ext::scale_value::{Composite, ValueDef};

/// Get a static regex for parsing identity data
fn get_identity_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r#"(\w+):\s*(?:Some\()?"([^"]*)"(?:\))?"#)
            .expect("Invalid regex pattern for identity parsing")
    })
}

/// Extract a u128 primitive from a Value using the proper API
fn extract_u128_from_value(value: &Value) -> Option<u128> {
    value.as_u128()
}

/// Decode PrometheusInfo from Value
/// Subtensor PrometheusInfo: { block: u64, version: u32, ip: u128, port: u16, ip_type: u8 }
pub fn decode_prometheus_info(value: &Value) -> Result<crate::types::PrometheusInfo> {
    // Try using Value's .at() API first for named/unnamed composite access
    // PrometheusInfo fields in order: block, version, ip, port, ip_type
    if let (Some(block_val), Some(version_val), Some(ip_val), Some(port_val), Some(ip_type_val)) = (
        value.at(0),
        value.at(1),
        value.at(2),
        value.at(3),
        value.at(4),
    ) {
        if let (Some(block), Some(version), Some(ip_u128), Some(port), Some(ip_type)) = (
            extract_u128_from_value(block_val),
            extract_u128_from_value(version_val),
            extract_u128_from_value(ip_val),
            extract_u128_from_value(port_val),
            extract_u128_from_value(ip_type_val),
        ) {
            let ip = utils::parse_ip_addr(ip_u128, ip_type as u8);
            return Ok(crate::types::PrometheusInfo::from_chain_data(
                block as u64,
                version as u32,
                ip.to_string(),
                port as u16,
                ip_type as u8,
            ));
        }
    }

    // Fall back to debug string parsing for compatibility with older formats
    let s = format!("{:?}", value);

    // Extract exactly: U64, U32, U128, U16, U8
    let block = utils::extract_u64(&s, 0)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing block (u64) in value: {}", s))?;
    let version = utils::extract_u32(&s, block.1)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing version (u32) in value: {}", s))?;
    let ip_u128 = utils::extract_u128(&s, version.1)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing ip (u128) in value: {}", s))?;
    let port = utils::extract_u16(&s, ip_u128.1)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing port (u16) in value: {}", s))?;
    let ip_type = utils::extract_u8(&s, port.1)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing ip_type (u8) in value: {}", s))?;

    let ip = utils::parse_ip_addr(ip_u128.0, ip_type.0);

    Ok(crate::types::PrometheusInfo::from_chain_data(
        block.0,
        version.0,
        ip.to_string(),
        port.0,
        ip_type.0,
    ))
}

/// Decode AxonInfo from a Value
/// Subtensor AxonInfo: { block: u64, version: u32, ip: u128, port: u16, ip_type: u8, protocol: u8, placeholder1: u8, placeholder2: u8 }
pub fn decode_axon_info(value: &Value) -> Result<crate::types::AxonInfo> {
    // Try using Value's .at() API first for composite access
    // AxonInfo fields in order: block, version, ip, port, ip_type, protocol, placeholder1, placeholder2
    if let (
        Some(block_val),
        Some(version_val),
        Some(ip_val),
        Some(port_val),
        Some(ip_type_val),
        Some(protocol_val),
        Some(placeholder1_val),
        Some(placeholder2_val),
    ) = (
        value.at(0),
        value.at(1),
        value.at(2),
        value.at(3),
        value.at(4),
        value.at(5),
        value.at(6),
        value.at(7),
    ) {
        if let (
            Some(block),
            Some(version),
            Some(ip_u128),
            Some(port),
            Some(ip_type),
            Some(protocol),
            Some(placeholder1),
            Some(placeholder2),
        ) = (
            extract_u128_from_value(block_val),
            extract_u128_from_value(version_val),
            extract_u128_from_value(ip_val),
            extract_u128_from_value(port_val),
            extract_u128_from_value(ip_type_val),
            extract_u128_from_value(protocol_val),
            extract_u128_from_value(placeholder1_val),
            extract_u128_from_value(placeholder2_val),
        ) {
            let ip = utils::parse_ip_addr(ip_u128, ip_type as u8);
            return Ok(crate::types::AxonInfo::from_chain_data(
                block as u64,
                version as u32,
                ip,
                port as u16,
                ip_type as u8,
                protocol as u8,
                placeholder1 as u8,
                placeholder2 as u8,
            ));
        }
    }

    // Fall back to debug string parsing for compatibility
    let s = format!("{:?}", value);

    // Extract exactly: U64, U32, U128, U16, U8, U8, U8, U8
    let block = utils::extract_u64(&s, 0)
        .ok_or_else(|| anyhow!("AxonInfo: missing block (u64) in value: {}", s))?;
    let version = utils::extract_u32(&s, block.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing version (u32) in value: {}", s))?;
    let ip_u128 = utils::extract_u128(&s, version.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing ip (u128) in value: {}", s))?;
    let port = utils::extract_u16(&s, ip_u128.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing port (u16) in value: {}", s))?;
    let ip_type = utils::extract_u8(&s, port.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing ip_type (u8) in value: {}", s))?;
    let protocol = utils::extract_u8(&s, ip_type.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing protocol (u8) in value: {}", s))?;
    let placeholder1 = utils::extract_u8(&s, protocol.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing placeholder1 (u8) in value: {}", s))?;
    let placeholder2 = utils::extract_u8(&s, placeholder1.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing placeholder2 (u8) in value: {}", s))?;

    let ip = utils::parse_ip_addr(ip_u128.0, ip_type.0);

    Ok(crate::types::AxonInfo::from_chain_data(
        block.0,
        version.0,
        ip,
        port.0,
        ip_type.0,
        protocol.0,
        placeholder1.0,
        placeholder2.0,
    ))
}

/// Helper to decode identity data from a map structure
/// Extracts key-value pairs from composite values
pub fn decode_identity_map(value: &Value) -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();

    // Try to extract named fields first using the proper Value API
    // Check for common identity fields by name
    let identity_fields = [
        "name",
        "display",
        "legal",
        "web",
        "riot",
        "email",
        "image",
        "twitter",
        "pgp_fingerprint",
        "additional",
    ];

    for field_name in identity_fields {
        if let Some(field_val) = value.at(field_name) {
            // Try to get the string value directly
            if let Some(s) = field_val.as_str() {
                if !s.is_empty() {
                    result.insert(field_name.to_string(), s.to_string());
                }
            } else if let Some(inner) = field_val.at(0) {
                // Handle Option<T> wrapped values (Some variant with inner value)
                if let Some(s) = inner.as_str() {
                    if !s.is_empty() {
                        result.insert(field_name.to_string(), s.to_string());
                    }
                }
            }
        }
    }

    // If we found fields using the API, return early
    if !result.is_empty() {
        return Ok(result);
    }

    // Fall back to regex parsing for compatibility with debug format
    let value_str = format!("{:?}", value);
    let re = get_identity_regex();

    for cap in re.captures_iter(&value_str) {
        if let (Some(key), Some(val)) = (cap.get(1), cap.get(2)) {
            let key_str = key.as_str().to_string();
            let val_str = val.as_str().to_string();
            // Only insert non-empty values
            if !val_str.is_empty() {
                result.insert(key_str, val_str);
            }
        }
    }

    Ok(result)
}

/// Decode a named composite (struct) from a Value
/// Extracts field names and values from composite structures
/// For named composites: returns HashMap of field_name -> cloned Value
/// For unnamed composites: returns HashMap of index (as string) -> cloned Value
pub fn decode_named_composite(value: &Value) -> Result<HashMap<String, Value>> {
    let mut result = HashMap::new();

    // Inspect the ValueDef to determine if it's a composite
    match &value.value {
        ValueDef::Composite(composite) => match composite {
            Composite::Named(fields) => {
                // Named composite: extract all field names and values
                for (name, val) in fields {
                    result.insert(name.clone(), val.clone());
                }
            }
            Composite::Unnamed(values) => {
                // Unnamed composite: use index as key
                for (idx, val) in values.iter().enumerate() {
                    result.insert(idx.to_string(), val.clone());
                }
            }
        },
        ValueDef::Variant(variant) => {
            // For variants, extract the inner composite values
            match &variant.values {
                Composite::Named(fields) => {
                    for (name, val) in fields {
                        result.insert(name.clone(), val.clone());
                    }
                }
                Composite::Unnamed(values) => {
                    for (idx, val) in values.iter().enumerate() {
                        result.insert(idx.to_string(), val.clone());
                    }
                }
            }
        }
        _ => {
            // Not a composite or variant, return empty map
            // This is not an error - the caller may expect this for non-composite values
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_prometheus_info() {
        // Create a test Value representing PrometheusInfo
        // Fields: block (u64), version (u32), ip (u128), port (u16), ip_type (u8)
        // Using unnamed composite as that's how subxt returns it
        let prometheus_value = Value::unnamed_composite([
            Value::u128(12345),      // block
            Value::u128(1),          // version
            Value::u128(2130706433), // ip: 127.0.0.1 as u128
            Value::u128(9933),       // port
            Value::u128(4),          // ip_type: IPv4
        ]);

        let result = decode_prometheus_info(&prometheus_value);
        assert!(
            result.is_ok(),
            "Failed to decode PrometheusInfo: {:?}",
            result
        );

        let info = result.unwrap();
        assert_eq!(info.block, 12345);
        assert_eq!(info.version, 1);
        assert_eq!(info.port, 9933);
        assert_eq!(info.ip_type, 4);
    }

    #[test]
    fn test_decode_axon_info() {
        // Create a test Value representing AxonInfo
        // Fields: block, version, ip, port, ip_type, protocol, placeholder1, placeholder2
        let axon_value = Value::unnamed_composite([
            Value::u128(54321),      // block
            Value::u128(2),          // version
            Value::u128(2130706433), // ip: 127.0.0.1 as u128
            Value::u128(8080),       // port
            Value::u128(4),          // ip_type: IPv4
            Value::u128(1),          // protocol
            Value::u128(0),          // placeholder1
            Value::u128(0),          // placeholder2
        ]);

        let result = decode_axon_info(&axon_value);
        assert!(result.is_ok(), "Failed to decode AxonInfo: {:?}", result);

        let info = result.unwrap();
        assert_eq!(info.block, 54321);
        assert_eq!(info.version, 2);
        assert_eq!(info.port, 8080);
        assert_eq!(info.ip_type, 4);
        assert_eq!(info.protocol, 1);
        assert_eq!(info.placeholder1, 0);
        assert_eq!(info.placeholder2, 0);
    }

    #[test]
    fn test_decode_identity_map_with_named_fields() {
        // Create a test Value with named identity fields
        let identity_value = Value::named_composite([
            ("name", Value::string("TestValidator")),
            ("web", Value::string("https://example.com")),
            ("email", Value::string("test@example.com")),
        ]);

        let result = decode_identity_map(&identity_value);
        assert!(
            result.is_ok(),
            "Failed to decode identity map: {:?}",
            result
        );

        let map = result.unwrap();
        assert_eq!(map.get("name"), Some(&"TestValidator".to_string()));
        assert_eq!(map.get("web"), Some(&"https://example.com".to_string()));
        assert_eq!(map.get("email"), Some(&"test@example.com".to_string()));
    }

    #[test]
    fn test_decode_identity_map_empty() {
        // Test with a primitive value (should return empty map)
        let primitive_value = Value::u128(42);

        let result = decode_identity_map(&primitive_value);
        assert!(result.is_ok());

        let map = result.unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn test_decode_named_composite_with_named_fields() {
        // Create a named composite value
        let composite = Value::named_composite([
            ("foo", Value::u128(123)),
            ("bar", Value::string("hello")),
            ("baz", Value::bool(true)),
        ]);

        let result = decode_named_composite(&composite);
        assert!(
            result.is_ok(),
            "Failed to decode named composite: {:?}",
            result
        );

        let map = result.unwrap();
        assert_eq!(map.len(), 3);
        assert!(map.contains_key("foo"));
        assert!(map.contains_key("bar"));
        assert!(map.contains_key("baz"));

        // Verify field values using as_u128, as_str, as_bool
        assert_eq!(map.get("foo").and_then(|v| v.as_u128()), Some(123));
        assert_eq!(map.get("bar").and_then(|v| v.as_str()), Some("hello"));
        assert_eq!(map.get("baz").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_decode_named_composite_with_unnamed_fields() {
        // Create an unnamed composite value
        let composite = Value::unnamed_composite([Value::u128(1), Value::u128(2), Value::u128(3)]);

        let result = decode_named_composite(&composite);
        assert!(result.is_ok());

        let map = result.unwrap();
        assert_eq!(map.len(), 3);

        // Unnamed composites use index as key
        assert!(map.contains_key("0"));
        assert!(map.contains_key("1"));
        assert!(map.contains_key("2"));

        assert_eq!(map.get("0").and_then(|v| v.as_u128()), Some(1));
        assert_eq!(map.get("1").and_then(|v| v.as_u128()), Some(2));
        assert_eq!(map.get("2").and_then(|v| v.as_u128()), Some(3));
    }

    #[test]
    fn test_decode_named_composite_with_primitive() {
        // Test with a primitive value (should return empty map)
        let primitive = Value::u128(42);

        let result = decode_named_composite(&primitive);
        assert!(result.is_ok());

        let map = result.unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn test_decode_named_composite_with_variant() {
        // Create a variant value with named fields
        let variant = Value::named_variant(
            "SomeVariant",
            [
                ("field1", Value::u128(100)),
                ("field2", Value::string("variant_data")),
            ],
        );

        let result = decode_named_composite(&variant);
        assert!(result.is_ok());

        let map = result.unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("field1"));
        assert!(map.contains_key("field2"));
    }

    #[test]
    fn test_static_regex_reuse() {
        // Call get_identity_regex multiple times to ensure it's properly cached
        let re1 = get_identity_regex();
        let re2 = get_identity_regex();

        // Both should point to the same regex instance
        assert!(
            std::ptr::eq(re1, re2),
            "Regex should be cached via OnceLock"
        );
    }
}
