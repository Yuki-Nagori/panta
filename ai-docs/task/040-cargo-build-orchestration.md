# 040 — 统一 Cargo 构建编排入口

- 状态：done
- 阶段：验证基础
- 依赖：[004](004-cargo-native-orchestration.md)、[039](039-ci-ffi-build-fix.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-17 / 2026-09-17

## 目标与背景

任务 039 为 CI 增加了单独的 `cargo build --locked -p panta-ffi` 步骤，以确保 native launcher 使用的 Rust staticlib 已经落盘。这能消除 Cargo 并发构建的竞态，但开发者需要记住额外命令，且 CI 与本地入口不一致。

本任务让原生的 `cargo build` 成为统一入口：在 launcher manifest 中同时声明 `panta-ffi` 的普通依赖和 build-dependency。普通依赖继续提供 build script 所需的 `DEP_PANTA_FFI_INCLUDE` metadata，build-dependency 则让 Cargo 在 launcher build script 前完成 staticlib。CI、README 和构建说明继续统一使用 `cargo build`，不新增需要记忆的命令。

## 必读

- [Cargo 与 build script 规范](../standards/cargo.md)
- [Rust 编码与工具链](../standards/rust.md)
- [构建、开发与部署](../architecture/build-and-development.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [仓库文件规范](../standards/repository-hygiene.md)

## 范围与非目标

范围：调整 launcher 的 Cargo manifest 构建图；让 CI 和当前构建文档继续使用 `cargo build`；为 CI 增加任务 012 所需的基础缓存（Cargo registry/git 与带校验的 `target/panta-tools`、`target/panta-deps`，覆盖 Qt staging/GTest）；保留 `panta-ffi` 与 workspace/native 的既有构建参数和环境变量。

非目标：不在 launcher build script 中递归调用 Cargo；不改变 CXX DTO、CMake target、Qt/Native 依赖供给、CTest 聚合或 Cargo 缓存策略；不把编排器扩展成通用任务运行器。

## 前置条件与待决策

Cargo 会区分普通依赖和 build-dependencies 的构建单元；同一 `panta-ffi` package 保留两条边，避免丢失 `links` metadata 注入，同时让 launcher build script 等待 staticlib 产物。该方案不新增 Cargo 子进程，现有 `CMAKE_*`、`CARGO_TARGET_DIR` 等环境传递保持不变。

## 实施步骤

1. 在 launcher manifest 中保留普通依赖并增加同包 build-dependency，固定顺序为 FFI staticlib → launcher build script/native。
2. 移除 CI 的分离预构建步骤，保持 README 与架构构建说明以 `cargo build` 为入口。
3. 验证干净 Debug target、metadata 注入、失败诊断、增量构建和可用的 native/CMake 测试。

## 预计改动

`crates/launcher/Cargo.toml`、`.github/workflows/ci.yml`、`README.md`、`ai-docs/architecture/build-and-development.md`、`ai-docs/task-index.md` 与本文件。

## 清理与兼容例外

删除 CI 中要求调用者手动记忆的独立 `panta-ffi` 预构建步骤和临时编排包；无兼容例外。`cargo build --locked` 保持为从干净树构建完整 Rust/native 链的统一入口。

## 验收标准

- [x] 干净 target 执行 `cargo build --locked` 时，Cargo 先完成 `panta-ffi` staticlib，再执行 launcher build script/native 构建。
- [x] `DEP_PANTA_FFI_INCLUDE` 仍注入 launcher build script；`CMAKE_GENERATOR`、`CMAKE_GENERATOR_PLATFORM`、`CMAKE_PREFIX_PATH`、`CARGO_TARGET_DIR` 等既有参数传递不变。
- [x] staticlib 缺失或任一构建失败时，诊断原样输出且入口返回非零码；不引入 Cargo 递归调用。
- [x] CI、README、架构文档和 task-index 使用 `cargo build` 同一入口；不再存在重复的 FFI 预构建步骤。
- [x] CI 依赖缓存键区分 OS、架构、LLVM 版本、锁文件和 native/Qt/Python 供给配置；可变 native/Cargo 构建树不跨运行恢复，缓存删除后仍可完整构建。
- [x] `cargo fmt`、相关 Rust 测试/Clippy、workflow 静态检查和可用 native 测试通过；无死代码或未登记兼容分支。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-17 | 设计记录：CI run `35197347393` | 确认额外预构建应收回 Cargo 统一入口 | 已确认普通依赖边不保证 staticlib 文件在 launcher build script 前落盘 |
| 2026-09-17 | `/private/tmp/panta-cargo-order-check` 最小 workspace；普通依赖 + build-dependency 双边 | 验证 Cargo 构建图是否先生成 staticlib | 通过：launcher build script 观察到 `liborder_ffi.a` 和 `liborder_ffi.rlib` 已存在；无递归 Cargo |
| 2026-09-17 | workflow cache 静态检查 | 验证缓存覆盖 Cargo registry/git、target/Qt staging，并按平台/ABI/generator/配置分键 | 已加入 `actions/cache@v4`；命中/删除缓存后的真实 run 待 CI 复跑 |
| 2026-09-17 | `CARGO_TARGET_DIR=/private/tmp/panta-cargo-build-order.TZyGyN cargo build --locked` | FFI staticlib 先生成，随后 native 构建 | 通过顺序验证：日志先出现 `Compiling panta-ffi`，再出现 `Compiling panta-launcher`；launcher 已进入 CMake/Qt configure，随后因当前沙箱无法解析 `download.qt.io` 失败，未掩盖 staticlib 缺失问题 |
| 2026-09-17 | `cargo metadata --locked --no-deps`、`cargo fmt --all -- --check`、`actionlint .github/workflows/ci.yml`、`git diff --check` | 清单、格式、workflow 和补丁静态检查通过 | 全部通过 |
| 2026-09-17 | `cargo test --locked --workspace --exclude panta-launcher`；`cargo clippy --locked --workspace --all-targets --exclude panta-launcher -- -D warnings`；完整 workspace Clippy 使用已缓存 Qt staging | Rust 侧回归检查通过 | 测试 15/15 通过；完整 workspace Clippy 通过且无 warning |
| 2026-09-17 | GitHub Actions run `35203709898`（`5a6f452`，Windows 日志确认 `Cache not found` 冷缓存）与 run `35204747014`（`ab0a130`，`Cache hit for restore-key`）三平台 | 干净树单一 `cargo build --locked`；缓存删除后仍可完整构建 | 通过：冷缓存 run 三平台完成全量 Build/Test/Format/Clippy（Windows 5m10s 含 Qt staging 与 native 构建），缓存命中 run 增量复跑全绿；run 日志可见缓存键含 OS/arch/compiler/generator/platform 与 Cargo.lock、rust-toolchain.toml、CI workflow、native CMake 文件哈希 |

## 风险与回退

同一 package 的普通依赖和 build-dependency 双边属于 Cargo 构建图的显式约束；未来升级 Cargo 时需复查 staticlib 是否仍在 launcher build script 前落盘。若行为改变，临时回退到 CI 显式预构建，但不在 launcher build script 中增加隐式 Cargo 调用。

## 决策与工作记录

- 2026-09-17：根据用户反馈创建任务；初步考虑独立编排 package，最小 Cargo workspace 复现后改用同一 `panta-ffi` 的普通依赖 + build-dependency 双边，让原生 `cargo build` 自身表达构建顺序。
- 2026-09-17（收尾）：三平台 CI 冷缓存 run `35203709898` 以单一 `cargo build --locked` 完成全量构建，缓存命中 run `35204747014` 增量复跑全绿，缓存删除后可完整构建的验收由前者证明；任务关闭。

## 完成摘要

已完成。统一入口保持 `cargo build --locked`：launcher manifest 以 `panta-ffi` 普通依赖 + build-dependency 双边表达 staticlib → build script 顺序；CI 删除独立 FFI 预构建步骤并加入按平台/编译器/generator/锁文件分键的缓存；冷缓存 run `35203709898` 证明干净树与缓存删除后均可完整构建，`35204747014` 证明缓存命中增量路径。遗留复查点：Cargo 升级后需复查 staticlib 是否仍在 launcher build script 前落盘（见风险与回退）。
