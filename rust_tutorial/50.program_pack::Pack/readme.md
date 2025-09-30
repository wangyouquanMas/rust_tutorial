目标：
1. 理解Pack 的用法

内容:
1. 用法
`program_pack::Pack` brings the `Pack` trait from `solana_sdk` into scope. That trait defines the static `pack`/`unpack` methods used by SPL account types; without importing it, calls like `Mint::unpack(...)` don’t resolve.