/// VTK WebGPU 开发基准：在真实原生窗口中测量场景更新到帧提交的间隔。

#include "../../support/qt/frame_submission_capture.hpp"
#include "../support/benchmark_statistics.hpp"
#include <QColor>
#include <QGuiApplication>
#include <QObject>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QSizeF>
#include <QString>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <algorithm>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <memory>
#include <panta/visualization/render_scene.hpp>
#include <qlogging.h>
#include <qtestsupport_core.h>
#include <qtestsupport_gui.h>
#include <vector>
#include <vtk_viewport.hpp>

namespace {

constexpr int kWarmupFrameCount = 30;
constexpr int kFrameCountPerSample = 60;
constexpr int kSampleCount = 3;
constexpr int kFrameWaitTimeoutMs = 10000;

} // namespace

class ViewportGpuBenchmark final : public QObject {
    Q_OBJECT

  private:
    QQuickWindow window_;
    std::unique_ptr<panta::test::FrameSubmissionCapture> capture_;
    // 指针而非直接成员：VTK/WebGPU 资源必须在事件循环仍活跃的
    // cleanupTestCase 显式拆除（与应用侧 QML 引擎拆除同序）；作为成员会在
    // qExec 之后的静态析构期释放，与 Win32 interactor 遗留状态冲突触发
    // 0xC0000005（任务 007 A/B 实测）。
    QQuickItem* viewport_parent_ = nullptr;
    panta::visualization::VtkViewport* viewport_ = nullptr;

    bool submit_scene_update(int frame_index) {
        const auto previous_count = capture_->count();
        panta::visualization::RenderScene scene;
        scene.revision = static_cast<std::uint64_t>(frame_index) + std::uint64_t{1};
        scene.background =
            QColor((frame_index * 37) % 256, (frame_index * 71) % 256, (frame_index * 113) % 256);
        viewport_->apply_state(scene);
        if (!capture_->wait_after(previous_count, std::chrono::milliseconds{kFrameWaitTimeoutMs})) {
            QTest::qFail("Timed out waiting for a VTK WebGPU frame submission", __FILE__, __LINE__);
            return false;
        }
        return true;
    }

    std::vector<double> measure_submission_intervals(int frame_count, int& frame_index) {
        const std::size_t first_frame = capture_->count();
        for (int frame = 0; frame < frame_count; ++frame) {
            if (!submit_scene_update(frame_index++)) {
                return {};
            }
        }
        const auto submitted_frame_times = capture_->times();
        std::vector<double> intervals;
        intervals.reserve(frame_count);
        for (std::size_t index = std::max<std::size_t>(first_frame, 1);
             index < submitted_frame_times.size(); ++index) {
            intervals.push_back(static_cast<double>(submitted_frame_times[index] -
                                                    submitted_frame_times[index - 1]) /
                                1.0e6);
        }
        return intervals;
    }

  private slots:
    void initTestCase() {
        const QString platform = QGuiApplication::platformName();
        if (platform == QStringLiteral("offscreen") || platform == QStringLiteral("minimal")) {
            QSKIP("VTK WebGPU benchmark requires a visible native graphics session");
        }

        viewport_parent_ = new QQuickItem(window_.contentItem());
        viewport_ = new panta::visualization::VtkViewport(viewport_parent_);
        viewport_parent_->setSize(QSizeF(1000, 700));
        viewport_->setSize(QSizeF(1000, 700));
        QSignalSpy initialized(viewport_, &panta::visualization::VtkViewport::sceneInitialized);
        window_.resize(1000, 700);
        window_.show();
        if (!QTest::qWaitForWindowExposed(&window_)) {
            QSKIP("Native benchmark window could not be exposed");
        }
        if (initialized.isEmpty() && !initialized.wait(10000)) {
            QSKIP("VTK WebGPU did not initialize a native rendering context");
        }

        capture_ = std::make_unique<panta::test::FrameSubmissionCapture>();
    }

    void measures_webgpu_frame_submission_interval() {
        qInfo() << "VTK WebGPU native-window benchmark; interval is measured from scene update "
                   "to the following VTK frame-submission log, not GPU execution or display "
                   "presentation time";
        int frame_index = 0;
        if (measure_submission_intervals(kWarmupFrameCount, frame_index).empty()) {
            return;
        }

        std::vector<double> intervals;
        intervals.reserve(static_cast<std::size_t>(kFrameCountPerSample) * kSampleCount);
        for (int sample = 0; sample < kSampleCount; ++sample) {
            const auto values = measure_submission_intervals(kFrameCountPerSample, frame_index);
            if (values.size() != kFrameCountPerSample) {
                return;
            }
            intervals.insert(intervals.end(), values.begin(), values.end());
        }

        const auto summary = panta::test::summarize_timings(intervals);
        if (!summary) {
            QTest::qFail("Invalid frame timing samples", __FILE__, __LINE__);
            return;
        }
        qInfo().nospace() << "VTK WebGPU frame submission (" << QGuiApplication::platformName()
                          << ", " << kSampleCount << " x " << kFrameCountPerSample
                          << " frames; p50/p95 ms): " << summary->p50 << "/" << summary->p95;
    }

    void cleanupTestCase() {
        capture_.reset();
        // 显式按应用侧同序拆除 VTK/WebGPU 资源：事件循环仍活跃时先行释放，
        // 避免拖到 qExec 之后的静态析构期与 Win32 interactor 遗留状态冲突。
        delete viewport_;
        viewport_ = nullptr;
        delete viewport_parent_;
        viewport_parent_ = nullptr;
        QTest::qWait(50);
    }
};

QTEST_MAIN(ViewportGpuBenchmark)
#include "viewport_benchmark.moc"
