#include "task_host.hpp"

namespace panta::bridge {

namespace {
constexpr int kPollIntervalMs = 10;

using panta::ffi::task_service_cancel;
using panta::ffi::task_service_drain;
using panta::ffi::task_service_new;
using panta::ffi::task_service_running;
using panta::ffi::task_service_submit;
} // namespace

TaskHost::TaskHost(QObject* parent) : QObject(parent), m_service(task_service_new()) {
    m_pollTimer.setInterval(kPollIntervalMs);
    connect(&m_pollTimer, &QTimer::timeout, this, &TaskHost::poll);
    m_pollTimer.start();
}

TaskHost::~TaskHost() = default;

quint32 TaskHost::runningTasks() const { return m_runningTasks; }

const QString& TaskHost::lastError() const { return m_lastError; }

qint64 TaskHost::submitTask(const QString& label, qint64 durationMs, bool fail) {
    try {
        const auto id = task_service_submit(*m_service, label.toStdString(),
                                            static_cast<std::uint64_t>(durationMs), fail);
        return static_cast<qint64>(id);
    } catch (const rust::Error& error) {
        const QString code = QString::fromUtf8(error.what());
        if (m_lastError != code) {
            m_lastError = code;
            emit lastErrorChanged();
        }
        return -1;
    }
}

bool TaskHost::cancelTask(quint64 taskId) { return task_service_cancel(*m_service, taskId); }

void TaskHost::poll() {
    for (const auto& event : task_service_drain(*m_service)) {
        switch (event.kind) {
        case panta::ffi::TaskEventKind::Started:
            emit taskStarted(event.task_id);
            break;
        case panta::ffi::TaskEventKind::Progress:
            emit taskProgress(event.task_id, event.percent);
            break;
        case panta::ffi::TaskEventKind::Succeeded:
            emit taskSucceeded(event.task_id);
            break;
        case panta::ffi::TaskEventKind::Failed:
            emit taskFailed(event.task_id, QString::fromUtf8(event.code.data(), event.code.size()),
                            QString::fromUtf8(event.detail.data(), event.detail.size()));
            break;
        case panta::ffi::TaskEventKind::Cancelled:
            emit taskCancelled(event.task_id,
                               QString::fromUtf8(event.code.data(), event.code.size()));
            break;
        }
    }
    const auto running = task_service_running(*m_service);
    if (running != m_runningTasks) {
        m_runningTasks = running;
        emit runningTasksChanged();
    }
}

} // namespace panta::bridge
