# OCCT 8.0.1 prebuilt SDKs

Panta 项目受信 CI 按[冻结的构建描述](../../tools/sdk/occt.cmake)生产的预编译
OCCT SDK，供 `sdk-provision`（任务 031）按 manifest 消费；开发者与普通 CI
不编译 OCCT 源码。三平台统一由本管线生产（维护者决策 2026-09-18：官方
仅有 Windows 归档，为保证跨平台工具链一致，全部走自托管制品）。

## 源码与可复现性

- 来源：<https://github.com/Open-Cascade-SAS/OCCT.git>
- tag `V8.0.1` 解引用 commit **`b8f597c677811d1f9f4d8a97f5ae2825c0353a42`**，
  未做任何修改（unmodified upstream sources）。
- 构建配置：Release、共享库（`BUILD_LIBRARY_TYPE=Shared`）；模块
  `Draw`/`Visualization`/`DETools` 关闭（渲染归 VTK、交互 DRAW 非本项目
  范围），其余默认启用（含 STEP 所需 DataExchange）。
- 每个归档内的 `panta-sdk.json` 记录 triple、源码 pin、构建开关与许可证
  入口（机器可读 provenance）。

## 许可证

- OCCT 按 **LGPL-2.1（附 OCCT 例外）** 再分发；本制品为动态链接的共享库，
  许可证文本与例外条款随每个归档的 `share/licenses/OCCT/` 提供
  （`LICENSE_LGPL_21.txt`、`OCCT_LGPL_EXCEPTION.txt`）。
- 源码可自上方 repository 在所示 tag 获取。

## 校验

每个平台资产附带同名 `.sha256`（SHA256）。使用与供给规则见
[依赖获取规范](../../ai-docs/standards/dependency-acquisition.md) 与
[native-dependency-supply](../../ai-docs/modules/native-dependency-supply.md)。
