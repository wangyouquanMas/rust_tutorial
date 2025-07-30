目标：
1. 如何声明
    fn  snake_case_function_name(){}

2. 函数内部调用其它函数

3. 函数参数(单参数，多参数)

4. 函数返回值

5. 函数返回值Self 是什么含义？
impl AccountInfo{
    fn new(is_writable: bool) -> Self{
        Self{ is_writable}
    }
}
