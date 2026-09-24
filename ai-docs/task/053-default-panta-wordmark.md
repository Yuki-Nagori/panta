# 053 — 默认视口立体 panta 字样

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[007 VTK 视口](007-vtk-quick-viewport.md)、[081 视觉体系](081-qml-visual-language-and-iconography.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-25

## 目标与视觉契约

重新设计并实现默认视口的 Welcome 场景。整体应用视觉方向遵循 [081](081-qml-visual-language-and-iconography.md)，本任务只负责 VTK 中的 Welcome 内容，不再建立另一套应用颜色和图标规范。视觉契约为浅色中性背景、居中的轻量小写 `panta`、保留原有冷暖过渡的彩色渐变、较浅的挤出厚度、下方固定且清楚可读的 50px `Welcome!` 文案与留白；字标尺寸通过 Welcome 专属相机适配边距控制，不改变 STL 视口比例。渐变仅作品牌装饰，不表示物理量或求解结果。Welcome 整体由 `native/visualization/src/vtk/welcome/` 单独维护，模型载入后隐藏文案。

本任务优先完成 081 已明确的 Welcome 外观子集；081 的应用级图标、主题与其余 QML 迁移仍保持独立计划。Nunito 字体尝试已撤回；圆环拼字 STL 仅保留为本地设计产物，用户已决定不接入应用，不能视为后续既定方案。

欢迎图形与真实求解结果分离，不修改工程数据。若使用仿真云图风格配色，应明确其装饰性质，不附带虚构物理量或结果色标。

## 必读

[VTK](../standards/vtk.md)、[可视化架构](../architecture/visualization.md)、[注释](../standards/comments.md)、[文件规范](../standards/repository-hygiene.md)、[验证](../standards/validation-and-review.md)、[提交](../standards/commits.md)。

## 实施

1. 在 VTK Welcome 模块内实现字形、浅挤出、平滑渐变材质和屏幕固定文案；保持 RenderScene 后端中立。
2. 用几何测试验证闭合拓扑、字孔、绕序、渐变覆盖、厚度与居中；场景测试验证文案显隐和资源释放。
3. 在真实 WebGPU 窗口确认最终视觉、初始相机、隐藏/恢复及销毁路径。
4. 删除临时注塑色带和失效测试、注释，更新可视化架构及 RenderScene 文档；无兼容例外。

## 验收

- [x] 053 使用的 Welcome 视觉子集已依据维护者认可的参考方向确定；081 的其余工作不在本任务内。
- [ ] 字形保持轻量清晰，浅挤出与装饰渐变符合选定方案，缩小后的 Welcome 字样和放大的固定文案在宽窄窗口完整显示。
- [ ] 几何与渲染检查、构建、相关测试及静态检查通过；记录真实窗口证据。
- [ ] 临时实现及废弃资源已按替换范围清理，文档、任务与索引同步。

## 历史验证与产物（不代表重新实现已完成）

- 2026-09-22：临时 VTK 字样已在 `6a1b9a8` 提交，采用内置字形挤出与装饰顶点色，移除旧测试球体；未接入外部字体或 STL。
- macOS 26.3.1 arm64、Qt 6.11.2、VTK 9.7.0；当时 `cargo test --locked --workspace` 通过，包含 50 项 native/QML CTest；格式、includes、qmllint、clang-tidy 与 diff 检查通过。日志：`artifacts/053/precommit-tests.log`。
- 当时已检查真实 macOS WebGPU 全尺寸与 640px 窄窗口；几何测试覆盖封闭面、边绕序、正体积、字孔 Euler 特征、居中及有效 RGB 色值。该证据不代替未来实现验收或 Windows/Linux GPU 验证。
- 圆环拼字候选保存于 `artifacts/053/panta-ring-text.stl`，约 123.1×36.4×4.4mm、管径 4.4mm、五个封闭字母。208656 个三角形，无零面积面且绕序一致；检查记录 `stl-check.json`、交互预览和生成脚本位于同目录。产物不提交、不接入应用。
- 历史 API 依据：[vtkVectorText](https://vtk.org/doc/nightly/html/classvtkVectorText.html)（2026-09-22 查阅；以锁定 SDK 头文件为准），不预定未来实现路线。

## 本次调整

- 2026-09-22：按用户要求将任务从 done 改为 planned，撤销完成勾选，原方案不作为最终交付。
- 2026-09-25：开始实现；用户反馈字样应减轻粗重感并保留原有彩色渐变，故将浅色背景、轻量字形和浅挤出冻结为 053 子集。081 仍负责应用级视觉体系，其余工作不阻塞本任务。
- 2026-09-25：真实窗口截图显示 `Welcome!` 文案过小且与字样间距不足；字号按维护者调整为 50px 并下移。字标使用更大的 Welcome 专属相机适配边距缩小，不影响导入 STL 的视口比例；需重新验收宽窗口留白与模型载入后的隐藏行为。
- 2026-09-25：macOS Cocoa 真实窗口确认 50px `Welcome!` 固定文案与缩小后的渐变 `panta` 均完整可见；当前需继续记录窄视口与载入模型时的验收。
- 2026-09-25：将字标几何测试从过时的 `default_wordmark_test` 命名重命名为 `welcome_wordmark_test`，与 Welcome 场景归属一致；独立场景生命周期测试仍由 `welcome_scene_test` 覆盖。历史验证记录中的旧 target / CTest 名称保留为当时的原始记录。
