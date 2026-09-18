#include "panta_ffi.h"
#include "rust/cxx.h"
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <optional>
#include <string>
#include <thread>

namespace {

using panta::ffi::task_service_cancel;
using panta::ffi::task_service_drain;
using panta::ffi::task_service_new;
using panta::ffi::task_service_recent_logs;
using panta::ffi::task_service_running;
using panta::ffi::task_service_submit;
using panta::ffi::TaskEvent;
using panta::ffi::TaskEventKind;
using panta::ffi::TaskService;

using Clock = std::chrono::steady_clock;

bool wait_running_zero(TaskService& service, std::chrono::milliseconds timeout) {
    const auto deadline = Clock::now() + timeout;
    while (Clock::now() < deadline) {
        if (task_service_running(service) == 0U) {
            return true;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(5));
    }
    return task_service_running(service) == 0U;
}

std::optional<TaskEvent> find_event(const rust::Vec<TaskEvent>& events, TaskEventKind kind) {
    std::optional<TaskEvent> found;
    for (const auto& event : events) {
        if (event.kind == kind) {
            found = event;
        }
    }
    return found;
}

TEST(FfiTaskService, SubmitSucceedsAndEmitsOrderedEvents) {
    auto service = task_service_new();
    const std::uint64_t id = task_service_submit(*service, "导入-α", 30, false);

    ASSERT_TRUE(wait_running_zero(*service, std::chrono::seconds(2)));
    const auto events = task_service_drain(*service);
    std::size_t started_index = events.size();
    std::size_t succeeded_index = events.size();
    for (std::size_t index = 0; index < events.size(); ++index) {
        if (events[index].task_id != id) {
            continue;
        }
        if (events[index].kind == TaskEventKind::Started && started_index == events.size()) {
            started_index = index;
        }
        if (events[index].kind == TaskEventKind::Succeeded && succeeded_index == events.size()) {
            succeeded_index = index;
        }
    }
    ASSERT_LT(started_index, succeeded_index);
    EXPECT_TRUE(find_event(events, TaskEventKind::Failed).has_value() == false);
    // 终态后重复拉取不产生新事件。
    EXPECT_TRUE(task_service_drain(*service).empty());

    // 结构化日志凭 task_id 关联任务与原因。
    bool has_submitted_log = false;
    bool has_succeeded_log = false;
    for (const auto& line : task_service_recent_logs(*service)) {
        if (line.task_id != id) {
            continue;
        }
        const std::string message(line.message);
        has_submitted_log = has_submitted_log || message.find("submitted") != std::string::npos;
        has_succeeded_log = has_succeeded_log || message.find("succeeded") != std::string::npos;
    }
    EXPECT_TRUE(has_submitted_log);
    EXPECT_TRUE(has_succeeded_log);
}

TEST(FfiTaskService, SimulatedFailureCarriesStructuredCode) {
    auto service = task_service_new();
    const std::uint64_t id = task_service_submit(*service, "fail-β", 20, true);

    ASSERT_TRUE(wait_running_zero(*service, std::chrono::seconds(2)));
    const auto events = task_service_drain(*service);
    const auto failed = find_event(events, TaskEventKind::Failed);
    if (!failed.has_value()) {
        ADD_FAILURE() << "未收到失败事件";
        return;
    }
    EXPECT_EQ(failed->task_id, id);
    EXPECT_EQ(failed->code, "task.simulated_failure");
    const std::string detail(failed->detail);
    EXPECT_NE(detail.find("fail-β"), std::string::npos);
}

TEST(FfiTaskService, CancelRunningTaskAndRejectLateCancel) {
    auto service = task_service_new();
    const std::uint64_t id = task_service_submit(*service, "cancel-γ", 30'000, false);

    EXPECT_TRUE(task_service_cancel(*service, id));
    // 取消请求只接受一次。
    EXPECT_FALSE(task_service_cancel(*service, id));

    std::optional<TaskEvent> cancelled;
    const auto deadline = Clock::now() + std::chrono::seconds(5);
    while (Clock::now() < deadline) {
        for (const auto& event : task_service_drain(*service)) {
            if (event.kind == TaskEventKind::Cancelled) {
                cancelled = event;
            }
        }
        if (cancelled.has_value()) {
            break;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(5));
    }
    if (!cancelled.has_value()) {
        ADD_FAILURE() << "未收到取消事件";
        return;
    }
    EXPECT_EQ(cancelled->code, "task.cancelled");
    EXPECT_EQ(task_service_running(*service), 0U);

    // 终态之后：迟到取消被拒绝且不产生新事件。
    EXPECT_FALSE(task_service_cancel(*service, id));
    EXPECT_TRUE(task_service_drain(*service).empty());
}

TEST(FfiTaskService, InvalidSubmitYieldsStructuredError) {
    auto service = task_service_new();
    EXPECT_THROW(static_cast<void>(task_service_submit(*service, "", 1, false)), rust::Error);
    EXPECT_THROW(static_cast<void>(task_service_submit(*service, "x", 60'001, false)), rust::Error);
    EXPECT_EQ(task_service_running(*service), 0U);
}

TEST(FfiTaskService, DestructionJoinsRunningWorkers) {
    const auto started = Clock::now();
    {
        auto service = task_service_new();
        static_cast<void>(task_service_submit(*service, "shutdown-δ", 30'000, false));
        static_cast<void>(task_service_submit(*service, "shutdown-ε", 30'000, false));
    }
    const auto elapsed =
        std::chrono::duration_cast<std::chrono::milliseconds>(Clock::now() - started);
    // 析构应在工作线程检查点快速返回，而非等待任务自然结束。
    EXPECT_LT(elapsed.count(), 30'000);
}

} // namespace
