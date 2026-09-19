# VTK 9.7.0 WebGPU/Cocoa prebuilt SDKs

Panta 项目受信 CI 按[冻结的构建描述](../../tools/sdk/vtk.cmake)生产的预编译
VTK SDK，提供 `RenderingWebGPU` 与平台 hardware window（macOS 为 Cocoa hardware window），供 `sdk-provision`
（任务 031）按 manifest 消费；开发者与普通 CI 不编译 VTK 源码。

## 源码与可复现性

- CI 源码与依赖资产均来自 GitHub：<https://github.com/Kitware/VTK.git>、
  <https://github.com/google/dawn/releases>
- tag `v9.7.0`（annotated tag 对象 `a78e2d950b5bf3aca6d78cc24b3153fd94cdaeec`）
  解引用 commit **`23f0a095621e91bbdbeace8451e22b950c8e5f46`**（"Update
  version number to 9.7.0"），未做任何修改（unmodified upstream sources）。
- 构建配置：Release、共享库、`VTK_GROUP_ENABLE_Qt=NO`、
  `VTK_ENABLE_WEBGPU=ON`、`VTK_MODULE_ENABLE_VTK_RenderingUI=YES`、
  `VTK_MODULE_ENABLE_VTK_RenderingWebGPU=YES`、
  `VTK_RELOCATABLE_INSTALL=ON`；macOS surface 使用
  `vtkCocoaHardwareWindow` 的 Metal layer/view；Dawn 使用 GitHub native
  Release `v20260720.160313` 的固定平台资产，并随 VTK SDK 一起提供其头文件、
  CMake package 和 native library。
- 每个归档内的 `panta-sdk.json` 记录 triple、源码 pin、构建开关与许可证
  入口（机器可读 provenance）。

## 许可证

- VTK 按 **BSD-3-Clause** 再分发；许可证文件随每个归档的
  `share/licenses/VTK/Copyright.txt` 提供（VTK 源码树内嵌的第三方组件
  许可同样以其 Copyright.txt 为准）。
- 本制品不链接 Qt；Qt Quick 应用侧的 Qt 供给与本制品独立。
- Dawn 按其上游许可证随制品提供；Dawn provenance、平台、资产 URL 和 SHA256
  写入 `panta-sdk.json`，native library 与 VTK 安装树共用 `lib` 布局。

## 校验

每个平台资产附带同名 `.sha256`（SHA256）。使用与供给规则见
[依赖获取规范](../../ai-docs/standards/dependency-acquisition.md) 与
[native-dependency-supply](../../ai-docs/modules/native-dependency-supply.md)。
