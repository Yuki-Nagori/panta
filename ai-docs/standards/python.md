# Python 自动化与包管理

查阅日期：2026-09-16。状态：项目规范草案，尚未完成工具链集成验证。

项目基线 Python 3.12+；属于后续自动化基础设施，不是 M0 桌面构建必需项。

## 官方依据

`venv` 提供隔离环境，虚拟环境通常应重建而非复制迁移。[Python 3.12 venv](https://docs.python.org/3.12/library/venv.html)

`pyproject.toml` 区分构建系统、项目元数据和工具配置；支持版本由 `requires-python` 表达。[PyPA pyproject 指南](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/)

## 项目规则

- 任务 014 才创建 Python 工具环境，明确 3.12 最低线及实测解释器版本。不把项目依赖安装到系统 Python，不提交 `.venv`。
- 项目使用统一 `pyproject.toml`；包名和构建后端在实施时确定。区分开发工具依赖、脚本运行依赖和将来的二进制扩展依赖，记录可重建的锁定方案。
- 代码使用四空格缩进、函数/模块 `snake_case`、类型 `PascalCase`，公共函数标注类型。格式与检查工具须在 task 中选定并固定，当前不假定已安装 Ruff、Black 或 pytest。
- 路径用 `pathlib`；外部进程用参数列表，明确 cwd、环境和退出码。禁止将用户输入拼成 shell 命令。
- CLI 入口显式定义；import 模块不能启动 GUI、修改工程或执行耗时计算。错误加上下文后向调用者报告，不能吞掉异常返回空成功结果。
- 未来绑定复用应用服务，默认不暴露 OCCT/VTK 内部指针；原生数组关联与生命周期参见 [FFI](ffi.md)。采用 pybind11 仍需独立任务，不因建立 Python 工具环境而提前引入嵌入式解释器。

## 验证

在新环境重建依赖，验证 `python -m` 入口、模块导入无副作用和空格/非 ASCII 路径。Python API 与纯工具脚本的支持范围分别记录；没有 headless 验证就不能承诺无显示服务运行。
