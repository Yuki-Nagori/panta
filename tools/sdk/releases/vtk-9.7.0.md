# VTK 9.7.0 prebuilt SDKs

Panta 项目受信 CI 按[冻结的构建描述](../../tools/sdk/vtk.cmake)生产的预编译
VTK SDK，供 `sdk-provision`（任务 031）按 manifest 消费；开发者与普通 CI
不编译 VTK 源码。

## 源码与可复现性

- 来源：<https://gitlab.kitware.com/vtk/vtk.git>
- tag `v9.7.0`（annotated tag 对象 `a78e2d950b5bf3aca6d78cc24b3153fd94cdaeec`）
  解引用 commit **`23f0a095621e91bbdbeace8451e22b950c8e5f46`**（"Update
  version number to 9.7.0"），未做任何修改（unmodified upstream sources）。
- 构建配置：Release、共享库、`VTK_QT_VERSION=6`、`VTK_GROUP_ENABLE_Qt=YES`、
  `VTK_MODULE_ENABLE_VTK_GUISupportQtQuick=YES`；Qt 为官方预编译 6.11.2
  （qtsdkrepository，SHA256 见 `native/cmake/qt-provision.cmake`）。
- 每个归档内的 `panta-sdk.json` 记录 triple、源码 pin、构建开关与许可证
  入口（机器可读 provenance）。

## 许可证

- VTK 按 **BSD-3-Clause** 再分发；许可证文件随每个归档的
  `share/licenses/VTK/Copyright.txt` 提供（VTK 源码树内嵌的第三方组件
  许可同样以其 Copyright.txt 为准）。
- 本制品链接 Qt 6.11.2（LGPL-3.0）但**不包含 Qt 二进制**；Qt 仍由
  `qt-provision` 从 Qt 官方渠道获取。若在下游分发物中捆绑 Qt 运行库，
  需另行满足 Qt LGPL 义务（任务 013 范围）。

## 校验

每个平台资产附带同名 `.sha256`（SHA256）。使用与供给规则见
[依赖获取规范](../../ai-docs/standards/dependency-acquisition.md) 与
[native-dependency-supply](../../ai-docs/modules/native-dependency-supply.md)。
