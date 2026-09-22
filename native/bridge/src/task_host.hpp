/// 后台任务的 Qt 集成宿主（任务 008）：持有 Rust TaskService，把拉取式
/// 事件转为 Qt 信号，供 ViewModel/QML 绑定。
///
/// 契约（ui-and-bridge.md / application-and-storage.md）：
/// - 对象归属 GUI 线程；Rust 工作线程不触碰任何 Qt 对象，事件只经
///   QTimer 轮询在 GUI 线程取回并发射；
/// - 成功提交后以 10ms 轮询；所有已提交任务的终态均被转发后停表。
///   不以 runningTasks == 0 停表，避免漏掉刚入队但尚未拉取的终态；
/// - lastError 保存最近一次提交失败的结构化错误码，用户可读摘要的
///   本地化由 022 的语言字典承接。
#pragma once

#include "panta_ffi.h"
#include <QObject>
#include <QSet>
#include <QTimer>
#include <QtQml/qqmlregistration.h>

namespace panta::bridge {

class TaskHost : public QObject {
    Q_OBJECT
    QML_ELEMENT
    /// 当前运行中的任务数；仅在数值变化时通知。
    Q_PROPERTY(quint32 runningTasks READ runningTasks NOTIFY runningTasksChanged)
    /// 最近一次 submitTask 失败的结构化错误码；成功提交不清除。
    Q_PROPERTY(QString lastError READ lastError NOTIFY lastErrorChanged)

  public:
    explicit TaskHost(QObject* parent = nullptr);
    ~TaskHost() override;

    [[nodiscard]] quint32 runningTasks() const;
    [[nodiscard]] const QString& lastError() const;

    /// 提交模拟慢任务，返回任务 ID；参数无效返回 -1 并记录 lastError。
    Q_INVOKABLE qint64 submitTask(const QString& label, qint64 durationMs, bool fail);
    /// 请求取消；终态或未知任务返回 false。
    Q_INVOKABLE bool cancelTask(quint64 taskId);

  signals:
    void runningTasksChanged();
    void lastErrorChanged();
    void taskStarted(quint64 taskId);
    /// percent 为 0-100 的阶段内进度（10% 步进发布）。
    void taskProgress(quint64 taskId, quint32 percent);
    void taskSucceeded(quint64 taskId);
    /// code 为机器可读错误码（如 task.simulated_failure）；detail 为诊断
    /// 上下文，面向日志/Console，不直接展示给用户。
    void taskFailed(quint64 taskId, const QString& code, const QString& detail);
    /// code 区分 task.cancelled（用户取消）与 task.shutdown（宿主关闭）。
    void taskCancelled(quint64 taskId, const QString& code);

  private:
    void poll();

    rust::Box<panta::ffi::TaskService> m_service;
    QTimer m_pollTimer;
    QSet<quint64> m_pendingTasks;
    quint32 m_runningTasks = 0;
    QString m_lastError;
};

} // namespace panta::bridge
