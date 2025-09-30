# 错误复盘 - Anchor 客户端账户类型混淆

## 错误1
- **背景**：在编写 `open_position_with_token22_nft_instr` 客户端逻辑，为 Anchor 程序构造调用指令。
- **错误**：导入并使用了 `fun_uniswap_v3::instructions::OpenPositionWithToken22Nft`，导致编译器报错，提示多个账户字段期望 `Signer`、`AccountLoader` 等，而实际提供的是 `Pubkey`。
- **原因**：Anchor 为同一个指令同时生成了面向链上程序的 `instructions` 结构体和面向客户端的 `accounts` 结构体。客户端应填入 `Pubkey`，却误用了只在链上上下文可用、包含借用约束的 `instructions` 版本。
- **方案**：改为导入并使用 `fun_uniswap_v3::accounts::OpenPositionWithToken22Nft`，让 `.accounts(...)` 接收预期的 `Pubkey` 参数即可恢复正常编译。

