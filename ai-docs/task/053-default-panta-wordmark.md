# 053 — 默认视口立体 panta 字样

- 状态：done
- 阶段：应用平台扩展
- 依赖：[007 VTK 视口](007-vtk-quick-viewport.md)
- 优先级：P1
- 负责人：Codex
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与范围

将默认蓝色测试球体替换为三维 `panta` 字样，具有可见厚度与斜视角，采用用户参考图的蓝—绿—黄—红注塑仿真云图配色及柔和高光。配色是欢迎场景的示意色，不是求解结果；不添加色标或物理量，不改变工程数据或新增交互命令。

追加设计输出：用户要求另做圆环、圆弧与圆柱笔画拼成的 `panta` STL 候选，保留本地生成脚本和预览图；先交付文件供评估，默认场景保持已回退的版本。STL 只保存几何，不包含配色。

## 必读

[VTK](../standards/vtk.md)、[可视化架构](../architecture/visualization.md)、[注释](../standards/comments.md)、[文件规范](../standards/repository-hygiene.md)、[验证](../standards/validation-and-review.md)、[提交](../standards/commits.md)。

## 实施

- 使用 VTK 内置字形生成轮廓并挤出，保留字孔与侧面；无外部字体资源。
- 使用顶点色与法线保持立体光照；默认相机适配字样与视口宽高比。
- 删除旧球体生成逻辑与失效描述；保持 RenderScene 的可见性与重建契约，无兼容例外。

## 验收

- [x] 默认视口显示清晰、居中的立体 panta，宽/窄窗口不裁字。
- [x] 云图色带平滑，字孔、侧面与法线正确；CPU 几何检查通过。
- [x] 构建、相关测试、格式及静态检查通过；真实 macOS WebGPU 窗口核对。
- [x] 同步文档、任务与索引，仅本地 commit，不 push。

## 验证与记录

- 2026-09-22：创建任务并登记索引。官方 vtkVectorText 文档与锁定头文件确认内置 ASCII 字形输出三角网格；现有默认球体位于 VTK 私有适配器。生成、法线与光照均保持在私有适配器内。
- 官方依据：[vtkVectorText](https://vtk.org/doc/nightly/html/classvtkVectorText.html)（2026-09-22 查阅；以本机 9.7.0 SDK 头文件为准）。

## 风险与回退

曲面法线和相机宽高比可能造成接缝或裁剪；验证网格边界、实际光照与宽窄窗口。失败时撤回本任务提交，保留工程数据。

- 2026-09-22：用户试用圆润字体后要求回退，恢复首次预览的 VTK 字形、厚度和云图配色；删除尝试的 Nunito 字体资源及轮廓转换依赖。
- 2026-09-22：用户收窄当前工作到 STL，界面修改停止推进。已生成 `artifacts/053/panta-ring-text.stl` 与独立交互预览 `panta-ring-text-preview.html`；OCCT 8.0.1 圆环/圆弧/胶囊笔画布尔并集，每字一个有效实体。尺寸约 123.1×36.4×4.4mm，管径 4.4mm，无底座，五字独立。
- STL 导出后清理接缝零面积面，208656 个三角形、5 个连通分量；每条无向边恰有两个面、相邻绕序一致、无零面积面。证据 `artifacts/053/stl-check.json`；生成源与构建脚本保存在同目录。浏览器实际渲染核对圆润笔画、字孔、云图预览及旋转操作。预览着色不写入 STL。


## 最终验证与交付

- 2026-09-22：用户确认维持当前 VTK 立体字样，STL 仅留作本地候选，不接入应用、不提交产物；代码、构建与资源中无 Nunito/STL 加载路径。
- macOS 26.3.1 arm64、Qt 6.11.2、VTK 9.7.0；仓库根目录 `cargo test --locked --workspace` 通过（含 50 项 native/QML CTest），记录在 `artifacts/053/precommit-tests.log`。几何测试覆盖封闭面、边绕序、正体积、字孔 Euler 特征、居中及有效 RGB 顶点色。
- 提交前修正几何测试中索引乘法的隐式整数扩宽告警，使用显式尺寸类型；相关几何测试再次通过。
- `cargo format --check`、`cargo lint includes --check`、`cargo lint qmllint`、`cargo lint clang-tidy` 与 diff 空白检查通过。
- 本机真实 WebGPU 窗口核对全尺寸与 640px 窄窗口：字样完整、光照/颜色正确，恢复窗口尺寸后正常；未将此视作 Windows/Linux GPU 验证。
- 本次提交同步实现、测试、构建和任务状态；删除旧球体生成及废弃字体尝试，无兼容例外，不 push。
