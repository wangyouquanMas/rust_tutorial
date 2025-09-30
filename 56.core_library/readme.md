目标：
1. 理解core library的作用

内容:
1. 用法
`core` is Rust’s foundational standard library: it contains the language primitives (traits like `Default`, ops traits, `Option`, `Result`, etc.) and works in `no_std` builds. `std` simply re-exports most of `core` plus OS-dependent features.

The `uint` macros you’re using expect the crate to expose `core` under the name `core_`. That’s why we added `use core as core_;` in `lib.rs`: when the macro expands to `$crate::core_::default::Default`, it resolves to the real `core` crate and finds the required traits. Without that alias, `$crate::core_` doesn’t exist, so the macro-generated impls fail to compile.