use crate::libraries::big_num::{U256, U512};

pub trait MulDiv {
    fn mul_div_floor(self, numerator: U256, denominator: U256) -> Option<U256>;
    fn mul_div_ceil(self, numerator: U256, denominator: U256) -> Option<U256>;
}

impl MulDiv for U256 {
    fn mul_div_floor(self, numerator: U256, denominator: U256) -> Option<U256> {
        if self.is_zero() || numerator.is_zero() {
            return Some(U256::zero());
        }
        if denominator.is_zero() {
            return None;
        }
        let product = U512::from(self) * U512::from(numerator);
        Some(U256::from(product / U512::from(denominator)))
    }

    fn mul_div_ceil(self, numerator: U256, denominator: U256) -> Option<U256> {
        if self.is_zero() || numerator.is_zero() {
            return Some(U256::zero());
        }
        if denominator.is_zero() {
            return None;
        }
        let product = U512::from(self) * U512::from(numerator);
        let quotient = product / U512::from(denominator);
        if product % U512::from(denominator) != U512::zero() {
            Some(U256::from(quotient + U512::one()))
        } else {
            Some(U256::from(quotient))
        }
    }
}

