目标：
1. 理解 arrayref 库的作用

内容:
1. 用法
`arrayref` is a small helper crate that lets you take fixed-size slices out of a byte slice via macros like `array_ref!`, `array_mut_ref!`, and `array_refs!`. Anchor programs often use it when parsing or mutating raw account data: it avoids manual indexing and bounds checks when turning `[u8]` buffers into `[u8; N]` or tuples of arrays.