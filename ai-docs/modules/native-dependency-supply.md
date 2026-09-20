# Native 依赖供给与 SDK 制品（规划）

[模块导航](README.md) · [依赖获取规范](../standards/dependency-acquisition.md) · [任务 031](../task/031-prebuilt-native-dependencies.md) · [任务 038](../task/038-native-sdk-artifact-production.md)

## 目标

VTK、OpenCASCADE 和 Netgen 是 CAE 主链路的 native 依赖，开发者和普通 CI 不应为每次构建重复编译它们。供给模块将固定版本、平台、架构、编译器 ABI、Qt 兼容范围和模块清单，交付可缓存、可校验的 SDK；CMake 只从 staging 根目录消费 `CONFIG` package。

预编译优先是构建契约，不是对上游发布形式的假设。上游没有完整 SDK 时，由受信 CI 生成一次项目制品，再由开发者下载；本地构建没有源码 fallback，也不能把 Python wheel 当作 C++ SDK。

## 制品生命周期

1. 031 维护依赖 manifest：版本、来源 URL、SHA256、目标 triple、ABI、Qt 兼容范围、CMake package 入口、模块和许可证。manifest 与供给实现已落地于 [native/cmake/sdk-provision.cmake](../../native/cmake/sdk-provision.cmake)（`panta_sdk_declare_version` / `panta_sdk_declare_asset` 登记，`panta_require_sdk` 供给并注入 `find_package(... CONFIG)`）。
2. 038 在固定工具链容器或 runner 中生成缺失的平台 SDK，记录源码 tag/commit、构建选项、依赖清单和许可证；制品生成不进入开发者的 `cargo build` 或普通 CMake 图。**ABI 互相锁定的依赖合并发布**：Netgen `USE_OCC=ON` 在 C++ 层链接构建时的 OCCT，而 OCCT 不承诺跨版本 C++ ABI 稳定——两者由同一管线、同一工具链生产并合并为一个 Release（`sdk-occt-netgen-<occt>-<netgen>`），升级成对进行，配对关系写入 Release 说明与 `panta-sdk.json`。
3. 发布前运行 package 自检：头文件、动态库、CMake config/imported targets、所需模块、运行库和许可证均存在，架构与 ABI 元数据一致；生成校验和、SBOM 和 provenance 记录。
4. 供给脚本按 manifest 下载到 `target/panta-deps/sdk/<name>/<version>/<triple>/`（Cargo 入口由 build.rs 注入该共享根；presets 直接 configure 时默认构建树内），归档缓存于 `<name>/archives/`，解包走临时目录并以原子 rename 发布；哈希不符删除归档，staging 标记（`.panta-sdk-provisioned`）损坏时按缓存归档重建。CMake 只接收对应 staging 根目录，并在缺包、哈希错误、配置文件缺失/歧义或 target 缺失时立即失败。
5. 007、009、010 只使用 imported targets 完成集成验证；升级必须先更新制品 manifest，再验证三平台最小窗口、几何和网格冒烟。

## 统一布局与诊断

建议归档内部布局为：

```text
<sdk-root>/
├── include/
├── lib/                 # 静态库、导入库或平台对应目录
├── bin/                 # 运行期 DLL/工具（按平台需要）
├── lib/cmake/<Package>/ # CONFIG package 与 targets
├── share/licenses/
└── panta-sdk.json       # 版本、triple、ABI、Qt、模块、来源和哈希
```

供给失败的诊断至少包含依赖名、期望版本、平台 triple、编译器/运行库 ABI、staging 路径、缺失 target 或模块、manifest 来源和修复动作。不能静默改用系统库、另一个版本或源码构建。

## 当前盘点结论（2026-09-18）

- 三个依赖的供给已全链路就绪：VTK 9.7.0（Release `sdk-vtk-9.7.0`）与 OCCT 8.0.1 + Netgen v6.2.2604（合并 Release `sdk-occt-netgen-8.0.1-6.2.2604`，成对发布——Netgen 链接其构建时的 OCCT，硬 ABI 锁定）均由 038 受信 CI 三平台生产并登记进 031 manifest；macOS 生产 consumer 烟测（occt/netgen/vtk 三依赖从 Release 真实下载消费）与离线复用全部通过。OCCT 弃用官方 Windows SDK（仅 Windows 有归档，跨平台工具链不一致，维护者决策 2026-09-18 三平台统一自托管）。
- 实证约束（009/010/007 消费时注意）：OCCT targets 无命名空间（`TKernel`/`TKDESTEP`）；Netgen 包配置为 `NetgenConfig.cmake`，`find_package` 必须用 `Netgen`（Linux ext4 大小写敏感）；Netgen 运行期加载依赖同平台 OCCT 资产（staging 相邻，消费侧处理 rpath/加载路径）；VTK WebGPU 目标为 `VTK::` 命名空间，Dawn 目标为 `dawn::webgpu_dawn`，不再含 Qt VTK GUI 集成。
- 剩余项属集成任务自身：007（视口运行时）、009（STEP 集成）、010（网格集成）的真实链接/运行冒烟，以及 Linux glibc 有效基线的运行验证回写。

供给实现的验证证据见 [任务 031](../task/031-prebuilt-native-dependencies.md)：ctest `Build.SdkProvision` 用 fixture SDK（file:// 下载、project NONE）驱动成功、哈希不符、缺资产、离线缓存、marker 重建、版本隔离、内嵌归档/包装目录、配置歧义等路径，三平台可同路径执行。在全平台 manifest 和 configure/package 冒烟证据完成前，031 保持进行中，007/009/010 不启动第三方源码构建。

## 边界

Qt、CMake 和 Ninja 的预编译供给由现有 005/020 负责；本模块只规定 VTK/OCCT/Netgen SDK 与其制品生产。运行时复制、签名校验和增量更新由 013、037 管理；本模块提供版本与许可证元数据供它们消费。
