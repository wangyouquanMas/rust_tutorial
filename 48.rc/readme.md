目标：
1. 理解Rc 的用法

内容:
1. 用法
`std::rc::Rc` is Rust’s single-threaded reference-counting smart pointer. It lets multiple owners share read-only access to the same heap-allocated data by keeping a count of active references and freeing that data automatically once the last `Rc` goes out of scope.