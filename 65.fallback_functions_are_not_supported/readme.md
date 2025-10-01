目标：
1. 理解 为什么 #[account（...）]属性生效，必须要导入 anchor_lang::prelude


内容:
1. 用法
The `#[account(...)]` attribute is defined by Anchor (it’s a procedural macro exported from `anchor_lang::prelude`). Without importing the prelude—or at least the specific macro—the compiler can’t resolve the attribute, so it either no-ops or errors. Bringing the prelude into scope ensures `#[account]`, `#[derive(Accounts)]`, etc., are actually applied and generate the trait impls you’re expecting.