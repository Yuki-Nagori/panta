# 技术规范索引

这些规范用于后续 task 的实现与评审。技术规范按各篇查阅日期附官方依据；通用工作规范标明项目约定，更新日期不等于外部资料查阅日期。不是官方文档的逐字翻译，也不是已验证依赖矩阵。

## 通用规范

| 主题 | 文档 | 何时阅读 |
|---|---|---|
| 无死代码与兼容例外 | [代码生命周期](code-lifecycle.md) | 替换/删除功能、修改版本处理 |
| 每次 commit 的一致性 | [提交规范](commits.md) | 每次提交前 |
| 注释与接口契约 | [注释规范](comments.md) | 新增或修改代码/API |
| 文件、产物与忽略规则 | [仓库文件规范](repository-hygiene.md) | 添加配置、目录、资产或依赖 |
| 文档与示例 | [文档规范](documentation.md) | 新建/修改文档、任务或示例 |
| 验证与变更评审 | [验证与评审](validation-and-review.md) | 实施和完成任务 |
| 测试目录与入口 | [测试规范](testing.md) | 新增、移动或注册 Rust/C++/QML 测试 |

## 技术规范

| 技术/主题 | 文档 | 适用范围 |
|---|---|---|
| 版本与依赖选型 | [技术基线](baseline.md) | 所有任务 |
| 依赖获取与主平台 | [依赖获取](dependency-acquisition.md) | 环境搭建、依赖升级与回退 |
| C++20 | [C++](cpp.md) | native 核心与适配器 |
| C++ 测试 | [GTest](gtest.md) | native 单元与行为测试 |
| Rust 2024 | [Rust](rust.md) | 应用平台 |
| Panta `.pa` DSL | [`.pa` 规则](pa.md) | DSL 源文件、格式化与校验 |
| Cargo | [Cargo](cargo.md) | workspace 与调度 |
| CMake | [CMake](cmake.md) | native 构建图 |
| Ninja | [Ninja](ninja.md) | 构建执行 |
| Python 3.12+ | [Python](python.md) | 后续工具/自动化 |
| QML | [QML](qml.md) | 界面组件 |
| Qt 6 | [Qt](qt.md) | QObject、线程、部署 |
| OpenCASCADE | [OCCT](occt.md) | 几何适配器 |
| Netgen | [Netgen](netgen.md) | 网格适配器 |
| VTK | [VTK](vtk.md) | 渲染与原生视口 |
| CXX 首选候选 | [CXX](cxx.md) | Rust ↔ C++ 服务桥接 |
| FFI / pybind11 候选 | [跨语言边界](ffi.md) | Rust/C++ 和后续 Python |

## 文档职责

| 内容 | 维护位置 |
|---|---|
| 产品范围、模块边界和数据流 | [架构总览](../architecture/README.md) 及主题文档 |
| 重要模块的具体设计、取舍和演进门槛 | [模块说明](../modules/README.md) |
| 项目选型、版本与兼容性状态 | [技术基线](baseline.md) |
| 编码规则、库使用约束及官方依据 | 对应技术规范 |
| 单次工作的范围、决策过程与验证证据 | 对应 task |
| 任务依赖、顺序和状态摘要 | [任务索引](../task-index.md) |

任务中的探索结论在验证后同步到架构或规范；任务保留过程与证据，不成为长期规则的唯一存放处。应用当前可用能力只在根 README 与相关使用说明中承诺，不能从 done 的文档维护任务推断功能已实现。

## 使用与维护

执行 task 前阅读其列出的规范。若官方资料与已锁定版本不一致，以所用版本的实际支持为事实依据，在 task 记录原因并同步规范；不得悄悄改变架构边界。规则冲突先记录并解决，任务文档不能偷偷重定义整个项目的基线。

官方链接含 `latest`、`stable`、`nightly` 或无版本路径时会随时间变化。实施者必须补充实际版本、固定版本文档/源码 tag 和实测结果；本次查阅没有把网页展示版本直接确定为项目依赖。

外部求解器内的 PETSc、hypre、CUDA、FEM/FVM/DG 不属于本仓库规范集合；它们的边界见 [[External / 非本仓库] MoldSolver](../architecture/external-moldsolver.md)。协议编码、存储格式和 Python binding 后端尚未定型，选定后先补规范再实现。
