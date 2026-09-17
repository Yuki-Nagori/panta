# Native 依赖供给与 SDK 制品（规划）

[模块导航](README.md) · [依赖获取规范](../standards/dependency-acquisition.md) · [任务 031](../task/031-prebuilt-native-dependencies.md) · [任务 038](../task/038-native-sdk-artifact-production.md)

## 目标

VTK、OpenCASCADE 和 Netgen 是 CAE 主链路的 native 依赖，开发者和普通 CI 不应为每次构建重复编译它们。供给模块将固定版本、平台、架构、编译器 ABI、Qt 兼容范围和模块清单，交付可缓存、可校验的 SDK；CMake 只从 staging 根目录消费 `CONFIG` package。

预编译优先是构建契约，不是对上游发布形式的假设。上游没有完整 SDK 时，由受信 CI 生成一次项目制品，再由开发者下载；本地构建没有源码 fallback，也不能把 Python wheel 当作 C++ SDK。

## 制品生命周期

1. 031 维护依赖 manifest：版本、来源 URL、SHA256、目标 triple、ABI、Qt 兼容范围、CMake package 入口、模块和许可证。
2. 038 在固定工具链容器或 runner 中生成缺失的平台 SDK，记录源码 tag/commit、构建选项、依赖清单和许可证；制品生成不进入开发者的 `cargo build` 或普通 CMake 图。
3. 发布前运行 package 自检：头文件、动态库、CMake config/imported targets、所需模块、运行库和许可证均存在，架构与 ABI 元数据一致；生成校验和、SBOM 和 provenance 记录。
4. 031 的供给脚本按 manifest 下载到 `target/panta-deps/<name>/<version>/<triple>/`，临时解包目录通过原子 rename 发布。CMake 只接收对应 staging 根目录，并在缺包、哈希错误、架构不符或 target 缺失时立即失败。
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

## 当前盘点结论（2026-09-17）

- OCCT `V8.0.1` 有官方 Windows 预编译归档并含 `OpenCASCADEConfig.cmake`；目前只将它作为 Windows 候选，尚未证明 Qt/编译器 ABI 集成。
- VTK `v9.7.0` 官方渠道确认源码归档、Python wheels 和 SDK 提示，本轮没有确认包含 `GUISupportQtQuick` 的三平台 C++ SDK；必须由 038 或新的可信上游包补齐。
- Netgen 更新 tag `v6.2.2607` 没有对应 release 预编译资产；当前清单的 `v6.2.2604` 保持不变，等待与 OCCT ABI 一起生产或取得 SDK。

在全平台 manifest 和 configure/package 冒烟证据完成前，031 保持进行中，007/009/010 不启动第三方源码构建。

## 边界

Qt、CMake 和 Ninja 的预编译供给由现有 005/020 负责；本模块只规定 VTK/OCCT/Netgen SDK 与其制品生产。运行时复制、签名校验和增量更新由 013、037 管理；本模块提供版本与许可证元数据供它们消费。
