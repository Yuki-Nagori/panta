# Native 依赖供给与 SDK 制品

[模块导航](README.md) · [依赖获取规范](../standards/dependency-acquisition.md) · [任务 031](../task/031-prebuilt-native-dependencies.md) · [任务 038](../task/038-native-sdk-artifact-production.md)

## 当前实现

VTK、OCCT、Netgen 与 GoogleTest SDK 的生产、自检和发布由独立仓库 [sdk-packaging](https://github.com/Jack-Lab-at-EIT/sdk-packaging) 负责；Panta 仅在 `native/cmake/sdk-provision.cmake` 维护消费端 manifest。供给逻辑校验版本、平台、SHA256、缓存归档与 staging package；配置失败时不会退回系统包或本地源码构建。VTK WebGPU 单独发布，OCCT 与启用 `USE_OCC` 的 Netgen 成对发布以保持 C++ ABI 一致。实际消费版本、资产和验证证据以任务 031/038 与 manifest 为准。

Qt、CMake 和 Ninja 使用各自的工具链供给入口；GoogleTest 虽由测试构建按需消费，仍属于本模块的 SDK manifest。实现与升级约束见[依赖获取规范](../standards/dependency-acquisition.md)。

## 尚未验收

任务 031 为 `deferred`，038 正在迁移到外部 SDK 生产仓库。已发布 SDK 和 fixture 供给测试不等于产品集成全部完成；剩余项包括桌面产品的依赖/许可证安装布局、Linux glibc 基线，以及制品 SBOM/provenance 的消费侧核验。具体证据和范围以对应任务为准。

## 消费边界

CMake 通过 `panta_require_sdk` 获取受控 staging 根并使用 imported targets。adapter 不自行下载、猜测系统路径或引入 FetchContent fallback。版本、哈希、架构、编译器 ABI、许可证和 package 元数据以对应 manifest 与发布记录为准。
