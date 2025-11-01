
/// Convert raw units (RAO) to TAO
/// 1 TAO = 1e9 RAO
pub fn rao_to_tao(rao: u128) -> f64 {
    rao as f64 / 1e9
}

/// Convert TAO to raw units (RAO)
pub fn tao_to_rao(tao: f64) -> u128 {
    (tao * 1e9) as u128
}

/// Create balance from RAO
pub fn from_rao(rao: u128) -> Balance {
    Balance { rao }
}

/// Create balance from TAO
pub fn from_tao(tao: f64) -> Balance {
    Balance {
        rao: tao_to_rao(tao),
    }
}

/// Balance representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Balance {
    /// Raw units (RAO)
    pub rao: u128,
}

impl Balance {
    /// Create from RAO
    pub fn from_rao(rao: u128) -> Self {
        Self { rao }
    }

    /// Create from TAO
    pub fn from_tao(tao: f64) -> Self {
        Self {
            rao: tao_to_rao(tao),
        }
    }

    /// Get as TAO
    pub fn as_tao(&self) -> f64 {
        rao_to_tao(self.rao)
    }

    /// Get as RAO
    pub fn as_rao(&self) -> u128 {
        self.rao
    }
}

impl std::ops::Add for Balance {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            rao: self.rao + other.rao,
        }
    }
}

impl std::ops::Sub for Balance {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            rao: self.rao.saturating_sub(other.rao),
        }
    }
}

impl std::fmt::Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.9} TAO", self.as_tao())
    }
}

