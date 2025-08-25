目标
1. 掌握如何创建测试函数
方法1：
如何在tests 目录内，就不需要 #[cfg(test)],直接
#[test]
fn test_add() {
    let result = add(2, 3);
    assert_eq!(result, 5);
}
方法2：
如果是在lib.rs中，就需要 #[cfg(test)]来表示该部分代码只会在runnint tests时候才会被编译。 
这样可以避免生成不必要的代码。


2. 如何输出日志
命令：RUST_LOG=info cargo test -- --nocapture
To ensure the logs are immediately visible, you can run the test with the --nocapture flag
The --nocapture flag ensures that cargo test doesn't suppress the standard output during the test execution


3. 如何打印日志？
{} 占位符的含义
{} 会自动调用变量的 Display trait 来格式化输出。如果变量实现了 Display trait（几乎所有基本类型都实现了），它会以标准格式打印该值。

举例：
let account_info = AccountInfo { is_writable: true };
println!("Account writable: {}", account_info.is_writable);

"Account writable: {}" 是格式化字符串。
account_info.is_writable 是将被插入的值，它是 bool 类型。
{} 占位符会将 account_info.is_writable 的值插入到字符串中，并打印出来。


4. 如何快速测试合约？
如何在合约中打印日志？
4.1 启动local validator 
命令： solana-test-validator

本地节点，解析速度太慢了，经常出现一笔交易，要过好几分钟才能在浏览器查看，


4.2 将rpc设置为 localnet
solana config set --url localhost
4.3 给钱包添加solana，当作测试币
solana airdrop 1000
4.4 给合约添加日志

msg!()日志在哪里看? 查看实验8
The msg!() macro in Solana is used to print simple string messages or formatted strings directly to the program log. This is the standard way to output logs for debugging or informational purposes in Rust-based Solana programs

emit!() 
The emit!() macro, provided by the Anchor framework, is designed specifically for emitting structured events. When you call emit!(), it serializes an event struct and writes it to the program log as base64-encoded binary data (prefixed by Program data:). This enables off-chain clients to parse and subscribe to these events in a structured manner, rather than just reading plain text log

https://www.anchor-lang.com/docs/features/events





4.5 编译合约

4.6 部署合约