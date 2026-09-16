# 跨语言边界与 Python binding 预留

查阅日期：2026-09-16。状态：项目规范草案，尚未完成工具链集成验证。

适用于 Rust↔C++，以及后续 Python bindings；不适用于外部求解器，后者继续走进程协议。

## 官方依据

Rust FFI 需要匹配表示与 ABI，明确回调生命周期以及外部异常/Rust panic 的展开边界。[Rustonomicon FFI](https://doc.rust-lang.org/nomicon/ffi.html)

pybind11 文档区分持有/释放 GIL 的执行环境，释放 GIL 的 native 代码不能任意使用 Python 对象。[pybind11 Miscellaneous](https://pybind11.readthedocs.io/en/stable/advanced/misc.html)

## 项目规则

- 任务 006 优先验证 [CXX](cxx.md) 并在通过后固定版本；具体生成/链接方案仍需实测。若验证发现不适配，再记录替代方案和原因。
- 边界接口先定义所有者、销毁方、有效期、线程和错误契约。字符串约定编码和长度，数组约定元素布局/对齐/数量，句柄约定失效行为。
- 不直接共享 `std::string`、Rust `String`、QObject 或第三方库对象的内存布局；需要共享时由工具生成的受支持桥接或显式 ABI 适配负责。
- 分配方提供对应释放入口；不能跨 allocator 随意释放内存。释放后句柄不能复用为另一个仍可被旧回调访问的对象。
- Rust panic 与 C++ 异常在受控边界转换为失败；明确 abort 与展开场景，禁止以捕获机制承诺段错误可恢复。
- 回调注销与任务结束有明确时序，关闭工程后拒绝旧修订事件。大数据先采用简单明确的所有权，再引入借用或零拷贝。
- Python binding 是后续单独任务：释放 GIL 前确认不访问 Python 状态，回调 Python 时重新获取必要执行上下文；不把 GIL 当作 native 全局互斥锁。

## 验证

成功返回只是最小检查；任务 006 还需验证错误转换、非 ASCII 文本、空数组、大长度拒绝和重复创建/释放。任务 008 再验证异步取消和关闭时回调，不把同步 FFI 测试当成线程安全证明。
