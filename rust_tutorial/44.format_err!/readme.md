目标：
1. 理解什么是format_err!

内容:
1. 使用

`format_err!` comes from the `anyhow` crate. To use it in a fresh project:

- Add `anyhow` to `Cargo.toml` (e.g. `anyhow = "1"`).
- Import the macro in the file where you call it: `use anyhow::{format_err, Result};`

After that, `format_err!("message")` will compile—no extra crate or module is needed.