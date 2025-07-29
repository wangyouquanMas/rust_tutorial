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