目标：
1. 理解lib.rs在 crate中的应用

内容:
1. 用法
`lib.rs` is the crate root: it defines the public surface of your library crate. Any `pub mod ...;` in that file tells the compiler which internal modules exist and makes them reachable for other modules (and for external crates when you `pub use`). If you don’t declare a module in `lib.rs` (or otherwise include it), the compiler never compiles it, so `use crate::error::ErrorCode;` fails. By adding `pub mod error;` (and optionally `pub use error::ErrorCode;`), you both compile that file and expose it to the rest of the crate.