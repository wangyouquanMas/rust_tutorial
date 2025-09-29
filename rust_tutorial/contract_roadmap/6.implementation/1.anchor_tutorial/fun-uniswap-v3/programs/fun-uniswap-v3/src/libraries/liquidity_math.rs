use crate::libraries::unsafe_math::add_delta;

pub fn add_liquidity_delta(liquidity: u128, delta: i128) -> Result<u128, &'static str> {
    if delta < 0 && liquidity < delta.unsigned_abs() {
        return Err("Liquidity underflow");
    }
    Ok(add_delta(liquidity, delta))
}

