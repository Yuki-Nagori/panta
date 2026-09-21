# VTK 9.7.0 WebGPU hardware-window SDKs

Panta 项目受信 CI 按[冻结的构建描述](../../tools/sdk/vtk.cmake)生产的预编译
VTK SDK，提供 `RenderingWebGPU` 与平台 hardware window（Linux 为 Wayland、macOS 为 Cocoa、Windows 为 Win32），供 `sdk-provision`
（任务 031）按 manifest 消费；开发者与普通 CI 不编译 VTK 源码。

## 源码与可复现性

- CI 源码与依赖均来自 GitHub：<https://github.com/Kitware/VTK.git>、
  <https://github.com/google/dawn.git>
- tag `v9.7.0`（annotated tag 对象 `a78e2d950b5bf3aca6d78cc24b3153fd94cdaeec`）
  解引用 commit **`23f0a095621e91bbdbeace8451e22b950c8e5f46`**（"Update
  version number to 9.7.0"），未做任何修改（unmodified upstream sources）。
- 构建配置：Release、C++20、共享库、`VTK_GROUP_ENABLE_Qt=NO`；通过
  `CMAKE_PROJECT_VTK_INCLUDE` 将 `RenderingWebGPU` 的上游 `cxx_std_17`
  target requirement 提升为 C++20；Dawn 按上游 source tag 构建并显式开启
  `DAWN_ENABLE_RTTI=ON`，不在 VTK 侧伪造 RTTI 或 ABI 符号；
  `VTK_GROUP_ENABLE_Rendering/StandAlone/Imaging/Parallel/Views/Web=DONT_WANT`，
  只直接启用 `RenderingUI` 与 `RenderingWebGPU` 及其传递依赖；
  `VTK_ENABLE_WRAPPING=OFF`，不打包 Python/Java 等包装层；
  `VTK_ENABLE_WEBGPU=ON`、`VTK_MODULE_ENABLE_VTK_RenderingUI=YES`、
  `VTK_MODULE_ENABLE_VTK_RenderingWebGPU=YES`、
  `VTK_RELOCATABLE_INSTALL=ON`；Linux 选择原生 Wayland（`VTK_USE_X=OFF`、
  `VTK_USE_Wayland=ON`），macOS 使用 `vtkCocoaHardwareWindow` 的 Metal
  layer/view，Windows 使用 `vtkWin32HardwareWindow`；Dawn source tag 为
  `v20260421.125655`（commit `b073946efbf0de690e2aeec16ef0d5c68362c951`，
  VTK 9.7.0 上游 WebGPU 文档指定），并随 VTK SDK 一起提供其头文件、CMake
  package 和 native library；Linux 额外随 VTK CMake package 提供上游遗漏的
  `FindWAYLAND.cmake` 与 `FindXKBCOMMON.cmake`，保证安装树可独立被消费。
- 每个归档内的 `panta-sdk.json` 记录 triple、源码 pin、构建开关与许可证
  入口（机器可读 provenance）。

## 覆盖重产记录

本 Release 维持单一 tag：管线重产时清空旧资产后全量重传（sdk-vtk.yml
`publish` job），归档字节随重产变化，消费侧以 `sdk-provision.cmake` 登记的
SHA256 为准（任务 049 起另有记录）。

## 许可证

- VTK 按 **BSD-3-Clause** 再分发；许可证文件随每个归档的
  `share/licenses/VTK/Copyright.txt` 提供（VTK 源码树内嵌的第三方组件
  许可同样以其 Copyright.txt 为准）。
- 本制品不链接 Qt；Qt Quick 应用侧的 Qt 供给与本制品独立。
- Dawn 按其上游许可证随制品提供；Dawn repository、tag、commit、RTTI 构建选项
  写入 `panta-sdk.json`，native library 与 VTK 安装树共用 `lib` 布局。

## 校验

每个平台资产附带同名 `.sha256`（SHA256）。使用与供给规则见
[依赖获取规范](../../ai-docs/standards/dependency-acquisition.md) 与
[native-dependency-supply](../../ai-docs/modules/native-dependency-supply.md)。
