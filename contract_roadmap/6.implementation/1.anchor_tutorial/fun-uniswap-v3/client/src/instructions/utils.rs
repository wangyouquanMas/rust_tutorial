

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