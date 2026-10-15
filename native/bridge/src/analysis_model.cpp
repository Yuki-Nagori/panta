#include "analysis_model.hpp"

#include "panta/visualization/mesh_source.hpp"
#include "panta_ffi.h"
#include "project_view_model.hpp"
#include "rust/cxx.h"
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QObject>
#include <QString>
#include <QVariantMap>
#include <algorithm>
#include <cmath>
#include <cstddef>
#include <memory>
#include <qtmetamacros.h>
#include <string>
#include <string_view>
#include <utility>

namespace panta::bridge {
namespace {
QString qt_string(const rust::String& text) {
    return QString::fromUtf8(std::string_view(text.data(), text.size()));
}
QVariantList fillResultGroups(bool withPressure) {
    QVariantList results{QVariantMap{{QStringLiteral("id"), QStringLiteral("fill-time")},
                                     {QStringLiteral("sourceText"), QStringLiteral("Fill time")}}};
    if (withPressure)
        results.append(QVariantMap{{QStringLiteral("id"), QStringLiteral("pressure")},
                                   {QStringLiteral("sourceText"), QStringLiteral("Pressure")}});
    return QVariantList{QVariantMap{{QStringLiteral("sourceText"), QStringLiteral("Fill")},
                                    {QStringLiteral("results"), results}}};
}
} // namespace

AnalysisModel::AnalysisModel(QObject* parent)
    : MeshSource(parent), m_pollTimer(this), m_playTimer(this),
      m_service(panta::ffi::analysis_service_new()) {
    m_pollTimer.setInterval(100);
    connect(&m_pollTimer, &QTimer::timeout, this, &AnalysisModel::poll);
    m_playTimer.setInterval(50);
    connect(&m_playTimer, &QTimer::timeout, this, [this] {
        setPlaybackTime(m_playbackTime +
                        static_cast<double>(m_playClock.restart()) / 10000.0 * m_duration);
    });
}

void AnalysisModel::setProjectModel(ProjectViewModel* project) {
    if (m_project == project)
        return;
    if (m_project)
        disconnect(m_project, nullptr, this, nullptr);
    m_project = project;
    if (project) {
        connect(project, &ProjectViewModel::planSettingsChanged, this, &AnalysisModel::syncContext);
        connect(project, &ProjectViewModel::projectChanged, this, &AnalysisModel::syncContext);
        connect(project, &ProjectViewModel::documentsChanged, this,
                &AnalysisModel::documentsChanged);
        connect(project, &ProjectViewModel::activeDocumentChanged, this, [this] {
            m_activeStudy.clear();
            emit documentsChanged();
            emit meshChanged();
        });
        connect(project, &ProjectViewModel::meshChanged, this, &AnalysisModel::meshChanged);
    }
    syncContext();
    emit contextChanged();
}

void AnalysisModel::syncContext() {
    const auto plan = m_project ? m_project->planSettings() : QVariantMap{};
    const QString importId = plan.value(QStringLiteral("importId")).toString();
    const QString next = m_project ? m_project->currentPath() + ":" + importId : QString{};
    if (next == m_context)
        return;
    // 取消后继续拉取，让后台线程退出。服务本身保留各零件的内存缓存。
    if (m_busy) {
        panta::ffi::analysis_cancel(*m_service);
        m_awaitingWorker = true;
    }
    m_context = next;
    m_importId = importId;
    if (!m_awaitingWorker)
        m_pollTimer.stop();
    pause();
    m_busy = false;
    m_meshReady = m_gateReady = m_resultReady = false;
    m_activeStudy.clear();
    m_studies.clear();
    m_mesh.reset();
    m_error.clear();
    m_duration = m_playbackTime = 0;
    m_scope.clear();
    if (m_project && !importId.isEmpty()) {
        const auto output = m_project->analysisInput().value(QStringLiteral("output")).toString();
        if (!output.isEmpty()) {
            const auto cache = panta::ffi::analysis_bind(*m_service, output.toStdString());
            publishCached(cache.root, cache.has_mesh, cache.has_gate, cache.has_fill,
                          cache.coordinates, cache.fill_times, cache.pressures, cache.gate_points,
                          cache.fill_time);
            noteRestored(cache.peak_pressure);
        }
    }
    emit stateChanged();
    emit documentsChanged();
    emit meshChanged();
    emit playbackChanged();
}

std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
AnalysisModel::mesh_snapshot() const {
    if (m_mesh && m_project &&
        (m_project->activeDocumentId() == m_importId || !m_activeStudy.isEmpty()))
        return m_mesh;
    return m_project ? m_project->mesh_snapshot() : nullptr;
}
bool AnalysisModel::placeholder_visible() const {
    return m_project ? m_project->placeholder_visible() : true;
}

void AnalysisModel::generateMesh(double edgeLength) { start("mesh", edgeLength); }
void AnalysisModel::analyze() {
    if (m_busy) {
        cancel();
        return;
    }
    const QString sequence =
        m_project ? m_project->planSettings().value(QStringLiteral("sequenceId")).toString()
                  : QString{};
    if (sequence == "gate-location")
        start("gate", m_edgeLength);
    else if (sequence == "fill")
        start("fill", m_edgeLength);
    else {
        m_error = tr("Select Gate Location or Fill analysis sequence.");
        emit stateChanged();
    }
}
void AnalysisModel::start(const QString& operation, double edgeLength) {
    if (m_busy || !m_project)
        return;
    syncContext();
    const auto plan = m_project->planSettings();
    if (m_importId.isEmpty() || m_project->activeDocumentId() != m_importId) {
        m_error = tr("Open the imported part before starting analysis.");
        emit stateChanged();
        return;
    }
    const auto input = m_project->analysisInput();
    if (input.value(QStringLiteral("units")).toString() != "millimeters") {
        m_error = tr("The cover workflow requires millimetres.");
        emit stateChanged();
        return;
    }
    const auto settings = plan.value(operation == "gate" ? QStringLiteral("gateLocationSettings")
                                                         : QStringLiteral("fillSettings"))
                              .toMap();
    const auto gate = plan.value(QStringLiteral("gateLocationSettings")).toMap();
    const auto fill = plan.value(QStringLiteral("fillSettings")).toMap();
    const QJsonObject request{
        {"source", input.value(QStringLiteral("source")).toString()},
        {"output", input.value(QStringLiteral("output")).toString()},
        {"edgeLength", edgeLength},
        {"moldTemperature", settings.value(QStringLiteral("moldTemperature")).toDouble()},
        {"meltTemperature", settings.value(QStringLiteral("meltTemperature")).toDouble()},
        {"flowRate", fill.value(QStringLiteral("flowRate")).toDouble()},
        {"switchVolume", fill.value(QStringLiteral("switchVolume")).toDouble()},
        {"numberOfGates", gate.value(QStringLiteral("numberOfGates")).toInt()}};
    try {
        const auto output = input.value(QStringLiteral("output")).toString();
        if (!output.isEmpty()) {
            // 事件范围用 bind 的规范化目录。同步上下文时导入记录可能还不可用。
            const auto cache = panta::ffi::analysis_bind(*m_service, output.toStdString());
            if (qt_string(cache.root) != m_scope) {
                publishCached(cache.root, cache.has_mesh, cache.has_gate, cache.has_fill,
                              cache.coordinates, cache.fill_times, cache.pressures,
                              cache.gate_points, cache.fill_time);
                noteRestored(cache.peak_pressure);
            }
            m_scope = qt_string(cache.root);
        }
        const std::string json =
            QJsonDocument(request).toJson(QJsonDocument::Compact).toStdString();
        panta::ffi::analysis_start(*m_service, operation.toStdString(), json);
        m_awaitingWorker = false;
        pause();
        m_operation = operation;
        m_edgeLength = edgeLength;
        m_busy = true;
        m_error.clear();
        m_progress = -1;
        m_phase = operation == "mesh" ? tr("Loading geometry / MMG remeshing") : tr("Preparing");
        m_console.clear();
        m_summary =
            tr("Operation: %1\nSource: %2\nCase: cover_fast.case.yaml\nEdge length: %3 mm\nMold / "
               "melt temperature: %4 / %5 °C\nFlow rate: %6 cm³/s\nV/P switch: %7 %\n\n")
                .arg(operation,
                     QFileInfo(input.value(QStringLiteral("source")).toString()).fileName())
                .arg(edgeLength)
                .arg(settings.value(QStringLiteral("moldTemperature")).toDouble())
                .arg(settings.value(QStringLiteral("meltTemperature")).toDouble())
                .arg(fill.value(QStringLiteral("flowRate")).toDouble())
                .arg(fill.value(QStringLiteral("switchVolume")).toDouble());
        m_outputDir.clear();
        m_activeStudy.clear();
        m_clock.start();
        m_pollTimer.start();
        m_logRuns.append(
            QVariantMap{{QStringLiteral("id"), m_context + ":" + QString::number(m_logRuns.size())},
                        {QStringLiteral("contextId"), m_context},
                        {QStringLiteral("scope"), m_scope},
                        {QStringLiteral("categories"), QVariantList{}},
                        {QStringLiteral("resultGroups"), QVariantList{}}});
        updateRun(tr("Starting %1\n").arg(operation));
        emit stateChanged();
        emit runStarted();
    } catch (const rust::Error& error) {
        m_error = QString::fromUtf8(error.what());
    }
    emit stateChanged();
}
void AnalysisModel::cancel() {
    if (m_busy)
        panta::ffi::analysis_cancel(*m_service);
}
void AnalysisModel::updateRun(const QString& text) {
    const int index = currentRunIndex();
    if (index < 0)
        return;
    m_console += QString(text).replace("\r\n", "\n").replace("\r", "\n");
    auto run = m_logRuns[index].toMap();
    QString status;
    if (m_busy) {
        const QString stage =
            m_progress >= 0 ? tr("Filling · %1 %").arg(m_progress * 100, 0, 'f', 1) : m_phase;
        status = tr("Status: Running\nStage: %1\nElapsed: %2 s\n")
                     .arg(stage)
                     .arg(static_cast<double>(m_clock.elapsed()) / 1000, 0, 'f', 1);
    }
    run[QStringLiteral("categories")] = QVariantList{
        QVariantMap{
            {QStringLiteral("id"), m_operation == "mesh" ? "mesh" : "analysis"},
            {QStringLiteral("sourceText"), m_operation == "mesh" ? "Mesh Log" : "Analysis Log"},
            {QStringLiteral("text"), m_summary + status}},
        QVariantMap{{QStringLiteral("id"), "solver"},
                    {QStringLiteral("sourceText"), "Solver Log"},
                    {QStringLiteral("text"), m_console}}};
    m_logRuns[index] = run;
}

void AnalysisModel::publishCached(const rust::String& root, bool hasMesh, bool hasGate,
                                  bool hasFill, const rust::Vec<double>& coordinates,
                                  const rust::Vec<double>& fillTimes,
                                  const rust::Vec<double>& pressures,
                                  const rust::Vec<double>& gatePoints, double fillTime) {
    m_scope = qt_string(root);
    m_meshReady = hasMesh;
    m_gateReady = hasGate;
    m_resultReady = hasFill;
    m_duration = hasFill ? fillTime : 0;
    m_playbackTime = m_duration;
    m_activeStudy.clear();
    m_studies.clear();
    m_error.clear();
    if (!hasMesh && !hasGate && !hasFill) {
        m_mesh.reset();
        return;
    }
    auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
    for (std::size_t i = 0; i + 2 < coordinates.size(); i += 3)
        mesh->vertices.push_back({coordinates[i], coordinates[i + 1], coordinates[i + 2]});
    mesh->fill_times.assign(fillTimes.begin(), fillTimes.end());
    mesh->pressures.assign(pressures.begin(), pressures.end());
    for (std::size_t i = 0; i + 2 < gatePoints.size(); i += 3)
        mesh->gate_points.push_back({gatePoints[i], gatePoints[i + 1], gatePoints[i + 2]});
    mesh->z_up = true;
    mesh->fill_duration = m_duration;
    if (hasFill) {
        mesh->fields_visible = true;
        mesh->pressure_visible = false;
        mesh->show_gates = false;
        mesh->show_edges = false;
    } else if (hasGate) {
        mesh->fields_visible = false;
        mesh->show_gates = true;
        mesh->show_edges = false;
        mesh->fill_times.clear();
        mesh->pressures.clear();
        mesh->fill_duration = 0;
    } else {
        mesh->fields_visible = false;
        mesh->show_gates = false;
        mesh->show_edges = true;
        mesh->fill_times.clear();
        mesh->pressures.clear();
    }
    m_mesh = std::move(mesh);
    if (!m_project || m_importId.isEmpty())
        return;
    const QString title = m_project->activeDocumentTitle();
    const QString base = title.isEmpty() ? m_importId : title;
    const auto append = [this, &base](const QString& suffix, const QString& label) {
        m_studies.append(QVariantMap{{QStringLiteral("id"), m_importId + suffix},
                                     {QStringLiteral("title"), base + label},
                                     {QStringLiteral("parentId"), m_importId},
                                     {QStringLiteral("kind"), QStringLiteral("analysis")},
                                     {QStringLiteral("state"), QStringLiteral("ready")},
                                     {QStringLiteral("message"), QString{}}});
    };
    if (hasGate)
        append(QStringLiteral(":gate"), QStringLiteral(" (Gate Location)"));
    if (hasFill)
        append(QStringLiteral(":fill"), QStringLiteral(" (Fill)"));
}

void AnalysisModel::noteRestored(double peakPressure) {
    if (m_context.isEmpty())
        return;
    if (contextHasLog()) {
        refreshCachedResults();
        return;
    }
    if (!m_meshReady && !m_gateReady && !m_resultReady)
        return;
    m_summary = tr("Restored from cache\n");
    if (m_resultReady)
        m_summary += tr("Fill time: %1 s\nPeak pressure: %2 MPa\n")
                         .arg(m_duration, 0, 'f', 3)
                         .arg(peakPressure, 0, 'f', 3);
    m_console.clear();
    m_logRuns.append(
        QVariantMap{{QStringLiteral("id"), m_context + ":" + QString::number(m_logRuns.size())},
                    {QStringLiteral("contextId"), m_context},
                    {QStringLiteral("scope"), m_scope},
                    {QStringLiteral("categories"), QVariantList{}},
                    {QStringLiteral("resultGroups"), QVariantList{}}});
    updateRun(tr("Restored from cache\n"));
    if (m_resultReady)
        attachFillResults();
}

void AnalysisModel::attachFillResults() {
    const int index = currentRunIndex();
    if (index < 0)
        return;
    auto run = m_logRuns[index].toMap();
    run[QStringLiteral("resultGroups")] = fillResultGroups(m_mesh && !m_mesh->pressures.empty());
    m_logRuns[index] = run;
}

void AnalysisModel::refreshCachedResults() {
    const int index = currentRunIndex();
    if (index < 0)
        return;
    auto run = m_logRuns[index].toMap();
    run[QStringLiteral("resultGroups")] =
        m_resultReady ? fillResultGroups(m_mesh && !m_mesh->pressures.empty()) : QVariantList{};
    m_logRuns[index] = run;
}

void AnalysisModel::finishScopedRun(int index, const panta::ffi::AnalysisEvent& event) {
    if (index < 0 || index >= m_logRuns.size())
        return;
    const auto kind = event.kind;
    const QString outcome = kind == panta::ffi::AnalysisEventKind::Failed      ? tr("Failed")
                            : kind == panta::ffi::AnalysisEventKind::Cancelled ? tr("Stopped")
                                                                               : tr("Completed");
    QString tail = tr("Status: %1\n").arg(outcome);
    if (kind == panta::ffi::AnalysisEventKind::Restored) {
        tail += tr("Restored from cache\n");
        if (event.has_fill)
            tail += tr("Fill time: %1 s\nPeak pressure: %2 MPa\n")
                        .arg(event.fill_time, 0, 'f', 3)
                        .arg(event.peak_pressure, 0, 'f', 3);
    } else {
        if (kind == panta::ffi::AnalysisEventKind::MeshReady)
            tail += tr("Triangles: %1\n").arg(event.coordinates.size() / 9);
        if (kind == panta::ffi::AnalysisEventKind::Completed)
            tail += tr("Fill time: %1 s\nPeak pressure: %2 MPa\n")
                        .arg(event.fill_time, 0, 'f', 3)
                        .arg(event.peak_pressure, 0, 'f', 3);
        if (kind == panta::ffi::AnalysisEventKind::GateReady && event.gate_points.size() == 3)
            tail += tr("Selected gate (mm): [%1, %2, %3]\nCandidates: top 3; one injection gate "
                       "used\n")
                        .arg(event.gate_points[0], 0, 'f', 3)
                        .arg(event.gate_points[1], 0, 'f', 3)
                        .arg(event.gate_points[2], 0, 'f', 3);
        if (kind == panta::ffi::AnalysisEventKind::Failed)
            tail += qt_string(event.message) + QStringLiteral("\n");
        if (!event.output_dir.empty())
            tail += QStringLiteral("\nArtifacts:\n") + qt_string(event.output_dir);
    }
    auto run = m_logRuns[index].toMap();
    auto categories = run.value(QStringLiteral("categories")).toList();
    const QString status = tr("Status: Running");
    for (auto& category : categories) {
        auto map = category.toMap();
        const QString id = map.value(QStringLiteral("id")).toString();
        auto text = map.value(QStringLiteral("text")).toString();
        if (id == QLatin1String("mesh") || id == QLatin1String("analysis")) {
            const int at = text.lastIndexOf(status);
            if (at < 0)
                continue;
            text.truncate(at);
            text += tail;
        } else if (id == QLatin1String("solver")) {
            text += qt_string(event.message);
            if (!event.output_dir.empty())
                text += QStringLiteral("\nArtifacts: ") + qt_string(event.output_dir) +
                        QStringLiteral("\n");
            else if (!text.endsWith(QLatin1Char('\n')))
                text += QLatin1Char('\n');
        } else {
            continue;
        }
        map[QStringLiteral("text")] = text;
        category = map;
    }
    run[QStringLiteral("categories")] = categories;
    if (kind == panta::ffi::AnalysisEventKind::MeshReady ||
        kind == panta::ffi::AnalysisEventKind::GateReady)
        run[QStringLiteral("resultGroups")] = QVariantList{};
    else if (kind == panta::ffi::AnalysisEventKind::Completed ||
             (kind == panta::ffi::AnalysisEventKind::Restored && event.has_fill))
        run[QStringLiteral("resultGroups")] = fillResultGroups(!event.pressures.empty());
    m_logRuns[index] = run;
}

int AnalysisModel::currentRunIndex() const {
    for (int i = static_cast<int>(m_logRuns.size()) - 1; i >= 0; --i) {
        if (m_logRuns[i].toMap().value(QStringLiteral("contextId")).toString() == m_context)
            return i;
    }
    return -1;
}

int AnalysisModel::runIndexForScope(const QString& scope) const {
    if (scope.isEmpty())
        return -1;
    for (int i = static_cast<int>(m_logRuns.size()) - 1; i >= 0; --i) {
        if (m_logRuns[i].toMap().value(QStringLiteral("scope")).toString() == scope)
            return i;
    }
    return -1;
}

bool AnalysisModel::contextHasLog() const {
    return std::any_of(m_logRuns.cbegin(), m_logRuns.cend(), [this](const QVariant& run) {
        return run.toMap().value(QStringLiteral("contextId")).toString() == m_context;
    });
}

bool AnalysisModel::foreignScope(const rust::String& scope) const {
    return !scope.empty() && qt_string(scope) != m_scope;
}

void AnalysisModel::poll() {
    for (const auto& event : panta::ffi::analysis_drain(*m_service)) {
        const auto text = qt_string(event.message);
        const bool terminal = event.kind != panta::ffi::AnalysisEventKind::Progress &&
                              event.kind != panta::ffi::AnalysisEventKind::Log;
        if (foreignScope(event.scope)) {
            if (terminal) {
                finishScopedRun(runIndexForScope(qt_string(event.scope)), event);
                m_awaitingWorker = false;
                if (!m_busy)
                    m_pollTimer.stop();
            }
            continue;
        }
        if (event.kind == panta::ffi::AnalysisEventKind::Log) {
            updateRun(text);
            continue;
        }
        if (event.kind == panta::ffi::AnalysisEventKind::Restored) {
            publishCached(event.scope, event.has_mesh, event.has_gate, event.has_fill,
                          event.coordinates, event.fill_times, event.pressures, event.gate_points,
                          event.fill_time);
            if (m_operation == QLatin1String("gate") && event.has_gate && !m_importId.isEmpty()) {
                m_activeStudy = m_importId + QStringLiteral(":gate");
                selectResult(QStringLiteral("gate"));
            } else if (m_operation == QLatin1String("fill") && event.has_fill &&
                       !m_importId.isEmpty()) {
                m_activeStudy = m_importId + QStringLiteral(":fill");
                selectResult(QStringLiteral("fill-time"));
            }
            m_busy = false;
            m_awaitingWorker = false;
            m_pollTimer.stop();
            m_summary += tr("\nStatus: Completed\nRestored from cache\n");
            if (event.has_fill)
                m_summary += tr("Fill time: %1 s\nPeak pressure: %2 MPa\n")
                                 .arg(event.fill_time, 0, 'f', 3)
                                 .arg(event.peak_pressure, 0, 'f', 3);
            updateRun(tr("Restored from cache\n"));
            if (event.has_fill)
                attachFillResults();
            emit documentsChanged();
            emit meshChanged();
            emit playbackChanged();
            continue;
        }
        if (event.kind == panta::ffi::AnalysisEventKind::Progress)
            m_phase = text;
        if (!event.output_dir.empty())
            m_outputDir = qt_string(event.output_dir);
        if (event.progress >= 0)
            m_progress = event.progress;
        if (event.kind == panta::ffi::AnalysisEventKind::Progress)
            continue;
        const bool wasBusy = m_busy;
        m_busy = false;
        m_awaitingWorker = false;
        m_pollTimer.stop();
        if (wasBusy) {
            const QString outcome =
                event.kind == panta::ffi::AnalysisEventKind::Failed      ? tr("Failed")
                : event.kind == panta::ffi::AnalysisEventKind::Cancelled ? tr("Stopped")
                                                                         : tr("Completed");
            m_summary +=
                tr("Status: %1\nElapsed: %2 s\n").arg(outcome).arg(event.elapsed, 0, 'f', 1);
            if (event.kind == panta::ffi::AnalysisEventKind::MeshReady)
                m_summary += tr("Triangles: %1\n").arg(event.coordinates.size() / 9);
            if (event.kind == panta::ffi::AnalysisEventKind::Completed)
                m_summary += tr("Fill time: %1 s\nPeak pressure: %2 MPa\n")
                                 .arg(event.fill_time, 0, 'f', 3)
                                 .arg(event.peak_pressure, 0, 'f', 3);
            if (event.kind == panta::ffi::AnalysisEventKind::GateReady &&
                event.gate_points.size() == 3)
                m_summary +=
                    tr("Selected gate (mm): [%1, %2, %3]\nCandidates: top 3; one injection "
                       "gate used\n")
                        .arg(event.gate_points[0], 0, 'f', 3)
                        .arg(event.gate_points[1], 0, 'f', 3)
                        .arg(event.gate_points[2], 0, 'f', 3);
            if (event.kind == panta::ffi::AnalysisEventKind::Failed)
                m_summary += text + "\n";
            m_summary += "\nArtifacts:\n" + m_outputDir;
            updateRun(text + "\nArtifacts: " + m_outputDir + "\n");
        } else {
            // 离开该零件后终态才到。日志已写在运行记录里，不能再用当前摘要覆盖。
            finishScopedRun(currentRunIndex(), event);
        }
        if (event.kind == panta::ffi::AnalysisEventKind::Failed)
            m_error = text;
        else if (event.kind != panta::ffi::AnalysisEventKind::Cancelled) {
            if (!event.coordinates.empty()) {
                auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
                for (std::size_t i = 0; i + 2 < event.coordinates.size(); i += 3)
                    mesh->vertices.push_back(
                        {event.coordinates[i], event.coordinates[i + 1], event.coordinates[i + 2]});
                mesh->fill_times.assign(event.fill_times.begin(), event.fill_times.end());
                mesh->pressures.assign(event.pressures.begin(), event.pressures.end());
                mesh->z_up = true;
                mesh->show_edges = event.kind == panta::ffi::AnalysisEventKind::MeshReady;
                mesh->fill_duration = event.fill_time;
                if (m_mesh)
                    mesh->gate_points = m_mesh->gate_points;
                m_mesh = std::move(mesh);
            }
            if (event.kind == panta::ffi::AnalysisEventKind::MeshReady) {
                m_meshReady = true;
                m_gateReady = m_resultReady = false;
                m_studies.clear();
                for (auto& previous : m_logRuns) {
                    auto value = previous.toMap();
                    if (value.value(QStringLiteral("contextId")).toString() == m_context) {
                        value[QStringLiteral("resultGroups")] = QVariantList{};
                        previous = value;
                    }
                }
            } else {
                for (auto& previous : m_logRuns) {
                    auto value = previous.toMap();
                    if (value.value(QStringLiteral("contextId")).toString() == m_context) {
                        value[QStringLiteral("resultGroups")] = QVariantList{};
                        previous = value;
                    }
                }
                const bool gate = event.kind == panta::ffi::AnalysisEventKind::GateReady;
                if (gate) {
                    m_gateReady = true;
                    m_resultReady = false;
                    m_studies.clear();
                    if (m_mesh) {
                        auto mesh =
                            std::make_shared<panta::visualization::SurfaceMeshSnapshot>(*m_mesh);
                        mesh->gate_points.clear();
                        for (std::size_t i = 0; i + 2 < event.gate_points.size(); i += 3)
                            mesh->gate_points.push_back({event.gate_points[i],
                                                         event.gate_points[i + 1],
                                                         event.gate_points[i + 2]});
                        mesh->fields_visible = false;
                        mesh->show_gates = true;
                        mesh->fill_times.clear();
                        mesh->pressures.clear();
                        mesh->fill_duration = 0;
                        m_mesh = std::move(mesh);
                    }
                } else {
                    m_resultReady = true;
                    m_duration = event.fill_time;
                    if (wasBusy)
                        updateRun(tr("Fill time: %1 s; peak pressure: %2 MPa\n")
                                      .arg(event.fill_time, 0, 'f', 3)
                                      .arg(event.peak_pressure, 0, 'f', 3));
                    m_playbackTime = m_duration;
                    attachFillResults();
                    emit playbackChanged();
                }
                const QString id = m_importId + (gate ? ":gate" : ":fill");
                const QString title =
                    m_project->activeDocumentTitle() + (gate ? " (Gate Location)" : " (Fill)");
                const QVariantMap study{{QStringLiteral("id"), id},
                                        {QStringLiteral("title"), title},
                                        {QStringLiteral("parentId"), m_importId},
                                        {QStringLiteral("kind"), "analysis"},
                                        {QStringLiteral("state"), "ready"},
                                        {QStringLiteral("message"), ""}};
                bool found = false;
                for (auto& entry : m_studies) {
                    if (entry.toMap().value(QStringLiteral("id")).toString() == id) {
                        entry = study;
                        found = true;
                        break;
                    }
                }
                if (!found)
                    m_studies.append(study);
                m_activeStudy = id;
            }
            emit documentsChanged();
            emit meshChanged();
        }
    }
    if (m_busy)
        updateRun({});
    emit stateChanged();
}

QVariantList AnalysisModel::openDocuments() const {
    auto docs = m_project ? m_project->openDocuments() : QVariantList{};
    docs.append(m_studies);
    return docs;
}
QString AnalysisModel::activeDocumentId() const {
    return m_activeStudy.isEmpty() && m_project ? m_project->activeDocumentId() : m_activeStudy;
}
QString AnalysisModel::activeDocumentTitle() const {
    for (const auto& study : m_studies) {
        const auto value = study.toMap();
        if (value.value(QStringLiteral("id")).toString() == m_activeStudy)
            return value.value(QStringLiteral("title")).toString();
    }
    return m_project ? m_project->activeDocumentTitle() : QString{};
}
void AnalysisModel::activateDocument(const QString& id) {
    if (id == m_activeStudy && !id.isEmpty())
        return;
    for (const auto& study : m_studies) {
        if (study.toMap().value(QStringLiteral("id")).toString() == id) {
            m_activeStudy = id;
            if (id.endsWith(":gate"))
                selectResult("gate");
            else
                selectResult("fill-time");
            emit documentsChanged();
            return;
        }
    }
    m_activeStudy.clear();
    selectResult("mesh");
    if (m_project)
        m_project->activateDocument(id);
    emit documentsChanged();
    emit meshChanged();
}
void AnalysisModel::closeDocument(const QString& id) {
    for (qsizetype i = 0; i < m_studies.size(); ++i) {
        if (m_studies[i].toMap().value(QStringLiteral("id")).toString() == id) {
            m_studies.removeAt(i);
            m_activeStudy.clear();
            emit documentsChanged();
            emit meshChanged();
            return;
        }
    }
    if (m_project)
        m_project->closeDocument(id);
}
void AnalysisModel::moveDocument(int fromIndex, int toIndex) {
    const int size = m_project ? static_cast<int>(m_project->openDocuments().size()) : 0;
    if (fromIndex < size && toIndex < size && m_project)
        m_project->moveDocument(fromIndex, toIndex);
}
void AnalysisModel::openImportRecord(const QString& id) {
    m_activeStudy.clear();
    selectResult("mesh");
    if (m_project)
        m_project->openImportRecord(id);
    emit documentsChanged();
    emit meshChanged();
}
void AnalysisModel::selectResult(const QString& id) {
    if (!m_mesh)
        return;
    pause();
    auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>(*m_mesh);
    mesh->pressure_visible = id == "pressure" && !mesh->pressures.empty();
    mesh->fields_visible = (id == "fill-time" || id == "pressure") && m_resultReady;
    mesh->show_gates = id == "gate";
    const bool documentChanged = mesh->fields_visible && m_activeStudy != m_importId + ":fill";
    if (mesh->fields_visible)
        m_activeStudy = m_importId + ":fill";
    m_mesh = std::move(mesh);
    setPlaybackTime(m_duration);
    emit meshChanged();
    if (documentChanged)
        emit documentsChanged();
    emit stateChanged();
}
void AnalysisModel::play() {
    if (!m_resultReady || m_busy || !m_mesh || m_mesh->pressure_visible || !m_mesh->fields_visible)
        return;
    if (m_playbackTime >= m_duration)
        m_playbackTime = 0;
    m_playClock.start();
    m_playTimer.start();
    emit playbackChanged();
}
void AnalysisModel::pause() {
    m_playTimer.stop();
    emit playbackChanged();
}
void AnalysisModel::stop() {
    pause();
    setPlaybackTime(0);
}
void AnalysisModel::setPlaybackTime(double seconds) {
    if (!std::isfinite(seconds))
        return;
    m_playbackTime = std::clamp(seconds, 0.0, m_duration);
    if (m_playbackTime >= m_duration)
        m_playTimer.stop();
    emit playbackChanged();
}
} // namespace panta::bridge
