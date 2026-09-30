# 内置材料资源

`pp-mineral-25.json` 是内置默认材料：聚丙烯（PP），矿物填充比例 25%，版本 ID 为 `pp-mineral-25-v1`。

温度为 K、应力为 Pa、剪切速率为 s⁻¹，比热为 J/(kg·K)、导热率为 W/(m·K)，PVT 比容为 m³/kg。资源包含 Cross-WLF、Modified Tait 及比热 / 导热率表。Rust 向界面提供处理温度和剪切限制摘要，转换为 °C 和 kPa。当前仅用于材料选择，不代表已经完成求解器适用性和材料完整性校验。

Rust 使用 `include_str!` 内置资源，运行时不访问 `target/`。参数变化应使用新版本 ID，避免已有材料引用的含义静默变化；完整材料定义和区域分配见 [分析配置](../../ai-docs/architecture/study-and-automation.md)。
