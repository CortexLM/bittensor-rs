//! Example: Balance arithmetic — construct, add, subtract, and display TAO values.
//!
//! Run with: cargo run -p bittensor-examples --example balance_arithmetic

use bittensor_core::balance::Balance;

fn main() {
    let a = Balance::from_tao(1.5);
    let b = Balance::from_rao(500_000_000);

    let sum = a.checked_add(b).expect("overflow");
    println!("1.5 TAO + 0.5 TAO = {} TAO ({})", sum.to_tao(), sum);

    let diff = a.checked_sub(b).expect("underflow");
    println!("1.5 TAO - 0.5 TAO = {} TAO ({})", diff.to_tao(), diff);

    let scaled = a.checked_mul(2).expect("overflow");
    println!("1.5 TAO * 2 = {} TAO", scaled.to_tao());

    // Saturating operations never panic
    let max = a.saturating_add(Balance::from_rao(u64::MAX));
    println!("Saturating add with u64::MAX: {} rao", max.to_rao());
}
