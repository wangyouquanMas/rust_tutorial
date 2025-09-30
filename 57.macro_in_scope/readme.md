目标：
1. 理解 macro 和 scope的关系

内容:
1. 用法
Rust attributes like `#[account(...)]` are procedural macros defined in Anchor. If the macro isn’t in scope, the compiler treats the attribute as an unknown tag and simply drops it—no generated code, no trait impls. In `states/pool.rs` you didn’t `use anchor_lang::prelude::*;`, so the `account` macro never gets imported. Without that import, the attribute doesn’t expand into the derived `ZeroCopy` implementations the macro normally emits, and `AccountLoader` fails its trait bound.

`use anchor_lang::prelude::*;` brings all of Anchor’s core macros and types into scope (`account`, `event`, `ZeroCopy`, `Pubkey`, etc.). Once the prelude is imported, the compiler finds the `account` macro, runs it during expansion, and the struct gains the auto-generated trait impls required by `AccountLoader`.