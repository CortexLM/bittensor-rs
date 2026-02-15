use crate::chain::BittensorClient;
use crate::core::constants::{MAX_TICK, MIN_TICK, RAOPERTAO, TICK_STEP};
use crate::types::LiquidityPosition;
use crate::utils::decoders::{
    decode_fixed_u64f64, decode_i32, decode_named_composite, decode_u128, decode_u64, decode_vec,
};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SWAP_PALLET: &str = "Swap";

async fn fetch_positions_value(
    client: &BittensorClient,
    netuid: u16,
    coldkey: &AccountId32,
) -> Result<Option<Value>> {
    // 1) Single key: coldkey
    if let Ok(val) = client
        .storage_with_keys(
            SWAP_PALLET,
            "Positions",
            vec![Value::from_bytes(coldkey.encode())],
        )
        .await
    {
        if val.is_some() {
            return Ok(val);
        }
    }
    // 2) Two keys: (netuid, coldkey)
    if let Ok(val) = client
        .storage_with_keys(
            SWAP_PALLET,
            "Positions",
            vec![
                Value::u128(netuid as u128),
                Value::from_bytes(coldkey.encode()),
            ],
        )
        .await
    {
        if val.is_some() {
            return Ok(val);
        }
    }
    // 3) Two keys reversed: (coldkey, netuid)
    if let Ok(val) = client
        .storage_with_keys(
            SWAP_PALLET,
            "Positions",
            vec![
                Value::from_bytes(coldkey.encode()),
                Value::u128(netuid as u128),
            ],
        )
        .await
    {
        if val.is_some() {
            return Ok(val);
        }
    }
    Ok(None)
}

pub async fn get_liquidity_list(
    client: &BittensorClient,
    netuid: u16,
    coldkey: &AccountId32,
    _block: Option<u64>,
) -> Result<Vec<LiquidityPosition>> {
    // Fetch global fees and sqrt price
    let fee_global_tao = read_fixed_u64f64(
        client,
        SWAP_PALLET,
        "FeeGlobalTao",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    let fee_global_alpha = read_fixed_u64f64(
        client,
        SWAP_PALLET,
        "FeeGlobalAlpha",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    let sqrt_price = read_fixed_u64f64(
        client,
        SWAP_PALLET,
        "AlphaSqrtPrice",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    let current_price = sqrt_price * sqrt_price;
    let current_tick = price_to_tick(current_price);

    // Fetch positions vector for coldkey
    let positions_val = fetch_positions_value(client, netuid, coldkey).await?;
    let Some(positions_val) = positions_val else {
        return Ok(Vec::new());
    };

    let mut out: Vec<LiquidityPosition> = Vec::new();

    // Try to decode as a vector of positions
    if let Ok(positions) = decode_vec(&positions_val, |v| Ok(v.clone())) {
        for position_val in positions {
            let (id, tick_low, tick_high, liquidity, pos_netuid) =
                parse_position_fields(&position_val);
            if let Some(n) = pos_netuid {
                if n as u16 != netuid {
                    continue;
                }
            } else {
                continue;
            }

            // Fetch ticks values (resilient key shapes)
            let tick_low_val = fetch_tick_value(client, tick_low, netuid).await?;
            let tick_high_val = fetch_tick_value(client, tick_high, netuid).await?;

            let tick_low_map = tick_low_val.as_ref();
            let tick_high_map = tick_high_val.as_ref();

            let tao_below = get_fees(
                current_tick,
                tick_low_map,
                tick_low,
                true,
                fee_global_tao,
                fee_global_alpha,
                false,
            );
            let tao_above = get_fees(
                current_tick,
                tick_high_map,
                tick_high,
                true,
                fee_global_tao,
                fee_global_alpha,
                true,
            );
            let alpha_below = get_fees(
                current_tick,
                tick_low_map,
                tick_low,
                false,
                fee_global_tao,
                fee_global_alpha,
                false,
            );
            let alpha_above = get_fees(
                current_tick,
                tick_high_map,
                tick_high,
                false,
                fee_global_tao,
                fee_global_alpha,
                true,
            );

            let (fees_tao, fees_alpha) = calculate_fees_for_position(
                liquidity as f64,
                fee_global_tao,
                fee_global_alpha,
                tao_below,
                tao_above,
                alpha_below,
                alpha_above,
            );

            let price_low = tick_to_price(tick_low) * RAOPERTAO as f64;
            let price_high = tick_to_price(tick_high) * RAOPERTAO as f64;

            out.push(LiquidityPosition {
                id: id.unwrap_or(0),
                price_low_rao: price_low.max(0.0) as u128,
                price_high_rao: price_high.max(0.0) as u128,
                liquidity_rao: liquidity,
                fees_tao_rao: if fees_tao <= 0.0 { 0 } else { fees_tao as u128 },
                fees_alpha_rao: if fees_alpha <= 0.0 {
                    0
                } else {
                    fees_alpha as u128
                },
                netuid,
            });
        }
    }
    Ok(out)
}

pub async fn get_current_subnet_price_rao(client: &BittensorClient, netuid: u16) -> Result<u128> {
    let sqrt_price = read_fixed_u64f64(
        client,
        SWAP_PALLET,
        "AlphaSqrtPrice",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    let price = sqrt_price * sqrt_price;
    Ok(if price <= 0.0 {
        0
    } else {
        (price * RAOPERTAO as f64) as u128
    })
}

fn parse_position_fields(value: &Value) -> (Option<u64>, i32, i32, u128, Option<u64>) {
    if let Ok(fields) = decode_named_composite(value) {
        let id = fields.get("id").and_then(|v| decode_u64(v).ok());
        let tick_low = fields
            .get("tick_low")
            .and_then(|v| decode_i32(v).ok())
            .unwrap_or(0);
        let tick_high = fields
            .get("tick_high")
            .and_then(|v| decode_i32(v).ok())
            .unwrap_or(0);
        let liquidity = fields
            .get("liquidity")
            .and_then(|v| decode_u128(v).ok())
            .unwrap_or(0);
        let netuid = fields.get("netuid").and_then(|v| decode_u64(v).ok());
        return (id, tick_low, tick_high, liquidity, netuid);
    }
    (None, 0, 0, 0, None)
}

async fn read_fixed_u64f64(
    client: &BittensorClient,
    pallet: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<f64> {
    let val = client
        .storage_with_keys(pallet, entry, keys)
        .await?
        .ok_or_else(|| anyhow::anyhow!(format!("{}.{} not found", pallet, entry)))?;
    Ok(decode_fixed_u64f64(&val).unwrap_or(0.0))
}

// Try multiple key shapes for Swap.Ticks storage
async fn fetch_tick_value(
    client: &BittensorClient,
    tick: i32,
    netuid: u16,
) -> Result<Option<Value>> {
    // 1) Single key: i128
    if let Ok(val) = client
        .storage_with_keys(SWAP_PALLET, "Ticks", vec![Value::i128(tick as i128)])
        .await
    {
        if val.is_some() {
            return Ok(val);
        }
    }
    // 2) Two keys: (netuid, tick)
    if let Ok(val) = client
        .storage_with_keys(
            SWAP_PALLET,
            "Ticks",
            vec![Value::u128(netuid as u128), Value::i128(tick as i128)],
        )
        .await
    {
        if val.is_some() {
            return Ok(val);
        }
    }
    // 3) Two keys reversed: (tick, netuid)
    if let Ok(val) = client
        .storage_with_keys(
            SWAP_PALLET,
            "Ticks",
            vec![Value::i128(tick as i128), Value::u128(netuid as u128)],
        )
        .await
    {
        if val.is_some() {
            return Ok(val);
        }
    }
    Ok(None)
}

fn get_fees(
    current_tick: i32,
    tick_val: Option<&Value>,
    tick_index: i32,
    quote: bool,
    global_fees_tao: f64,
    global_fees_alpha: f64,
    above: bool,
) -> f64 {
    let tick_fee_key = if quote {
        "fees_out_tao"
    } else {
        "fees_out_alpha"
    };
    let tick_fee_value = tick_val
        .and_then(|v| extract_fixed_field(v, tick_fee_key))
        .unwrap_or(0.0);
    let global_fee_value = if quote {
        global_fees_tao
    } else {
        global_fees_alpha
    };
    if above {
        if tick_index <= current_tick {
            global_fee_value - tick_fee_value
        } else {
            tick_fee_value
        }
    } else if tick_index <= current_tick {
        tick_fee_value
    } else {
        global_fee_value - tick_fee_value
    }
}

fn calculate_fees_for_position(
    liquidity_frac_rao: f64,
    global_fees_tao: f64,
    global_fees_alpha: f64,
    tao_fees_below_low: f64,
    tao_fees_above_high: f64,
    alpha_fees_below_low: f64,
    alpha_fees_above_high: f64,
) -> (f64, f64) {
    let fee_tao_agg = global_fees_tao - tao_fees_below_low - tao_fees_above_high;
    let fee_alpha_agg = global_fees_alpha - alpha_fees_below_low - alpha_fees_above_high;
    let fees_tao = liquidity_frac_rao * fee_tao_agg;
    let fees_alpha = liquidity_frac_rao * fee_alpha_agg;
    (fees_tao, fees_alpha)
}

fn extract_fixed_field(value: &Value, key: &str) -> Option<f64> {
    // Try proper SCALE decoding first
    if let Ok(fields) = decode_named_composite(value) {
        if let Some(field_val) = fields.get(key) {
            return decode_fixed_u64f64(field_val).ok();
        }
    }
    None
}

fn price_to_tick(price: f64) -> i32 {
    if price <= 0.0 {
        return 0;
    }
    let tick = (price.ln() / TICK_STEP.ln()) as i32;
    tick.clamp(MIN_TICK, MAX_TICK)
}

fn tick_to_price(tick: i32) -> f64 {
    if !(MIN_TICK..=MAX_TICK).contains(&tick) {
        return 0.0;
    }
    TICK_STEP.powi(tick)
}
