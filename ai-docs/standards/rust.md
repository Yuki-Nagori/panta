# Rust 编码与工具链

查阅日期：2026-09-16。状态：项目规范草案，尚未完成工具链集成验证。

适用于 `crates/`、launcher 和 Rust 构建辅助代码。stable 表示工具链渠道；可复现构建仍需固定实际版本。

## 官方依据

Rust 2024 edition 随 Rust 1.85.0 发布；这只是 edition 的起点，不代表本项目所有依赖的最低 Rust 版本。[Rust Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)

跨语言接口需要明确 ABI、所有权及异常展开规则；不能假定 Rust 类型天然可供 C/C++ 使用。[Rustonomicon FFI](https://doc.rust-lang.org/nomicon/ffi.html)

## 项目规则

- 采用 edition 2024；任务 001 固定经过验证的 stable 版本，记录 `rust-version` 策略。升级编译器与依赖分别评估，不能以浮动 stable 代替验证记录。
- crate 名使用 `chronos-` 前缀，模块/函数 `snake_case`、类型 `PascalCase`。默认使用 rustfmt；Clippy 规则由基础设施任务固定，不能盲目对第三方启用本项目 lint。
- 领域库用结构化 `Result`，在应用边界补充上下文；不在正常输入、文件读写和协议处理路径使用 `unwrap`/`expect` 代替错误处理。
- ID 使用 newtype；持久化 DTO 和内存领域对象分离。文件路径采用 `Path`/`PathBuf`，不假定操作系统路径一定能无损转成 UTF-8 字符串。
- `unsafe` 集中在边界模块，每处注明安全前提、所有者和线程条件。不得为了通过类型检查无依据实现 `Send`/`Sync`。
- 大型数据使用明确所有者和借用/共享策略；`Arc` 解决共享所有权，不自动保证内部修改安全。异步运行时是否引入由具体任务决定，不先堆叠多套 runtime。
- panic 不作为业务错误；FFI 捕获策略依赖 panic 配置，`panic=abort` 不可由 `catch_unwind` 恢复。引擎隔离依靠外部进程，不能靠捕获 panic 承诺所有故障可恢复。

## 验证

落地后使用 `cargo fmt --all -- --check` 和对实际 targets 的 Clippy 检查；领域测试覆盖输入修订、取消、错误转换。涉及 FFI 时同时运行 native 侧测试。当前没有 Cargo 工程，上述命令尚不可运行。
