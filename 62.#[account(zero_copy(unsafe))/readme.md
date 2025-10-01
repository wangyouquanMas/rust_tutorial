目标：
1. 理解 
use fun_uniswap_v3::accounts;
use fun_uniswap_v3::instruction; 的用法


内容:
1. 用法
You were still calling `raydium_accounts::CreatePool` from the client, but after regenerating the Anchor code the `CreatePool` account struct actually lives in the `accounts` module that Anchor exports for every program. Because the module path no longer matched, Rust couldn’t resolve the type and the build failed.

**Rust concepts involved**

- Module paths and visibility: `fun_uniswap_v3::accounts::CreatePool` is the generated path; `raydium_accounts` doesn’t exist anymore, so the compiler complains.
- Crate exports: Anchor’s macros regenerate the `accounts` and `instruction` modules from the program; your client must use the exact paths they expose.
- Type resolution during imports: the compiler resolves types strictly by their module path; any mismatch is an unresolved-type error.

**Fix applied**

I updated the client to use the correct modules and re-exported the `create_pool` instruction from the program:

- Replaced `raydium_accounts::CreatePool` with `accounts::CreatePool`, and similarly `raydium_instruction::CreatePool` with `instruction::CreatePool`.
  
```92:107:client/src/instructions/amm_instructions.rs
        .accounts(accounts::CreatePool {
            pool_creator: program.payer(),
            ...
        })
        .args(instruction::CreatePool {
            sqrt_price_x64,
            open_time,
        })
```

- Added a `create_pool` entry function in `src/lib.rs` so the IDL exports the instruction for the client to use.

After aligning those module paths, `cargo build --release` succeeded.