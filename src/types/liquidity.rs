use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub id: u64,
    pub price_low_rao: u128,
    pub price_high_rao: u128,
    pub liquidity_rao: u128,
    pub fees_tao_rao: u128,
    pub fees_alpha_rao: u128,
    pub netuid: u16,
}

impl LiquidityPosition {
    /// Convert a liquidity position to token amounts (Alpha RAO, TAO RAO)
    pub fn to_token_amounts(&self, current_subnet_price_rao: u128) -> (u128, u128) {
        let sqrt_price_low = (self.price_low_rao as f64).sqrt();
        let sqrt_price_high = (self.price_high_rao as f64).sqrt();
        let sqrt_current = (current_subnet_price_rao as f64).sqrt();

        let liquidity = self.liquidity_rao as f64;
        let (amount_alpha, amount_tao) = if sqrt_current < sqrt_price_low {
            (
                liquidity * (1.0 / sqrt_price_low - 1.0 / sqrt_price_high),
                0.0,
            )
        } else if sqrt_current > sqrt_price_high {
            (0.0, liquidity * (sqrt_price_high - sqrt_price_low))
        } else {
            (
                liquidity * (1.0 / sqrt_current - 1.0 / sqrt_price_high),
                liquidity * (sqrt_current - sqrt_price_low),
            )
        };

        let alpha_rao = if amount_alpha <= 0.0 {
            0
        } else {
            amount_alpha as u128
        };
        let tao_rao = if amount_tao <= 0.0 {
            0
        } else {
            amount_tao as u128
        };
        (alpha_rao, tao_rao)
    }
}
