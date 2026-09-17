# 020 — 托管引导：CMake/Ninja 二进制供给

- 状态：done
- 阶段：M0
- 依赖：[004](004-cargo-native-orchestration.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

004 已接通 Cargo→CMake 调度，但仍要求本机预装 CMake/Ninja；[依赖获取](../standards/dependency-acquisition.md) 的托管原则要求"开发者只装 rustup 与平台编译器"。本任务实现工具二进制的自动供给：首次构建时按固定清单下载 CMake 与 Ninja 的预编译资产到构建树托管目录，缺失时引导，使 README 环境要求降为最终形态。依赖满足、任务已 ready，尚未实施。

## 必读

- [规范：依赖获取](../standards/dependency-acquisition.md)
- [规范：cargo](../standards/cargo.md)
- [架构：build-and-development](../architecture/build-and-development.md)

## 范围与非目标

范围：build.rs 引导逻辑（下载/校验/缓存 CMake 与 Ninja）、SHA256 校验与回填、跨平台资产（macOS/Linux/Windows）、缺失时的可定位诊断。

非目标：Qt/VTK/OCCT/Netgen 的获取与构建（各自任务）；CI 缓存共享（012）。

## 前置条件与待决策

- 固定版本已在依赖获取清单（CMake 4.4.3、Ninja 1.13.2）；实施时补齐各平台资产 URL 与 SHA256。
- Ninja 采用各平台预编译 release 资产；若上游没有可复核资产，另立制品任务，不在开发者构建中编译源码。缓存目录布局建议为 `target/` 下托管目录，随 profile 共享。

## 实施步骤

1. 在 build.rs 中实现"定位 → 缺失时按清单获取 → 校验 → 注入 PATH/显式路径"的供给链。
2. 记录 macOS/Linux/Windows 三平台资产与校验，回填依赖获取文档。
3. 验证：无 CMake/Ninja 的 PATH 下 `cargo build` 全链成功；损坏下载可检出并被拒绝。

## 预计改动

`crates/launcher/build.rs`（或新调度模块）、依赖获取文档、README 环境要求。

本轮实际边界：新增 `crates/launcher/src/provision.rs`（build.rs 经 `#[path]` 复用，单元测试挂 launcher 测试构建）；build.rs 接入"定位 → 下载 → SHA256 校验 → 解包 → 注入路径"，托管 Ninja 时向 CMake 传 `CMAKE_MAKE_PROGRAM`；`sha2` 进入 workspace/build/dev 依赖。缓存位于根 `target/panta-tools/`（archives + 解包目录 + marker）。README 环境要求已是最终形态（写明工具由构建引导拉取），本任务使其成真，未改动；Linux aarch64 无官方 CMake 资产，`CMAKE` 旁路诊断覆盖。

## 清理与兼容例外

移除"要求预装 CMake/Ninja"的临时说明；无兼容层。

## 验收标准

- [x] 干净环境（PATH 无 cmake/ninja）`cargo build --locked` 全链成功且产物可运行。
- [x] 下载校验（SHA256）生效，损坏产物被拒绝并给出可定位诊断。
- [x] 三平台资产清单与校验和回填文档；README 环境要求同步收敛。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| — | 尚未执行 | 无实现证据 |
| 2026-09-17 | 首次下载实测（macOS arm64 下载三平台资产）：CMake mac universal `0c5d…64fa`、linux x86_64 `d6c8…c6cc`、windows x86_64 `4d52…26ab`；Ninja mac `c990…321b`、linux `5749…bead6`、win `07fc…8dc65` | SHA256 已回填 dependency-acquisition.md；解包布局与 `cmake --version`/`ninja --version`（4.4.3 / 1.13.2）逐一验证 |
| 2026-09-17 | `cargo test -p panta-launcher --locked`（macOS arm64） | 9/9：launcher 现有 4 测试 + provision 单元测试 5（hex、宿主资产表、SHA256 已知向量、嵌套布局定位、缺失返回 None）。cargo test 不执行 build script 内测试，故 provision.rs 由 build.rs `#[path]` 与 main.rs cfg(test) 模块共享 |
| 2026-09-17 | 干净 PATH E2E：`PATH=/tmp/panta-clean-bin:/usr/bin:/bin:/usr/sbin:/sbin cargo build --locked`（无系统 cmake/ninja，/tmp/panta-clean-bin 仅含 cargo/rustc 符号链接） | 通过：自动下载并校验 cmake/ninja 至 `target/panta-tools/`；新构建树 CMakeCache 中 `CMAKE_COMMAND`/`CMAKE_MAKE_PROGRAM` 均指向托管产物；全链构建成功，产物 `panta-native` 离屏启动存活（QT_QPA_PLATFORM=offscreen，进程 4 秒存活确认） |
| 2026-09-17 | 损坏下载拒绝：假 curl 替身注入损坏归档（marker/解包目录/归档清空后强制重跑） | 通过：退出码 101，诊断"ninja 归档 SHA256 不符，已拒绝进入构建：预期 c990…/实际 8bd9…；归档已删除，重试将重新下载（URL）"，归档与解包目录均未残留 |
| 2026-09-17 | 离线复用：真实归档放回（假 curl 仍在 PATH 最前，联网即失败） | 通过：归档哈希命中跳过下载，解包后全链成功；再次强制重跑（archives 目录移走），marker+二进制命中，构建成功 |
| 2026-09-17 | `cargo fmt --all -- --check`；`cargo clippy --locked --workspace --all-targets -- -D warnings`；`git diff --check` | 通过，Clippy 0 warning |
| 2026-09-17 | GitHub Actions run `35222847994` | Windows 分支回归 | 失败：`set_executable` 参数在非 unix 构建未使用（`-D unused-variables`）；本机 macOS clippy 覆盖不到该 cfg 分支 |
| 2026-09-17 | 修复后 GitHub Actions run `35223745676`（`9063cae`，三平台） | 供给模块在三平台编译、PATH 定位路径无回归 | 通过：Build/Test/Format/Clippy 全绿（Windows 8m49s），runner 自带 cmake/ninja 走 PATH 路径 |

## 风险与回退

下载源不可达或校验失败会阻塞首次构建；保留"CMAKE 环境变量指定本机 cmake"的旁路并写明诊断。回退仅撤销本任务变更，恢复"要求本机预装"的 004 状态。

## 决策与工作记录

- 2026-09-16：自任务 004 的托管原则拆分立task；004 交付调度与诊断，本任务交付二进制供给。
- 2026-09-17：依赖 004 已完成且范围/验收明确，状态调整为 ready。根据预编译优先规则，CMake/Ninja 只消费带 SHA256 的官方或可信预编译资产；缺少资产时另立项目制品任务，不回退本地源码构建。
- 2026-09-17（方案）：定位顺序为 `CMAKE` 环境变量 → PATH → `target/panta-tools/` 托管缓存 → 固定资产下载；marker 记录资产 SHA256，与解包二进制同时有效即可离线复用，marker 不符（版本升级）先清场不覆盖。下载用平台自带 curl，CMake 包用平台自带 tar 解压（macOS/Windows bsdtar 兼容 zip），Ninja zip 用 `cmake -E tar`（libarchive）；解包后显式 chmod 可执行位。托管 Ninja 向 CMake 传 `CMAKE_MAKE_PROGRAM`，不依赖 PATH。SHA256 于首次下载实测并回填固定清单；Linux aarch64 无官方 CMake 资产，走 `CMAKE` 旁路并给出诊断。
- 2026-09-17（实施）：落地上表。已知限制：下载进度在成功 run 中被 cargo 隐藏（失败时完整输出）；CI runner 自带 cmake/ninja，走 PATH 定位路径，托管下载路径由本机 E2E 覆盖，三平台 CI 验证编译与 PATH 路径无回归。

## 完成摘要

已完成。开发者环境要求收敛为 git + rustup + 平台编译器：PATH 无 CMake/Ninja 时构建引导按固定资产（SHA256 实测回填固定清单）自动下载校验到根 `target/panta-tools/`，干净 PATH 全链构建、损坏归档拒绝、两档离线复用均在本机 E2E 验证；三平台 CI run `35223745676` 确认供给模块编译与 PATH 定位路径无回归。已知边界：下载进度走 curl 的 stderr 实时可见（stdout 诊断在成功 run 中被 cargo 隐藏）；Linux aarch64 无官方 CMake 资产，`CMAKE` 旁路诊断覆盖；Windows 非 unix 分支的 cfg 错误由 CI 抓出并已修复（`9063cae`）。
