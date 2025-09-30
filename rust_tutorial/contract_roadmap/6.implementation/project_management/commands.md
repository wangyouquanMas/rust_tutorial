
cargo run --release -- create-config \
     --config-index <u16> \
     --tick-spacing <u16> \
     --trade-fee-rate <u32> \
     --protocol-fee-rate <u32> \
     --fund-fee-rate <u32>
    
cargo run --release -- create-config 1 60 600 1200 800





cargo run --release -- create-pool --config-index <index> --price <initial_price> --mint0 <MINT0_PUBKEY> --mint1 <MINT1_PUBKEY> --open-time <unix_timestamp>

cargo run --release -- create-pool --config-index 1 --price 1.0 --mint0  EfqZcJLcmRLTTr5cuafEQ6c4WBWHiBYYc94Fo9vaMZGt  --mint1  So11111111111111111111111111111111111111112


cargo run --manifest-path /root/rust_tutorial/rust_tutorial/contract_roadmap/6.implementation/1.anchor_tutorial/fun-uniswap-v3/client/Cargo.toml -- open-position --tick-lower-price 0.8 --tick-upper-price 1.2 --input-amount 1000 --is-base-0 --with-metadata