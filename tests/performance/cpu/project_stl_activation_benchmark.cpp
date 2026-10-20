// 080 / 088 的手动真实工程基准；测量 Mesh sidecar 激活/重载、页签快切与快照内存。
#include "project_operation_helpers.hpp"
#include "project_view_model.hpp"
#include <QCoreApplication>
#include <QEventLoop>
#include <QObject>
#include <QString>
#include <QSysInfo>
#include <QTimer>
#include <QVariant>
#include <QtCore/qtversion.h>
#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <ctime>
#include <exception>
#include <filesystem>
#include <iomanip>
#include <iostream>
#include <memory>
#include <optional>
#include <panta/visualization/mesh_source.hpp>
#include <ratio>
#include <string>
#include <system_error>
#include <utility>
#include <vector>

#if defined(Q_OS_MACOS)
#include <sys/resource.h>
#endif

#if defined(Q_OS_MACOS)
#include <mach/kern_return.h>
#include <mach/mach_init.h>
#include <mach/message.h>
#include <mach/task.h>
#include <mach/task_info.h>
#elif defined(Q_OS_LINUX)
#include <fstream>
#include <unistd.h>
#elif defined(Q_OS_WIN)
#include <psapi.h>
#include <windows.h>
#endif

using panta::test::finish_project_operation;

namespace {

using Clock = std::chrono::steady_clock;
using panta::bridge::ProjectViewModel;
using panta::visualization::SurfaceMeshSnapshot;

struct MemorySample {
    std::optional<std::uint64_t> resident_bytes;
    std::optional<std::uint64_t> peak_bytes;
};

struct TimedSample {
    double wall_ms = 0.0;
    std::optional<double> process_cpu_ms;
};

struct MeasurementStart {
    Clock::time_point wall;
    std::optional<double> process_cpu_ms;
};

struct LoadedDocument {
    QString id;
    QString name;
    std::uint64_t mesh_payload_estimate_bytes = 0;
};

struct MeshCacheStats {
    std::size_t resident_documents = 0;
    std::uint64_t estimated_resident_mesh_bytes = 0;
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
#if defined(Q_OS_MACOS)
    struct rusage usage{};
    if (getrusage(RUSAGE_SELF, &usage) != 0) {
        return std::nullopt;
    }
    return static_cast<std::uint64_t>(usage.ru_maxrss);
#elif defined(Q_OS_LINUX)
    // VmHWM 是 Linux 提供的进程 RSS 高水位，单位为 kB。
    std::ifstream status("/proc/self/status");
    std::string key;
    while (status >> key) {
        if (key == "VmHWM:") {
            std::uint64_t peak_kib = 0;
            std::string unit;
            if (!(status >> peak_kib >> unit) || unit != "kB") {
                return std::nullopt;
            }
            return peak_kib * 1024U;
        }
        std::string remainder;
        std::getline(status, remainder);
    }
    return std::nullopt;
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

MemorySample sample_memory() { return {resident_bytes(), peak_rss_bytes()}; }

double mib(std::uint64_t bytes) { return static_cast<double>(bytes) / (1024.0 * 1024.0); }

std::optional<double> process_cpu_ms() {
#if defined(Q_OS_MACOS)
    struct rusage usage{};
    if (getrusage(RUSAGE_SELF, &usage) != 0) {
        return std::nullopt;
    }
    const auto to_microseconds = [](const auto& value) {
        return static_cast<std::int64_t>(value.tv_sec) * 1'000'000 + value.tv_usec;
    };
    return static_cast<double>(to_microseconds(usage.ru_utime) + to_microseconds(usage.ru_stime)) /
           1000.0;
#elif defined(Q_OS_LINUX)
    const std::clock_t ticks = std::clock();
    if (ticks == static_cast<std::clock_t>(-1)) {
        return std::nullopt;
    }
    return static_cast<double>(ticks) * 1000.0 / CLOCKS_PER_SEC;
#elif defined(Q_OS_WIN)
    FILETIME creation_time{};
    FILETIME exit_time{};
    FILETIME kernel_time{};
    FILETIME user_time{};
    if (!GetProcessTimes(GetCurrentProcess(), &creation_time, &exit_time, &kernel_time,
                         &user_time)) {
        return std::nullopt;
    }
    ULARGE_INTEGER kernel_ticks{};
    ULARGE_INTEGER user_ticks{};
    kernel_ticks.LowPart = kernel_time.dwLowDateTime;
    kernel_ticks.HighPart = kernel_time.dwHighDateTime;
    user_ticks.LowPart = user_time.dwLowDateTime;
    user_ticks.HighPart = user_time.dwHighDateTime;
    return static_cast<double>(kernel_ticks.QuadPart + user_ticks.QuadPart) / 10'000.0;
#else
    return std::nullopt;
#endif
}

MeasurementStart start_measurement() {
    const auto cpu = process_cpu_ms();
    return {Clock::now(), cpu};
}

TimedSample finish_measurement(const MeasurementStart& start,
                               Clock::time_point wall_end = Clock::now()) {
    const auto cpu_end = process_cpu_ms();
    std::optional<double> cpu_elapsed;
    if (start.process_cpu_ms && cpu_end) {
        cpu_elapsed = *cpu_end - *start.process_cpu_ms;
    }
    return {std::chrono::duration<double, std::milli>(wall_end - start.wall).count(), cpu_elapsed};
}

std::uint64_t file_bytes(const std::filesystem::path& path) {
    return static_cast<std::uint64_t>(std::filesystem::file_size(path));
}

std::uint64_t directory_bytes(const std::filesystem::path& directory) {
    std::error_code error;
    const bool exists = std::filesystem::is_directory(directory, error);
    if (error) {
        throw std::filesystem::filesystem_error("inspect sidecar directory", directory, error);
    }
    if (!exists) {
        return 0;
    }
    std::uint64_t bytes = 0;
    std::filesystem::recursive_directory_iterator iterator(directory, error);
    const std::filesystem::recursive_directory_iterator end;
    if (error) {
        throw std::filesystem::filesystem_error("iterate sidecar directory", directory, error);
    }
    while (iterator != end) {
        const bool regular_file = iterator->is_regular_file(error);
        if (error) {
            throw std::filesystem::filesystem_error("inspect sidecar file", iterator->path(),
                                                    error);
        }
        if (regular_file) {
            const bool symlink = iterator->is_symlink(error);
            if (error) {
                throw std::filesystem::filesystem_error("inspect sidecar symlink", iterator->path(),
                                                        error);
            }
            if (!symlink) {
                bytes += file_bytes(iterator->path());
            }
        }
        iterator.increment(error);
        if (error) {
            throw std::filesystem::filesystem_error("iterate sidecar directory", directory, error);
        }
    }
    return bytes;
}

void print_project_storage(const std::filesystem::path& manifest_path) {
    const auto manifest_bytes = file_bytes(manifest_path);
    const auto sidecar_bytes = directory_bytes(manifest_path.parent_path() / "assets" / "meshes");
    std::cout << "project_storage manifest_bytes=" << manifest_bytes
              << " mesh_sidecar_bytes=" << sidecar_bytes
              << " total_bytes=" << manifest_bytes + sidecar_bytes
              << " scope=manifest_plus_mesh_sidecars excludes=source_STL,lock_files,VTK_GPU\n";
}

void print_memory(const char* phase, const MemorySample& sample,
                  std::uint64_t estimated_resident_mesh_bytes) {
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
    std::cout << " estimated_resident_mesh_payload_mib=" << std::fixed << std::setprecision(2)
              << mib(estimated_resident_mesh_bytes) << '\n';
}

std::pair<double, double> percentiles_ms(std::vector<double> samples) {
    if (samples.empty()) {
        return {0.0, 0.0};
    }
    std::sort(samples.begin(), samples.end());
    const auto at_percentile = [&samples](double percentile) {
        const auto rank =
            static_cast<std::size_t>(std::ceil(percentile * static_cast<double>(samples.size())));
        const auto index = std::clamp<std::size_t>(rank, 1, samples.size()) - 1;
        return samples[index];
    };
    return {at_percentile(0.50), at_percentile(0.95)};
}

void print_optional_ms(const std::optional<double>& value) {
    if (value) {
        std::cout << std::fixed << std::setprecision(3) << *value;
    } else {
        std::cout << "unavailable";
    }
}

void print_samples(const std::string& phase, const std::vector<TimedSample>& samples,
                   const char* scope) {
    std::vector<double> wall_ms;
    std::vector<double> cpu_ms;
    wall_ms.reserve(samples.size());
    cpu_ms.reserve(samples.size());
    for (const auto& sample : samples) {
        wall_ms.push_back(sample.wall_ms);
        if (sample.process_cpu_ms) {
            cpu_ms.push_back(*sample.process_cpu_ms);
        }
    }
    const bool has_cpu_samples = !cpu_ms.empty();
    const auto [wall_p50, wall_p95] = percentiles_ms(std::move(wall_ms));
    const auto [cpu_p50, cpu_p95] = percentiles_ms(std::move(cpu_ms));
    std::cout << phase << " samples=" << samples.size() << " wall_p50_ms=" << std::fixed
              << std::setprecision(3) << wall_p50 << " wall_p95_ms=" << wall_p95
              << " process_cpu_p50_ms=";
    print_optional_ms(has_cpu_samples ? std::optional<double>(cpu_p50) : std::nullopt);
    std::cout << " process_cpu_p95_ms=";
    print_optional_ms(has_cpu_samples ? std::optional<double>(cpu_p95) : std::nullopt);
    std::cout << " scope=" << scope << '\n';
}

QString document_state(const ProjectViewModel& view_model, const QString& id) {
    const auto documents = view_model.openDocuments();
    for (const auto& value : documents) {
        const auto document = value.toMap();
        if (document.value(QStringLiteral("id")).toString() == id) {
            return document.value(QStringLiteral("state")).toString();
        }
    }
    return {};
}

bool document_ready(const ProjectViewModel& view_model, const QString& id) {
    if (view_model.activeDocumentId() != id || view_model.mesh_snapshot() == nullptr) {
        return false;
    }
    return document_state(view_model, id) == QStringLiteral("ready");
}

template <typename Predicate>
bool wait_for_activation(ProjectViewModel& view_model, Predicate&& ready, int timeout_ms) {
    if (ready()) {
        return true;
    }
    QEventLoop loop;
    QTimer timeout;
    timeout.setSingleShot(true);
    const auto check = [&loop, &ready]() {
        if (ready()) {
            loop.quit();
        }
    };
    const auto active_connection =
        QObject::connect(&view_model, &ProjectViewModel::activeDocumentChanged, &loop, check);
    const auto documents_connection =
        QObject::connect(&view_model, &ProjectViewModel::documentsChanged, &loop, check);
    QObject::connect(&view_model, &ProjectViewModel::meshChanged, &loop, check);
    QObject::connect(&timeout, &QTimer::timeout, &loop, &QEventLoop::quit);
    timeout.start(timeout_ms);
    loop.exec();
    QObject::disconnect(active_connection);
    QObject::disconnect(documents_connection);
    return ready();
}

// 用每个资产先前跨 FFI 得到的显示 DTO 容量估算 Rust Mesh payload；DTO 本身不按页签保留。
MeshCacheStats mesh_cache_stats(const ProjectViewModel& view_model,
                                const std::vector<LoadedDocument>& documents) {
    MeshCacheStats stats;
    for (const auto& document : documents) {
        if (document_state(view_model, document.id) == QStringLiteral("ready")) {
            ++stats.resident_documents;
            stats.estimated_resident_mesh_bytes += document.mesh_payload_estimate_bytes;
        }
    }
    return stats;
}

int fail(const std::string& message) {
    std::cerr << "error: " << message << '\n';
    return 1;
}

} // namespace

int main(int argc, char* argv[]) try {
    QCoreApplication app(argc, argv);
    const auto arguments = app.arguments();
    if (arguments.size() < 2 || arguments.size() > 3) {
        return fail("usage: panta_bridge_project_stl_activation_benchmark <project.panta> "
                    "[reload_iterations=31]");
    }
    bool iterations_ok = true;
    const int reload_iterations = arguments.size() == 3 ? arguments[2].toInt(&iterations_ok) : 31;
    if (!iterations_ok || reload_iterations < 1) {
        return fail("reload_iterations must be a positive integer");
    }

    std::cout << "benchmark=project_mesh_activation build="
#if defined(NDEBUG)
              << "Release"
#else
              << "Debug"
#endif
              << " qt=" << qVersion()
              << " architecture=" << QSysInfo::currentCpuArchitecture().toStdString()
              << " reload_iterations=" << reload_iterations
              << " mesh_cache_policy=ProjectService_default"
              << " memory_scope=ProjectViewModel+Rust_activation excludes=VTK_GPU"
              << " cpu_clock=platform_process_cpu_time\n";

    const std::filesystem::path manifest_path(arguments[1].toStdU16String());
    std::vector<TimedSample> project_open_samples;
    std::vector<TimedSample> project_open_admission_samples;
    project_open_admission_samples.reserve(static_cast<std::size_t>(reload_iterations));
    project_open_samples.reserve(static_cast<std::size_t>(reload_iterations));
    for (int sample = 0; sample < reload_iterations; ++sample) {
        const auto start = start_measurement();
        ProjectViewModel probe;
        const bool accepted = probe.openProject(arguments[1]);
        project_open_admission_samples.push_back(finish_measurement(start));
        if (!finish_project_operation(probe, accepted)) {
            return fail("openProject failed: " + probe.error().toStdString());
        }
        project_open_samples.push_back(finish_measurement(start));
    }
    print_samples("project_open_admission", project_open_admission_samples,
                  "new_ProjectViewModel+background_command_admission;excludes_completion_wait");
    print_samples("project_open", project_open_samples,
                  "new_ProjectViewModel+worker_admission+manifest_read_validate+queued_publication;"
                  "warm_os_page_cache");

    auto view_model = std::make_unique<ProjectViewModel>();
    auto& vm = *view_model;
    const auto baseline_memory = sample_memory();
    print_memory("service_baseline", baseline_memory, 0);
    if (!finish_project_operation(vm, vm.openProject(arguments[1]))) {
        return fail("openProject failed: " + vm.error().toStdString());
    }
    print_project_storage(manifest_path);

    std::vector<TimedSample> save_samples;
    std::vector<TimedSample> save_admission_samples;
    save_admission_samples.reserve(static_cast<std::size_t>(reload_iterations));
    save_samples.reserve(static_cast<std::size_t>(reload_iterations));
    for (int sample = 0; sample < reload_iterations; ++sample) {
        const auto start = start_measurement();
        const bool accepted = vm.saveProject();
        save_admission_samples.push_back(finish_measurement(start));
        if (!finish_project_operation(vm, accepted)) {
            return fail("saveProject failed: " + vm.error().toStdString());
        }
        save_samples.push_back(finish_measurement(start));
    }
    print_samples("project_save_admission", save_admission_samples,
                  "background_command_admission;excludes_completion_wait");
    print_samples(
        "project_save", save_samples,
        "worker_admission+atomic_manifest_write+sync+queued_publication;sidecars_unchanged");

    const auto ids = vm.importedPartIds();
    const auto names = vm.importedPartNames();
    if (ids.isEmpty() || ids.size() != names.size()) {
        return fail("project has no consistent saved mesh records");
    }

    std::vector<LoadedDocument> loaded;
    loaded.reserve(static_cast<std::size_t>(ids.size()));
    for (qsizetype index = 0; index < ids.size(); ++index) {
        const QString& id = ids[index];
        const auto start = start_measurement();
        vm.openImportRecord(id);
        const auto ready = [&vm, &id]() { return document_ready(vm, id); };
        if (!wait_for_activation(vm, ready, 60000)) {
            return fail("activation timed out or failed for " + names[index].toStdString());
        }
        const auto sample = finish_measurement(start);
        const auto snapshot = vm.mesh_snapshot();
        if (!snapshot || snapshot->vertices.empty()) {
            return fail("activation returned an empty mesh for " + names[index].toStdString());
        }
        const auto mesh_payload_estimate_bytes =
            static_cast<std::uint64_t>(snapshot->vertices.capacity()) *
            static_cast<std::uint64_t>(sizeof(snapshot->vertices[0]));
        loaded.push_back({id, names[index], mesh_payload_estimate_bytes});
        std::cout << "initial_activation file=" << names[index].toStdString()
                  << " triangles=" << snapshot->vertices.size() / 3
                  << " active_dto_capacity_mib=" << std::fixed << std::setprecision(2)
                  << mib(mesh_payload_estimate_bytes) << " elapsed_ms=" << std::setprecision(3)
                  << sample.wall_ms << " process_cpu_ms=";
        print_optional_ms(sample.process_cpu_ms);
        std::cout << '\n';
        const auto stats = mesh_cache_stats(vm, loaded);
        print_memory(("cache_after_activation_" + std::to_string(index + 1)).c_str(),
                     sample_memory(), stats.estimated_resident_mesh_bytes);
    }

    const auto all_loaded_memory = sample_memory();
    const auto cache_stats = mesh_cache_stats(vm, loaded);
    std::cout << "mesh_cache_state resident_documents=" << cache_stats.resident_documents
              << " unloaded_documents=" << loaded.size() - cache_stats.resident_documents
              << " estimated_resident_mesh_payload_mib=" << std::fixed << std::setprecision(2)
              << mib(cache_stats.estimated_resident_mesh_bytes) << '\n';
    print_memory("mesh_cache_after_opening_all_tabs", all_loaded_memory,
                 cache_stats.estimated_resident_mesh_bytes);

    if (loaded.size() > 1) {
        constexpr int kSwitchCycles = 31;
        std::vector<double> selection_call_samples_ms;
        std::vector<TimedSample> selection_ready_samples;
        const auto expected_samples = static_cast<std::size_t>(kSwitchCycles) * loaded.size();
        selection_call_samples_ms.reserve(expected_samples);
        selection_ready_samples.reserve(expected_samples);
        std::size_t cache_misses = 0;
        for (int cycle = 0; cycle < kSwitchCycles; ++cycle) {
            for (const auto& document : loaded) {
                cache_misses += document_state(vm, document.id) == QStringLiteral("unloaded");
                const auto start = start_measurement();
                vm.activateDocument(document.id);
                const auto selection_returned = Clock::now();
                const auto ready = [&vm, &document] { return document_ready(vm, document.id); };
                if (!wait_for_activation(vm, ready, 60000)) {
                    return fail("document selection or cache reload timed out for " +
                                document.name.toStdString());
                }
                const auto completed = Clock::now();
                selection_call_samples_ms.push_back(
                    std::chrono::duration<double, std::milli>(selection_returned - start.wall)
                        .count());
                selection_ready_samples.push_back(finish_measurement(start, completed));
            }
        }
        const auto [call_p50, call_p95] = percentiles_ms(selection_call_samples_ms);
        std::vector<double> ready_ms;
        std::vector<double> cpu_ms;
        ready_ms.reserve(selection_ready_samples.size());
        cpu_ms.reserve(selection_ready_samples.size());
        for (const auto& sample : selection_ready_samples) {
            ready_ms.push_back(sample.wall_ms);
            if (sample.process_cpu_ms) {
                cpu_ms.push_back(*sample.process_cpu_ms);
            }
        }
        const bool has_cpu_samples = !cpu_ms.empty();
        const auto [ready_p50, ready_p95] = percentiles_ms(std::move(ready_ms));
        const auto [cpu_p50, cpu_p95] = percentiles_ms(std::move(cpu_ms));
        std::cout << "document_selection samples=" << selection_call_samples_ms.size()
                  << " cache_misses=" << cache_misses << " call_p50_ms=" << std::fixed
                  << std::setprecision(3) << call_p50 << " call_p95_ms=" << call_p95
                  << " ready_p50_ms=" << ready_p50 << " ready_p95_ms=" << ready_p95
                  << " process_cpu_p50_ms=";
        print_optional_ms(has_cpu_samples ? std::optional<double>(cpu_p50) : std::nullopt);
        std::cout << " process_cpu_p95_ms=";
        print_optional_ms(has_cpu_samples ? std::optional<double>(cpu_p95) : std::nullopt);
        std::cout << " scope=ViewModel_selection_call_and_selection_to_ready\n";
    } else {
        std::cout << "cache_hit=unavailable reason=project_has_fewer_than_two_documents\n";
    }

    for (const auto& document : loaded) {
        std::vector<TimedSample> reload_samples;
        reload_samples.reserve(static_cast<std::size_t>(reload_iterations));
        for (int iteration = 0; iteration < reload_iterations; ++iteration) {
            vm.activateDocument(document.id);
            const auto ready = [&vm, &document]() { return document_ready(vm, document.id); };
            if (!wait_for_activation(vm, ready, 60000)) {
                return fail("document selection timed out before close for " +
                            document.name.toStdString());
            }
            std::weak_ptr<const SurfaceMeshSnapshot> prior_snapshot;
            {
                const auto snapshot = vm.mesh_snapshot();
                if (!snapshot) {
                    return fail("cached snapshot disappeared before close");
                }
                prior_snapshot = snapshot;
            }
            vm.closeDocument(document.id);
            if (!prior_snapshot.expired()) {
                return fail("closed document retained its mesh snapshot");
            }

            const auto start = start_measurement();
            vm.openImportRecord(document.id);
            if (!wait_for_activation(vm, ready, 60000)) {
                return fail("reload timed out or failed for " + document.name.toStdString());
            }
            reload_samples.push_back(finish_measurement(start));
            const int completed = iteration + 1;
            if (completed == 1 || completed == 10 || completed == 50 ||
                completed == reload_iterations) {
                const std::string phase = "reload_" + document.id.toStdString() + "_iteration_" +
                                          std::to_string(completed);
                print_memory(phase.c_str(), sample_memory(),
                             mesh_cache_stats(vm, loaded).estimated_resident_mesh_bytes);
            }
        }
        print_samples("file_reload file=" + document.name.toStdString(), reload_samples,
                      "cache=warm_os_page_cache");
    }

    const auto before_close_memory = sample_memory();
    print_memory("after_reloads_all_open", before_close_memory,
                 mesh_cache_stats(vm, loaded).estimated_resident_mesh_bytes);
    std::vector<TimedSample> close_samples;
    close_samples.reserve(loaded.size());
    for (const auto& document : loaded) {
        vm.activateDocument(document.id);
        const auto ready = [&vm, &document]() { return document_ready(vm, document.id); };
        if (!wait_for_activation(vm, ready, 60000)) {
            return fail("document did not reactivate before close: " + document.name.toStdString());
        }
        std::weak_ptr<const SurfaceMeshSnapshot> prior_snapshot;
        {
            const auto snapshot = vm.mesh_snapshot();
            if (!snapshot) {
                return fail("document has no mesh snapshot before close");
            }
            prior_snapshot = snapshot;
        }
        const auto start = start_measurement();
        vm.closeDocument(document.id);
        close_samples.push_back(finish_measurement(start));
        if (!prior_snapshot.expired()) {
            return fail("closed document retained its mesh snapshot");
        }
    }
    print_samples("close_documents", close_samples, "ViewModel_closeDocument_and_snapshot_release");
    vm.closeDocument(QStringLiteral("welcome"));
    QCoreApplication::processEvents(QEventLoop::AllEvents, 50);
    print_memory("all_documents_closed", sample_memory(), 0);
    view_model.reset();
    QCoreApplication::processEvents(QEventLoop::AllEvents, 50);
    print_memory("service_destroyed", sample_memory(), 0);
    return 0;
} catch (const std::exception& exception) {
    std::fprintf(stderr, "error: uncaught benchmark exception: %s\n", exception.what());
    return 1;
} catch (...) {
    std::fputs("error: unknown benchmark exception\n", stderr);
    return 1;
}
