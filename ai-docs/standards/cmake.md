# CMake target 与依赖规范

查阅日期：2026-09-16。状态：项目规范草案，尚未完成工具链集成验证。

适用于 `native/` 和其包含的 QML 模块构建。CMake/Ninja 的精确版本由依赖基线任务确定，在线文档的 latest 不等于项目锁定版本。

## 官方依据

CMake 用 targets 和 usage requirements 表达构建关系，`PRIVATE`、`PUBLIC`、`INTERFACE` 控制依赖属性传播。[Buildsystem](https://cmake.org/cmake/help/latest/manual/cmake-buildsystem.7.html)

`CMakePresets.json` 可共享，`CMakeUserPresets.json` 用于本机配置；preset schema 版本与 CMake 版本相关。[Presets](https://cmake.org/cmake/help/latest/manual/cmake-presets.7.html)

## 项目规则

- 使用源码树外构建；标准、include、defines、链接和告警都优先绑定 target。禁止为了一个 adapter 设置整个工程的 include/link 目录。
- 自有目标设 `CXX_STANDARD 20`、`CXX_STANDARD_REQUIRED ON`、`CXX_EXTENSIONS OFF` 或等价约束；PUBLIC 头需要 C++20 时向消费者传播要求。
- 依赖优先使用已发现的 imported targets；若上游包没有合适导出，在本地 adapter 内封装，不让平台库名散布在多个业务 CMake 文件。
- 明确列出源码、QML 和资源。生成代码使用带输入依赖与产物声明的构建规则，避免只在 configure 时执行且遗漏增量追踪。
- 共享 presets 不包含个人路径；本机前缀通过 user presets 或文档化参数传入。Cargo 调度与直接诊断构建使用同一套有效配置，避免两套默认值漂移。
- `cmake_minimum_required` 依据实际用到的 API 和验证结果确定。不要因为查到最新文档就直接要求最新 CMake。
- install 规则定义运行产物和布局；测试通过 CTest 注册。native 层不得无条件调用发起构建的同一个 Cargo target。

## 验证

检查从干净目录 configure/build/install、构建配置切换和找不到依赖时的诊断；查看 usage requirements 是否将 adapter 的私有库泄露到领域 core。具体 preset 名在任务 003 确定后再提供可复制命令。
