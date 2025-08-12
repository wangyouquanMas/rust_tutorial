use crate::big_num::U128;
use crate::fixed_point_64;
// To understand the working of binary search, consider the following illustration:

// Consider an array arr[] = {2, 5, 8, 12, 16, 23, 38, 56, 72, 91}, and the target = 23.


#[test]
fn test_get_liquidity_from_amount_0(){
    let sqrt_ratio_a_x64 = 583337266871351552;
    let sqrt_ratio_b_x64 = 58333726687135155;
    let amount_0 = 10000;
    let liquidity = get_liquidity_from_amount_0(sqrt_ratio_a_x64,sqrt_ratio_b_x64,amount_0);
    println!("liquidity: {}",liquidity);
}



fn get_liquidity_from_amount_0(mut sqrt_ratio_a_x64:u128,mut sqrt_ratio_b_x64:u128,amount_0:u64) -> u128{
    //sqrt_ratio_a_x64 should hold the smaller value 
    //TODO: why ? 
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64{
        std::mem::swap(&mut sqrt_ratio_a_x64,&mut sqrt_ratio_b_x64);
    }

    let intermediate = U128::from(sqrt_ratio_a_x64).mul_div_floor(
        U128::from(sqrt_ratio_b_x64),
        U128::from(fixed_point_64::Q64),
    )
    .unwrap();


    U128::from(amount_0)
        .mul_div_floor(
            intermediate,
            U128::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64),
        )
        .unwrap()
        .as_u128()
}