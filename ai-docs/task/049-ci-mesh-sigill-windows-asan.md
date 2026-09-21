# 049 — CI 修复：Netgen Linux 制品 ISA 基线与 Windows ASan 链接

- 状态：in-progress
- 阶段：验证基础
- 依赖：[038](038-native-sdk-artifact-production.md)、[042](042-unified-llvm-toolchain.md)、[010](010-netgen-adapter-smoke.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-21 / 2026-09-21

## 目标与背景

main 分支自 2026-09-20 起三连红（首次红为 push 携带 884465f + cbca56b + 9e4568b 的 run
35517972679；最新为 35556778266）。排查确认两个互相独立的根因：

1. **Linux mesh 测试启动即 SIGILL（Illegal instruction）**，影响所有全新重建
   native 树的 job（lint clippy/clang-tidy/cppcheck、native-coverage、sanitize
   ubuntu）：`gtest_discover_tests` 构建期运行 `panta_mesh_ir_test` /
   `panta_mesh_netgen_test` 直接崩溃、无输出。实证：发布的
   `netgen-6.2.2604-linux-x86_64.tar.gz` 的 `libnglib.so` 含大量 AVX-512 指令
   （zmm 寄存器、`vpermt2d`，llvm-objdump 反汇编计数 >4000）；两个 mesh 测试经
   `panta_mesh` 链接 netgen 共享库，加载期执行其静态初始化即触发。根因是 Netgen
   上游 `USE_NATIVE_ARCH` 默认 `ON`：Linux 走 `-march=native`（PUBLIC 传播整棵
   构建树），制品绑定 038 生产 runner 的 CPU。macOS 制品在 Apple M 系基线内天然
   可移植（上游对 Apple 不加 flag）、Windows 走 `/arch` 探测恰好与消费 runner
   同为 AVX-512，故两平台未暴露。首次红时序吻合：dcb103f（最后一次绿）尚无
   Netgen 消费方，cbca56b 起测试才在 CI Linux 上第一次加载 `libnglib.so`。
2. **Windows sanitizer 链接失败**（`lld-link: error: undefined symbol:
   __asan_init` 等）：CMake 对 clang-cl 直接以 lld-link 链接，绕过驱动器的
   compiler-rt 注入，`add_link_options(-fsanitize=address)` 对 lld-link 只是
   未知前缀被忽略；clang 驱动器在 `/MD` 下的注入（`asan_dynamic` 导入库 +
   `-include:__asan_seh_interceptor` + `-wholearchive:` dynamic_runtime_thunk，
   见 LLVM 22 `clang/lib/Driver/ToolChains/MSVC.cpp`）需要显式复刻。运行期 DLL
   解析（编译器资源目录进 PATH）ctest 侧已就绪，构建期 discovery 侧缺失。

完成后的可观察行为：`cargo sanitize` 三平台绿；Linux 全新重建的 lint/coverage
job 不再在 mesh 测试 discovery 崩溃。

## 必读

- [依赖获取](../standards/dependency-acquisition.md)、[native-dependency-supply](../modules/native-dependency-supply.md)
- [CMake 规范](../standards/cmake.md)、[Rust 规范](../standards/rust.md)、[注释规范](../standards/comments.md)
- [提交规范](../standards/commits.md)、[验证与评审](../standards/validation-and-review.md)

## 范围与非目标

范围：`tools/sdk/netgen.cmake` 关闭 `USE_NATIVE_ARCH`；SDK 管线以新 Release tag
重产并登记三平台 Netgen 资产（042 已决策"不覆盖已有发布资产"，OCCT 资产不变）；
`native/cmake/build-policy.cmake` 在 MSVC+ASan 下显式链接 compiler-rt 动态运行库；
launcher 构建期测试环境补 ASan DLL 解析路径（Windows）。

非目标：不为 netgen 调优 ISA 基线（如 x86-64-v2/v3），性能口径归 [048](048-performance-testing.md)；
不处理 OCCT 资产；不改 sanitizer 矩阵口径。

## 前置条件与待决策

- 已决策（维护者 2026-09-21）：保持单一 tag `sdk-occt-netgen-8.0.1-6.2.2604`，
  重产资产按管线既有 `--clobber` 语义覆盖；本次仅覆盖三个 netgen 资产（缺陷源），
  OCCT 资产不动、哈希继续有效。Netgen 仍链接 038 管线同次生产的 OCCT 8.0.1
  （同源码 pin、同配置，ABI 同构），覆盖理由在 release notes 中显式说明。
  （与 042"不覆盖已有发布资产"的差异由维护者本次决策覆盖。）
- SDK 管线（sdk-occt-netgen.yml）以含本修复的分支 ref 触发 `publish=false`，
  制品经本地反汇编验证后按维护者决策覆盖发布。

## 实施步骤

1. `tools/sdk/netgen.cmake` 增加 `-DUSE_NATIVE_ARCH=OFF` 并同步
   `panta-sdk.json` 选项记录。
2. `native/cmake/build-policy.cmake`：MSVC 且启用 address 时，经
   `clang-cl -print-resource-dir` 定位 compiler-rt，按 clang 驱动器 `/MD` 注入
   等价物显式链接；资源或库缺失在 configure 期 FATAL_ERROR。
3. `panta-build` 抽出 compiler-rt 运行库目录解析；launcher build.rs 在
   sanitizer 构建期把该目录注入测试环境（Windows）；`tests/src/main.rs` 的
   `sanitizer_test_env` 改用同一实现，删除内联副本。
4. 触发 SDK 管线，验证三平台制品（Linux 反汇编确认无 AVX-512），实测 SHA256
   后更新 `sdk-provision.cmake` 与 release notes，按单一 tag 覆盖发布 netgen 资产。
5. 三平台 CI 复验（本仓库唯一可执行三平台门禁的场所）。

## 预计改动

- `tools/sdk/netgen.cmake`（现存）：构建配置 + provenance json。
- `tools/sdk/releases/occt-netgen-8.0.1-6.2.2604.md`（现存）：补充 ISA 基线
  配置说明与覆盖重产记录。
- `native/cmake/sdk-provision.cmake`（现存）：netgen 三平台 SHA256 与 ABI 注记。
- `native/cmake/build-policy.cmake`（现存）：Windows ASan 链接。
- `crates/panta-build/src/lib.rs`（现存）、`crates/launcher/build.rs`（现存）、
  `tests/src/main.rs`（现存）：构建期 ASan DLL 环境注入与去重。

## 清理与兼容例外

- 删除 `tests/src/main.rs` 中 compiler-rt 目录解析的内联实现（被共享实现替代）。
- 无兼容例外。

## 验收标准

- [ ] 覆盖发布的 `netgen-6.2.2604-linux-x86_64.tar.gz` 的 `libnglib.so`/
      `libngcore.so` 反汇编无 zmm/AVX-512 指令；三平台 `panta-sdk.json` 记录
      `USE_NATIVE_ARCH: false`。
- [ ] Windows sanitizer 构建能链接全部测试可执行（无 `__asan_*` 未定义），
      构建期 discovery 与 ctest 均可加载 ASan 动态运行库。
- [ ] `sdk-provision.cmake` 三平台 netgen SHA256 与 Release 覆盖后的实测一致；
      marker 哈希失配触发旧 staging 自动重建。
- [ ] main 分支 CI 全绿。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-21 | 本地（macOS arm64）：旧制品 `llvm-objdump -d libnglib.so` grep zmm | 大量 AVX-512 指令 | 4170 处 zmm、97 处 vpermt2d（根因实证） |
| — | r2 制品同口径反汇编 | 零命中 | 待执行 |
| — | macOS：`cargo build --locked --workspace` + `cargo test --locked --workspace` | 全通过（netgen staging 哈希不变则不重下） | 待执行 |
| — | macOS：`cargo sanitize`、`cargo lint`、`cargo format --check` | 全通过 | 待执行 |
| — | 三平台 CI（push 后 run） | 全绿 | 待执行 |

## 风险与回退

- r2 制品若仍不可移植（验证步骤漏网）：CI 会以同形态 SIGILL 复现，回退为
  manifest 指回旧制品无意义（旧制品即缺陷源）；需在构建描述层面进一步收紧
  （显式 `-march` 基线）后再次覆盖重产。
- Windows ASan 显式链接如与 lld-link 版本行为不符：CI Windows sanitizer job
  直接暴露链接错误，回退点在 `build-policy.cmake` 单一位置。
- 旧 Release 与旧 manifest 哈希保留在 git 历史中，可随时回退登记。

## 决策与工作记录

- 2026-09-21：创建任务。根因实证记录于上表；方案依据：Netgen 上游
  CMakeLists `USE_NATIVE_ARCH` option（tag v6.2.2604 commit 3ee489c）、LLVM 22
  `MSVC.cpp` asan 链接注入序列。
- 2026-09-21：维护者决策单一 tag 覆盖发布（取代初稿的新 tag 方案）；为避免
  OCCT 哈希无谓漂移，覆盖动作只针对三个 netgen 资产，管线 `publish=true`
  的全量覆盖路径保留给未来整对重产场景。

## 完成摘要

未完成。
