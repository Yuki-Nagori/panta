# 056 — QML 周边 C++ 简化与性能评审

- 状态：done
- 阶段：应用平台扩展
- 依赖：[055 QML 整理](055-qml-review-and-cleanup.md)、[007 原生视口](007-vtk-quick-viewport.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-22

## 范围与目标

整体检查 QML 周边已实现的 C++：入口与模块注册、Shell/服务 bridge、图标 provider、CaeViewport 与原生 VTK 宿主及其测试。注重简化代码、所有权/信号生命周期和 GUI 线程性能，修正有证据的问题。保持当前外观和业务边界，053 仍为 planned；不实现计划中的引擎重载、主题服务或字样重设计。

## 实施与验证计划

- 检查资源加载、重复分配/转换、几何同步、信号频率与渲染调度，先确认既有缓存和合并机制，避免重复缓存或无证据的优化。
- 删除无用分支、重复状态或失效注释；保留跨平台资源释放和异步任务保护。
- 性能结论记录实际场景与测量方法，区分代码路径分析、微基准和真实窗口；不从微基准推断整机帧率。
- 对行为修复补最小回归，运行适用的构建、测试、格式/静态检查及真实窗口验证。

## 验收

- [x] 已检查上述实现，记录发现、处理与未覆盖边界。
- [x] 简化不改变当前外观，正确处理 QObject/平台 surface 生命周期。
- [x] 性能发现有可复查依据，未引入重复缓存或固定帧率刷新。
- [x] 相关测试和检查通过，任务/模块说明同步，仅本地 commit。

## 评审与验证记录

- 2026-09-22：工作区干净，登记任务并开始检查。

### 发现与处理

- **视口析构崩溃**：真实窗口回归和 LLDB 确认 `QQuickItem::~QQuickItem → windowChanged → VtkViewport` 回调在 `Impl` 销毁后仍访问其连接句柄。析构先断开自身回调；新增无需 GPU 的窗口归属/排队刷新析构回归。
- **重复 GPU 提交**：原实现的 `apply_state` 与 `geometryChange` 分别直接渲染。同一次调用中连续 100 次状态+宽度更新，实际日志计得 199 次 Render；统一排队刷新后为 1 次，重复相同外观/仅推进修订号不重绘。构建场景、应用状态与绘制分开，移除初始化和状态提交中的重复设置。
- **窗口与几何同步**：将窗口连接集中到 `bind_window`，包括构造时已有父窗口的情况；移除重复 visible/visibility 订阅。跨窗口先释放旧资源再按 CPU 状态重建；祖先平移同步原生区域但不重绘；隐藏/零尺寸恢复会补帧。
- **空闲轮询**：TaskHost 原来常开 10ms 定时器（基线用例在 60ms 空闲期间观察到 4 次唤醒）。改为成功提交后启动，已提交任务的终态全部转发后停止；不以服务运行数归零停表。回归覆盖零时长成功/失败、空闲后再提交、终态信号中同步提交下一任务；既有取消/进度/响应性用例继续通过。当前首页未实例化 TaskHost，因此不把此改动宣称为首页 CPU 降幅。
- **字样生成分配**：边使用信息由每边一个动态 vector 改为方向+计数；按已知数量预分配顶点/颜色/细分三角形；前后表面复用颜色计算。输出的 4960 个顶点、颜色和 9912 个三角面索引逐项完全相同，不改 053 的当前外观或 planned 状态。
- **其它简化/核对**：PathHost 直接传递错误输出参数，删除三处临时字符串与重复中转分支；修正入口日志和 CPU 状态注释。检查静态模块注册、ShellViewModel 通知去重、CaeViewport 所有权、图标 provider 的局部绘图对象和平台 surface 实现；未引入第二层图标缓存，也未扩展尚未实现的热重载/i18n 服务。

### 测量与验证

环境：仓库根目录，macOS 26.3.1 arm64、Qt 6.11.2、VTK 9.7.0；主构建为 Debug。证据保存在忽略目录 `artifacts/056/`。

| 检查 | 实际结果 |
|---|---|
| `cargo build --locked`、`cargo test --locked --workspace` | 通过；52/52 native/QML CTest，包含新增任务宿主与无头析构回归；真实 GPU 用例默认明确跳过 |
| `PANTA_TEST_NATIVE_VIEWPORT=1 QT_QPA_PLATFORM=cocoa target/native/debug/app/panta_qml_viewport_module_test` | 5/5 QtTest 检查通过；覆盖延迟显示、合帧、相同状态、祖先移动、隐藏/零尺寸恢复、跨窗口重建与析构；基线 199 帧、修复后 1 帧；详见 viewport-before/after.log |
| `cargo sanitize` | ASan+UBSan 52/52、TSan 39/39 通过；沿用仓库的自有代码插桩边界，Qt/VTK 预编译库及真实 GPU 路径不等于已插桩覆盖 |
| `cargo lint clang-tidy --check`、`cargo lint includes --check`、`cargo lint qmllint --check` | 通过；include-cleaner 的新增头文件问题已修正后复验 |
| `cargo format` 与 `git diff --check` | 通过；提交前再执行格式检查 |
| 最新 Panta Preview 桌面构建 | 重启并目视核对，首页布局、字样几何和着色保持原样 |

字样微基准：`artifacts/056/CMakeLists.txt` 构建独立 Release 对比程序，AppleClang 17.0.0（两版同编译器/优化配置、同一 VTK SDK），每版先预热 10 次，再交错运行 7 轮×150 次。每次 CPU 生成中位耗时 **1169.28µs → 892.274µs（约减少 24%）**；逐项数据比较通过。复现命令：

```sh
cmake -S artifacts/056 -B artifacts/056/build -DCMAKE_BUILD_TYPE=Release \
  -DVTK_DIR="$PWD/target/panta-deps/sdk/vtk/9.7.0/macos-arm64/lib/cmake/vtk-9.7"
cmake --build artifacts/056/build
artifacts/056/build/wordmark_bench
```

该数字只衡量当前小型默认网格的 CPU 生成，不包括 GPU 初始化、实际呈现耗时或应用启动；未做整体帧率/内存峰值宣称。Windows/Wayland 真实窗口与跨屏迁移本机无法验证；现有轴对齐原生区域不支持任意 QML transform/clip，本轮未扩展该能力。
