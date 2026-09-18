# OCCT 8.0.1 + Netgen v6.2.2604 prebuilt SDKs

Panta 项目受信 CI 按[冻结的构建描述](../../tools/sdk/)生产的预编译 OCCT 与
Netgen SDK，供 `sdk-provision`（任务 031）按 manifest 消费；开发者与普通
CI 不编译两者源码。**两者在同一管线以同一工具链生产并发布**：Netgen
`USE_OCC=ON` 链接本 Release 的 OCCT（同 triple、同编译器），升级必须成对
进行（Netgen↔OCCT ABI 锁定，见依赖规范）。

## 源码与可复现性

- OCCT：<https://github.com/Open-Cascade-SAS/OCCT.git> tag `V8.0.1`
  解引用 commit **`b8f597c677811d1f9f4d8a97f5ae2825c0353a42`**
- Netgen：<https://github.com/NGSolve/netgen.git> tag `v6.2.2604`
  解引用 commit **`3ee489c7d58fdbc2a6708cca3cbaefaae506dc17`**
- 两者均未修改（unmodified upstream sources）。
- OCCT 构建配置：Release、共享库；模块 `Draw`/`Visualization`/`DETools`
  关闭，`USE_FREETYPE`/`USE_XLIB` 关闭（渲染归 VTK；制品不依赖系统第三方
  库）；其余默认启用（含 STEP 所需 DataExchange）。
- Netgen 构建配置：Release、共享库；`USE_OCC=ON`、`USE_GUI/USE_PYTHON/
  USE_MPI/USE_JPEG/USE_MPEG` 关闭。
- 每个归档内的 `panta-sdk.json` 记录 triple、源码 pin、构建开关、依赖与
  许可证入口（机器可读 provenance）。

## 许可证

- OCCT 按 **LGPL-2.1（附 OCCT 例外）** 再分发：`share/licenses/OCCT/`
  （`LICENSE_LGPL_21.txt`、`OCCT_LGPL_EXCEPTION.txt`）。
- Netgen 按 **LGPL-2.1** 再分发：`share/licenses/Netgen/LICENSE`。
- 两者均为动态链接共享库；源码可自上方 repository 在所示 tag 获取。

## 校验

每个平台资产附带同名 `.sha256`（SHA256）。使用与供给规则见
[依赖获取规范](../../ai-docs/standards/dependency-acquisition.md) 与
[native-dependency-supply](../../ai-docs/modules/native-dependency-supply.md)。
