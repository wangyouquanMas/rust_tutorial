cargo run --release -- create-pool --config-index <index> --price <initial_price> --mint0 <MINT0_PUBKEY> --mint1 <MINT1_PUBKEY> --open-time <unix_timestamp>

cargo run --release -- create-pool --config-index 1 --price 1.0 --mint0   --mint1  



cargo run --release -- create-config \
     --config-index <u16> \
     --tick-spacing <u16> \
     --trade-fee-rate <u32> \
     --protocol-fee-rate <u32> \
     --fund-fee-rate <u32>
    
cargo run --release -- create-config 1 60 600 1200 800