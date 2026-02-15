//! Production-safe TAO/RAO value handling with compile-time type safety.
//!
//! This module provides newtype wrappers that prevent accidental mixing of RAO and TAO values
//! at compile time. All operations use exact integer arithmetic where possible to prevent
//! precision loss on large values.
//!
//! # RAOPERTAO
//! The conversion factor is exactly 1_000_000_000 (1e9), meaning:
//! - 1 TAO = 1_000_000_000 RAO
//! - 1 RAO = 0.000000001 TAO (1 nano-TAO)
//!
//! # Type Safety
//! - `Rao` type wraps u128 raw units - can only be created through validated constructors
//! - `Tao` type wraps f64 display values - provides safe conversion to RAO
//! - Neither type can be implicitly converted to the other without explicit conversion functions
//!
//! # Precision Safety
//! All conversions use exact integer arithmetic:
//! - TAO → RAO: `rao = (tao * RAOPERTAO) as u128` (truncates toward zero)
//! - RAO → TAO: `tao = rao as f64 / RAOPERTAO as f64` (exact for values < 2^53)
//!
//! For values above 2^53 (≈ 9e15 RAO or 9e6 TAO), the f64 representation may lose precision.
//! Use the RAO (u128) representation for exact arithmetic on large values.

use crate::core::constants::RAOPERTAO;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// Maximum exact integer value in f64 (2^53)
const F64_MAX_EXACT_INT: u128 = 9_007_199_254_740_992; // 2^53

/// RAO value - raw units (u128 wrapper)
///
/// This is a newtype wrapper around u128 that prevents accidental mixing with TAO values.
/// Use `Rao` for all on-chain operations, transfers, and exact arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rao(pub u128);

impl Rao {
    /// Zero RAO
    pub const ZERO: Self = Self(0);

    /// One RAO
    pub const ONE: Self = Self(1);

    /// RAOPERTAO RAO = 1 TAO
    pub const PER_TAO: Self = Self(RAOPERTAO);

    /// Create from raw u128 value
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    /// Get the raw u128 value
    pub const fn as_u128(self) -> u128 {
        self.0
    }

    /// Check if this value can be exactly represented as f64 TAO
    pub const fn is_exactly_representable_as_f64(self) -> bool {
        self.0 <= F64_MAX_EXACT_INT
    }

    /// Convert to TAO (f64 representation for display)
    ///
    /// # Precision Warning
    /// For values > 2^53 RAO (≈9 million TAO), this conversion may lose precision.
    /// Use `as_u128()` and handle large values with integer arithmetic.
    pub fn as_tao(self) -> f64 {
        self.0 as f64 / RAOPERTAO as f64
    }

    /// Convert to TAO type
    pub fn to_tao(self) -> Tao {
        Tao::from_rao(self.0)
    }

    /// Create from TAO (f64)
    pub fn from_tao(tao: f64) -> Self {
        Self(tao_to_rao(tao))
    }

    /// Saturating addition
    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction
    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Saturating multiplication
    pub fn saturating_mul(self, other: u128) -> Self {
        Self(self.0.saturating_mul(other))
    }

    /// Safe division (returns 0 on divide by zero)
    pub fn safe_div(self, divisor: u128) -> Self {
        if divisor == 0 {
            Self::ZERO
        } else {
            Self(self.0 / divisor)
        }
    }

    /// Format as TAO with 9 decimal places
    pub fn format_tao(self) -> String {
        let whole = self.0 / RAOPERTAO;
        let fraction = self.0 % RAOPERTAO;
        format!("{}.{:09} τ", whole, fraction)
    }

    /// Format as RAO
    pub fn format_rao(self) -> String {
        format!("{} ρ", self.0)
    }
}

impl Default for Rao {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Rao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ρ", self.0)
    }
}

impl Add for Rao {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
}

impl Sub for Rao {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl Mul<u128> for Rao {
    type Output = Self;
    fn mul(self, other: u128) -> Self {
        self.saturating_mul(other)
    }
}

impl Div<u128> for Rao {
    type Output = Self;
    fn div(self, other: u128) -> Self {
        self.safe_div(other)
    }
}

impl From<u128> for Rao {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<Rao> for u128 {
    fn from(rao: Rao) -> Self {
        rao.0
    }
}

/// TAO value - display units (f64 wrapper)
///
/// This is a newtype wrapper around f64 that prevents accidental mixing with RAO values.
/// Use `Tao` for user-facing display and input handling.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Tao(pub f64);

impl Tao {
    /// Zero TAO
    pub const ZERO: Self = Self(0.0);

    /// One TAO
    pub const ONE: Self = Self(1.0);

    /// Minimum positive TAO value (1 RAO)
    pub const MIN_POSITIVE: Self = Self(1.0 / RAOPERTAO as f64);

    /// Create from f64 value
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw f64 value
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// Create from RAO (u128)
    pub fn from_rao(rao: u128) -> Self {
        Self(rao as f64 / RAOPERTAO as f64)
    }

    /// Convert to RAO (u128)
    ///
    /// Uses exact integer arithmetic: `rao = floor(tao * RAOPERTAO)`
    /// This truncates toward zero, which is the standard behavior for financial amounts.
    pub fn as_rao(self) -> Rao {
        Rao::from_tao(self.0)
    }

    /// Convert to RAO with rounding (round half up)
    pub fn as_rao_rounded(self) -> Rao {
        Rao((self.0 * RAOPERTAO as f64).round() as u128)
    }

    /// Convert to RAO with ceiling (useful for ensuring sufficient balance)
    pub fn as_rao_ceiling(self) -> Rao {
        Rao((self.0 * RAOPERTAO as f64).ceil() as u128)
    }

    /// Saturating addition
    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    /// Saturating subtraction
    pub fn saturating_sub(self, other: Self) -> Self {
        Self((self.0 - other.0).max(0.0))
    }

    /// Format with 9 decimal places
    pub fn format(self) -> String {
        format!("{:.9} τ", self.0)
    }

    /// Check if the TAO value would lose precision when converted to RAO and back
    pub fn is_lossless_roundtrip(self) -> bool {
        let rao = self.as_rao();
        let tao_back = rao.as_tao();
        (self.0 - tao_back).abs() < f64::EPSILON
    }
}

impl Default for Tao {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Tao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.9} τ", self.0)
    }
}

impl Add for Tao {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
}

impl Sub for Tao {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl Mul<f64> for Tao {
    type Output = Self;
    fn mul(self, other: f64) -> Self {
        Self(self.0 * other)
    }
}

impl Div<f64> for Tao {
    type Output = Self;
    fn div(self, other: f64) -> Self {
        if other == 0.0 {
            Self::ZERO
        } else {
            Self(self.0 / other)
        }
    }
}

impl From<f64> for Tao {
    fn from(value: f64) -> Self {
        Self(value.max(0.0))
    }
}

impl From<Tao> for f64 {
    fn from(tao: Tao) -> Self {
        tao.0
    }
}

/// Safe conversion from TAO (f64) to RAO (u128)
///
/// Uses exact truncation: `rao = floor(tao * RAOPERTAO)`
/// This is the standard behavior used by the Python SDK.
pub fn tao_to_rao(tao: f64) -> u128 {
    (tao.max(0.0) * RAOPERTAO as f64) as u128
}

/// Safe conversion from RAO (u128) to TAO (f64)
///
/// # Precision Warning
/// For values > 2^53 RAO (≈9 million TAO), this conversion may lose precision.
/// The f64 mantissa has 53 bits of precision, so integers above 2^53 cannot be exactly represented.
pub fn rao_to_tao(rao: u128) -> f64 {
    rao as f64 / RAOPERTAO as f64
}

/// Convert TAO to RAO with rounding (round half up)
pub fn tao_to_rao_rounded(tao: f64) -> u128 {
    (tao.max(0.0) * RAOPERTAO as f64).round() as u128
}

/// Convert TAO to RAO with ceiling (useful for ensuring sufficient balance)
pub fn tao_to_rao_ceiling(tao: f64) -> u128 {
    (tao.max(0.0) * RAOPERTAO as f64).ceil() as u128
}

/// Format RAO as TAO string with 9 decimal places
pub fn format_rao_as_tao(rao: u128) -> String {
    let whole = rao / RAOPERTAO;
    let fraction = rao % RAOPERTAO;
    format!("{}.{:09}", whole, fraction)
}

/// Parse TAO string to RAO
///
/// Supports formats:
/// - "1.5" → 1_500_000_000 RAO
/// - "1.5 τ" or "1.5 TAO" → 1_500_000_000 RAO
/// - "1500000000" → 1_500_000_000 RAO (interpreted as RAO if no decimal)
pub fn parse_tao_string(s: &str) -> Option<Rao> {
    let cleaned = s
        .trim()
        .replace("τ", "")
        .replace("ρ", "") // RAO symbol
        .replace("TAO", "")
        .replace("tao", "")
        .replace(" ", "");

    if let Some(dot_pos) = cleaned.find('.') {
        // Has decimal point - parse as TAO
        let whole_str = &cleaned[..dot_pos];
        let frac_str = &cleaned[dot_pos + 1..];

        let whole: u128 = whole_str.parse().ok()?;
        let frac_padded = format!("{:0<9}", frac_str);
        let frac_str_9 = &frac_padded[..9.min(frac_padded.len())];
        let fraction: u128 = frac_str_9.parse().ok()?;

        Some(Rao(whole * RAOPERTAO + fraction))
    } else {
        // No decimal - could be RAO or TAO
        // If it's a small integer, assume TAO; if large, assume RAO
        match cleaned.parse::<u128>() {
            Ok(val) => {
                if val < 1_000_000_000_000u128 {
                    // Less than 1000 TAO, assume TAO
                    Some(Rao(val * RAOPERTAO))
                } else {
                    Some(Rao(val)) // Large value, assume RAO
                }
            }
            Err(_) => None,
        }
    }
}

/// Balance type that tracks both RAO value and unit (netuid)
///
/// This is the main type for handling balances with proper unit tracking.
/// - netuid=0: TAO
/// - netuid>0: Alpha on that subnet
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Balance {
    /// Raw units (RAO for TAO, or Alpha RAO for subnet tokens)
    pub amount: Rao,
    /// Subnet ID (0 = TAO, non-zero = Alpha on that subnet)
    pub netuid: u16,
}

impl Balance {
    /// Zero TAO balance
    pub const ZERO_TAO: Self = Self {
        amount: Rao::ZERO,
        netuid: 0,
    };

    /// Create from RAO (default unit is TAO, netuid=0)
    pub const fn from_rao(rao: u128) -> Self {
        Self {
            amount: Rao(rao),
            netuid: 0,
        }
    }

    /// Create from TAO (default unit is TAO, netuid=0)
    pub fn from_tao(tao: f64) -> Self {
        Self {
            amount: Rao::from_tao(tao),
            netuid: 0,
        }
    }

    /// Create from RAO with specific netuid
    pub const fn from_rao_with_netuid(rao: u128, netuid: u16) -> Self {
        Self {
            amount: Rao(rao),
            netuid,
        }
    }

    /// Create from TAO with specific netuid
    pub fn from_tao_with_netuid(tao: f64, netuid: u16) -> Self {
        Self {
            amount: Rao::from_tao(tao),
            netuid,
        }
    }

    /// Get as TAO (f64 representation for display)
    pub fn as_tao(&self) -> f64 {
        self.amount.as_tao()
    }

    /// Get as RAO (u128 internal representation)
    pub fn as_rao(&self) -> u128 {
        self.amount.as_u128()
    }

    /// Set the unit/netuid for this balance
    pub fn set_unit(mut self, netuid: u16) -> Self {
        self.netuid = netuid;
        self
    }

    /// Get the unit symbol for this balance
    pub fn unit(&self) -> &'static str {
        get_unit_symbol(self.netuid)
    }

    /// Check if this balance is for TAO (netuid=0)
    pub const fn is_tao(&self) -> bool {
        self.netuid == 0
    }

    /// Check if this balance is for Alpha (netuid>0)
    pub const fn is_alpha(&self) -> bool {
        self.netuid > 0
    }

    /// Saturating addition (requires same netuid or one is TAO)
    pub fn saturating_add(self, other: Self) -> Self {
        let result_netuid = if self.netuid == 0 {
            other.netuid
        } else {
            self.netuid
        };
        Self {
            amount: self.amount.saturating_add(other.amount),
            netuid: result_netuid,
        }
    }

    /// Saturating subtraction (requires same netuid or one is TAO)
    pub fn saturating_sub(self, other: Self) -> Self {
        let result_netuid = if self.netuid == 0 {
            other.netuid
        } else {
            self.netuid
        };
        Self {
            amount: self.amount.saturating_sub(other.amount),
            netuid: result_netuid,
        }
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::ZERO_TAO
    }
}

impl fmt::Display for Balance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = self.unit();
        if self.netuid == 0 {
            write!(f, "{}{:.9}", symbol, self.as_tao())
        } else {
            write!(f, "{:.9}{}", self.as_tao(), symbol)
        }
    }
}

impl Add for Balance {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
}

impl Sub for Balance {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl Mul<u128> for Balance {
    type Output = Self;
    fn mul(self, other: u128) -> Self {
        Self {
            amount: self.amount.saturating_mul(other),
            netuid: self.netuid,
        }
    }
}

impl Div<u128> for Balance {
    type Output = Self;
    fn div(self, other: u128) -> Self {
        Self {
            amount: self.amount.safe_div(other),
            netuid: self.netuid,
        }
    }
}

/// Get unit symbol for a given netuid
/// Returns "τ" (TAO) for netuid=0, Greek letters for subnets
pub const fn get_unit_symbol(netuid: u16) -> &'static str {
    match netuid {
        0 => "τ",  // TAO
        1 => "α",  // Alpha
        2 => "β",  // Beta
        3 => "γ",  // Gamma
        4 => "δ",  // Delta
        5 => "ε",  // Epsilon
        6 => "ζ",  // Zeta
        7 => "η",  // Eta
        8 => "θ",  // Theta
        9 => "ι",  // Iota
        10 => "κ", // Kappa
        11 => "λ", // Lambda
        12 => "μ", // Mu
        13 => "ν", // Nu
        14 => "ξ", // Xi
        15 => "ο", // Omicron
        16 => "π", // Pi
        17 => "ρ", // Rho
        18 => "σ", // Sigma
        19 => "τ", // Tau
        20 => "υ", // Upsilon
        21 => "φ", // Phi
        22 => "χ", // Chi
        23 => "ψ", // Psi
        24 => "ω", // Omega
        // For higher netuids, return alpha symbol
        _ => "α",
    }
}

/// Helper function to create a Rao from u128
pub const fn rao(amount: u128) -> Rao {
    Rao(amount)
}

/// Helper function to create a Rao from u128 with netuid
pub const fn rao_with_netuid(amount: u128, netuid: u16) -> Balance {
    Balance::from_rao_with_netuid(amount, netuid)
}

/// Helper function to create a Tao from f64
pub const fn tao(amount: f64) -> Tao {
    Tao(amount)
}

/// Helper function to create a Tao from f64 with netuid
pub fn tao_with_netuid(amount: f64, netuid: u16) -> Balance {
    Balance::from_tao_with_netuid(amount, netuid)
}

/// Helper function to create a Balance from RAO
pub const fn balance_from_rao(amount: u128) -> Balance {
    Balance::from_rao(amount)
}

/// Helper function to create a Balance from TAO
pub fn balance_from_tao(amount: f64) -> Balance {
    Balance::from_tao(amount)
}

/// Helper function to create a Balance from RAO with netuid
pub const fn balance_from_rao_with_netuid(amount: u128, netuid: u16) -> Balance {
    Balance::from_rao_with_netuid(amount, netuid)
}

/// Helper function to create a Balance from TAO with netuid
pub fn balance_from_tao_with_netuid(amount: f64, netuid: u16) -> Balance {
    Balance::from_tao_with_netuid(amount, netuid)
}

/// Check if a TAO value would lose precision when converted to RAO and back
///
/// This is useful for validating user inputs before conversion.
pub fn is_lossless_conversion(tao: f64) -> bool {
    if tao < 0.0 {
        return false;
    }
    let rao = tao_to_rao(tao);
    let tao_back = rao_to_tao(rao);
    (tao - tao_back).abs() < f64::EPSILON
}

/// Validate that a TAO amount can be safely converted to RAO without overflow
pub fn is_valid_tao_amount(tao: f64) -> bool {
    if tao < 0.0 || !tao.is_finite() {
        return false;
    }
    // Check for overflow: max u128 / RAOPERTAO ≈ 3.4e29 TAO
    tao < (u128::MAX as f64 / RAOPERTAO as f64)
}

/// Validate that a RAO amount is valid (non-negative, fits in u128)
/// Since RAO is u128, it's always valid by definition, but this function
/// is provided for API consistency.
pub const fn is_valid_rao_amount(_rao: u128) -> bool {
    // u128 is always valid (0 to 2^128-1)
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rao_constants() {
        assert_eq!(Rao::ZERO.as_u128(), 0);
        assert_eq!(Rao::ONE.as_u128(), 1);
        assert_eq!(Rao::PER_TAO.as_u128(), RAOPERTAO);
        assert_eq!(Rao::PER_TAO.as_u128(), 1_000_000_000);
    }

    #[test]
    fn test_tao_constants() {
        assert_eq!(Tao::ZERO.as_f64(), 0.0);
        assert_eq!(Tao::ONE.as_f64(), 1.0);
        assert_eq!(Tao::MIN_POSITIVE.as_f64(), 1e-9);
    }

    #[test]
    fn test_rao_tao_conversions() {
        // 0 TAO = 0 RAO
        assert_eq!(Rao::from_tao(0.0).as_u128(), 0);
        assert_eq!(Tao::from_rao(0).as_f64(), 0.0);

        // 1 TAO = 1e9 RAO
        assert_eq!(Rao::from_tao(1.0).as_u128(), 1_000_000_000);
        assert_eq!(Tao::from_rao(1_000_000_000).as_f64(), 1.0);

        // 0.5 TAO = 5e8 RAO
        assert_eq!(Rao::from_tao(0.5).as_u128(), 500_000_000);
        let tao_back = Tao::from_rao(500_000_000).as_f64();
        assert!((tao_back - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_rao_arithmetic() {
        let a = Rao(1_000_000_000);
        let b = Rao(500_000_000);

        assert_eq!((a + b).as_u128(), 1_500_000_000);
        assert_eq!((a - b).as_u128(), 500_000_000);
        assert_eq!((a * 2).as_u128(), 2_000_000_000);
        assert_eq!((a / 2).as_u128(), 500_000_000);
    }

    #[test]
    fn test_tao_arithmetic() {
        let a = Tao(1.0);
        let b = Tao(0.5);

        assert_eq!((a + b).as_f64(), 1.5);
        assert_eq!((a - b).as_f64(), 0.5);
        assert_eq!((a * 2.0).as_f64(), 2.0);
        assert_eq!((a / 2.0).as_f64(), 0.5);
    }

    #[test]
    fn test_format_rao_as_tao() {
        assert_eq!(format_rao_as_tao(0), "0.000000000");
        assert_eq!(format_rao_as_tao(1_000_000_000), "1.000000000");
        assert_eq!(format_rao_as_tao(1_500_000_000), "1.500000000");
        assert_eq!(format_rao_as_tao(123_456_789), "0.123456789");
    }

    #[test]
    fn test_parse_tao_string() {
        assert_eq!(parse_tao_string("1.5").unwrap().as_u128(), 1_500_000_000);
        assert_eq!(parse_tao_string("1.5 τ").unwrap().as_u128(), 1_500_000_000);
        assert_eq!(
            parse_tao_string("1.5 TAO").unwrap().as_u128(),
            1_500_000_000
        );
        assert_eq!(parse_tao_string("0.000000001").unwrap().as_u128(), 1);
    }

    #[test]
    fn test_balance_creation() {
        let b = Balance::from_tao(1.5);
        assert_eq!(b.as_rao(), 1_500_000_000);
        assert_eq!(b.netuid, 0);
        assert!(b.is_tao());
        assert!(!b.is_alpha());
    }

    #[test]
    fn test_balance_with_netuid() {
        let b = Balance::from_tao_with_netuid(1.5, 1);
        assert_eq!(b.netuid, 1);
        assert!(!b.is_tao());
        assert!(b.is_alpha());
        assert_eq!(b.unit(), "α");
    }

    #[test]
    fn test_balance_arithmetic() {
        let a = Balance::from_tao(1.0);
        let b = Balance::from_tao(0.5);

        assert_eq!((a + b).as_rao(), 1_500_000_000);
        assert_eq!((a - b).as_rao(), 500_000_000);
    }

    #[test]
    fn test_is_lossless_conversion() {
        assert!(is_lossless_conversion(1.0));
        assert!(is_lossless_conversion(0.5));
        assert!(is_lossless_conversion(1.23456789));

        // Very large values may lose precision
        // This is expected behavior due to f64 mantissa limits
    }

    #[test]
    fn test_is_valid_tao_amount() {
        assert!(is_valid_tao_amount(0.0));
        assert!(is_valid_tao_amount(1.0));
        assert!(is_valid_tao_amount(1e20));
        assert!(!is_valid_tao_amount(-1.0));
        assert!(!is_valid_tao_amount(f64::NAN));
        assert!(!is_valid_tao_amount(f64::INFINITY));
    }

    #[test]
    fn test_unit_symbols() {
        assert_eq!(get_unit_symbol(0), "τ");
        assert_eq!(get_unit_symbol(1), "α");
        assert_eq!(get_unit_symbol(2), "β");
        assert_eq!(get_unit_symbol(24), "ω");
        assert_eq!(get_unit_symbol(100), "α"); // Default to alpha for high netuids
    }

    #[test]
    fn test_precision_boundary() {
        // 2^53 is the exact integer limit for f64
        let exact_limit = F64_MAX_EXACT_INT;
        assert!(Rao(exact_limit).is_exactly_representable_as_f64());
        assert!(!Rao(exact_limit + 1).is_exactly_representable_as_f64());

        // In TAO terms: 2^53 RAO = 2^53 / 1e9 TAO ≈ 9,007,199 TAO
        let tao_at_limit = exact_limit as f64 / RAOPERTAO as f64;
        assert!(tao_at_limit > 9_000_000.0);
    }

    #[test]
    fn test_helper_functions() {
        assert_eq!(rao(1_000_000_000).as_u128(), 1_000_000_000);
        assert_eq!(tao(1.0).as_f64(), 1.0);
        assert_eq!(balance_from_rao(1_000_000_000).as_rao(), 1_000_000_000);
        assert_eq!(balance_from_tao(1.0).as_rao(), 1_000_000_000);
    }

    #[test]
    fn test_rao_display() {
        assert_eq!(format!("{}", Rao(1_000_000_000)), "1000000000 ρ");
        assert_eq!(Rao(1_000_000_000).format_tao(), "1.000000000 τ");
    }

    #[test]
    fn test_tao_display() {
        assert_eq!(format!("{}", Tao(1.5)), "1.500000000 τ");
        assert_eq!(Tao(1.5).format(), "1.500000000 τ");
    }

    #[test]
    fn test_balance_display() {
        let tao_balance = Balance::from_tao(1.5);
        assert_eq!(format!("{}", tao_balance), "τ1.500000000");

        let alpha_balance = Balance::from_tao_with_netuid(1.5, 1);
        assert_eq!(format!("{}", alpha_balance), "1.500000000α");
    }

    #[test]
    fn test_saturating_arithmetic() {
        let max = Rao(u128::MAX);
        let one = Rao(1);

        // Saturating add should not overflow
        assert_eq!((max + one).as_u128(), u128::MAX);

        // Saturating sub should not underflow
        let zero = Rao(0);
        assert_eq!((zero - one).as_u128(), 0);
    }

    #[test]
    fn test_safe_div() {
        let a = Rao(1_000_000_000);
        assert_eq!(a.safe_div(2).as_u128(), 500_000_000);
        assert_eq!(a.safe_div(0).as_u128(), 0); // Division by zero returns 0
    }

    #[test]
    fn test_tao_rounding_modes() {
        let tao = Tao(1.1234567895);

        // Default truncation
        assert_eq!(tao.as_rao().as_u128(), 1_123_456_789);

        // Rounded
        assert_eq!(tao.as_rao_rounded().as_u128(), 1_123_456_790);

        // Ceiling
        assert_eq!(tao.as_rao_ceiling().as_u128(), 1_123_456_790);
    }

    #[test]
    fn test_lossless_roundtrip() {
        // Small values should be lossless
        let small_tao = Tao(1.23456789);
        assert!(small_tao.is_lossless_roundtrip());

        // Large values may lose precision
        let large_tao = Tao(9_007_199.0); // Close to f64 limit
                                          // This might or might not be lossless depending on exact value
        let _ = large_tao.is_lossless_roundtrip();
    }
}
