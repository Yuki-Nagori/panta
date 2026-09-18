# Netgen v6.2.2604 prebuilt SDKs

Panta 项目受信 CI 按[冻结的构建描述](../../tools/sdk/netgen.cmake)生产的
预编译 Netgen SDK，供 `sdk-provision`（任务 031）按 manifest 消费；开发者
与普通 CI 不编译 Netgen 源码。

## 源码与可复现性

- 来源：<https://github.com/NGSolve/netgen.git>
- tag `v6.2.2604` 解引用 commit **`3ee489c7d58fdbc2a6708cca3cbaefaae506dc17`**，
  未做任何修改（unmodified upstream sources）。
- 构建配置：Release、共享库；`USE_OCC=ON`，链接同一管线的 OCCT 8.0.1
  制品（commit `b8f597c677811d1f9f4d8a97f5ae2825c0353a42`，同 triple、
  同工具链——Netgen↔OCCT ABI 匹配见依赖规范）；`USE_GUI/USE_PYTHON/
  USE_MPI/USE_JPEG/USE_MPEG` 关闭。
- 运行本制品需要同管线的 OCCT SDK（`sdk-occt-8.0.1` Release，同 triple）。
- 每个归档内的 `panta-sdk.json` 记录 triple、源码 pin、构建开关、依赖与
  许可证入口（机器可读 provenance）。

## 许可证

- Netgen 按 **LGPL-2.1** 再分发；本制品为动态链接的共享库，许可证文本随
  每个归档的 `share/licenses/Netgen/COPYING` 提供。
- 源码可自上方 repository 在所示 tag 获取。

## 校验

每个平台资产附带同名 `.sha256`（SHA256）。使用与供给规则见
[依赖获取规范](../../ai-docs/standards/dependency-acquisition.md) 与
[native-dependency-supply](../../ai-docs/modules/native-dependency-supply.md)。
