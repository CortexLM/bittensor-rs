//! Signing message construction for synapse verification.

/// Constructs the signing message from the synapse verification fields.
///
/// The format is `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"`,
/// matching the Python SDK exactly. Fields are dot-separated with no spaces.
pub fn signing_message(
    nonce: u64,
    dendrite_hotkey: &str,
    axon_hotkey: &str,
    uuid: &str,
    body_hash: &str,
) -> String {
    format!("{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signing_message_format() {
        let msg = signing_message(
            1234567890,
            "5DendriteHotkey123",
            "5AxonHotkey456",
            "550e8400-e29b-41d4-a716-446655440000",
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
        );
        assert_eq!(
            msg,
            "1234567890.5DendriteHotkey123.5AxonHotkey456.550e8400-e29b-41d4-a716-446655440000.a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
        );
    }

    #[test]
    fn signing_message_empty_fields() {
        let msg = signing_message(0, "", "", "", "");
        assert_eq!(msg, "0....");
    }

    #[test]
    fn signing_message_no_spaces() {
        let msg = signing_message(1, "key_a", "key_b", "uuid-1", "hash123");
        assert!(!msg.contains(' '));
        assert_eq!(msg, "1.key_a.key_b.uuid-1.hash123");
    }

    #[test]
    fn signing_message_dot_separated_exactly_five_parts() {
        let msg = signing_message(42, "dhk", "ahk", "uid", "bhash");
        let parts: Vec<&str> = msg.split('.').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], "42");
        assert_eq!(parts[1], "dhk");
        assert_eq!(parts[2], "ahk");
        assert_eq!(parts[3], "uid");
        assert_eq!(parts[4], "bhash");
    }
}
