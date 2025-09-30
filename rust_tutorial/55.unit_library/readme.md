目标：
1. 理解unit crate 的作用

内容:
1. 用法
The `uint` crate supplies the `construct_uint!` macro you invoke in `big_num.rs`. That macro expands into all of the big-integer plumbing (`U128`, `U256`, `U512`, `U1024`)—bit operations, shifts, conversions, etc. Without pulling in `uint`, those types never get generated, so `crate::libraries::big_num::U128` doesn’t exist. By depending on `uint` (you’ve pointed at Raydium’s fork), the macro becomes available, its generated code compiles, and the rest of the crate can import `U128` as expected.