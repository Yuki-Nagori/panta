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
  重产资产由管线 `publish=true` 收口 job 覆盖发布（清空旧资产后全量重传）；
  VTK 管线同口径。本次 occt/netgen/vtk 三个 SDK 全部重产，manifest 哈希按
  Release 实测 sidecar 回填。Netgen 仍链接 038 管线同次生产的 OCCT 8.0.1
  （同源码 pin、同配置，ABI 同构）。（与 042"不覆盖已有发布资产"的差异由
  维护者本次决策覆盖。）
- 已决策（维护者 2026-09-21）：不在本地下载制品做反汇编验证；ISA 可移植性
  由管线 selfcheck + PR 三平台 CI（Linux mesh 测试真实加载 netgen）实测，
  Windows ASan 链接由 PR CI sanitizer job 实测。

## 实施步骤

1. `tools/sdk/netgen.cmake` 增加 `-DUSE_NATIVE_ARCH=OFF` 并同步
   `panta-sdk.json` 选项记录。
2. `native/cmake/build-policy.cmake`：MSVC 且启用 address 时，经
   `clang-cl -print-resource-dir` 定位 compiler-rt，按 clang 驱动器 `/MD` 注入
   等价物显式链接；资源或库缺失在 configure 期 FATAL_ERROR。
3. `panta-build` 抽出 compiler-rt 运行库目录解析；launcher build.rs 在
   sanitizer 构建期把该目录注入测试环境（Windows）；`tests/src/main.rs` 的
   `sanitizer_test_env` 改用同一实现，删除内联副本。
4. 以分支 ref 触发两条管线重产（publish=false 先行验证管线与自检），代码
   review 通过后以 `publish=true` 重产并覆盖发布；manifest 从 Release 的
   `.sha256` sidecar 实测回填。
5. 三平台 CI 复验（PR CI 即最终验证场所）。

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

- [ ] 覆盖发布的 `netgen-6.2.2604-linux-x86_64.tar.gz` 可在无 AVX-512 的
      Linux runner 上正常加载（main CI mesh 测试全过）。
- [ ] Windows sanitizer 构建能链接全部测试可执行（无 `__asan_*` 未定义），
      构建期 discovery 与 ctest 均可加载 ASan 动态运行库（main CI 实测）。
- [x] `sdk-provision.cmake` occt/netgen/vtk 九项 SHA256 与覆盖后 Release
      sidecar 一致；marker 哈希失配触发本地与 CI 旧 staging 自动重建。
- [ ] push 后 main CI 全绿。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-21 | 本地（macOS arm64）：旧制品 `llvm-objdump -d libnglib.so` grep zmm | 大量 AVX-512 指令 | 4170 处 zmm、97 处 vpermt2d（根因实证） |
| 2026-09-21 | 本地（macOS arm64，clang++/LLVM 22.1.7）：`cargo format --check`、`cargo check --locked --workspace --all-targets`、`cargo test --locked --workspace` | 全通过 | 全通过（含 launcher build.rs 改动后的强制重编译） |
| 2026-09-21 | 本地（macOS arm64）：`cargo lint`（clippy/clang-tidy/includes/cppcheck/cmake/qmllint/machete） | 零发现 | 退出码 0 |
| 2026-09-21 | 本地（macOS arm64）：`cargo sanitize`（asan-ubsan + tsan 双树） | 49/49 ×2 | 100% passed（两棵插桩树 ctest 全过） |
| 2026-09-21 | 分支 dispatch sdk-occt-netgen.yml / sdk-vtk.yml（publish=false） | 三平台生产 + selfcheck 全绿 | run 35559174921（49m/24m/26m）、35559176795（70m/22m/40m）全 ✓ |
| 2026-09-21 | publish=true 重产覆盖发布 | 单一 tag 清空旧资产后重传，收口 job 正常 | run 35566443162（22m/50m/38m + publish 20s）、35566444816（32m/36m/70m + publish 20s）全 ✓ |
| 2026-09-21 | 九项 SHA256 回填 sdk-provision | 与 Release sidecar 逐一对应 | occt/netgen/vtk ×三平台已按 sidecar 实测替换（本 commit） |
| — | push 后 main CI：Linux mesh 测试 / Windows sanitizer / 全矩阵 | 全绿 | 待执行（维护者自行 push） |

## 风险与回退

- 重产制品若仍不可移植：PR CI 会以同形态 SIGILL 复现，回退为 manifest 指回
  旧制品无意义（旧制品即缺陷源）；需在构建描述层面进一步收紧（显式 `-march`
  基线）后再次覆盖重产。
- Windows ASan 显式链接如与 lld-link 版本行为不符：CI Windows sanitizer job
  直接暴露链接错误，回退点在 `build-policy.cmake` 单一位置。
- 旧 Release 与旧 manifest 哈希保留在 git 历史中，可随时回退登记。

## 决策与工作记录

- 2026-09-21：创建任务。根因实证记录于上表；方案依据：Netgen 上游
  CMakeLists `USE_NATIVE_ARCH` option（tag v6.2.2604 commit 3ee489c）、LLVM 22
  `MSVC.cpp` asan 链接注入序列。
- 2026-09-21：维护者决策单一 tag 覆盖发布（取代初稿的新 tag 方案）；
  为避免 OCCT 哈希无谓漂移的初版方案随后由维护者扩展为 occt/netgen/vtk
  三 SDK 全部重产覆盖。管线 publish=true 收口 job 全绿，单一 tag 覆盖语义
  （清空旧资产后重传）获 CI 实证。
- 2026-09-21：流程调整——PR #3 被维护者 squash 合入 main（d5c0955）后，
  维护者决策改为 main 直接推进、哈希回填单独成 commit、push 由维护者
  自行执行；制品反汇编验证按维护者决策跳过，由 main CI 实测替代。

## 完成摘要

未完成。
