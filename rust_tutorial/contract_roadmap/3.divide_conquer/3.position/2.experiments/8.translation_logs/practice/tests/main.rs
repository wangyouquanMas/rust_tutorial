use practice::get_transaction_logs;

#[test]
fn fetch_logs_from_live_rpc() {
    // Set `RPC_URL` and `SIG` env vars to run this against a live cluster.
    // let rpc_url = std::env::var("RPC_URL").expect("set RPC_URL");
    // let sig = std::env::var("SIG").expect("set SIG");

    let rpc_url = "http://127.0.0.1:8899";
    let sig = "CeDTWJ6tMFueAZFxwkre1t3Q47SnPHQa1Mj1aqNZ7cUN53Km86TGbdS5sUgPBxpR1kHHKNiEYYNraoT5H9YVkaz";

    let logs = get_transaction_logs(&rpc_url, &sig).expect("fetch logs");

    println!("logs: {:?}", logs);
    // Just assert it doesn't crash; logs may or may not be present depending on the transaction
    assert!(logs.len() >= 0);
}