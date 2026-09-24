// 开发侧 VTK 导航 CPU 消融基准；不创建图形窗口或提交 GPU 帧。
#include "navigation/viewport_camera.hpp"
#include "navigation/viewport_orientation.hpp"
#include <QElapsedTimer>
#include <QtCore/qlogging.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <algorithm>
#include <array>
#include <cmath>
#include <vector>
#include <vtkActor.h>
#include <vtkCamera.h>
#include <vtkCubeSource.h>
#include <vtkNew.h>
#include <vtkPolyDataMapper.h>
#include <vtkRenderWindow.h>
#include <vtkRenderer.h>
#include <vtkSmartPointer.h>

namespace {

constexpr int kBenchmarkSampleCount = 31;
constexpr int kBenchmarkOperationsPerSample = 1000;

struct TimingSummary {
    double median_ns_per_operation;
    double p95_ns_per_operation;
};

template <typename Operation> TimingSummary measure_cpu_batches(Operation&& operation) {
    for (int i = 0; i < kBenchmarkOperationsPerSample; ++i) {
        operation();
    }

    std::vector<double> samples;
    samples.reserve(kBenchmarkSampleCount);
    for (int sample = 0; sample < kBenchmarkSampleCount; ++sample) {
        QElapsedTimer timer;
        timer.start();
        for (int i = 0; i < kBenchmarkOperationsPerSample; ++i) {
            operation();
        }
        samples.push_back(static_cast<double>(timer.nsecsElapsed()) /
                          kBenchmarkOperationsPerSample);
    }
    std::sort(samples.begin(), samples.end());
    return {samples[kBenchmarkSampleCount / 2],
            samples[static_cast<int>(std::ceil(0.95 * kBenchmarkSampleCount)) - 1]};
}

} // namespace

class ViewportNavigationCpuBenchmark final : public QObject {
    Q_OBJECT

  private slots:
    void measures_navigation_cpu_ablation() {
        vtkNew<vtkCamera> camera;
        camera->SetPosition(0.0, 0.0, 10.0);
        camera->SetFocalPoint(0.0, 0.0, 0.0);
        camera->SetViewUp(0.0, 1.0, 0.0);
        const auto start = panta::visualization::ViewportCamera::capture(camera);
        const panta::visualization::CameraFrame frame{{1.0, 0.0, 0.0}, {0.0, 0.0, 1.0}};
        const std::array<double, 3> focal_point = {0.5, 0.25, 0.0};
        const auto target =
            panta::visualization::ViewportCamera::make_target(frame, focal_point, start.distance);
        // 循环采样不同姿态，避免反复提交相同值只测到 VTK setter 的快速返回。
        unsigned int frame_index = 0;
        const auto next_progress = [&] { return static_cast<double>(frame_index++ % 32U) / 31.0; };
        volatile double camera_pose_sink = 0.0;
        const auto camera_pose = measure_cpu_batches([&] {
            const auto pose =
                panta::visualization::ViewportCamera::interpolate(start, target, next_progress());
            camera_pose_sink = pose.orientation[0] + pose.focal_point[0] + pose.distance;
        });

        panta::visualization::ViewportOrientation orientation;
        vtkNew<vtkRenderWindow> render_window;
        orientation.attach(render_window);
        orientation.update(camera);
        std::array<vtkSmartPointer<vtkCamera>, 32> sample_cameras;
        for (auto& sample_camera : sample_cameras) {
            sample_camera = vtkSmartPointer<vtkCamera>::New();
            panta::visualization::ViewportCamera::apply(
                sample_camera,
                panta::visualization::ViewportCamera::interpolate(start, target, next_progress()));
        }
        const auto overlay_sync = measure_cpu_batches(
            [&] { orientation.update(sample_cameras[frame_index++ % sample_cameras.size()]); });

        panta::visualization::ViewportOrientation no_overlay_orientation;
        const auto no_overlay_sync =
            measure_cpu_batches([&] { no_overlay_orientation.update(camera); });

        panta::visualization::ViewportOrientation steady_orientation;
        vtkNew<vtkRenderWindow> steady_render_window;
        steady_orientation.attach(steady_render_window);
        steady_orientation.update(camera);
        const auto steady_overlay_sync =
            measure_cpu_batches([&] { steady_orientation.update(camera); });

        panta::visualization::ViewportOrientation picking_orientation;
        vtkNew<vtkRenderWindow> picking_window;
        picking_window->SetSize(1000, 800);
        picking_orientation.attach(picking_window);
        vtkNew<vtkCamera> picking_camera;
        picking_camera->SetPosition(0.0, 0.0, 5.0);
        picking_camera->SetFocalPoint(0.0, 0.0, 0.0);
        picking_camera->SetViewUp(0.0, 1.0, 0.0);
        picking_orientation.update(picking_camera);
        volatile int picked_direction = -1;
        const auto cube_face_pick = measure_cpu_batches([&] {
            const auto picked = picking_orientation.cube_direction(940, 704, 1000, 800);
            picked_direction = picked.has_value() ? static_cast<int>(*picked) : -1;
        });

        vtkNew<vtkCubeSource> cube;
        vtkNew<vtkPolyDataMapper> mapper;
        mapper->SetInputConnection(cube->GetOutputPort());
        vtkNew<vtkActor> actor;
        actor->SetMapper(mapper);
        vtkNew<vtkRenderer> scene_renderer;
        scene_renderer->AddActor(actor);
        scene_renderer->SetActiveCamera(camera);
        render_window->AddRenderer(scene_renderer);
        const auto clipping_reset =
            measure_cpu_batches([&] { scene_renderer->ResetCameraClippingRange(); });
        const auto transition_without_overlay = measure_cpu_batches([&] {
            const auto pose =
                panta::visualization::ViewportCamera::interpolate(start, target, next_progress());
            panta::visualization::ViewportCamera::apply(camera, pose);
            scene_renderer->ResetCameraClippingRange();
        });
        const auto full_tick = measure_cpu_batches([&] {
            const auto pose =
                panta::visualization::ViewportCamera::interpolate(start, target, next_progress());
            panta::visualization::ViewportCamera::apply(camera, pose);
            scene_renderer->ResetCameraClippingRange();
            orientation.update(camera);
        });

        qInfo().nospace() << "navigation CPU ablation (" << kBenchmarkSampleCount << " samples x "
                          << kBenchmarkOperationsPerSample
                          << " iterations; p50/p95 ns per operation): "
                          << "camera interpolation=" << camera_pose.median_ns_per_operation << "/"
                          << camera_pose.p95_ns_per_operation
                          << ", overlay sync=" << overlay_sync.median_ns_per_operation << "/"
                          << overlay_sync.p95_ns_per_operation << ", no overlay renderer update="
                          << no_overlay_sync.median_ns_per_operation << "/"
                          << no_overlay_sync.p95_ns_per_operation
                          << ", steady overlay sync=" << steady_overlay_sync.median_ns_per_operation
                          << "/" << steady_overlay_sync.p95_ns_per_operation
                          << ", cube face pick=" << cube_face_pick.median_ns_per_operation << "/"
                          << cube_face_pick.p95_ns_per_operation
                          << ", clipping reset=" << clipping_reset.median_ns_per_operation << "/"
                          << clipping_reset.p95_ns_per_operation << ", transition without overlay="
                          << transition_without_overlay.median_ns_per_operation << "/"
                          << transition_without_overlay.p95_ns_per_operation
                          << ", full transition tick=" << full_tick.median_ns_per_operation << "/"
                          << full_tick.p95_ns_per_operation;
        QVERIFY(camera_pose_sink != 0.0);
        QVERIFY(camera_pose.median_ns_per_operation > 0.0);
        QVERIFY(overlay_sync.median_ns_per_operation > 0.0);
        QVERIFY(no_overlay_sync.median_ns_per_operation > 0.0);
        QVERIFY(steady_overlay_sync.median_ns_per_operation > 0.0);
        QVERIFY(cube_face_pick.median_ns_per_operation > 0.0);
        QCOMPARE(picked_direction,
                 static_cast<int>(panta::visualization::CubeDirection::PositiveZ));
        QVERIFY(clipping_reset.median_ns_per_operation > 0.0);
        QVERIFY(transition_without_overlay.median_ns_per_operation > 0.0);
        QVERIFY(full_tick.median_ns_per_operation > 0.0);
    }
};

QTEST_APPLESS_MAIN(ViewportNavigationCpuBenchmark)
#include "viewport_navigation_cpu_benchmark.moc"
