//! Header constants and serialization for the Bittensor synapse protocol.
//!
//! All header keys use the `bt_header_` prefix pattern matching the Python SDK.

/// Header key constants matching the Python SDK's `to_headers()` output.
pub mod keys {
    pub const NAME: &str = "name";
    pub const TIMEOUT: &str = "timeout";
    pub const HEADER_SIZE: &str = "header_size";
    pub const TOTAL_SIZE: &str = "total_size";
    pub const COMPUTED_BODY_HASH: &str = "computed_body_hash";

    pub const AXON_PREFIX: &str = "bt_header_axon_";
    pub const DENDRITE_PREFIX: &str = "bt_header_dendrite_";
    pub const INPUT_OBJ_PREFIX: &str = "bt_header_input_obj_";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_strings_are_correct() {
        assert_eq!(keys::AXON_PREFIX, "bt_header_axon_");
        assert_eq!(keys::DENDRITE_PREFIX, "bt_header_dendrite_");
        assert_eq!(keys::INPUT_OBJ_PREFIX, "bt_header_input_obj_");
    }

    #[test]
    fn top_level_keys_match_python() {
        assert_eq!(keys::NAME, "name");
        assert_eq!(keys::TIMEOUT, "timeout");
        assert_eq!(keys::HEADER_SIZE, "header_size");
        assert_eq!(keys::TOTAL_SIZE, "total_size");
        assert_eq!(keys::COMPUTED_BODY_HASH, "computed_body_hash");
    }
}
