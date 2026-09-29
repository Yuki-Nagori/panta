// 080 的手动真实工程基准；测量工程内 STL 激活/重载、页签快切与快照内存。
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
#include <exception>
#include <iomanip>
#include <iostream>
#include <memory>
#include <optional>
#include <panta/visualization/mesh_source.hpp>
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

namespace {

using Clock = std::chrono::steady_clock;
using panta::bridge::ProjectViewModel;
using panta::visualization::SurfaceMeshSnapshot;

struct MemorySample {
    std::optional<std::uint64_t> resident_bytes;
    std::optional<std::uint64_t> peak_bytes;
};

struct LoadedDocument {
    QString id;
    QString name;
    std::uint64_t mesh_payload_estimate_bytes = 0;
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

MemorySample sample_memory() { return {resident_bytes(), peak_rss_bytes()}; }

double mib(std::uint64_t bytes) { return static_cast<double>(bytes) / (1024.0 * 1024.0); }

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
    QObject::connect(&timeout, &QTimer::timeout, &loop, &QEventLoop::quit);
    timeout.start(timeout_ms);
    loop.exec();
    QObject::disconnect(active_connection);
    QObject::disconnect(documents_connection);
    return ready();
}

// 用每个资产先前跨 FFI 得到的显示 DTO 容量估算 Rust Mesh payload；DTO 本身不按页签保留。
std::uint64_t resident_mesh_payload_estimate(const ProjectViewModel& view_model,
                                             const std::vector<LoadedDocument>& documents) {
    std::uint64_t bytes = 0;
    for (const auto& document : documents) {
        if (document_state(view_model, document.id) == QStringLiteral("ready")) {
            bytes += document.mesh_payload_estimate_bytes;
        }
    }
    return bytes;
}

std::size_t cached_document_count(const ProjectViewModel& view_model,
                                  const std::vector<LoadedDocument>& documents) {
    return static_cast<std::size_t>(std::count_if(
        documents.cbegin(), documents.cend(), [&view_model](const LoadedDocument& document) {
            return document_state(view_model, document.id) == QStringLiteral("ready");
        }));
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

    std::cout << "benchmark=project_stl_activation build="
#if defined(NDEBUG)
              << "Release"
#else
              << "Debug"
#endif
              << " qt=" << qVersion()
              << " architecture=" << QSysInfo::currentCpuArchitecture().toStdString()
              << " reload_iterations=" << reload_iterations
              << " mesh_cache_policy=ProjectService_default"
              << " memory_scope=ProjectViewModel+Rust_activation excludes=VTK_GPU\n";

    ProjectViewModel view_model;
    const auto baseline_memory = sample_memory();
    print_memory("service_baseline", baseline_memory, 0);
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
        const auto start = Clock::now();
        view_model.openImportRecord(id);
        const auto ready = [&view_model, &id]() { return document_ready(view_model, id); };
        if (!wait_for_activation(view_model, ready, 60000)) {
            return fail("activation timed out or failed for " + names[index].toStdString());
        }
        const double elapsed_ms =
            std::chrono::duration<double, std::milli>(Clock::now() - start).count();
        const auto snapshot = view_model.mesh_snapshot();
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
                  << elapsed_ms << '\n';
        print_memory(("cache_after_activation_" + std::to_string(index + 1)).c_str(),
                     sample_memory(), resident_mesh_payload_estimate(view_model, loaded));
    }

    const auto all_loaded_memory = sample_memory();
    const auto resident_mesh_bytes = resident_mesh_payload_estimate(view_model, loaded);
    std::cout << "mesh_cache_state resident_documents=" << cached_document_count(view_model, loaded)
              << " unloaded_documents=" << loaded.size() - cached_document_count(view_model, loaded)
              << " estimated_resident_mesh_payload_mib=" << std::fixed << std::setprecision(2)
              << mib(resident_mesh_bytes) << '\n';
    print_memory("mesh_cache_after_opening_all_tabs", all_loaded_memory, resident_mesh_bytes);

    if (loaded.size() > 1) {
        constexpr int kSwitchCycles = 31;
        std::vector<double> selection_call_samples_ms;
        std::vector<double> selection_ready_samples_ms;
        const auto expected_samples = static_cast<std::size_t>(kSwitchCycles) * loaded.size();
        selection_call_samples_ms.reserve(expected_samples);
        selection_ready_samples_ms.reserve(expected_samples);
        std::size_t cache_misses = 0;
        for (int cycle = 0; cycle < kSwitchCycles; ++cycle) {
            for (const auto& document : loaded) {
                cache_misses +=
                    document_state(view_model, document.id) == QStringLiteral("unloaded");
                const auto start = Clock::now();
                view_model.activateDocument(document.id);
                const auto selection_returned = Clock::now();
                const auto ready = [&view_model, &document] {
                    return document_ready(view_model, document.id);
                };
                if (!wait_for_activation(view_model, ready, 60000)) {
                    return fail("document selection or cache reload timed out for " +
                                document.name.toStdString());
                }
                const auto completed = Clock::now();
                selection_call_samples_ms.push_back(
                    std::chrono::duration<double, std::milli>(selection_returned - start).count());
                selection_ready_samples_ms.push_back(
                    std::chrono::duration<double, std::milli>(completed - start).count());
            }
        }
        std::cout << "document_selection samples=" << selection_call_samples_ms.size()
                  << " cache_misses=" << cache_misses << " call_p50_ms=" << std::fixed
                  << std::setprecision(3) << percentile_ms(selection_call_samples_ms, 0.50)
                  << " call_p95_ms=" << percentile_ms(selection_call_samples_ms, 0.95)
                  << " ready_p50_ms=" << percentile_ms(selection_ready_samples_ms, 0.50)
                  << " ready_p95_ms=" << percentile_ms(selection_ready_samples_ms, 0.95)
                  << " scope=ViewModel_selection_call_and_selection_to_ready\n";
    }

    for (const auto& document : loaded) {
        std::vector<double> reload_samples_ms;
        reload_samples_ms.reserve(static_cast<std::size_t>(reload_iterations));
        for (int iteration = 0; iteration < reload_iterations; ++iteration) {
            view_model.activateDocument(document.id);
            const auto ready = [&view_model, &document]() {
                return document_ready(view_model, document.id);
            };
            if (!wait_for_activation(view_model, ready, 60000)) {
                return fail("document selection timed out before close for " +
                            document.name.toStdString());
            }
            std::weak_ptr<const SurfaceMeshSnapshot> prior_snapshot;
            {
                const auto snapshot = view_model.mesh_snapshot();
                if (!snapshot) {
                    return fail("cached snapshot disappeared before close");
                }
                prior_snapshot = snapshot;
            }
            view_model.closeDocument(document.id);
            if (!prior_snapshot.expired()) {
                return fail("closed document retained its mesh snapshot");
            }

            const auto start = Clock::now();
            view_model.openImportRecord(document.id);
            if (!wait_for_activation(view_model, ready, 60000)) {
                return fail("reload timed out or failed for " + document.name.toStdString());
            }
            reload_samples_ms.push_back(
                std::chrono::duration<double, std::milli>(Clock::now() - start).count());
            const int completed = iteration + 1;
            if (completed == 1 || completed == 10 || completed == 50 ||
                completed == reload_iterations) {
                const std::string phase = "reload_" + document.id.toStdString() + "_iteration_" +
                                          std::to_string(completed);
                print_memory(phase.c_str(), sample_memory(),
                             resident_mesh_payload_estimate(view_model, loaded));
            }
        }
        std::cout << "file_reload file=" << document.name.toStdString()
                  << " samples=" << reload_samples_ms.size() << " p50_ms=" << std::fixed
                  << std::setprecision(3) << percentile_ms(reload_samples_ms, 0.50)
                  << " p95_ms=" << percentile_ms(reload_samples_ms, 0.95)
                  << " cache=warm_os_page_cache\n";
    }

    const auto before_close_memory = sample_memory();
    print_memory("after_reloads_all_open", before_close_memory,
                 resident_mesh_payload_estimate(view_model, loaded));
    for (const auto& document : loaded) {
        view_model.closeDocument(document.id);
    }
    view_model.closeDocument(QStringLiteral("welcome"));
    QCoreApplication::processEvents(QEventLoop::AllEvents, 50);
    print_memory("all_documents_closed", sample_memory(), 0);
    return 0;
} catch (const std::exception& exception) {
    std::fprintf(stderr, "error: uncaught benchmark exception: %s\n", exception.what());
    return 1;
} catch (...) {
    std::fputs("error: unknown benchmark exception\n", stderr);
    return 1;
}
