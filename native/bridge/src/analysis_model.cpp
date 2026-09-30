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
    // 切换零件或工程后回收旧作业，迟到结果不能发布到新的上下文。
    panta::ffi::analysis_cancel(*m_service);
    m_service = panta::ffi::analysis_service_new();
    m_context = next;
    m_importId = importId;
    m_pollTimer.stop();
    pause();
    m_busy = false;
    m_meshReady = m_gateReady = m_resultReady = false;
    m_activeStudy.clear();
    m_studies.clear();
    m_mesh.reset();
    m_error.clear();
    m_duration = m_playbackTime = 0;
    emit stateChanged();
    emit documentsChanged();
    emit meshChanged();
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
        const std::string json =
            QJsonDocument(request).toJson(QJsonDocument::Compact).toStdString();
        panta::ffi::analysis_start(*m_service, operation.toStdString(), json);
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
    if (m_logRuns.isEmpty())
        return;
    m_console += QString(text).replace("\r\n", "\n").replace("\r", "\n");
    auto run = m_logRuns.last().toMap();
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
    m_logRuns.last() = run;
}

void AnalysisModel::poll() {
    for (const auto& event : panta::ffi::analysis_drain(*m_service)) {
        const auto text = qt_string(event.message);
        if (event.kind == panta::ffi::AnalysisEventKind::Log) {
            updateRun(text);
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
        m_busy = false;
        m_pollTimer.stop();
        const QString outcome = event.kind == panta::ffi::AnalysisEventKind::Failed ? tr("Failed")
                                : event.kind == panta::ffi::AnalysisEventKind::Cancelled
                                    ? tr("Stopped")
                                    : tr("Completed");
        m_summary += tr("Status: %1\nElapsed: %2 s\n").arg(outcome).arg(event.elapsed, 0, 'f', 1);
        if (event.kind == panta::ffi::AnalysisEventKind::MeshReady)
            m_summary += tr("Triangles: %1\n").arg(event.coordinates.size() / 9);
        if (event.kind == panta::ffi::AnalysisEventKind::Completed)
            m_summary += tr("Fill time: %1 s\nPeak pressure: %2 MPa\n")
                             .arg(event.fill_time, 0, 'f', 3)
                             .arg(event.peak_pressure, 0, 'f', 3);
        if (event.kind == panta::ffi::AnalysisEventKind::GateReady && event.gate_points.size() == 3)
            m_summary +=
                tr("Selected gate (mm): [%1, %2, %3]\nCandidates: top 3; one injection gate used\n")
                    .arg(event.gate_points[0], 0, 'f', 3)
                    .arg(event.gate_points[1], 0, 'f', 3)
                    .arg(event.gate_points[2], 0, 'f', 3);
        if (event.kind == panta::ffi::AnalysisEventKind::Failed)
            m_summary += text + "\n";
        m_summary += "\nArtifacts:\n" + m_outputDir;
        updateRun(text + "\nArtifacts: " + m_outputDir + "\n");
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
                    updateRun(tr("Fill time: %1 s; peak pressure: %2 MPa\n")
                                  .arg(event.fill_time, 0, 'f', 3)
                                  .arg(event.peak_pressure, 0, 'f', 3));
                    m_playbackTime = m_duration;
                    auto run = m_logRuns.last().toMap();
                    QVariantList results{QVariantMap{{QStringLiteral("id"), "fill-time"},
                                                     {QStringLiteral("sourceText"), "Fill time"}}};
                    if (m_mesh && !m_mesh->pressures.empty())
                        results.append(QVariantMap{{QStringLiteral("id"), "pressure"},
                                                   {QStringLiteral("sourceText"), "Pressure"}});
                    run[QStringLiteral("resultGroups")] =
                        QVariantList{QVariantMap{{QStringLiteral("sourceText"), "Fill"},
                                                 {QStringLiteral("results"), results}}};
                    m_logRuns.last() = run;
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
