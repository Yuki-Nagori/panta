// TaskHost 的 Qt 集成验证（任务 008）：事件转信号、取消、无效输入与
// GUI 线程在慢任务期间保持响应。QSignalSpy 属 QtTest；自定义 main 提供
// QCoreApplication。

#include <gtest/gtest.h>
#include <QCoreApplication>
#include <QEventLoop>
#include <QSignalSpy>
#include <QTimer>
#include <QtTest>

#include "task_host.hpp"

namespace {

using panta::bridge::TaskHost;

TEST(TaskHost, SubmitSucceedsEmitsSignalsAndRunningProperty) {
    TaskHost host;
    QSignalSpy startedSpy(&host, &TaskHost::taskStarted);
    QSignalSpy succeededSpy(&host, &TaskHost::taskSucceeded);
    QSignalSpy runningSpy(&host, &TaskHost::runningTasksChanged);

    const qint64 id = host.submitTask(QStringLiteral("host-α"), 30, false);
    ASSERT_GE(id, 0);

    ASSERT_TRUE(succeededSpy.wait(2000));
    ASSERT_EQ(startedSpy.count(), 1);
    EXPECT_EQ(startedSpy.at(0).at(0).toULongLong(), static_cast<qulonglong>(id));
    EXPECT_EQ(succeededSpy.at(0).at(0).toULongLong(), static_cast<qulonglong>(id));

    // 运行数 1 → 0 各通知一次；终态后稳定为 0。
    QTRY_COMPARE(runningSpy.count(), 2);
    QTRY_COMPARE(host.runningTasks(), quint32{0});
}

TEST(TaskHost, CancelRunningTaskAndRejectLateCancel) {
    TaskHost host;
    QSignalSpy cancelledSpy(&host, &TaskHost::taskCancelled);

    const qint64 id = host.submitTask(QStringLiteral("host-cancel"), 30'000, false);
    ASSERT_GE(id, 0);
    EXPECT_TRUE(host.cancelTask(static_cast<quint64>(id)));
    // 重复请求被拒。
    EXPECT_FALSE(host.cancelTask(static_cast<quint64>(id)));

    ASSERT_TRUE(cancelledSpy.wait(5000));
    EXPECT_EQ(cancelledSpy.at(0).at(0).toULongLong(), static_cast<qulonglong>(id));
    EXPECT_EQ(cancelledSpy.at(0).at(1).toString(), QStringLiteral("task.cancelled"));
    QTRY_COMPARE(host.runningTasks(), quint32{0});
    // 终态后的迟到取消被拒。
    EXPECT_FALSE(host.cancelTask(static_cast<quint64>(id)));
}

TEST(TaskHost, InvalidSubmitRecordsStructuredError) {
    TaskHost host;
    QSignalSpy errorSpy(&host, &TaskHost::lastErrorChanged);

    EXPECT_EQ(host.submitTask(QString(), 1, false), -1);
    EXPECT_EQ(host.lastError(), QStringLiteral("task.empty_label"));
    EXPECT_EQ(host.submitTask(QStringLiteral("x"), 60'001, false), -1);
    EXPECT_TRUE(host.lastError().startsWith(QStringLiteral("task.invalid_duration")));
    EXPECT_GE(errorSpy.count(), 1);
    QTRY_COMPARE(host.runningTasks(), quint32{0});
}

TEST(TaskHost, ProgressSignalsAreBoundedMonotonic) {
    TaskHost host;
    QSignalSpy progressSpy(&host, &TaskHost::taskProgress);
    QSignalSpy succeededSpy(&host, &TaskHost::taskSucceeded);

    const qint64 id = host.submitTask(QStringLiteral("host-progress"), 300, false);
    ASSERT_GE(id, 0);
    ASSERT_TRUE(succeededSpy.wait(3000));

    // 300ms 任务按 10% 步进：若干次进度、总量有界且单调递增。
    ASSERT_GE(progressSpy.count(), 1);
    EXPECT_LE(progressSpy.count(), 10);
    quint32 previous = 0;
    for (const auto& call : progressSpy) {
        EXPECT_EQ(call.at(0).toULongLong(), static_cast<qulonglong>(id));
        const quint32 percent = call.at(1).toUInt();
        EXPECT_GT(percent, previous);
        EXPECT_LE(percent, 100U);
        previous = percent;
    }
}

TEST(TaskHost, GuiThreadStaysResponsiveDuringSlowTask) {
    TaskHost host;
    // 500ms 慢任务；GUI 线程事件循环期间的 10ms 心跳若被阻塞将接近 0。
    const qint64 id = host.submitTask(QStringLiteral("host-slow"), 500, false);
    ASSERT_GE(id, 0);

    int heartbeats = 0;
    QTimer ticker;
    ticker.setInterval(10);
    QObject::connect(&ticker, &QTimer::timeout, [&heartbeats] { ++heartbeats; });
    ticker.start();

    QEventLoop loop;
    QObject::connect(&host, &TaskHost::taskSucceeded, &loop, &QEventLoop::quit);
    QTimer::singleShot(5000, &loop, &QEventLoop::quit);
    loop.exec();

    EXPECT_GE(heartbeats, 20);
    QTRY_COMPARE(host.runningTasks(), quint32{0});
    Q_UNUSED(id);
}

}  // namespace

int main(int argc, char** argv) {
    QCoreApplication app(argc, argv);
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
