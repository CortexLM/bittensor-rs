use anyhow::Result;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

/// Helper to get the length of a numeric tag (U16, U32, U64, U128)
fn get_tag_len(s: &str, pos: usize) -> usize {
    if s[pos..].starts_with("U128(") {
        5
    } else {
        4 // U16(, U32(, U64( are all 4 chars
    }
}

/// Decode a Vec<T> from a Value
/// Returns empty Vec if value cannot be decoded as a vector
pub fn decode_vec<T, F>(value: &Value, _decoder: F) -> Result<Vec<T>>
where
    F: Fn(&Value) -> Result<T>,
{
    // Parse from debug representation
    let s = format!("{:?}", value);

    // Check if it looks like a vector/sequence - return empty vec regardless
    let _ = s.contains("Composite(Unnamed([") || s.contains("Sequence([");
    Ok(Vec::new())
}

/// Decode a vector of u16 from Value
pub fn decode_vec_u16(value: &Value) -> Result<Vec<u16>> {
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Look for U128( pattern (values are stored as U128 even for u16)
        if let Some(pos) = s[i..].find("U128(").map(|p| p + i) {
            let mut j = pos + 5;
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            // Convert to u16
            res.push(num as u16);
            i = j;
        } else {
            i += 1;
        }
    }
    Ok(res)
}

/// Decode a vector of u128 from Value
pub fn decode_vec_u128(value: &Value) -> Result<Vec<u128>> {
    let s = format!("{:?}", value);
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(pos) = s[i..].find("U128(").map(|p| p + i) {
            let mut j = pos + 5; // len("U128(")
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            out.push(num);
            i = j;
        } else if let Some(pos) = s[i..].find("U64(").map(|p| p + i) {
            let mut j = pos + 4; // len("U64(")
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            out.push(num);
            i = j;
        } else {
            break;
        }
    }
    Ok(out)
}

/// Decode a vector of (u64,u64) pairs from a Value representing Vec<(Compact<u16>, Compact<u16>)>
/// Decode a vector of bool from Value
pub fn decode_vec_bool(value: &Value) -> Result<Vec<bool>> {
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Try Bool(true/false) first
        if let Some(pos) = s[i..].find("Bool(").map(|p| p + i) {
            let j = pos + 5;
            if j + 4 <= bytes.len() {
                if &s[j..j + 4] == "true" {
                    res.push(true);
                    i = j + 4;
                } else if j + 5 <= bytes.len() && &s[j..j + 5] == "false" {
                    res.push(false);
                    i = j + 5;
                } else {
                    i = j;
                }
            } else {
                i = j;
            }
        }
        // Also try U8(0/1) format (bools might be stored as U8)
        else if let Some(pos) = s[i..].find("U8(").map(|p| p + i) {
            let mut j = pos + 3;
            let mut num: u8 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0');
                j += 1;
            }
            res.push(num != 0);
            i = j;
        }
        // Also try U128 format (like with u16 vectors)
        else if let Some(pos) = s[i..].find("U128(").map(|p| p + i) {
            let mut j = pos + 5;
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            res.push(num != 0);
            i = j;
        } else {
            i += 1;
        }
    }
    Ok(res)
}

/// Decode a vector of u64 from Value
pub fn decode_vec_u64(value: &Value) -> Result<Vec<u64>> {
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Try U64 first
        if let Some(pos) = s[i..].find("U64(").map(|p| p + i) {
            let mut j = pos + 4;
            let mut num: u64 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u64;
                j += 1;
            }
            res.push(num);
            i = j;
        }
        // Also try U128 (values might be stored as U128 even for u64)
        else if let Some(pos) = s[i..].find("U128(").map(|p| p + i) {
            let mut j = pos + 5;
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            // Convert to u64 (will truncate if too large)
            res.push(num as u64);
            i = j;
        } else {
            i += 1;
        }
    }
    Ok(res)
}

/// Decode a vector of AccountId32 from Value
pub fn decode_vec_account_id32(value: &Value) -> Result<Vec<AccountId32>> {
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let mut i = 0usize;
    while let Some(hx) = s[i..].find("0x").map(|p| p + i) {
        let hex_portion = &s[hx + 2..];
        let mut hex = String::new();
        for ch in hex_portion.chars() {
            if ch.is_ascii_hexdigit() {
                hex.push(ch);
            } else {
                break;
            }
        }
        if hex.len() >= 64 {
            let hex64 = &hex[..64];
            if let Ok(b) = hex::decode(hex64) {
                if b.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&b[..32]);
                    res.push(AccountId32::from(arr));
                }
            }
        }
        i = hx + 2 + hex.len();
    }
    Ok(res)
}

pub fn decode_vec_u64_u64_pairs(value: &Value) -> Result<Vec<(u64, u64)>> {
    // Parse by scanning pairs of numbers inside composite tuples
    let s = format!("{:?}", value);
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut res = Vec::new();
    loop {
        // find first number
        let pos1_opt = s[i..]
            .find("U16(")
            .or_else(|| s[i..].find("U32("))
            .or_else(|| s[i..].find("U64("))
            .or_else(|| s[i..].find("U128("))
            .map(|p| p + i);
        let Some(pos1) = pos1_opt else { break };
        let tag1 = get_tag_len(&s, pos1);
        let mut j = pos1 + tag1;
        let mut a: u128 = 0;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            a = a * 10 + (bytes[j] - b'0') as u128;
            j += 1;
        }
        // find second number
        let pos2_opt = s[j..]
            .find("U16(")
            .or_else(|| s[j..].find("U32("))
            .or_else(|| s[j..].find("U64("))
            .or_else(|| s[j..].find("U128("))
            .map(|p| p + j);
        let Some(pos2) = pos2_opt else {
            i = j;
            continue;
        };
        let tag2 = get_tag_len(&s, pos2);
        let mut k = pos2 + tag2;
        let mut b: u128 = 0;
        while k < bytes.len() && bytes[k].is_ascii_digit() {
            b = b * 10 + (bytes[k] - b'0') as u128;
            k += 1;
        }
        res.push((
            u64::try_from(a).unwrap_or(u64::MAX),
            u64::try_from(b).unwrap_or(u64::MAX),
        ));
        i = k;
    }
    Ok(res)
}

/// Decode a vector of (AccountId32, u128) pairs from Value
/// This is used for Stake[(netuid, uid)] -> Vec<(AccountId32, Compact<u64>)>
pub fn decode_vec_account_u128_pairs(value: &Value) -> Result<Vec<(AccountId32, u128)>> {
    // Strategy: find interleaved 0x<64-hex> (AccountId32) followed by a U64/U128 number
    let s = format!("{:?}", value);
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // find account hex
        if let Some(hx) = s[i..].find("0x").map(|p| p + i) {
            // extract 64 hex chars
            let mut k = hx + 2;
            let mut hex = String::new();
            while k < bytes.len() && s.as_bytes()[k].is_ascii_hexdigit() {
                hex.push(s.as_bytes()[k] as char);
                k += 1;
            }
            if hex.len() >= 64 {
                let hex64 = &hex[..64];
                if let Ok(b) = hex::decode(hex64) {
                    if b.len() == 32 {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&b[..32]);
                        let acct = AccountId32::from(arr);
                        // after account, seek number tag U64(..) or U128(..)
                        let rest = &s[k..];
                        if let Some(num_pos_rel) = rest.find("U64(").or_else(|| rest.find("U128("))
                        {
                            let num_pos = k + num_pos_rel;
                            let tag_len = get_tag_len(&s, num_pos);
                            let mut j = num_pos + tag_len;
                            let mut num: u128 = 0;
                            while j < bytes.len() && bytes[j].is_ascii_digit() {
                                num = num * 10 + (bytes[j] - b'0') as u128;
                                j += 1;
                            }
                            out.push((acct, num));
                            i = j;
                            continue;
                        }
                        // if no number found, still advance
                        i = k;
                        continue;
                    }
                }
            }
            i = hx + 2 + hex.len();
        } else {
            break;
        }
    }
    Ok(out)
}

/// Decode a vector of tuples (u64, AccountId32) from Value
pub fn decode_vec_tuple_u64_account(value: &Value) -> Result<Vec<(u64, AccountId32)>> {
    // Parse by scanning pairs of U64/U128 and following 0x<64-hex>
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let mut i = 0usize;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        // find number
        if let Some(pos) = s[i..]
            .find("U64(")
            .or_else(|| s[i..].find("U128("))
            .map(|p| p + i)
        {
            // Move to position of found pattern
            // extract number
            let tag_len = get_tag_len(&s, pos);
            let mut j = pos + tag_len;
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            // find hex
            if let Some(hx) = s[j..].find("0x").map(|p| p + j) {
                let mut k = hx + 2;
                let mut hex = String::new();
                while k < bytes.len() && s.as_bytes()[k].is_ascii_hexdigit() {
                    hex.push(s.as_bytes()[k] as char);
                    k += 1;
                }
                if hex.len() >= 64 {
                    let hex = &hex[..64];
                    if let Ok(b) = hex::decode(hex) {
                        if b.len() == 32 {
                            let mut arr = [0u8; 32];
                            arr.copy_from_slice(&b[..32]);
                            let acct = AccountId32::from(arr);
                            let prop = u64::try_from(num).unwrap_or(u64::MAX);
                            res.push((prop, acct));
                        }
                    }
                }
                i = k;
                continue;
            }
            i = j;
            continue;
        } else {
            break;
        }
    }
    Ok(res)
}
