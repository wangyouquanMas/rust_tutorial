Step1: create amm config account 
cargo run --release -- create-config \
     --config-index <u16> \
     --tick-spacing <u16> \
     --trade-fee-rate <u32> \
     --protocol-fee-rate <u32> \
     --fund-fee-rate <u32>
    
cargo run --release -- create-config 1 60 600 1200 800




Step2: create pool
cargo run --release -- create-pool --config-index <index> --price <initial_price> --mint0 <MINT0_PUBKEY> --mint1 <MINT1_PUBKEY> --open-time <unix_timestamp>

cargo run --release -- create-pool --config-index 1 --price 1.0 --mint0  EfqZcJLcmRLTTr5cuafEQ6c4WBWHiBYYc94Fo9vaMZGt  --mint1  So11111111111111111111111111111111111111112


Step3: Open position 
cargo run --manifest-path /root/rust_tutorial/rust_tutorial/contract_roadmap/6.implementation/1.anchor_tutorial/fun-uniswap-v3/client/Cargo.toml -- open-position --tick-lower-price 0.8 --tick-upper-price 1.2 --input-amount 1000 --is-base-0 --with-metadata


Step4: Swap
cargo run --manifest-path /root/rust_tutorial/rust_tutorial/contract_roadmap/6.implementation/1.anchor_tutorial/fun-uniswap-v3/client/Cargo.toml -- \
  swap \
  --input-token  5Kar97KcLTHFP7iMMbRgB74zXHVL43yEmURwD3Z6RvdT \
  --output-token EErNVGXUNTDSx2w26sgLfs657zPqSbp8YFFbnifqbF8Y  \
  --base-in \
  --amount 100 \
  --limit-price 1.05



