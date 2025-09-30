目标：
1. 理解FromStr 的用法

内容:
1. 用法
It imports `std::str::FromStr`, bringing the trait into scope so types that implement it—like `Pubkey`—expose the associated `from_str` method. Without this `use`, the compiler won’t find `Pubkey::from_str` even though it’s implemented.