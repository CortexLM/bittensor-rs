use anyhow::Result;
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
use sp_core::sr25519;
use std::str::FromStr;

/// SS58 format constant for Bittensor (42 = "bt")
pub const SS58_FORMAT: u16 = 42;

/// Trait for converting types to SS58 address format
pub trait AccountId32ToSS58 {
    /// Convert to SS58 address string
    fn to_ss58(&self) -> String;
}

impl AccountId32ToSS58 for AccountId32 {
    fn to_ss58(&self) -> String {
        encode_ss58(self)
    }
}

impl AccountId32ToSS58 for sr25519::Public {
    fn to_ss58(&self) -> String {
        self.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT))
    }
}

/// Encode AccountId32 to SS58 string
pub fn encode_ss58(account: &AccountId32) -> String {
    account.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT))
}

/// Decode SS58 string to AccountId32
pub fn decode_ss58(ss58: &str) -> Result<AccountId32> {
    AccountId32::from_str(ss58)
        .or_else(|_| {
            let (account, _format) = AccountId32::from_ss58check_with_version(ss58)?;
            Ok(account)
        })
        .map_err(|e: sp_core::crypto::PublicError| {
            anyhow::anyhow!("Failed to decode SS58 address: {:?}", e)
        })
}

/// Validate SS58 address format
pub fn is_valid_ss58(ss58: &str) -> bool {
    decode_ss58(ss58).is_ok()
}

pub mod serde_account {
    use super::{decode_ss58, encode_ss58};
    use serde::{Deserialize, Deserializer, Serializer};
    use sp_core::crypto::AccountId32;

    pub fn serialize<S>(account: &AccountId32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode_ss58(account))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AccountId32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        decode_ss58(&value).map_err(serde::de::Error::custom)
    }
}

pub mod serde_account_vec {
    use super::{decode_ss58, encode_ss58};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use sp_core::crypto::AccountId32;

    pub fn serialize<S>(accounts: &[AccountId32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let values: Vec<String> = accounts.iter().map(encode_ss58).collect();
        values.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<AccountId32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = Vec::<String>::deserialize(deserializer)?;
        let mut accounts = Vec::with_capacity(values.len());
        for value in values {
            accounts.push(decode_ss58(&value).map_err(serde::de::Error::custom)?);
        }
        Ok(accounts)
    }
}

pub mod serde_account_map {
    use super::{decode_ss58, encode_ss58};
    use serde::ser::SerializeMap;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use sp_core::crypto::AccountId32;
    use std::collections::HashMap;

    pub fn serialize<S, V>(map: &HashMap<AccountId32, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        V: Serialize,
    {
        let mut state = serializer.serialize_map(Some(map.len()))?;
        for (key, value) in map {
            state.serialize_entry(&encode_ss58(key), value)?;
        }
        state.end()
    }

    pub fn deserialize<'de, D, V>(deserializer: D) -> Result<HashMap<AccountId32, V>, D::Error>
    where
        D: Deserializer<'de>,
        V: Deserialize<'de>,
    {
        let raw = HashMap::<String, V>::deserialize(deserializer)?;
        let mut map = HashMap::with_capacity(raw.len());
        for (key, value) in raw {
            let account = decode_ss58(&key).map_err(serde::de::Error::custom)?;
            map.insert(account, value);
        }
        Ok(map)
    }
}
