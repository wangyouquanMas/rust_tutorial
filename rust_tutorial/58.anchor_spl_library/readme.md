目标：
1. 理解 #[derive(Account)] 和 anchor_spl的关系

内容:
1. 用法
We hit a compile error in `create_pool.rs` because the code derives `Accounts` and uses SPL token types from `anchor_spl`, but that crate wasn’t listed in `Cargo.toml`. Adding `anchor-spl = { version = "0.31.0", features = ["token-interface"] }` to the program’s dependencies fixes the missing import. Run `anchor build`/`cargo check` to confirm all good.