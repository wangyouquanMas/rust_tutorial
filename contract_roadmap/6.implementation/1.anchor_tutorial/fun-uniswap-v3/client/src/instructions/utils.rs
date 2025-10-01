use fun_uniswap_v3::libraries::fixed_point_64;
use std::ops::{DerefMut, Mul, Neg};


pub fn multipler(decimals: u8) -> f64{
    (10_i32).checked_pow(decimals.try_into().unwrap()).unwrap() as f64
}

pub fn price_to_sqrt_price_x64(price: f64, decimals_0: u8, decimals_1:u8) -> u128{
     let price_with_decimals = price*multipler(decimals_1) / multipler(decimals_0);
     price_to_x64(price_with_decimals.sqrt())
}

pub fn price_to_x64(price: f64) -> u128{
    (price * fixed_point_64::Q64 as f64) as u128
}

pub fn tick_with_spacing(tick: i32, tick_spacing: i32) -> i32 {
    let mut compressed = tick / tick_spacing;
    if tick < 0 && tick % tick_spacing != 0 {
        compressed -= 1; // round towards negative infinity
    }
    compressed * tick_spacing
}


pub fn amount_with_slippage(amount: u64, slippage: f64, round_up: bool) -> u64 {
    if round_up {
        (amount as f64).mul(1_f64 + slippage).ceil() as u64
    } else {
        (amount as f64).mul(1_f64 - slippage).floor() as u64
    }
}
