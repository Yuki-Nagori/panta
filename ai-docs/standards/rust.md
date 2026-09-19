# Rust 编码与工具链

查阅日期：2026-09-17。状态：workspace Clippy 已启用 warning-as-error；FFI、native 依赖接入仍按对应 task 验证。

适用于 `crates/`、launcher 和 Rust 构建辅助代码。stable 表示工具链渠道；可复现构建仍需固定实际版本。

## 官方依据

Rust 2024 edition 随 Rust 1.85.0 发布；这只是 edition 的起点，不代表本项目所有依赖的最低 Rust 版本。[Rust Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)

跨语言接口需要明确 ABI、所有权及异常展开规则；不能假定 Rust 类型天然可供 C/C++ 使用。[Rustonomicon FFI](https://doc.rust-lang.org/nomicon/ffi.html)

## 项目规则

- 采用 edition 2024；工具链由 `rust-toolchain.toml` 固定在 stable 1.98.1（任务 001，2026-09-16 验证）。workspace `rust-version = "1.88"` 是承诺下限，跟随首选 CXX release（当时 1.0.202）的 MSRV，而非 edition 的最低要求 1.85；依赖或 CXX 要求更高时上调并记录原因。升级编译器与依赖分别评估，不能以浮动 stable 代替验证记录。
- crate 名使用 `panta-` 前缀，模块/函数 `snake_case`、类型 `PascalCase`。默认使用 rustfmt；workspace 内 Clippy 以 `cargo clippy --locked --workspace --all-targets -- -D warnings` 运行，warning 直接失败。第三方源码不纳入本项目 lint。
- 领域库用结构化 `Result`，在应用边界补充上下文；不在正常输入、文件读写和协议处理路径使用 `unwrap`/`expect` 代替错误处理。
- ID 使用 newtype；持久化 DTO 和内存领域对象分离。文件路径采用 `Path`/`PathBuf`，不假定操作系统路径一定能无损转成 UTF-8 字符串。
- `unsafe` 集中在边界模块，每处注明安全前提、所有者和线程条件。workspace 默认 `unsafe_code = "deny"`；当前登记的 allow 仅包括 CXX 桥接生成胶水和 `panta-foundation::crash` 专用模块，均不得向业务代码外扩。新增手写 unsafe 需专用模块并逐块写 `// SAFETY:`。不得为了通过类型检查无依据实现 `Send`/`Sync`。
- 大型数据使用明确所有者和借用/共享策略；`Arc` 解决共享所有权，不自动保证内部修改安全。异步运行时是否引入由具体任务决定，不先堆叠多套 runtime。
- panic 不作为业务错误；FFI 捕获策略依赖 panic 配置，`panic=abort` 不可由 `catch_unwind` 恢复。引擎隔离依靠外部进程，不能靠捕获 panic 承诺所有故障可恢复。
- 单元测试留在被测 `.rs` 的 `#[cfg(test)]` 模块中以访问私有项；公共 API 的黑盒/跨模块测试放对应 crate 的 `tests/`。根跨语言聚合测试只放 [测试规范](testing.md) 规定的 `tests/integration/`。

## 验证

`cargo fmt --all -- --check` 与 `cargo clippy --locked --workspace --all-targets -- -D warnings` 是 Rust-only 质量入口；跨语言项目验证使用根 `cargo format`、`cargo lint` 和 `cargo test`。领域测试覆盖输入修订、取消、错误转换，随首个业务 crate 建立。涉及 FFI 时同时运行 native 侧测试。
