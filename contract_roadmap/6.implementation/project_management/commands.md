cargo run --release -- create-pool --config-index <index> --price <initial_price> --mint0 <MINT0_PUBKEY> --mint1 <MINT1_PUBKEY> --open-time <unix_timestamp>


cargo run --release --manifest-path /root/rust_tutorial/contract_roadmap/6.implementation/1.anchor_tutorial/fun-uniswap-v3/Cargo.toml -- create-pool --config-index 8 --price 1.0 --mint0 13tkegLcV4EnLVmohrVzAjzZ36xRM3vuyvjXT8uk5Zok --mint1 So11111111111111111111111111111111111111112

cargo run --release -- create-config \
     --config-index <u16> \
     --tick-spacing <u16> \
     --trade-fee-rate <u32> \
     --protocol-fee-rate <u32> \
     --fund-fee-rate <u32>
    
cargo run --release create-config --config-index 8 --tick-spacing 60 --trade-fee-rate 600 --protocol-fee-rate 1200 --fund-fee-rate 800





cargo run --manifest-path /root/rust_tutorial/contract_roadmap/6.implementation/1.anchor_tutorial/fun-uniswap-v3/client/Cargo.toml -- open-position --tick-lower-price 0.9 --tick-upper-price 1.1 --is-base-0 --input-amount 1000000 --with-metadata