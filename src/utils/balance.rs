//! Balance type for handling TAO/RAO amounts
//!
//! This module provides a Balance type similar to the Python SDK's Balance class,
//! handling conversions between TAO and RAO (the smallest unit).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use crate::config::{RAO_PER_TAO, RAO_SYMBOL, TAO_SYMBOL};

/// Balance type representing an amount of TAO/RAO
///
/// Internally stores the value in RAO (the smallest unit).
/// 1 TAO = 1,000,000,000 RAO (10^9)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Balance {
    /// Amount in RAO (smallest unit)
    rao: u64,
}

impl Balance {
    /// Create a zero balance
    pub const fn zero() -> Self {
        Self { rao: 0 }
    }

    /// Create from RAO amount
    pub const fn from_rao(rao: u64) -> Self {
        Self { rao }
    }

    /// Create from TAO amount
    pub fn from_tao(tao: f64) -> Self {
        Self {
            rao: (tao * RAO_PER_TAO as f64) as u64,
        }
    }

    /// Get the RAO amount
    pub const fn rao(&self) -> u64 {
        self.rao
    }

    /// Get the TAO amount
    pub fn tao(&self) -> f64 {
        self.rao as f64 / RAO_PER_TAO as f64
    }

    /// Check if balance is zero
    pub const fn is_zero(&self) -> bool {
        self.rao == 0
    }

    /// Format as TAO with symbol
    pub fn format_tao(&self) -> String {
        format!("{}{:.9}", TAO_SYMBOL, self.tao())
    }

    /// Format as RAO with symbol
    pub fn format_rao(&self) -> String {
        format!("{}{}", RAO_SYMBOL, self.rao)
    }

    /// Convert to a human-readable string (auto-selects unit)
    pub fn to_human(&self) -> String {
        if self.rao >= RAO_PER_TAO {
            self.format_tao()
        } else {
            self.format_rao()
        }
    }

    /// Parse from string (supports both TAO and RAO formats)
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        
        // Remove symbol if present
        let s = s.trim_start_matches(TAO_SYMBOL).trim_start_matches(RAO_SYMBOL);
        
        // Try parsing as float (TAO)
        if let Ok(tao) = s.parse::<f64>() {
            return Some(Self::from_tao(tao));
        }
        
        // Try parsing as integer (RAO)
        if let Ok(rao) = s.parse::<u64>() {
            return Some(Self::from_rao(rao));
        }
        
        None
    }
}

impl fmt::Display for Balance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{:.4}", TAO_SYMBOL, self.tao())
    }
}

impl Add for Balance {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            rao: self.rao.saturating_add(other.rao),
        }
    }
}

impl AddAssign for Balance {
    fn add_assign(&mut self, other: Self) {
        self.rao = self.rao.saturating_add(other.rao);
    }
}

impl Sub for Balance {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            rao: self.rao.saturating_sub(other.rao),
        }
    }
}

impl SubAssign for Balance {
    fn sub_assign(&mut self, other: Self) {
        self.rao = self.rao.saturating_sub(other.rao);
    }
}

impl Mul<u64> for Balance {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self {
        Self {
            rao: self.rao.saturating_mul(rhs),
        }
    }
}

impl Mul<f64> for Balance {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self {
            rao: (self.rao as f64 * rhs) as u64,
        }
    }
}

impl Div<u64> for Balance {
    type Output = Self;

    fn div(self, rhs: u64) -> Self {
        Self {
            rao: self.rao / rhs,
        }
    }
}

impl From<u64> for Balance {
    fn from(rao: u64) -> Self {
        Self::from_rao(rao)
    }
}

impl From<Balance> for u64 {
    fn from(balance: Balance) -> u64 {
        balance.rao
    }
}

impl std::iter::Sum for Balance {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Balance::zero(), |acc, x| acc + x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_conversions() {
        let b = Balance::from_tao(1.0);
        assert_eq!(b.rao(), RAO_PER_TAO);
        assert!((b.tao() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_balance_arithmetic() {
        let a = Balance::from_tao(1.0);
        let b = Balance::from_tao(0.5);
        
        let sum = a + b;
        assert!((sum.tao() - 1.5).abs() < 1e-9);
        
        let diff = a - b;
        assert!((diff.tao() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_balance_display() {
        let b = Balance::from_tao(1.5);
        let s = b.to_string();
        assert!(s.contains("1.5"));
    }

    #[test]
    fn test_balance_parse() {
        let b = Balance::parse("1.5").unwrap();
        assert!((b.tao() - 1.5).abs() < 1e-9);
    }
}
