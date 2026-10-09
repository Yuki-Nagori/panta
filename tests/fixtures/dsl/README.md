# DSL 输出回归夹具

本仓库自建文本，无外部数据或单位；按仓库许可证分发。

`artifacts.pa` 覆盖 locale 别名、跨 context 声明顺序、独立注释、复数、消息状态、旧 source、翻译注释、缺少翻译及 XML 特殊字符。
`artifacts.formatted.pa` 与 `artifacts.ts` 由 task110 拆分前的 `panta-dslc` 生成，是确定性输出基线。格式保留源码声明顺序，TS 使用 context / 消息键的索引顺序。
