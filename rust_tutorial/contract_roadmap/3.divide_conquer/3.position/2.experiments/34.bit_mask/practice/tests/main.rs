use uint::construct_uint;

construct_uint! {
    pub struct U1024(16);
}

#[test]
fn test_bit_mask_and_log() {
    // minimal reproduction of the mask-and test
    let bit_pos: usize = 511;
    //U1024::one() returns the 1024-bit unsigned integer value equal to 1.
    //
    let one = U1024::one();
    println!("one:{:?}",one);
    println!("one (decimal) = {}", one);     

    let mask = U1024::one() << bit_pos;
    println!("mask:{:?}",mask);

    // let masked = bit_map & mask;
    // println!("bit_pos: {}", bit_pos);
    // println!("bit_map == mask: {}", bit_map == mask);
    // println!("masked is nonzero: {}", masked != U1024::default());
    // assert!(masked != U1024::default());
}