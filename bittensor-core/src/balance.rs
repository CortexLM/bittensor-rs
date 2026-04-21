use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// 1 TAO = 10^9 rao. The on-chain unit is rao; TAO is the human-readable form.
const RAO_PER_TAO: u64 = 1_000_000_000;

/// Fixed-point balance type storing value in rao (10^-9 TAO).
///
/// All on-chain balances are integers in rao. This type wraps that
/// representation and provides conversion to/from TAO with 9 decimal
/// places, matching the Python SDK's `Balance` class.
///
/// # Examples
/// ```
/// use bittensor_core::balance::Balance;
///
/// let one_tao = Balance::ONE_TAO;
/// assert_eq!(one_tao.to_rao(), 1_000_000_000);
/// assert_eq!(one_tao.to_tao(), 1.0);
///
/// let half = Balance::from_tao(0.5);
/// assert_eq!(half.to_rao(), 500_000_000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct Balance {
    rao: u64,
}

impl Balance {
    /// Zero balance.
    pub const ZERO: Balance = Balance { rao: 0 };
    /// Exactly 1 TAO (10^9 rao).
    pub const ONE_TAO: Balance = Balance { rao: RAO_PER_TAO };

    /// Create from a raw rao value.
    pub fn from_rao(rao: u64) -> Self {
        Self { rao }
    }

    /// Create from a TAO float. Rounded to the nearest rao.
    pub fn from_tao(tao: f64) -> Self {
        let rao = (tao * RAO_PER_TAO as f64).round() as u64;
        Self { rao }
    }

    /// Return the raw rao value.
    pub fn to_rao(self) -> u64 {
        self.rao
    }

    /// Return the value as a TAO float.
    pub fn to_tao(self) -> f64 {
        self.rao as f64 / RAO_PER_TAO as f64
    }

    /// Checked addition. Returns `None` on overflow.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.rao.checked_add(other.rao).map(|rao| Self { rao })
    }

    /// Checked subtraction. Returns `None` on underflow.
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.rao.checked_sub(other.rao).map(|rao| Self { rao })
    }

    /// Checked scalar multiplication. Returns `None` on overflow.
    pub fn checked_mul(self, scalar: u64) -> Option<Self> {
        self.rao.checked_mul(scalar).map(|rao| Self { rao })
    }

    /// Checked division. Returns the integer quotient or `None` if `other` is zero.
    pub fn checked_div(self, other: Self) -> Option<u64> {
        if other.rao == 0 {
            return None;
        }
        Some(self.rao / other.rao)
    }

    /// Saturating addition. Clamps at `u64::MAX` rao on overflow.
    pub fn saturating_add(self, other: Self) -> Self {
        Self { rao: self.rao.saturating_add(other.rao) }
    }

    /// Saturating subtraction. Clamps at zero on underflow.
    pub fn saturating_sub(self, other: Self) -> Self {
        Self { rao: self.rao.saturating_sub(other.rao) }
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Balance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tao = self.to_tao();
        write!(f, "{tao:.9}")
    }
}

impl FromStr for Balance {
    type Err = crate::error::BittensorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tao: f64 = s.parse().map_err(|_| {
            crate::error::BittensorError::Balance(format!("invalid balance string: {s}"))
        })?;
        Ok(Self::from_tao(tao))
    }
}

impl Add for Balance {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs).expect("balance addition overflow")
    }
}

impl Sub for Balance {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).expect("balance subtraction underflow")
    }
}

impl Mul<u64> for Balance {
    type Output = Self;

    fn mul(self, scalar: u64) -> Self {
        self.checked_mul(scalar).expect("balance multiplication overflow")
    }
}

impl Div for Balance {
    type Output = u64;

    fn div(self, rhs: Self) -> u64 {
        self.checked_div(rhs).expect("balance division by zero")
    }
}

impl Serialize for Balance {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Balance {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_rao_identity() {
        let b = Balance::from_rao(1_500_000_000);
        assert_eq!(b.to_rao(), 1_500_000_000);
    }

    #[test]
    fn from_tao_roundtrip() {
        let b = Balance::from_tao(1.5);
        assert_eq!(b.to_rao(), 1_500_000_000);
        let result = b.to_tao();
        let diff = (result - 1.5).abs();
        assert!(diff < 1e-10, "expected ~1.5, got {result}");
    }

    #[test]
    fn one_tao_equals_billion_rao() {
        assert_eq!(Balance::ONE_TAO.to_rao(), 1_000_000_000);
    }

    #[test]
    fn zero_balance() {
        assert_eq!(Balance::ZERO.to_rao(), 0);
        assert_eq!(Balance::ZERO.to_tao(), 0.0);
    }

    #[test]
    fn display_nine_decimal_places() {
        let b = Balance::from_tao(1.5);
        assert_eq!(format!("{b}"), "1.500000000");
    }

    #[test]
    fn display_small_balance() {
        let b = Balance::from_rao(1);
        assert_eq!(format!("{b}"), "0.000000001");
    }

    #[test]
    fn display_zero() {
        assert_eq!(format!("{}", Balance::ZERO), "0.000000000");
    }

    #[test]
    fn from_str_valid() {
        let b: Balance = "1.5".parse().expect("parse");
        assert_eq!(b, Balance::from_tao(1.5));
    }

    #[test]
    fn from_str_integer() {
        let b: Balance = "100".parse().expect("parse");
        assert_eq!(b, Balance::from_tao(100.0));
    }

    #[test]
    fn from_str_invalid() {
        let result = "abc".parse::<Balance>();
        assert!(result.is_err());
    }

    #[test]
    fn add_balances() {
        let a = Balance::from_tao(1.0);
        let b = Balance::from_tao(2.5);
        let sum = a + b;
        assert_eq!(sum, Balance::from_tao(3.5));
    }

    #[test]
    fn sub_balances() {
        let a = Balance::from_tao(5.0);
        let b = Balance::from_tao(2.0);
        let diff = a - b;
        assert_eq!(diff, Balance::from_tao(3.0));
    }

    #[test]
    fn mul_by_scalar() {
        let a = Balance::from_tao(2.0);
        let result = a * 3;
        assert_eq!(result, Balance::from_tao(6.0));
    }

    #[test]
    fn div_balances() {
        let a = Balance::from_tao(10.0);
        let b = Balance::from_tao(2.0);
        assert_eq!(a / b, 5);
    }

    #[test]
    fn checked_add_overflow() {
        let max = Balance::from_rao(u64::MAX);
        assert!(max.checked_add(Balance::from_rao(1)).is_none());
    }

    #[test]
    fn checked_sub_underflow() {
        let a = Balance::from_rao(5);
        let b = Balance::from_rao(10);
        assert!(a.checked_sub(b).is_none());
    }

    #[test]
    fn checked_mul_overflow() {
        let a = Balance::from_rao(u64::MAX);
        assert!(a.checked_mul(2).is_none());
    }

    #[test]
    fn checked_div_by_zero() {
        let a = Balance::from_rao(100);
        assert!(a.checked_div(Balance::ZERO).is_none());
    }

    #[test]
    fn saturating_add() {
        let max = Balance::from_rao(u64::MAX);
        let result = max.saturating_add(Balance::from_rao(1));
        assert_eq!(result, max);
    }

    #[test]
    fn saturating_sub() {
        let a = Balance::from_rao(5);
        let b = Balance::from_rao(10);
        let result = a.saturating_sub(b);
        assert_eq!(result, Balance::ZERO);
    }

    #[test]
    fn ordering() {
        let a = Balance::from_tao(1.0);
        let b = Balance::from_tao(2.0);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn serialization_roundtrip() {
        let b = Balance::from_tao(1.5);
        let json = serde_json::to_string(&b).expect("serialize");
        let deserialized: Balance = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, deserialized);
    }

    #[test]
    fn scale_codec_roundtrip() {
        let b = Balance::from_tao(42.0);
        let encoded = b.encode();
        let decoded: Balance = Decode::decode(&mut &encoded[..]).expect("decode");
        assert_eq!(b, decoded);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(Balance::default(), Balance::ZERO);
    }
}
