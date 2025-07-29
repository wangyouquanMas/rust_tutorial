借用与所有权
直接相关于: 堆栈优化
将  let global_config = &self.global_config;
  let creator = &self.creator;
改为：
  let creator_key = self.creator.key();

减少了堆栈使用量，直接解决了内存问题


问题背景：

# Rust Ownership特性在解决栈内存问题中的应用

在解决栈内存过大问题时，我们利用了Rust的以下ownership特性：

## 1. 避免不必要的引用嵌套
**原代码**:
```rust
let global_config = &self.global_config;
let creator = &self.creator;
let token = &self.token;
// ...
bonding_curve.token_mint = token.key();
```

**修改后**:
```rust
// 直接存储需要的值，不存储中间引用
let creator_key = self.creator.key();
let token_key = self.token.key();
// ...
self.bonding_curve.token_mint = token_key;
```

这减少了栈上的引用层级，每个中间引用都会占用栈空间。

## 2. 直接访问而非引用拷贝
**原代码**:
```rust
let bonding_curve = &mut self.bonding_curve;
bonding_curve.virtual_sol_reserves = global_config.initial_virtual_sol_reserves_config;
```

**修改后**:
```rust
self.bonding_curve.virtual_sol_reserves = self.global_config.initial_virtual_sol_reserves_config;
```

直接访问`self`的字段，避免了在栈上创建额外的可变引用。

## 3. 值拷贝替代大结构引用
对于小型数据（如`PublicKey`），我们使用值拷贝而非引用整个大型结构：
```rust
// 只复制需要的数据(PublicKey)，而不是引用整个Account结构
let creator_key = self.creator.key(); // 只占用一个PublicKey的大小
```

## 4. 减少日志输出中的临时对象创建
**原代码**:
```rust
msg!("  - Virtual SOL reserves: {}", bonding_curve.virtual_sol_reserves);
msg!("  - Virtual token reserves: {}", bonding_curve.virtual_token_reserves);
// 多行日志...
```

**修改后**:
移除了这些日志，避免了每次格式化字符串时在栈上创建临时对象。

## 5. 控制变量作用域
变量的声明被移到更接近使用点的位置，减少了它们在栈上的生存时间，这不是直接减少栈使用峰值，但是一种良好的内存管理实践。

这些ownership特性的巧妙应用，让我们在不改变程序功能的情况下，成功减少了栈内存使用，解决了原来超出Solana 4KB栈限制的问题。