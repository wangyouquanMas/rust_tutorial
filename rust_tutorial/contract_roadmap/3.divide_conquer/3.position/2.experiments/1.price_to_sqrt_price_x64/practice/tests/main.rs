// Define fixed_point_64 module locally
mod fixed_point_64 {
    pub const Q64: u128 = 1 << 64;
}

#[test]
fn test_price_to_sqrt_price_x64() {
    let price = 1.0;
    let decimals_0 = 6;
    let decimals_1 = 6;

    let sqrt_price_x64 = price_to_sqrt_price_x64(price, decimals_0, decimals_1);
    println!("sqrt_price_x64: {}", sqrt_price_x64);

    let price_from_x64 = sqrt_price_x64_to_price(sqrt_price_x64, decimals_0, decimals_1);
    println!("price_from_x64: {}", price_from_x64);

    assert_eq!(price, price_from_x64);
}

pub fn sqrt_price_x64_to_price(price: u128, decimals_0: u8, decimals_1: u8) -> f64 {
    from_x64_price(price).powi(2) * multipler(decimals_0) / multipler(decimals_1)
}

pub fn from_x64_price(price: u128) -> f64 {
    price as f64 / fixed_point_64::Q64 as f64
}

pub fn price_to_sqrt_price_x64(price: f64, decimals_0: u8, decimals_1: u8) -> u128 {
    //add logs to check inputs
    println!("price: {}", price);
    println!("decimals_0: {}", decimals_0);
    println!("decimals_1: {}", decimals_1);

    let price_with_decimals = price * multipler(decimals_1) / multipler(decimals_0);
    println!("price_with_decimals: {}", price_with_decimals);

    println!("price_with_decimals.sqrt(): {}", price_with_decimals.sqrt());

    price_to_x64(price_with_decimals.sqrt())
}

pub fn price_to_x64(price: f64) -> u128 {
    (price * fixed_point_64::Q64 as f64) as u128
}

pub fn multipler(decimals: u8) -> f64 {
    (10_i32).checked_pow(decimals.try_into().unwrap()).unwrap() as f64
}
