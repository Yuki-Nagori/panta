/// VTK WebGPU 手动基准：默认硬件离屏，原生窗口提交间隔保留独立对照。
#include "../../support/qt/frame_submission_capture.hpp"
#include "../support/benchmark_statistics.hpp"
#include "../support/offscreen_vtk_renderer.hpp"
#include <QColor>
#include <QGuiApplication>
#include <QObject>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QSizeF>
#include <QString>
#include <QtCore/qtenvironmentvariables.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <memory>
#include <panta/visualization/render_scene.hpp>
#include <qlogging.h>
#include <qtestsupport_core.h>
#include <qtestsupport_gui.h>
#include <ratio>
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
    std::unique_ptr<panta::test::OffscreenVtkRenderer> offscreen_;
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
        std::vector<double> intervals;
        intervals.reserve(frame_count);
        for (int frame = 0; frame < frame_count; ++frame) {
            const auto start = std::chrono::steady_clock::now();
            if (!submit_scene_update(frame_index++)) {
                return {};
            }
            intervals.push_back(
                std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start)
                    .count());
        }
        return intervals;
    }

  private slots:
    void initTestCase() {
        const QString platform = QGuiApplication::platformName();
        if (platform == QStringLiteral("offscreen") || platform == QStringLiteral("minimal")) {
            QFAIL("VTK WebGPU benchmark requires a native graphics session (surface hidden by "
                  "default)");
        }

        if (qgetenv("PANTA_BENCHMARK_PRESENTATION") != "1") {
            const double ratio = window_.devicePixelRatio();
            try {
                offscreen_ = std::make_unique<panta::test::OffscreenVtkRenderer>(
                    QSize(static_cast<int>(1000 * ratio), static_cast<int>(700 * ratio)));
            } catch (const std::exception& error) {
                QFAIL(error.what());
            }
            return;
        }

        viewport_parent_ = new QQuickItem(window_.contentItem());
        viewport_ = new panta::visualization::VtkViewport(viewport_parent_);
        viewport_parent_->setSize(QSizeF(1000, 700));
        viewport_->setSize(QSizeF(1000, 700));
        QSignalSpy initialized(viewport_, &panta::visualization::VtkViewport::sceneInitialized);
        window_.resize(1000, 700);
        window_.show();
        if (!QTest::qWaitForWindowExposed(&window_)) {
            QFAIL("Native benchmark window could not be exposed");
        }
        if (initialized.isEmpty() && !initialized.wait(10000)) {
            QFAIL("VTK WebGPU did not initialize a native rendering context");
        }

        capture_ = std::make_unique<panta::test::FrameSubmissionCapture>();
    }

    void measures_webgpu_frames() {
        if (offscreen_) {
            std::vector<double> submissions;
            std::vector<double> completions;
            try {
                for (int index = 0; index < kWarmupFrameCount + kSampleCount * kFrameCountPerSample;
                     ++index) {
                    panta::visualization::RenderScene scene;
                    scene.background =
                        QColor((index * 37) % 256, (index * 71) % 256, (index * 113) % 256);
                    const auto timing = offscreen_->render(scene);
                    if (index >= kWarmupFrameCount) {
                        submissions.push_back(timing.submitMs);
                        completions.push_back(timing.completionMs);
                    }
                }
            } catch (const std::exception& error) {
                QFAIL(error.what());
            }
            const auto submit = panta::test::summarize_timings(submissions);
            const auto complete = panta::test::summarize_timings(completions);
            if (!submit || !complete) {
                QFAIL("Invalid VTK offscreen timing samples");
            }
            qInfo().nospace() << "VTK offscreen (" << offscreen_->backend() << ", "
                              << submissions.size()
                              << " frames; CPU update to submit p50/p95 ms): " << submit->p50 << "/"
                              << submit->p95;
            qInfo().nospace() << "VTK offscreen (CPU update to queue completion p50/p95 ms): "
                              << complete->p50 << "/" << complete->p95;
            return;
        }

        qInfo() << "VTK WebGPU native-window benchmark; interval is measured from scene update "
                   "to the following VTK frame-submission log, not GPU execution or display "
                   "presentation time";
        int frame_index = 0;
        if (measure_submission_intervals(kWarmupFrameCount, frame_index).size() !=
            kWarmupFrameCount) {
            QFAIL("Incomplete VTK warmup sample batch");
        }

        std::vector<double> intervals;
        intervals.reserve(static_cast<std::size_t>(kFrameCountPerSample) * kSampleCount);
        for (int sample = 0; sample < kSampleCount; ++sample) {
            const auto values = measure_submission_intervals(kFrameCountPerSample, frame_index);
            if (values.size() != kFrameCountPerSample) {
                QFAIL("Incomplete VTK submission sample batch");
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
        offscreen_.reset();
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
