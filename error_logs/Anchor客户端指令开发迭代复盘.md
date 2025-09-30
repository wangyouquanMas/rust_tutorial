# Anchor 客户端指令开发迭代复盘

## 背景
在为 `fun-uniswap-v3` 项目补全客户端命令（`open_position`、`swap` 等）时，需要直接通过 Anchor Client 构造和发送指令。由于对 Anchor 为同一条指令生成的多套结构体理解不够清晰，导致多次编译报错及逻辑卡点。

## 错误 1：误用 `instructions` 模块中的指令账户结构体
- **背景**：实现 `open_position_with_token22_nft_instr` 客户端逻辑，准备通过 `.request().accounts(...)` 构造交易。
- **错误**：导入了 `fun_uniswap_v3::instructions::OpenPositionWithToken22Nft`，结果编译报错，提示各字段需要 `Signer<'info>`、`AccountLoader<'info, …>`，而实际传入的是 `Pubkey`。
- **原因**：`instructions` 模块内的结构体只在链上处理逻辑时使用（带生命周期和借用约束），客户端应使用 `accounts` 模块下的版本，它们的字段类型就是 `Pubkey`，与客户端环境匹配。
- **方案**：改为 `use fun_uniswap_v3::accounts::OpenPositionWithToken22Nft;`，重新编译即可通过。

## 错误 2：没有导出 `swap` 指令却在客户端调用
- **背景**：为 CLI 新增 `swap` 子命令，并在客户端复用 `swap_instr` 封装。
- **错误**：`cargo check` 提示找不到 `fun_uniswap_v3::accounts::SwapSingle`，即便导入 `instructions::SwapSingle` 也会出现大量 “expected Signer, found Pubkey” 的类型不匹配。
- **原因**：链上程序 `lib.rs` 并未导出 `pub fn swap(...)`，Anchor 因此不会在 `accounts` 模块中生成 `SwapSingle`，客户端自然找不到对应结构体。强行改成 `instructions::SwapSingle` 仍然不对，因为那是链上上下文所需的类型。
- **方案**：先在程序中补齐 `swap` 指令入口（或确认已有），重新运行 Anchor 生成的代码后，客户端再导入 `fun_uniswap_v3::accounts::SwapSingle`。确保链上指令已暴露，客户端代码才能正常编译。

## 错误 3：盲目接受编译器建议导致类型连锁错误
- **背景**：面对 “cannot find `SwapSingle` in `accounts`” 的报错时，编译器提示“尝试使用 `instructions::SwapSingle`”。
- **错误**：直接采纳提示，导致 `.accounts(...)` 里所有字段都与预期类型不符，引出更多 `expected Signer<'info>` 与 `Pubkey` 的不匹配报错。
- **原因**：IDE/编译器的 fix 建议只基于名字匹配，而不了解 Anchor 中同名结构体承担的不同角色。盲目套用会把链上专用类型拉进客户端，造成更复杂的类型问题。
- **方案**：遇到建议时先确认语义是否合适；必要时回到 Anchor 生成的 `accounts`、`instructions` 模块理解差异，再选择正确的导入。

## 总结
- 牢记 Anchor 会为同一条指令生成「链上使用」「客户端使用」两套结构体，客户端应优先引用 `accounts` 模块。
- 客户端在调用某个指令前，确保链上程序确实导出了对应入口函数，否则不会生成配套的 `accounts::*` 结构体。
- 对编译器/IDE 的自动提示保持警惕，验证后再接受，避免将错误导入放大成更多类型问题。
