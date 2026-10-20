# Native 依赖供给与 SDK 制品

[模块导航](README.md) · [依赖获取规范](../standards/dependency-acquisition.md) · [任务 031](../task/031-prebuilt-native-dependencies.md) · [任务 038](../task/038-native-sdk-artifact-production.md)

## 当前实现

固定版本的 VTK、OCCT 与 Netgen SDK 已由受信 CI 生产并发布，manifest 和 CMake 消费入口位于 `native/cmake/sdk-provision.cmake`。供给逻辑校验版本、平台、SHA256、缓存归档与 staging package；配置失败时不会退回系统包或本地源码构建。VTK WebGPU 单独发布，OCCT 与启用 `USE_OCC` 的 Netgen 成对发布以保持 C++ ABI 一致。实际版本、资产和消费证据以任务 031/038 与 manifest 为准。

Qt、CMake、Ninja 和 GoogleTest 有各自的工具链供给入口，不属于本模块的 SDK 清单。实现与升级约束见[依赖获取规范](../standards/dependency-acquisition.md)。

## 尚未验收

任务 031 与 038 均为 `deferred`。已发布 SDK 和 fixture 供给测试不等于产品集成全部完成；剩余项包括桌面产品的依赖/许可证安装布局、Linux glibc 基线，以及制品 SBOM/provenance 发布流程。具体证据和范围以对应任务为准。

## 消费边界

CMake 通过 `panta_require_sdk` 获取受控 staging 根并使用 imported targets。adapter 不自行下载、猜测系统路径或引入 FetchContent fallback。版本、哈希、架构、编译器 ABI、许可证和 package 元数据以对应 manifest 与发布记录为准。
