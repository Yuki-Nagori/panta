# 109 — Rust 后台执行容量与生命周期收敛

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[108](108-project-write-coordination.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-08

## 目标与范围

为预检、只读激活及元数据写入建立共享执行基础，分离模拟任务执行体和调度生命周期，定义请求编号、排队 / 取消、容量与终态回收；保持各消费者的领域状态机和写入提交规则。明确服务销毁后写入收尾所有者；结合大文件测量确定容量，不盲目 join GUI 或加入通用异步运行时。

本任务对应 [106 架构评审](106-rust-architecture-review.md) 的后续建议；确认导入后台化仍由 063 实施。保持 UI 与正常工程 schema，不将新基础接口描述为已经实现。

## 必读

- [分层规则](../standards/layering.md)
- [Rust 规范](../standards/rust.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)
- [文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)

## 验收标准

- [x] 快速取消 / 替换、容量限制、终态清理、服务销毁与写入完成回执通过；记录容量测量依据。
- [x] 替换实现与失效引用已删除，无新增兼容分支。
- [x] Cargo 工作区聚合、构建、格式及完整 lint 通过；记录具体平台与未覆盖点。

## 验证计划与结果

macOS / Rust 1.98.1、仓库根目录：`cargo test --locked --workspace` 通过 Rust 工作区、qmllint 和 72/72 个 native / QML CTest；core 单元 68、工程集成 29、FFI 单元 27 通过。`cargo build --locked`、`cargo format --check`、`cargo lint --check` 通过。`cargo coverage` 门禁通过：全局函数 90.79%、行 95.02%；branch 未测。定向 Miri（nightly-2026-09-15、关闭隔离、target/miri）core 单元 68 全通过；nightly 对 fetch_update 的两处弃用提示不改变仓库 stable / MSRV 下的接口选择。

回归包括在途容量 / worker 上限、拒绝后账目回滚、启动失败与 panic 回执、关闭准入并等待已接受作业、排队预检替换、旧代次取消 / 结果释放、满载元数据拒绝不改变清单、服务销毁后提交成功 / 失败、独立子进程的应用退出写入收尾、任务终态消费回收及请求编号耗尽。退出协议由 Rust 实现，C++ 入口只安排在 QML 引擎销毁后调用。

`cargo performance cpu --samples 3` 全部通过：辅助正确性、Rust 后台容量 / 网格基准、native CPU 各场景。新基准已接入 CPU / all 聚合，源码在 tests/performance/cpu，显式注册为 panta-core example，不进入功能测试或 CI 时间门禁。未运行 GPU 基准或真实窗口；QML / UI 尺寸未修改，本批跨平台 CI 尚未执行。

### 容量测量

独立 Rust CPU 进程、Release；Mac16,12、10 逻辑 CPU、24 GiB 内存。通过公共预检 API 使用 5 万 / 50 万三角面的 binary STL（2,500,084 / 25,000,084 bytes），提交 1 / 4 / 8 / 64 个请求并在消费后释放快照。最终 `/usr/bin/time -l target/release/examples/background_execution` 成功；峰值 RSS 223,805,440 bytes（约 213.4 MiB），不含 Qt / VTK。

| 50 万三角面请求数 | 整批耗时 ms | 完成延迟 p95 ms |
|---|---:|---:|
| 1 | 17.652 | 17.624 |
| 4 | 87.514 | 87.513 |
| 8 | 27.745 | 27.744 |
| 64 | 222.904 | 215.321 |

这是单轮容量复核，包含调度 / OS 页缓存波动，不作为加速比例或 CI 时间门禁。初始读取策略取 2 个并行解析、64 个在途作业，已验证 64 请求突发与有界执行；写入独立 1 / 32，模拟任务 2 / 64，避免读取占满后阻塞提交收尾。数量上限不能限制单文件或全部待消费结果的字节大小；不是最优容量声明。激活在飞 / 待消费结果与模拟记录各最多 256，需按领域 drain 回收。

## 工作记录

- 2026-10-08：锁释放修复 dae9aaa 与流程规则 9bdf379 已推送；[CI 37789241094](https://github.com/Yuki-Nagori/panta/actions/runs/37789241094) 全部适用作业通过，包括 Linux / macOS / Windows Cargo 聚合、Rust / native 覆盖率、Miri、三平台 sanitizer 和适用质量检查。原 Linux 失败作业本轮成功，CI 收尾完成；未把路径门控跳过项计为实际运行通过。独立 subagent 复核验收记录与完成摘要通过。

- 2026-10-08：按用户要求，将小范围 CI 修复的 commit / push / CI 等待流程登记在 AGENTS.md；本次锁释放修复将随流程文档一并推送，跟进对应提交的运行，失败继续修复。独立 subagent 复核本次规则与用户授权一致；文档差异检查通过。

- 2026-10-08：独立 subagent review 检查锁释放、失败 / 提交语义、复制句柄断言、fs4 Unix / Windows 源码和文档一致性，未发现代码行为阻断项；按评审修正原 CI 根因的证据范围及提交点注释。评审为只读检查，未将本机验证或源码检查写成 Linux / Windows 新提交通过。后续小 CI 修复归所属 task，任务完成后独立 review 的规则登记在 AGENTS.md。

- 2026-10-08：CI 收尾：Linux 工程回归报 project write pending。新增句柄复制回归在修复前稳定返回 WouldBlock；依据 [flock(2)](https://man7.org/linux/man-pages/man2/flock.2.html) 与 fs4 源码，租约销毁改为显式 unlock 后关闭句柄，保留争用拒绝及 rename 提交点，解锁诊断不改变已提交结果。与该报错一致的释放机制已确定复现，但不能据此证明原 CI 发生了 fork 继承。macOS Cargo 工作区聚合（69 core 单元、72/72 native / QML）、构建、完整 lint、格式及新回归的 Miri 检查通过；修复提交的 Linux CI 尚待复验。按用户要求撤掉未提交的独立 CI task，更新本任务，新增独立 subagent review 环节。

- 2026-10-08：[CI 37751188859](https://github.com/Yuki-Nagori/panta/actions/runs/37751188859) 的 Linux 工程集成回归报 project write pending，其余检查成功；在本任务 CI 收尾中复现描述符共享延长锁寿命的机制并修正租约释放；原 CI 的具体触发条件尚未直接捕获，不将本次失败写为跨平台验收通过。

- 2026-10-08：登记剩余 Rust 架构任务，按 108 完成后的顺序实施。
- 2026-10-08：预检、激活、元数据提交及模拟任务接入共享有界执行。删除逐任务句柄积累，分离模拟执行体，任务 drain 回收终态；激活限制在飞 / 未消费结果，取消旧代次读取并释放结果。
- 2026-10-08：补齐应用正常退出收尾；QML 服务销毁后由 Rust 关闭后台写入准入并等已接受提交，宿主关闭后的提交错误仍写 stderr。补充容量、失败、关闭、编号与 FFI 回归及手动 CPU 负载。
- 2026-10-08：完整 review 后清理空错误回调、统一执行账目锁入口和共享测试夹具挂载，删除旧启动路径及失效 join 描述。修复函数 coverage 门禁缺口，最终 Cargo 聚合、质量检查、CPU 套件和 Miri 通过，状态 / 索引同步 done。

## 完成摘要

109 本地与 CI 验收完成：共享有界执行与各领域状态机分离，容量拒绝 / 取消 / 终态回收有回归，已接受写入由后台持有并在正常退出时收尾。CXX 仍只转发，C++ 只安排宿主退出顺序，QML 外观保持；确认导入后台化继续由 063 跟踪。
