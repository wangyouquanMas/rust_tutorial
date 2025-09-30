目标：
1. 理解super库的用法

内容:
1. 用法
`super` walks up the Rust module tree. In `client/src/instructions/amm_instructions.rs`, the `use super::super::ClientConfig;` path means:

- `super` → the parent module (`client/src/instructions/mod.rs`).
- `super::super` → the parent of that parent, which is the crate root (`client/src/main.rs`).

From there it can import `ClientConfig`, which is defined at the top level. So `super` isn’t a library—it’s just how you reach items defined in ancestor modules.