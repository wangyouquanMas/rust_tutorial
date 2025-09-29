pub fn add_delta(x: u128, y: i128) -> u128 {
    if y < 0 {
        x - y.unsigned_abs()
    } else {
        x + y as u128
    }
}

