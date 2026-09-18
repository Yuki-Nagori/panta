# Python 自动化与包管理

查阅日期：2026-09-18。状态：uv + pyproject + uv.lock 已用于 CMake formatter；Python 业务运行时和 binding 仍未接入。

项目基线 Python 3.12+；属于后续自动化基础设施，不是 M0 桌面构建必需项。

## 官方依据

`venv` 提供隔离环境，虚拟环境通常应重建而非复制迁移。[Python 3.12 venv](https://docs.python.org/3.12/library/venv.html)

`pyproject.toml` 区分构建系统、项目元数据和工具配置；支持版本由 `requires-python` 表达。[PyPA pyproject 指南](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/)

## 项目规则

- Python 工具统一由 uv 管理，`pyproject.toml` 声明 `requires-python = ">=3.12,<3.15"`，`uv.lock` 提交并用于 CI；不把项目依赖安装到系统 Python，不提交 `.venv`。
- 当前只声明开发工具依赖 `cmakelang==0.6.13`，不引入运行时、binding 或业务包；增加 Python 工具时按用途拆分 dependency group 并更新锁文件。
- 代码使用四空格缩进、函数/模块 `snake_case`、类型 `PascalCase`，公共函数标注类型。CMake 格式通过 `uv run --locked cmake-format --check`，规则检查通过 `uv run --locked cmake-lint`；工具失败必须返回非零。Cargo 根入口负责调用 uv，当前没有 Python 业务源码，不假定已安装 Ruff、Black 或 pytest。
- 路径用 `pathlib`；外部进程用参数列表，明确 cwd、环境和退出码。禁止将用户输入拼成 shell 命令。
- CLI 入口显式定义；import 模块不能启动 GUI、修改工程或执行耗时计算。错误加上下文后向调用者报告，不能吞掉异常返回空成功结果。
- 未来绑定复用应用服务，默认不暴露 OCCT/VTK 内部指针；原生数组关联与生命周期参见 [FFI](ffi.md)。采用 pybind11 仍需独立任务，不因建立 Python 工具环境而提前引入嵌入式解释器。

## 验证

在新环境执行 `uv lock`/`uv run --locked` 重建依赖，验证工具版本、格式失败码和空格/非 ASCII 路径。Python API 与纯工具脚本的支持范围分别记录；没有 headless 验证就不能承诺无显示服务运行。
