//! Balance utilities for TAO/RAO conversions
//! Matches Python SDK implementation in bittensor.utils.balance

use crate::core::constants::RAOPERTAO;

/// Convert raw units (RAO) to TAO
/// 1 TAO = RAOPERTAO RAO (exactly 1e9)
pub fn rao_to_tao(rao: u128) -> f64 {
    rao as f64 / RAOPERTAO as f64
}

/// Convert TAO to raw units (RAO)
/// Uses exact integer arithmetic: rao = int(tao * RAOPERTAO)
pub fn tao_to_rao(tao: f64) -> u128 {
    (tao * RAOPERTAO as f64) as u128
}

/// Create balance from RAO
pub fn from_rao(rao: u128) -> Balance {
    Balance::from_rao(rao)
}

/// Create balance from TAO
pub fn from_tao(tao: f64) -> Balance {
    Balance::from_tao(tao)
}

/// Balance representation with unit tracking
/// Matches Python SDK's Balance class
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Balance {
    /// Raw units (RAO) - internal representation uses u128 for exact precision
    pub rao: u128,
    /// Subnet ID for unit tracking (0 = TAO, non-zero = Alpha on that subnet)
    pub netuid: u16,
}

impl Balance {
    /// Create from RAO (default unit is TAO, netuid=0)
    pub fn from_rao(rao: u128) -> Self {
        Self { rao, netuid: 0 }
    }

    /// Create from RAO with specific netuid
    pub fn from_rao_with_netuid(rao: u128, netuid: u16) -> Self {
        Self { rao, netuid }
    }

    /// Create from TAO (default unit is TAO, netuid=0)
    pub fn from_tao(tao: f64) -> Self {
        Self {
            rao: tao_to_rao(tao),
            netuid: 0,
        }
    }

    /// Create from TAO with specific netuid
    pub fn from_tao_with_netuid(tao: f64, netuid: u16) -> Self {
        Self {
            rao: tao_to_rao(tao),
            netuid,
        }
    }

    /// Get as TAO (f64 representation for display)
    pub fn as_tao(&self) -> f64 {
        rao_to_tao(self.rao)
    }

    /// Get as RAO (u128 internal representation)
    pub fn as_rao(&self) -> u128 {
        self.rao
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

    /// Get the RAO unit symbol for this balance
    pub fn rao_unit(&self) -> &'static str {
        get_unit_symbol(self.netuid)
    }

    /// Check if this balance is for TAO (netuid=0)
    pub fn is_tao(&self) -> bool {
        self.netuid == 0
    }

    /// Check if this balance is for Alpha (netuid>0)
    pub fn is_alpha(&self) -> bool {
        self.netuid > 0
    }
}

impl std::ops::Add for Balance {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        // Allow arithmetic between same netuid or with TAO (netuid=0)
        // If either is TAO, result takes the other's netuid
        let result_netuid = if self.netuid == 0 {
            other.netuid
        } else {
            self.netuid
        };
        Self {
            rao: self.rao.saturating_add(other.rao),
            netuid: result_netuid,
        }
    }
}

impl std::ops::Sub for Balance {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let result_netuid = if self.netuid == 0 {
            other.netuid
        } else {
            self.netuid
        };
        Self {
            rao: self.rao.saturating_sub(other.rao),
            netuid: result_netuid,
        }
    }
}

impl std::ops::Mul<u128> for Balance {
    type Output = Self;
    fn mul(self, other: u128) -> Self {
        Self {
            rao: self.rao.saturating_mul(other),
            netuid: self.netuid,
        }
    }
}

impl std::ops::Div<u128> for Balance {
    type Output = Self;
    fn div(self, other: u128) -> Self {
        if other == 0 {
            Self {
                rao: 0,
                netuid: self.netuid,
            }
        } else {
            Self {
                rao: self.rao / other,
                netuid: self.netuid,
            }
        }
    }
}

impl std::fmt::Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = self.unit();
        if self.netuid == 0 {
            write!(f, "{}{:.9}", symbol, self.as_tao())
        } else {
            write!(f, "{:.9}{}", self.as_tao(), symbol)
        }
    }
}

/// Get unit symbol for a given netuid
/// Returns "τ" (TAO) for netuid=0, Greek letters for subnets
pub fn get_unit_symbol(netuid: u16) -> &'static str {
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
        // For higher netuids, return numeric format
        _ => "α", // Default to Alpha symbol for higher netuids
    }
}

/// Helper function to create a Balance from an int (RAO)
pub fn rao(amount: u128) -> Balance {
    Balance::from_rao(amount)
}

/// Helper function to create a Balance from a float (TAO)
pub fn tao(amount: f64) -> Balance {
    Balance::from_tao(amount)
}

/// Helper function to create a Balance from RAO with netuid
pub fn rao_with_netuid(amount: u128, netuid: u16) -> Balance {
    Balance::from_rao_with_netuid(amount, netuid)
}

/// Helper function to create a Balance from TAO with netuid
pub fn tao_with_netuid(amount: f64, netuid: u16) -> Balance {
    Balance::from_tao_with_netuid(amount, netuid)
}
