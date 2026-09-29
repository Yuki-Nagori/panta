// 080 的手动真窗口基准：用保存的 STL 快照驱动单个 VTK WebGPU viewport。
#include "panta/visualization/mesh_source.hpp"
#include "panta/visualization/render_scene.hpp"
#include "project_view_model.hpp"
#include <QEventLoop>
#include <QGuiApplication>
#include <QLoggingCategory>
#include <QObject>
#include <QQuickWindow>
#include <QString>
#include <QStringList>
#include <QSysInfo>
#include <QTimer>
#include <QVariant>
#include <QtCore/qbytearrayalgorithms.h>
#include <QtCore/qnamespace.h>
#include <QtCore/qtenvironmentvariables.h>
#include <QtCore/qtversion.h>
#include <QtLogging>
#include <algorithm>
#include <atomic>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <exception>
#include <iomanip>
#include <iostream>
#include <memory>
#include <optional>
#include <ratio>
#include <string>
#include <vector>

#if defined(Q_OS_MACOS)
#include <mach/kern_return.h>
#include <mach/mach_init.h>
#include <mach/message.h>
#include <mach/task.h>
#include <mach/task_info.h>
#include <sys/resource.h>
#elif defined(Q_OS_LINUX)
#include <fstream>
#include <sys/resource.h>
#include <unistd.h>
#elif defined(Q_OS_WIN)
#include <psapi.h>
#include <windows.h>
#endif

#include <vtk_viewport.hpp>

namespace {

using Clock = std::chrono::steady_clock;
using panta::bridge::ProjectViewModel;
using panta::visualization::RenderScene;
using panta::visualization::VtkViewport;

struct LoadedDocument {
    QString id;
    QString name;
    std::uint64_t snapshot_capacity_bytes = 0;
};

QtMessageHandler previous_message_handler = nullptr;
std::atomic<int> submitted_frames{0};

void capture_frame_submissions(QtMsgType type, const QMessageLogContext& context,
                               const QString& message) {
    if (context.category != nullptr && qstrcmp(context.category, "panta.viewport") == 0 &&
        message.startsWith(QStringLiteral("frame submitted"))) {
        submitted_frames.fetch_add(1, std::memory_order_relaxed);
        return;
    }
    if (context.category != nullptr && qstrcmp(context.category, "panta.viewport") == 0 &&
        message.startsWith(QStringLiteral("surface synchronized"))) {
        return;
    }
    if (previous_message_handler != nullptr) {
        previous_message_handler(type, context, message);
    }
}

class FrameCapture final {
  public:
    FrameCapture() {
        QLoggingCategory::setFilterRules(QStringLiteral("panta.viewport.debug=true"));
        previous_message_handler = qInstallMessageHandler(capture_frame_submissions);
    }
    ~FrameCapture() {
        qInstallMessageHandler(previous_message_handler);
        QLoggingCategory::setFilterRules(QString{});
    }
};

class DiagnosticPause final {
  public:
    DiagnosticPause()
        : phases_(QString::fromLocal8Bit(qgetenv("PANTA_BENCH_PAUSE_PHASE"))
                      .split(',', Qt::SkipEmptyParts)) {
        if (phases_.isEmpty()) {
            return;
        }
        duration_ms_ = qEnvironmentVariableIntValue("PANTA_BENCH_PAUSE_MS", &valid_);
        valid_ = valid_ && duration_ms_ > 0;
    }

    [[nodiscard]] bool valid() const { return valid_; }

    void at(const char* phase) const {
        if (!phases_.contains(QString::fromLatin1(phase))) {
            return;
        }
        std::cout << "diagnostic_pause phase=" << phase
                  << " pid=" << QCoreApplication::applicationPid()
                  << " duration_ms=" << duration_ms_ << '\n';
        QEventLoop loop;
        QTimer::singleShot(duration_ms_, &loop, &QEventLoop::quit);
        loop.exec();
    }

  private:
    QStringList phases_;
    int duration_ms_ = 0;
    bool valid_ = true;
};

struct MemorySample {
    std::optional<std::uint64_t> resident_bytes;
    std::optional<std::uint64_t> peak_bytes;
};

std::optional<std::uint64_t> resident_bytes() {
#if defined(Q_OS_MACOS)
    mach_task_basic_info_data_t info{};
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), MACH_TASK_BASIC_INFO, reinterpret_cast<task_info_t>(&info),
                  &count) != KERN_SUCCESS) {
        return std::nullopt;
    }
    return static_cast<std::uint64_t>(info.resident_size);
#elif defined(Q_OS_LINUX)
    std::ifstream statm("/proc/self/statm");
    std::uint64_t virtual_pages = 0;
    std::uint64_t resident_pages = 0;
    if (!(statm >> virtual_pages >> resident_pages)) {
        return std::nullopt;
    }
    (void)virtual_pages;
    const long page_size = sysconf(_SC_PAGESIZE);
    if (page_size <= 0) {
        return std::nullopt;
    }
    return resident_pages * static_cast<std::uint64_t>(page_size);
#elif defined(Q_OS_WIN)
    PROCESS_MEMORY_COUNTERS_EX counters{};
    counters.cb = sizeof(counters);
    if (!GetProcessMemoryInfo(GetCurrentProcess(),
                              reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&counters),
                              sizeof(counters))) {
        return std::nullopt;
    }
    return static_cast<std::uint64_t>(counters.WorkingSetSize);
#else
    return std::nullopt;
#endif
}

std::optional<std::uint64_t> peak_rss_bytes() {
#if defined(Q_OS_MACOS) || defined(Q_OS_LINUX)
    rusage usage{};
    if (getrusage(RUSAGE_SELF, &usage) != 0) {
        return std::nullopt;
    }
#if defined(Q_OS_MACOS)
    return static_cast<std::uint64_t>(usage.ru_maxrss);
#else
    return static_cast<std::uint64_t>(usage.ru_maxrss) * 1024U;
#endif
#elif defined(Q_OS_WIN)
    PROCESS_MEMORY_COUNTERS_EX counters{};
    counters.cb = sizeof(counters);
    if (!GetProcessMemoryInfo(GetCurrentProcess(),
                              reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&counters),
                              sizeof(counters))) {
        return std::nullopt;
    }
    return static_cast<std::uint64_t>(counters.PeakWorkingSetSize);
#else
    return std::nullopt;
#endif
}

double mib(std::uint64_t bytes) { return static_cast<double>(bytes) / (1024.0 * 1024.0); }

void print_memory(const char* phase, const MemorySample& sample,
                  std::uint64_t retained_snapshot_bytes) {
    std::cout << "memory phase=" << phase << " resident_mib=";
    if (sample.resident_bytes) {
        std::cout << std::fixed << std::setprecision(2) << mib(*sample.resident_bytes);
    } else {
        std::cout << "unavailable";
    }
    std::cout << " peak_rss_mib=";
    if (sample.peak_bytes) {
        std::cout << std::fixed << std::setprecision(2) << mib(*sample.peak_bytes);
    } else {
        std::cout << "unavailable";
    }
    std::cout << " retained_snapshot_capacity_mib=" << std::fixed << std::setprecision(2)
              << mib(retained_snapshot_bytes) << '\n';
}

double percentile_ms(std::vector<double> samples, double percentile) {
    if (samples.empty()) {
        return 0.0;
    }
    std::sort(samples.begin(), samples.end());
    const auto rank =
        static_cast<std::size_t>(std::ceil(percentile * static_cast<double>(samples.size())));
    const auto index = std::clamp<std::size_t>(rank, 1, samples.size()) - 1;
    return samples[index];
}

std::uint64_t retained_payload(const std::vector<LoadedDocument>& documents) {
    std::uint64_t bytes = 0;
    for (const auto& document : documents) {
        bytes += document.snapshot_capacity_bytes;
    }
    return bytes;
}

bool document_ready(const ProjectViewModel& view_model, const QString& id) {
    if (view_model.activeDocumentId() != id || view_model.mesh_snapshot() == nullptr) {
        return false;
    }
    const auto documents = view_model.openDocuments();
    return std::any_of(documents.cbegin(), documents.cend(), [&id](const QVariant& value) {
        const auto document = value.toMap();
        return document.value(QStringLiteral("id")).toString() == id &&
               document.value(QStringLiteral("state")).toString() == QStringLiteral("ready");
    });
}

template <typename Predicate> bool wait_until(Predicate&& ready, int timeout_ms) {
    if (ready()) {
        return true;
    }
    QEventLoop loop;
    QTimer poll;
    QTimer timeout;
    poll.setInterval(1);
    timeout.setSingleShot(true);
    QObject::connect(&poll, &QTimer::timeout, &loop, [&loop, &ready]() {
        if (ready()) {
            loop.quit();
        }
    });
    QObject::connect(&timeout, &QTimer::timeout, &loop, &QEventLoop::quit);
    poll.start();
    timeout.start(timeout_ms);
    loop.exec();
    poll.stop();
    return ready();
}

int fail(const std::string& message) {
    std::cerr << "error: " << message << '\n';
    return 1;
}

} // namespace

int main(int argc, char* argv[]) try {
    QGuiApplication app(argc, argv);
    std::cout << std::unitbuf;
    const auto arguments = app.arguments();
    if (arguments.size() < 2 || arguments.size() > 3) {
        return fail("usage: panta_project_stl_viewport_gpu_benchmark <project.panta> [samples]");
    }
    if (QGuiApplication::platformName() == QStringLiteral("offscreen")) {
        return fail("GPU benchmark requires a visible native graphics session");
    }
    constexpr int kDefaultSamples = 31;
    bool sample_count_ok = true;
    const int kSamples =
        arguments.size() == 3 ? arguments[2].toInt(&sample_count_ok) : kDefaultSamples;
    if (!sample_count_ok || kSamples <= 0) {
        return fail("samples must be a positive integer");
    }
    const DiagnosticPause diagnostic_pause;
    if (!diagnostic_pause.valid()) {
        return fail("PANTA_BENCH_PAUSE_PHASE requires a positive PANTA_BENCH_PAUSE_MS");
    }
    const FrameCapture frame_capture;

    std::cout << "benchmark=project_stl_viewport_gpu build="
#if defined(NDEBUG)
              << "Release"
#else
              << "Debug"
#endif
              << " qt=" << qVersion()
              << " architecture=" << QSysInfo::currentCpuArchitecture().toStdString()
              << " samples_per_document=" << kSamples
              << " malloc_stack_logging=" << qgetenv("MallocStackLogging").constData()
              << " malloc_stack_logging_no_compact="
              << qEnvironmentVariableIsSet("MallocStackLoggingNoCompact")
              << " metric=VtkViewport_frame_submitted excludes=GPU_completion_and_dedicated_VRAM\n";

    ProjectViewModel view_model;
    QQuickWindow window;
    window.resize(1280, 800);
    window.setColor(Qt::white);
    VtkViewport viewport(window.contentItem());
    viewport.setSize(QSizeF(1280, 800));
    RenderScene scene;
    std::uint64_t scene_revision = 0;
    const auto sync_viewport = [&]() {
        scene.revision = ++scene_revision;
        scene.mesh = view_model.mesh_snapshot();
        scene.primitive_visible = scene.mesh != nullptr || view_model.placeholder_visible();
        viewport.apply_state(scene);
    };
    QObject::connect(&view_model, &panta::visualization::MeshSource::meshChanged, &viewport,
                     sync_viewport);
    bool scene_initialized = false;
    QObject::connect(&viewport, &VtkViewport::sceneInitialized, &app,
                     [&scene_initialized]() { scene_initialized = true; });
    const int before_window_show = submitted_frames.load(std::memory_order_relaxed);
    window.show();
    if (!wait_until([&window]() { return window.isExposed(); }, 10000)) {
        return fail("native benchmark window was not exposed");
    }
    if (!wait_until([&scene_initialized]() { return scene_initialized; }, 30000)) {
        return fail("VTK WebGPU scene was not initialized");
    }
    if (!wait_until(
            [before_window_show]() {
                return submitted_frames.load(std::memory_order_relaxed) > before_window_show;
            },
            30000)) {
        return fail("VTK WebGPU scene did not submit a frame");
    }
    print_memory("native_viewport_baseline", {resident_bytes(), peak_rss_bytes()}, 0);
    diagnostic_pause.at("native_viewport_baseline");

    if (!view_model.openProject(arguments[1])) {
        return fail("openProject failed: " + view_model.error().toStdString());
    }
    const auto ids = view_model.importedPartIds();
    const auto names = view_model.importedPartNames();
    if (ids.isEmpty() || ids.size() != names.size()) {
        return fail("project has no consistent saved STL records");
    }
    std::vector<LoadedDocument> loaded;
    loaded.reserve(static_cast<std::size_t>(ids.size()));
    for (qsizetype index = 0; index < ids.size(); ++index) {
        const QString& id = ids[index];
        const int before_frame = submitted_frames.load(std::memory_order_relaxed);
        const auto start = Clock::now();
        view_model.openImportRecord(id);
        const auto ready = [&]() {
            return document_ready(view_model, id) &&
                   submitted_frames.load(std::memory_order_relaxed) > before_frame;
        };
        if (!wait_until(ready, 60000)) {
            return fail("activation or VTK frame timed out for " + names[index].toStdString());
        }
        const double elapsed_ms =
            std::chrono::duration<double, std::milli>(Clock::now() - start).count();
        const auto snapshot = view_model.mesh_snapshot();
        if (!snapshot || snapshot->vertices.empty()) {
            return fail("activation returned an empty mesh for " + names[index].toStdString());
        }
        const auto payload_bytes = static_cast<std::uint64_t>(snapshot->vertices.capacity()) *
                                   static_cast<std::uint64_t>(sizeof(snapshot->vertices[0]));
        loaded.push_back({id, names[index], payload_bytes});
        std::cout << "initial_render file=" << names[index].toStdString()
                  << " triangles=" << snapshot->vertices.size() / 3
                  << " activation_to_frame_ms=" << std::fixed << std::setprecision(3) << elapsed_ms
                  << '\n';
    }

    const auto largest = std::max_element(
        loaded.begin(), loaded.end(), [](const LoadedDocument& lhs, const LoadedDocument& rhs) {
            return lhs.snapshot_capacity_bytes < rhs.snapshot_capacity_bytes;
        });
    if (largest != loaded.end() && view_model.activeDocumentId() != largest->id) {
        const int before_frame = submitted_frames.load(std::memory_order_relaxed);
        view_model.activateDocument(largest->id);
        if (!wait_until(
                [before_frame]() {
                    return submitted_frames.load(std::memory_order_relaxed) > before_frame;
                },
                30000)) {
            return fail("largest STL did not reach a VTK frame");
        }
    }
    print_memory("all_snapshots_retained_largest_active", {resident_bytes(), peak_rss_bytes()},
                 retained_payload(loaded));
    diagnostic_pause.at("all_snapshots_retained_largest_active");

    for (const auto& document : loaded) {
        const auto alternate = std::find_if(
            loaded.cbegin(), loaded.cend(),
            [&document](const LoadedDocument& candidate) { return candidate.id != document.id; });
        if (alternate == loaded.cend()) {
            return fail("GPU frame sampling requires at least two imported documents");
        }
        std::vector<double> frame_samples_ms;
        frame_samples_ms.reserve(kSamples);
        for (int sample = 0; sample < kSamples; ++sample) {
            // ViewModel 对已活动页签不会重复发射 meshChanged；先切到另一页，
            // 让每个样本都实际经历一次目标模型的 VTK 场景更新。
            if (view_model.activeDocumentId() == document.id) {
                const int before_alternate = submitted_frames.load(std::memory_order_relaxed);
                view_model.activateDocument(alternate->id);
                if (!wait_until(
                        [before_alternate]() {
                            return submitted_frames.load(std::memory_order_relaxed) >
                                   before_alternate;
                        },
                        30000)) {
                    return fail("VTK alternate frame submit timed out for " +
                                alternate->name.toStdString());
                }
            }
            const int before_frame = submitted_frames.load(std::memory_order_relaxed);
            const auto start = Clock::now();
            view_model.activateDocument(document.id);
            if (!wait_until(
                    [before_frame]() {
                        return submitted_frames.load(std::memory_order_relaxed) > before_frame;
                    },
                    30000)) {
                return fail("VTK frame submit timed out for " + document.name.toStdString());
            }
            frame_samples_ms.push_back(
                std::chrono::duration<double, std::milli>(Clock::now() - start).count());
        }
        std::cout << "vtk_frame_submit file=" << document.name.toStdString()
                  << " samples=" << frame_samples_ms.size() << " p50_ms=" << std::fixed
                  << std::setprecision(3) << percentile_ms(frame_samples_ms, 0.50)
                  << " p95_ms=" << percentile_ms(frame_samples_ms, 0.95)
                  << " boundary=scene_change_to_VTK_submit_not_GPU_completion\n";
        std::cout << "memory_after_frame_cycles file=" << document.name.toStdString() << ' ';
        print_memory("document_complete", {resident_bytes(), peak_rss_bytes()},
                     retained_payload(loaded));
    }
    print_memory("after_frame_cycles", {resident_bytes(), peak_rss_bytes()},
                 retained_payload(loaded));
    diagnostic_pause.at("after_frame_cycles");

    bool pause_value_ok = true;
    const int pause_ms =
        qEnvironmentVariableIsSet("PANTA_BENCH_KEEP_WINDOW_MS")
            ? qEnvironmentVariableIntValue("PANTA_BENCH_KEEP_WINDOW_MS", &pause_value_ok)
            : 0;
    if (!pause_value_ok || pause_ms < 0) {
        return fail("PANTA_BENCH_KEEP_WINDOW_MS must be a non-negative integer");
    }
    if (pause_ms > 0 && largest != loaded.end()) {
        const int before_frame = submitted_frames.load(std::memory_order_relaxed);
        view_model.activateDocument(largest->id);
        if (!wait_until(
                [before_frame]() {
                    return submitted_frames.load(std::memory_order_relaxed) > before_frame;
                },
                30000)) {
            return fail("largest STL did not render before the window pause");
        }
        std::cout << "window_pause_ms=" << pause_ms
                  << " active_file=" << largest->name.toStdString() << '\n';
        QEventLoop pause_loop;
        QTimer::singleShot(pause_ms, &pause_loop, &QEventLoop::quit);
        pause_loop.exec();
    }

    for (const auto& document : loaded) {
        view_model.closeDocument(document.id);
    }
    view_model.closeDocument(QStringLiteral("welcome"));
    const int before_empty_frame = submitted_frames.load(std::memory_order_relaxed);
    if (!wait_until(
            [before_empty_frame]() {
                return submitted_frames.load(std::memory_order_relaxed) > before_empty_frame;
            },
            30000)) {
        return fail("empty viewport frame did not submit after closing all documents");
    }
    print_memory("all_documents_closed", {resident_bytes(), peak_rss_bytes()}, 0);
    diagnostic_pause.at("all_documents_closed");
    return 0;
} catch (const std::exception& exception) {
    std::fprintf(stderr, "error: uncaught benchmark exception: %s\n", exception.what());
    return 1;
} catch (...) {
    std::fputs("error: unknown benchmark exception\n", stderr);
    return 1;
}
