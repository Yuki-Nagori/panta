/// VTK WebGPU 开发基准：在真实原生窗口中测量场景更新到帧提交的间隔。

#include <QColor>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QGuiApplication>
#include <QLoggingCategory>
#include <QObject>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QSizeF>
#include <QString>
#include <QTimer>
#include <QtCore/qlogging.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <panta/visualization/render_scene.hpp>
#include <qtestsupport_gui.h>
#include <utility>
#include <vector>
#include <vtk_viewport.hpp>

namespace {

constexpr int kWarmupFrameCount = 30;
constexpr int kFrameCountPerSample = 60;
constexpr int kSampleCount = 3;
constexpr int kFrameWaitTimeoutMs = 10000;

QElapsedTimer frame_clock;
std::vector<qint64> submitted_frame_times;
QtMessageHandler previous_handler = nullptr;
QEventLoop* frame_wait_loop = nullptr;

void capture_frame_submissions(QtMsgType type, const QMessageLogContext& context,
                               const QString& message) {
    const bool viewport_debug =
        context.category != nullptr && std::strcmp(context.category, "panta.viewport") == 0;
    if (viewport_debug && message.startsWith(QStringLiteral("frame submitted"))) {
        if (frame_clock.isValid()) {
            submitted_frame_times.push_back(frame_clock.nsecsElapsed());
        }
        if (frame_wait_loop != nullptr) {
            frame_wait_loop->quit();
        }
        return;
    }
    if (viewport_debug && message == QStringLiteral("surface synchronized")) {
        return;
    }
    if (previous_handler != nullptr) {
        previous_handler(type, context, message);
    } else {
        std::fprintf(stderr, "%s\n", qPrintable(message));
    }
}

std::vector<double> summarize_percentiles(std::vector<double> values) {
    std::sort(values.begin(), values.end());
    const auto p95_index =
        static_cast<std::size_t>(std::ceil(static_cast<double>(values.size()) * 0.95)) - 1;
    return {values[values.size() / 2], values[p95_index]};
}

} // namespace

class ViewportGpuBenchmark final : public QObject {
    Q_OBJECT

  private:
    QQuickWindow window_;
    QQuickItem viewport_parent_{window_.contentItem()};
    panta::visualization::VtkViewport viewport_{&viewport_parent_};

    bool submit_scene_update(int frame_index) {
        const int previous_count = static_cast<int>(submitted_frame_times.size());
        panta::visualization::RenderScene scene;
        scene.revision = static_cast<std::uint64_t>(frame_index) + std::uint64_t{1};
        scene.background =
            QColor((frame_index * 37) % 256, (frame_index * 71) % 256, (frame_index * 113) % 256);
        QEventLoop wait_loop;
        frame_wait_loop = &wait_loop;
        QTimer::singleShot(kFrameWaitTimeoutMs, &wait_loop, &QEventLoop::quit);
        viewport_.apply_state(scene);
        wait_loop.exec();
        frame_wait_loop = nullptr;
        if (static_cast<int>(submitted_frame_times.size()) == previous_count) {
            QTest::qFail("Timed out waiting for a VTK WebGPU frame submission", __FILE__, __LINE__);
            return false;
        }
        return true;
    }

    std::vector<double> measure_submission_intervals(int frame_count, int& frame_index) {
        const std::size_t first_frame = submitted_frame_times.size();
        for (int frame = 0; frame < frame_count; ++frame) {
            if (!submit_scene_update(frame_index++)) {
                return {};
            }
        }
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

        viewport_parent_.setSize(QSizeF(1000, 700));
        viewport_.setSize(QSizeF(1000, 700));
        QSignalSpy initialized(&viewport_, &panta::visualization::VtkViewport::sceneInitialized);
        window_.resize(1000, 700);
        window_.show();
        if (!QTest::qWaitForWindowExposed(&window_)) {
            QSKIP("Native benchmark window could not be exposed");
        }
        if (initialized.isEmpty() && !initialized.wait(10000)) {
            QSKIP("VTK WebGPU did not initialize a native rendering context");
        }

        QLoggingCategory::setFilterRules(QStringLiteral("panta.viewport.debug=true"));
        previous_handler = qInstallMessageHandler(capture_frame_submissions);
        submitted_frame_times.clear();
        frame_clock.start();
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

        const auto summary = summarize_percentiles(std::move(intervals));
        qInfo().nospace() << "VTK WebGPU frame submission (" << QGuiApplication::platformName()
                          << ", " << kSampleCount << " x " << kFrameCountPerSample
                          << " frames; p50/p95 ms): " << summary[0] << "/" << summary[1];
    }

    void cleanupTestCase() {
        if (previous_handler != nullptr) {
            qInstallMessageHandler(previous_handler);
            previous_handler = nullptr;
        }
        QLoggingCategory::setFilterRules(QString());
    }
};

QTEST_MAIN(ViewportGpuBenchmark)
#include "viewport_gpu_benchmark.moc"
