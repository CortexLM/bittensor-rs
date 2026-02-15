//! Balance utilities for TAO/RAO conversions.
//!
//! Matches the Python SDK implementation in `bittensor.utils.balance`.
//! All amounts are in RAO (1 TAO = 1e9 RAO) unless explicitly stated otherwise.

pub use crate::utils::balance_newtypes::{
    balance_from_rao, balance_from_rao_with_netuid, balance_from_tao,
    balance_from_tao_with_netuid, format_rao_as_tao, get_unit_symbol, is_lossless_conversion,
    is_valid_rao_amount, is_valid_tao_amount, parse_tao_string, rao, rao_to_tao, rao_with_netuid,
    tao, tao_to_rao, tao_to_rao_ceiling, tao_to_rao_rounded, tao_with_netuid, Balance, Rao, Tao,
};