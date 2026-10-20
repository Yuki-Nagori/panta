#include "project_view_model.hpp"

#include "panta/visualization/mesh_source.hpp"
#include "panta_ffi.h"
#include "path_host.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QFuture>
#include <QFutureWatcher>
#include <QLatin1Char>
#include <QMetaType>
#include <QObject>
#include <QPromise>
#include <QString>
#include <QUrl>
#include <QVariantMap>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qnamespace.h>
#include <QtCore/qtmetamacros.h>
#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <functional>
#include <future>
#include <limits>
#include <memory>
#include <optional>
#include <panta/qt_adapter/standard_paths.hpp>
#include <qlogging.h>
#include <rust/cxx.h>
#include <string>
#include <system_error>
#include <thread>
#include <utility>

namespace panta::bridge {

namespace {

QString default_project_location() {
    const QString documents =
        panta::qt_adapter::standard_location(panta::qt_adapter::StandardLocation::Documents);
    return documents.isEmpty() ? QString{} : QDir(documents).filePath(QStringLiteral("panta"));
}

QVariantMap numeric_constraint_map(const panta::ffi::NumericConstraint& constraint) {
    return {{QStringLiteral("minimum"), constraint.minimum},
            {QStringLiteral("maximum"), constraint.maximum},
            {QStringLiteral("hasMinimum"), constraint.has_minimum},
            {QStringLiteral("hasMaximum"), constraint.has_maximum},
            {QStringLiteral("minimumInclusive"), constraint.minimum_inclusive},
            {QStringLiteral("maximumInclusive"), constraint.maximum_inclusive},
            {QStringLiteral("integer"), constraint.integer},
            {QStringLiteral("unit"), QString::fromUtf8(constraint.unit)}};
}

QString translated_project_error(const QString& code) {
    const QString message_key =
        QString::fromUtf8(panta::ffi::project_error_message_key(code.toStdString()));
    return QCoreApplication::translate("ProjectViewModel", message_key.toUtf8().constData());
}

QVariantMap draft_validation_map(const panta::ffi::ProcessDraftValidation& result) {
    QStringList errors;
    errors.reserve(static_cast<qsizetype>(result.errors.size()));
    for (const auto& error : result.errors) {
        errors.push_back(QString::fromUtf8(error));
    }
    return {{QStringLiteral("valid"), result.valid}, {QStringLiteral("errors"), errors}};
}

double draft_number(const QVariantMap& values, const QString& key) {
    bool ok = false;
    const double value = values.value(key).toDouble(&ok);
    return ok ? value : std::numeric_limits<double>::quiet_NaN();
}

QVariantList choice_catalog(const rust::Vec<panta::ffi::ChoiceDefinition>& catalog) {
    QVariantList result;
    for (const auto& entry : catalog) {
        result.append(
            QVariantMap{{QStringLiteral("id"), QString::fromUtf8(entry.id)},
                        {QStringLiteral("sourceText"), QString::fromUtf8(entry.source_text)}});
    }
    return result;
}

QVariantMap material_definition(const panta::ffi::MaterialDefinition& material) {
    QVariantList properties;
    for (const auto& property : material.properties) {
        properties.append(
            QVariantMap{{QStringLiteral("sourceText"), QString::fromUtf8(property.source_text)},
                        {QStringLiteral("value"), QString::fromUtf8(property.value)}});
    }
    return {{QStringLiteral("familySourceText"), QString::fromUtf8(material.family_source_text)},
            {QStringLiteral("id"), QString::fromUtf8(material.id)},
            {QStringLiteral("sourceText"), QString::fromUtf8(material.source_text)},
            {QStringLiteral("properties"), properties}};
}

QVariantList variable_values(const rust::Vec<panta::ffi::VariableValue>& values) {
    QVariantList result;
    result.reserve(static_cast<qsizetype>(values.size()));
    for (const auto& value : values) {
        result.append(QVariantMap{{QStringLiteral("name"), QString::fromUtf8(value.name)},
                                  {QStringLiteral("type"), QString::fromUtf8(value.value_type)},
                                  {QStringLiteral("value"), QString::fromUtf8(value.value)}});
    }
    return result;
}

QVariantList variable_diagnostics(const rust::Vec<panta::ffi::VariableDiagnostic>& diagnostics) {
    QVariantList result;
    result.reserve(static_cast<qsizetype>(diagnostics.size()));
    for (const auto& diagnostic : diagnostics) {
        result.append(QVariantMap{
            {QStringLiteral("code"), QString::fromUtf8(diagnostic.code)},
            {QStringLiteral("message"), QString::fromUtf8(diagnostic.message)},
            {QStringLiteral("startOffset"),
             QVariant::fromValue<qulonglong>(diagnostic.start_offset)},
            {QStringLiteral("endOffset"), QVariant::fromValue<qulonglong>(diagnostic.end_offset)},
            {QStringLiteral("line"), QVariant::fromValue<qulonglong>(diagnostic.line)},
            {QStringLiteral("column"), QVariant::fromValue<qulonglong>(diagnostic.column)}});
    }
    return result;
}

} // namespace

ProjectViewModel::ProjectViewModel(QObject* parent)
    : MeshSource(parent), m_defaultLocation(default_project_location()),
      m_analysisSequences(choice_catalog(panta::ffi::analysis_sequence_catalog())),
      m_meshTypes(choice_catalog(panta::ffi::mesh_type_catalog())),
      m_defaultMeshType(QString::fromUtf8(panta::ffi::default_mesh_type())),
      m_service(panta::ffi::project_service_new()) {
    const auto material = panta::ffi::default_material();
    if (material.error.code.empty()) {
        m_defaultMaterial = material_definition(material.value);
    } else {
        fail(material.error);
    }
    connect(this, &ProjectViewModel::activeDocumentChanged, this,
            &ProjectViewModel::refreshPlanSettings);
    connect(this, &ProjectViewModel::projectChanged, this, &ProjectViewModel::refreshPlanSettings);

    // 激活结果为拉取式队列（073）；10ms 轮询与 TaskHost 节奏一致，仅在
    // 存在 Loading 文档时运转。
    m_activationPoll.setInterval(10);
    connect(&m_activationPoll, &QTimer::timeout, this, &ProjectViewModel::drain_activations);
    m_metadataConfirmationPoll.setInterval(10);
    connect(&m_metadataConfirmationPoll, &QTimer::timeout, this,
            &ProjectViewModel::finish_metadata_confirmation);
    m_importPreviewPoll.setInterval(10);
    connect(&m_importPreviewPoll, &QTimer::timeout, this, &ProjectViewModel::finish_import_preview);
    m_stlImportPoll.setInterval(10);
    connect(&m_stlImportPoll, &QTimer::timeout, this, &ProjectViewModel::finish_stl_import);
    reset_documents();
}

std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
ProjectViewModel::mesh_snapshot() const {
    // 绘制刷新路径只读取这个本地共享快照，不能在这里跨 CXX 调 Rust。
    // Welcome / 空白 / 加载中返回空，由 placeholder_visible 区分欢迎字样与空白。
    return m_activeMesh;
}

bool ProjectViewModel::placeholder_visible() const {
    // 占位字样由活动 Welcome 文档决定；全部关闭后视口留白。
    for (const auto& item : m_openDocuments) {
        const auto document = item.toMap();
        if (document.value(QStringLiteral("id")).toString() == m_activeDocumentId) {
            return document.value(QStringLiteral("kind")).toString() == QStringLiteral("welcome");
        }
    }
    return false;
}

std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
ProjectViewModel::pull_service_mesh(const QString& importId) const {
    const auto result =
        panta::ffi::project_service_mesh_snapshot_for_import(*m_service, importId.toStdString());
    if (!result.error.code.empty()) {
        qWarning("ProjectViewModel mesh snapshot failed: %s: %s",
                 QString::fromUtf8(result.error.code).toStdString().c_str(),
                 QString::fromUtf8(result.error.detail).toStdString().c_str());
        return nullptr;
    }
    const auto& snapshot = result.value;
    if (snapshot.coordinates.empty() || snapshot.coordinates.size() % 9 != 0) {
        return nullptr;
    }
    auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
    mesh->project_revision = snapshot.revision;
    mesh->vertices.reserve(snapshot.coordinates.size() / 3);
    for (std::size_t offset = 0; offset < snapshot.coordinates.size(); offset += 3) {
        mesh->vertices.push_back({snapshot.coordinates[offset], snapshot.coordinates[offset + 1],
                                  snapshot.coordinates[offset + 2]});
    }
    return mesh;
}

const QString& ProjectViewModel::defaultLocation() const { return m_defaultLocation; }

QUrl ProjectViewModel::defaultLocationUrl() const {
    return m_defaultLocation.isEmpty() ? QUrl{} : QUrl::fromLocalFile(m_defaultLocation);
}

const QString& ProjectViewModel::lastCreatedPath() const { return m_lastCreatedPath; }

const QString& ProjectViewModel::error() const { return m_error; }

const QString& ProjectViewModel::errorCode() const { return m_errorCode; }

const QString& ProjectViewModel::errorCategory() const { return m_errorCategory; }

const QString& ProjectViewModel::errorDetail() const { return m_errorDetail; }

const QString& ProjectViewModel::currentPath() const { return m_currentPath; }

const QString& ProjectViewModel::currentName() const { return m_currentName; }

bool ProjectViewModel::dirty() const { return m_dirty; }

const QStringList& ProjectViewModel::importedPartNames() const { return m_importedPartNames; }

const QString& ProjectViewModel::importedPartName() const { return m_importedPartName; }

const QString& ProjectViewModel::importedAssetPath() const { return m_importedAssetPath; }

QVariantList ProjectViewModel::meshTypes() const { return m_meshTypes; }

QString ProjectViewModel::defaultMeshType() const { return m_defaultMeshType; }

QVariantMap ProjectViewModel::defaultMaterial() const { return m_defaultMaterial; }

bool ProjectViewModel::fillSettingsConfirmationPending() const {
    return m_pendingConfirmation == ConfirmationKind::FillSettings;
}

bool ProjectViewModel::gateLocationSettingsConfirmationPending() const {
    return m_pendingConfirmation == ConfirmationKind::GateLocationSettings;
}

bool ProjectViewModel::materialConfirmationPending() const {
    return m_pendingConfirmation == ConfirmationKind::Material;
}

QVariantList ProjectViewModel::analysisSequences() const { return m_analysisSequences; }

QVariantMap ProjectViewModel::planSettings() const { return m_planSettings; }

void ProjectViewModel::refreshPlanSettings() {
    const auto settings =
        panta::ffi::project_service_plan_settings(*m_service, m_activeDocumentId.toStdString());
    QVariantList profile;
    for (const auto& point : settings.fill_settings.holding_profile) {
        profile.append(QVariantMap{{QStringLiteral("duration"), point.duration_seconds},
                                   {QStringLiteral("pressure"), point.pressure_percent}});
    }
    const auto& fill = settings.fill_settings;
    const QVariantMap fillSettings{
        {QStringLiteral("moldTemperature"), fill.mold_temperature_celsius},
        {QStringLiteral("meltTemperature"), fill.melt_temperature_celsius},
        {QStringLiteral("flowRate"), fill.flow_rate_cm3_per_second},
        {QStringLiteral("switchVolume"), fill.switch_over_volume_percent},
        {QStringLiteral("fiberOrientation"), fill.fiber_orientation},
        {QStringLiteral("crystallization"), fill.crystallization},
        {QStringLiteral("holdingProfile"), profile}};
    const auto& gate = settings.gate_location_settings;
    const QVariantMap gateSettings{
        {QStringLiteral("machineId"), QString::fromUtf8(gate.machine_id)},
        {QStringLiteral("machineSourceText"), QString::fromUtf8(gate.machine_source_text)},
        {QStringLiteral("algorithmId"), QString::fromUtf8(gate.algorithm_id)},
        {QStringLiteral("algorithmSourceText"), QString::fromUtf8(gate.algorithm_source_text)},
        {QStringLiteral("moldTemperature"), gate.mold_temperature_celsius},
        {QStringLiteral("meltTemperature"), gate.melt_temperature_celsius},
        {QStringLiteral("numberOfGates"), gate.number_of_gates}};
    const auto& constraints = settings.process_constraints;
    const QVariantMap processConstraints{
        {QStringLiteral("temperature"), numeric_constraint_map(constraints.temperature)},
        {QStringLiteral("flowRate"), numeric_constraint_map(constraints.flow_rate)},
        {QStringLiteral("switchOverVolume"),
         numeric_constraint_map(constraints.switch_over_volume)},
        {QStringLiteral("holdingDuration"), numeric_constraint_map(constraints.holding_duration)},
        {QStringLiteral("holdingPressure"), numeric_constraint_map(constraints.holding_pressure)},
        {QStringLiteral("numberOfGates"), numeric_constraint_map(constraints.number_of_gates)},
        {QStringLiteral("holdingProfileMaxPoints"),
         QVariant::fromValue(constraints.holding_profile_max_points)}};
    const QVariantMap next{
        {QStringLiteral("projectPath"), QString::fromUtf8(settings.project_path)},
        {QStringLiteral("revision"), QVariant::fromValue(settings.revision)},
        {QStringLiteral("importId"), QString::fromUtf8(settings.import_id)},
        {QStringLiteral("meshType"), QString::fromUtf8(settings.mesh_type)},
        {QStringLiteral("sequenceId"), QString::fromUtf8(settings.sequence_id)},
        {QStringLiteral("sequenceSourceText"), QString::fromUtf8(settings.sequence_source_text)},
        {QStringLiteral("fillSettings"), fillSettings},
        {QStringLiteral("fillSettingsConfirmed"), settings.fill_settings_confirmed},
        {QStringLiteral("gateLocationSettings"), gateSettings},
        {QStringLiteral("gateLocationSettingsConfirmed"),
         settings.gate_location_settings_confirmed},
        {QStringLiteral("processConstraints"), processConstraints},
        {QStringLiteral("materialId"), QString::fromUtf8(settings.material_id)},
        {QStringLiteral("materialSourceText"), QString::fromUtf8(settings.material_source_text)}};
    if (next != m_planSettings) {
        m_planSettings = next;
        emit planSettingsChanged();
    }
}

bool ProjectViewModel::setAnalysisSequence(const QString& projectPath, quint64 revision,
                                           const QString& importId, const QString& sequenceId) {
    std::string path;
    std::string id;
    std::string sequence;
    QString conversionError;
    if (!toBoundaryText(projectPath, &path, &conversionError) ||
        !toBoundaryText(importId, &id, &conversionError) ||
        !toBoundaryText(sequenceId, &sequence, &conversionError)) {
        return fail(conversionError);
    }
    if (!applySnapshot(panta::ffi::project_service_set_analysis_sequence(*m_service, path, revision,
                                                                         id, sequence))) {
        return false;
    }
    refreshPlanSettings();
    clearError();
    return true;
}

bool ProjectViewModel::setMaterial(const QString& projectPath, quint64 revision,
                                   const QString& importId, const QString& materialId) {
    // 发布完成快照会同步发出 Qt 通知；此时也不能重入启动下一次确认。
    if (m_pendingConfirmation != ConfirmationKind::None) {
        return false;
    }
    std::string path;
    std::string id;
    std::string material;
    QString conversionError;
    if (!toBoundaryText(projectPath, &path, &conversionError) ||
        !toBoundaryText(importId, &id, &conversionError) ||
        !toBoundaryText(materialId, &material, &conversionError)) {
        return fail(conversionError);
    }
    const auto result = panta::ffi::project_service_begin_material_confirmation(
        *m_service, path, revision, id, material);
    if (!result.error.code.empty()) {
        return fail(result.error);
    }
    const bool pending = result.value;
    clearError();
    if (pending) {
        set_pending_confirmation(ConfirmationKind::Material);
    } else {
        // 相同材料无需写盘，仍按完成事件结束弹窗。
        refreshPlanSettings();
        emit materialConfirmationFinished(true);
    }
    return true;
}

bool ProjectViewModel::setFillSettings(const QString& projectPath, quint64 revision,
                                       const QString& importId, const QVariantMap& settings) {
    if (m_pendingConfirmation != ConfirmationKind::None) {
        return false;
    }
    std::string path;
    std::string id;
    QString conversionError;
    if (!toBoundaryText(projectPath, &path, &conversionError) ||
        !toBoundaryText(importId, &id, &conversionError)) {
        return fail(conversionError);
    }
    // 只适配 Qt 值类型；范围与工程有效性由 Rust 校验。
    bool converted =
        settings.value(QStringLiteral("fiberOrientation")).metaType().id() == QMetaType::Bool &&
        settings.value(QStringLiteral("crystallization")).metaType().id() == QMetaType::Bool;
    const auto number = [&](const QString& key) {
        bool ok = false;
        const double value = settings.value(key).toDouble(&ok);
        converted = converted && ok;
        return value;
    };
    panta::ffi::FillSettings candidate;
    candidate.mold_temperature_celsius = number(QStringLiteral("moldTemperature"));
    candidate.melt_temperature_celsius = number(QStringLiteral("meltTemperature"));
    candidate.flow_rate_cm3_per_second = number(QStringLiteral("flowRate"));
    candidate.switch_over_volume_percent = number(QStringLiteral("switchVolume"));
    candidate.fiber_orientation = settings.value(QStringLiteral("fiberOrientation")).toBool();
    candidate.crystallization = settings.value(QStringLiteral("crystallization")).toBool();
    for (const auto& value : settings.value(QStringLiteral("holdingProfile")).toList()) {
        const auto point = value.toMap();
        bool durationOk = false;
        bool pressureOk = false;
        const double duration = point.value(QStringLiteral("duration")).toDouble(&durationOk);
        const double pressure = point.value(QStringLiteral("pressure")).toDouble(&pressureOk);
        converted = converted && durationOk && pressureOk;
        candidate.holding_profile.push_back({duration, pressure});
    }
    if (!converted) {
        return fail(QStringLiteral("project.command_invalid"), QStringLiteral("validation"),
                    QStringLiteral("invalid process settings value"));
    }
    const auto result = panta::ffi::project_service_begin_fill_settings_confirmation(
        *m_service, path, revision, id, std::move(candidate));
    if (!result.error.code.empty()) {
        return fail(result.error);
    }
    const bool pending = result.value;
    clearError();
    if (pending) {
        set_pending_confirmation(ConfirmationKind::FillSettings);
    } else {
        refreshPlanSettings();
        emit fillSettingsConfirmationFinished(true);
    }
    return true;
}

QVariantMap ProjectViewModel::validateFillSettingsDraft(const QVariantMap& settings) const {
    panta::ffi::FillSettings candidate;
    candidate.mold_temperature_celsius = draft_number(settings, QStringLiteral("moldTemperature"));
    candidate.melt_temperature_celsius = draft_number(settings, QStringLiteral("meltTemperature"));
    candidate.flow_rate_cm3_per_second = draft_number(settings, QStringLiteral("flowRate"));
    candidate.switch_over_volume_percent = draft_number(settings, QStringLiteral("switchVolume"));
    candidate.fiber_orientation = settings.value(QStringLiteral("fiberOrientation")).toBool();
    candidate.crystallization = settings.value(QStringLiteral("crystallization")).toBool();
    for (const auto& value : settings.value(QStringLiteral("holdingProfile")).toList()) {
        const auto point = value.toMap();
        candidate.holding_profile.push_back({draft_number(point, QStringLiteral("duration")),
                                             draft_number(point, QStringLiteral("pressure"))});
    }
    return draft_validation_map(panta::ffi::validate_fill_settings_draft(std::move(candidate)));
}

QVariantMap ProjectViewModel::validateHoldingProfileDraft(const QVariantList& points) const {
    rust::Vec<panta::ffi::HoldingProfilePoint> profile;
    for (const auto& value : points) {
        const auto point = value.toMap();
        profile.push_back({draft_number(point, QStringLiteral("duration")),
                           draft_number(point, QStringLiteral("pressure"))});
    }
    return draft_validation_map(panta::ffi::validate_holding_profile_draft(std::move(profile)));
}

bool ProjectViewModel::setGateLocationSettings(const QString& projectPath, quint64 revision,
                                               const QString& importId,
                                               const QVariantMap& settings) {
    if (m_pendingConfirmation != ConfirmationKind::None)
        return false;
    std::string path;
    std::string id;
    std::string machine;
    std::string algorithm;
    QString conversionError;
    if (!toBoundaryText(projectPath, &path, &conversionError) ||
        !toBoundaryText(importId, &id, &conversionError) ||
        !toBoundaryText(settings.value(QStringLiteral("machineId")).toString(), &machine,
                        &conversionError) ||
        !toBoundaryText(settings.value(QStringLiteral("algorithmId")).toString(), &algorithm,
                        &conversionError))
        return fail(conversionError);
    bool moldOk = false;
    bool meltOk = false;
    bool gatesOk = false;
    const double mold = settings.value(QStringLiteral("moldTemperature")).toDouble(&moldOk);
    const double melt = settings.value(QStringLiteral("meltTemperature")).toDouble(&meltOk);
    const double gates = settings.value(QStringLiteral("numberOfGates")).toDouble(&gatesOk);
    // 拒绝 DTO 整数转换中的截断；业务范围由 Rust 校验。
    if (!moldOk || !meltOk || !gatesOk || !std::isfinite(gates) || gates < 0 ||
        gates > std::numeric_limits<std::uint32_t>::max() || std::floor(gates) != gates)
        return fail(QStringLiteral("project.command_invalid"), QStringLiteral("validation"),
                    QStringLiteral("invalid gate location settings value"));
    panta::ffi::GateLocationSettings candidate;
    candidate.machine_id = machine;
    candidate.algorithm_id = algorithm;
    candidate.mold_temperature_celsius = mold;
    candidate.melt_temperature_celsius = melt;
    candidate.number_of_gates = static_cast<std::uint32_t>(gates);
    const auto result = panta::ffi::project_service_begin_gate_location_settings_confirmation(
        *m_service, path, revision, id, std::move(candidate));
    if (!result.error.code.empty()) {
        return fail(result.error);
    }
    const bool pending = result.value;
    clearError();
    if (pending)
        set_pending_confirmation(ConfirmationKind::GateLocationSettings);
    else {
        refreshPlanSettings();
        emit gateLocationSettingsConfirmationFinished(true);
    }
    return true;
}

QVariantMap ProjectViewModel::validateGateLocationSettingsDraft(const QVariantMap& settings) const {
    return draft_validation_map(panta::ffi::validate_gate_location_settings_draft(
        draft_number(settings, QStringLiteral("moldTemperature")),
        draft_number(settings, QStringLiteral("meltTemperature")),
        draft_number(settings, QStringLiteral("numberOfGates")),
        settings.value(QStringLiteral("machineId")).toString().toStdString()));
}

void ProjectViewModel::set_pending_confirmation(ConfirmationKind kind) {
    const auto previous = m_pendingConfirmation;
    if (previous == kind) {
        return;
    }
    m_pendingConfirmation = kind;
    if (kind == ConfirmationKind::None) {
        m_metadataConfirmationPoll.stop();
    } else {
        m_metadataConfirmationPoll.start();
    }
    if (previous == ConfirmationKind::Material || kind == ConfirmationKind::Material) {
        emit materialConfirmationPendingChanged();
    }
    if (previous == ConfirmationKind::GateLocationSettings ||
        kind == ConfirmationKind::GateLocationSettings) {
        emit gateLocationSettingsConfirmationPendingChanged();
    }
    if (previous == ConfirmationKind::FillSettings || kind == ConfirmationKind::FillSettings) {
        emit fillSettingsConfirmationPendingChanged();
    }
}

void ProjectViewModel::finish_metadata_confirmation() {
    const auto kind = m_pendingConfirmation;
    if (kind == ConfirmationKind::None) {
        return;
    }
    panta::ffi::ProjectConfirmationResult result;
    switch (kind) {
    case ConfirmationKind::Material:
        result = panta::ffi::project_service_finish_material_confirmation(*m_service);
        break;
    case ConfirmationKind::FillSettings:
        result = panta::ffi::project_service_finish_fill_settings_confirmation(*m_service);
        break;
    case ConfirmationKind::GateLocationSettings:
        result = panta::ffi::project_service_finish_gate_location_settings_confirmation(*m_service);
        break;
    case ConfirmationKind::None:
        return;
    }
    if (result.error.code.empty() && !result.value) {
        return;
    }
    bool succeeded = false;
    if (!result.error.code.empty()) {
        fail(result.error);
    } else if (applySnapshot(panta::ffi::project_service_current(*m_service))) {
        // 当前待完成类型保持到快照通知结束，防止同步回调重入下一次写入。
        refreshPlanSettings();
        clearError();
        succeeded = true;
    }
    set_pending_confirmation(ConfirmationKind::None);
    if (kind == ConfirmationKind::Material) {
        emit materialConfirmationFinished(succeeded);
    } else if (kind == ConfirmationKind::FillSettings) {
        emit fillSettingsConfirmationFinished(succeeded);
    } else {
        emit gateLocationSettingsConfirmationFinished(succeeded);
    }
}

const QString& ProjectViewModel::importedMeshType() const { return m_importedMeshType; }

const QString& ProjectViewModel::importedUnits() const { return m_importedUnits; }

const QString& ProjectViewModel::importedDimensions() const { return m_importedDimensions; }

quint64 ProjectViewModel::importedTriangleCount() const { return m_importedTriangleCount; }

bool ProjectViewModel::importPreviewPending() const { return m_importPreviewPending; }

bool ProjectViewModel::importPreviewReady() const { return m_importPreviewReady; }

const QString& ProjectViewModel::importPreviewName() const { return m_importPreviewName; }

const QString& ProjectViewModel::importPreviewDimensions() const {
    return m_importPreviewDimensions;
}

quint64 ProjectViewModel::importPreviewTriangleCount() const {
    return m_importPreviewTriangleCount;
}

bool ProjectViewModel::importPending() const { return m_importPending; }

quint64 ProjectViewModel::importRequestId() const { return m_importRequest; }

bool ProjectViewModel::createProject(const QString& rawName, const QString& rawLocation) {
    clearError();
    std::string name;
    std::string location;
    QString conversionError;
    if (!toBoundaryText(rawName.trimmed(), &name, &conversionError) ||
        !toBoundaryText(QDir::cleanPath(QDir::fromNativeSeparators(rawLocation.trimmed())),
                        &location, &conversionError)) {
        return fail(conversionError);
    }
    const auto snapshot = panta::ffi::project_service_create(*m_service, location, name);
    if (snapshot.error.code.empty())
        cancelImportPreview();
    if (!applySnapshot(snapshot)) {
        return false;
    }
    if (!refreshImports()) {
        return false;
    }
    resetVariableDocument();
    reset_documents();
    m_lastCreatedPath = m_currentPath;
    emit projectCreated(m_lastCreatedPath);
    return true;
}

void ProjectViewModel::clearError() {
    if (m_error.isEmpty() && m_errorCode.isEmpty() && m_errorCategory.isEmpty() &&
        m_errorDetail.isEmpty()) {
        return;
    }
    m_error.clear();
    m_errorCode.clear();
    m_errorCategory.clear();
    m_errorDetail.clear();
    emit errorChanged();
}

void ProjectViewModel::retranslateError() {
    if (m_errorCode.isEmpty()) {
        return;
    }
    const QString message = translated_project_error(m_errorCode);
    if (m_error == message) {
        return;
    }
    m_error = message;
    emit errorChanged();
}

bool ProjectViewModel::openProject(const QString& path) {
    clearError();
    std::string boundaryPath;
    QString conversionError;
    if (!toBoundaryText(QDir::cleanPath(QDir::fromNativeSeparators(path.trimmed())), &boundaryPath,
                        &conversionError)) {
        return fail(conversionError);
    }
    const auto snapshot = panta::ffi::project_service_open(*m_service, boundaryPath);
    if (snapshot.error.code.empty())
        cancelImportPreview();
    if (!applySnapshot(snapshot)) {
        return false;
    }
    if (!refreshImports()) {
        return false;
    }
    resetVariableDocument();
    reset_documents();
    emit projectOpened(m_currentPath);
    return true;
}

bool ProjectViewModel::openProjectUrl(const QUrl& url) {
    const QString path = localPath(url);
    return path.isEmpty() ? fail(QStringLiteral("project.file_not_local")) : openProject(path);
}

QString ProjectViewModel::localPath(const QUrl& url) const {
    return url.isLocalFile() ? url.toLocalFile() : QString{};
}

bool ProjectViewModel::saveProject() {
    clearError();
    const auto snapshot = panta::ffi::project_service_save(*m_service);
    if (!applySnapshot(snapshot)) {
        return false;
    }
    emit projectSaved(m_currentPath);
    return true;
}

bool ProjectViewModel::beginLoadVariableDocument(quint64 expectedGeneration,
                                                 quint64 expectedRevision) {
    clearError();
    return beginVariableDocument({}, false, expectedGeneration, expectedRevision);
}

bool ProjectViewModel::beginSaveVariableDocument(const QString& source, quint64 expectedGeneration,
                                                 quint64 expectedRevision) {
    clearError();
    std::string boundarySource;
    QString conversionError;
    if (!toBoundaryText(source, &boundarySource, &conversionError)) {
        return fail(conversionError);
    }
    return beginVariableDocument(boundarySource, true, expectedGeneration, expectedRevision);
}

void ProjectViewModel::startVariableWaiter(std::function<void()> work) {
    std::thread(std::move(work)).detach();
}

bool ProjectViewModel::beginVariableDocument(const std::string& source, bool save,
                                             quint64 expectedGeneration, quint64 expectedRevision) {
    using Request = rust::Box<panta::ffi::VariableDocumentRequest>;
    using Completion = rust::Box<panta::ffi::VariableDocumentCompletion>;
    using Result = std::shared_ptr<Completion>;
    using Watcher = QFutureWatcher<Result>;
    const auto input = std::make_shared<std::promise<std::optional<Request>>>();
    const auto ready = std::make_shared<std::future<std::optional<Request>>>(input->get_future());
    const auto result = std::make_shared<QPromise<Result>>();
    auto watcher = std::make_unique<Watcher>();
    result->start();
    watcher->setFuture(result->future());
    connect(watcher.get(), &Watcher::finished, watcher.get(), &QObject::deleteLater);
    connect(
        watcher.get(), &Watcher::finished, this,
        [this, future = result->future(), expectedGeneration] {
            // Rust 准入被拒绝时等待线程只有退出回执，没有变量结果。
            if (future.resultCount() == 0) {
                return;
            }
            const auto completion = future.result();
            const auto document = panta::ffi::project_service_finish_variable_document(
                *m_service, std::move(*completion));
            const bool succeeded =
                expectedGeneration == m_variableGeneration && applyVariableDocument(document);
            emit variableDocumentFinished(expectedGeneration, succeeded);
        },
        Qt::QueuedConnection);
    try {
        // 先确保等待线程存在，再准入可能写盘的 Rust 请求。gate 不持有 QObject/服务。
        // std::thread 构造失败会抛异常；Qt 的 QThread::start 则可能仅 warning 且永不发 finished。
        startVariableWaiter([ready, result] {
            auto request = ready->get();
            if (request) {
                result->addResult(std::make_shared<Completion>(
                    panta::ffi::variable_document_wait(std::move(*request))));
            }
            result->finish();
        });
    } catch (const std::system_error& error) {
        return fail(QStringLiteral("project.io"), QStringLiteral("io"),
                    QString::fromUtf8(error.what()));
    }
    // watcher 归 ViewModel，销毁会撤销回调；后台只持有 future 状态，GUI 不 join。
    watcher.release()->setParent(this);
    auto request = panta::ffi::project_service_begin_variable_document(
        *m_service, static_cast<std::uint64_t>(expectedGeneration),
        static_cast<std::uint64_t>(expectedRevision), source, save);
    if (!panta::ffi::variable_document_request_accepted(*request)) {
        input->set_value(std::nullopt);
        // 拒绝的请求没有后台 receiver，wait 在此只取回结构化准入错误。
        const auto document = panta::ffi::project_service_finish_variable_document(
            *m_service, panta::ffi::variable_document_wait(std::move(request)));
        return applyVariableDocument(document);
    }
    input->set_value(std::move(request));
    return true;
}

bool ProjectViewModel::renameProject(const QString& name) {
    clearError();
    std::string boundaryName;
    QString conversionError;
    if (!toBoundaryText(name.trimmed(), &boundaryName, &conversionError)) {
        return fail(conversionError);
    }
    const auto snapshot = panta::ffi::project_service_execute(
        *m_service,
        panta::ffi::ProjectCommand{panta::ffi::ProjectCommandKind::Rename, boundaryName});
    return applySnapshot(snapshot);
}

bool ProjectViewModel::importStl(const QString& rawPath, const QString& rawMeshType,
                                 const QString& rawUnits, bool showImportLog) {
    clearError();
    std::string path;
    std::string meshType;
    std::string units;
    QString conversionError;
    if (!toBoundaryText(QDir::cleanPath(QDir::fromNativeSeparators(rawPath.trimmed())), &path,
                        &conversionError) ||
        !toBoundaryText(rawMeshType.trimmed(), &meshType, &conversionError) ||
        !toBoundaryText(rawUnits.trimmed(), &units, &conversionError)) {
        return fail(conversionError);
    }
    const auto result =
        panta::ffi::project_service_import_stl(*m_service, path, meshType, units, showImportLog);
    if (!result.error.code.empty()) {
        return fail(result.error);
    }
    return publish_import(result.value);
}

bool ProjectViewModel::beginImportStl(const QString& rawPath, const QString& rawMeshType,
                                      const QString& rawUnits, bool showImportLog) {
    clearError();
    std::string path;
    std::string meshType;
    std::string units;
    QString conversionError;
    if (!toBoundaryText(QDir::cleanPath(QDir::fromNativeSeparators(rawPath.trimmed())), &path,
                        &conversionError) ||
        !toBoundaryText(rawMeshType.trimmed(), &meshType, &conversionError) ||
        !toBoundaryText(rawUnits.trimmed(), &units, &conversionError)) {
        return fail(conversionError);
    }
    const auto result = panta::ffi::project_service_begin_stl_import(*m_service, path, meshType,
                                                                     units, showImportLog);
    if (!result.error.code.empty()) {
        return fail(result.error);
    }
    m_importRequest = result.value;
    m_importPending = true;
    m_stlImportPoll.start();
    emit importPendingChanged();
    return true;
}

bool ProjectViewModel::cancelImport() {
    if (!m_importPending || m_importRequest == 0) {
        return false;
    }
    if (!panta::ffi::project_service_cancel_stl_import(*m_service, m_importRequest)) {
        return false;
    }
    m_stlImportPoll.stop();
    m_importRequest = 0;
    m_importPending = false;
    emit importPendingChanged();
    return true;
}

void ProjectViewModel::finish_stl_import() {
    const auto request = m_importRequest;
    if (!m_importPending || request == 0) {
        m_stlImportPoll.stop();
        return;
    }
    const auto result = panta::ffi::project_service_finish_stl_import(*m_service, request);
    if (!result.ready) {
        return;
    }
    m_stlImportPoll.stop();
    m_importRequest = 0;
    m_importPending = false;
    bool succeeded = false;
    if (!result.error.code.empty()) {
        fail(result.error);
        cancelImportPreview();
    } else {
        succeeded = publish_import(result.value);
    }
    emit projectImportFinished(request, succeeded);
    emit importPendingChanged();
}

bool ProjectViewModel::publish_import(const panta::ffi::ProjectImport& imported) {
    const auto snapshot = panta::ffi::project_service_current(*m_service);
    if (!applySnapshot(snapshot) || !refreshImports()) {
        return false;
    }
    panta::ffi::project_service_workspace_publish_import(*m_service, imported.id,
                                                         imported.display_name);
    refresh_workspace();
    emit projectImported(m_importedAssetPath);
    return true;
}

void ProjectViewModel::cancelImportPreview() {
    m_importPreviewPoll.stop();
    if (m_previewRequest != 0) {
        panta::ffi::project_service_cancel_stl_preview(*m_service, m_previewRequest);
        m_previewRequest = 0;
    }
    const bool changed = m_importPreviewPending || m_importPreviewReady;
    m_importPreviewPending = false;
    m_importPreviewReady = false;
    m_importPreviewName.clear();
    m_importPreviewDimensions.clear();
    m_importPreviewTriangleCount = 0;
    if (changed)
        emit importPreviewChanged();
}

bool ProjectViewModel::inspectStl(const QString& rawPath) {
    cancelImportPreview();
    clearError();
    std::string path;
    QString conversionError;
    if (!toBoundaryText(QDir::cleanPath(QDir::fromNativeSeparators(rawPath.trimmed())), &path,
                        &conversionError)) {
        return fail(conversionError);
    }
    const auto result = panta::ffi::project_service_begin_stl_preview(*m_service, path);
    if (!result.error.code.empty())
        return fail(result.error);
    m_previewRequest = result.value;
    m_importPreviewPending = true;
    m_importPreviewPoll.start();
    emit importPreviewChanged();
    return true;
}

void ProjectViewModel::finish_import_preview() {
    const auto request = m_previewRequest;
    const auto result = panta::ffi::project_service_finish_stl_preview(*m_service, request);
    if (!result.ready && result.error.code.empty())
        return;
    m_importPreviewPoll.stop();
    m_importPreviewPending = false;
    const bool succeeded = result.error.code.empty();
    if (succeeded) {
        m_importPreviewReady = true;
        m_importPreviewName = QString::fromUtf8(result.value.source_name);
        m_importPreviewDimensions = QString::fromUtf8(panta::ffi::format_project_dimensions(
            result.value.size_x, result.value.size_y, result.value.size_z, ""));
        m_importPreviewTriangleCount = result.value.triangle_count;
    } else {
        fail(result.error);
    }
    // 错误或属性通知可重入并替换请求；旧完成事件不能再驱动新表单。
    if (m_previewRequest != request)
        return;
    emit importPreviewChanged();
    if (m_previewRequest == request)
        emit importPreviewFinished(request, succeeded);
}

bool ProjectViewModel::fail(const panta::ffi::ProjectDiagnostic& diagnostic) {
    return fail(QString::fromUtf8(diagnostic.code), QString::fromUtf8(diagnostic.category),
                QString::fromUtf8(diagnostic.detail));
}

bool ProjectViewModel::applySnapshot(const panta::ffi::ProjectSnapshotResult& result) {
    return result.error.code.empty() ? applySnapshot(result.value) : fail(result.error);
}

bool ProjectViewModel::applyVariableDocument(const panta::ffi::VariableDocumentResult& result) {
    if (!result.error.code.empty()) {
        if (!m_variableDiagnostics.isEmpty()) {
            m_variableDiagnostics.clear();
            emit variableDocumentChanged();
        }
        return fail(result.error);
    }
    const auto diagnostics = variable_diagnostics(result.diagnostics);
    if (!diagnostics.isEmpty()) {
        m_variableDiagnostics = diagnostics;
        emit variableDocumentChanged();
        const auto first = diagnostics.constFirst().toMap();
        return fail(first.value(QStringLiteral("code")).toString(), QStringLiteral("validation"),
                    first.value(QStringLiteral("message")).toString());
    }
    m_variableSource = QString::fromUtf8(result.source);
    m_variableValues = variable_values(result.values);
    m_variableDiagnostics.clear();
    m_variableGeneration = static_cast<quint64>(result.generation);
    m_variableRevision = static_cast<quint64>(result.revision);
    emit variableDocumentChanged();
    return true;
}

void ProjectViewModel::resetVariableDocument() {
    const auto generation =
        static_cast<quint64>(panta::ffi::project_service_variable_document_generation(*m_service));
    if (m_variableSource.isEmpty() && m_variableValues.isEmpty() &&
        m_variableDiagnostics.isEmpty() && m_variableGeneration == generation &&
        m_variableRevision == 0) {
        return;
    }
    m_variableSource.clear();
    m_variableValues.clear();
    m_variableDiagnostics.clear();
    m_variableGeneration = generation;
    m_variableRevision = 0;
    emit variableDocumentChanged();
}

bool ProjectViewModel::fail(const QString& code, const QString& category, const QString& detail) {
    const QString message = translated_project_error(code);
    if (m_errorCode != code || m_error != message || m_errorCategory != category ||
        m_errorDetail != detail) {
        m_errorCode = code;
        m_error = message;
        m_errorCategory = category;
        m_errorDetail = detail;
        emit errorChanged();
    }
    return false;
}

bool ProjectViewModel::applySnapshot(const panta::ffi::ProjectSnapshot& snapshot) {
    const QString path = QString::fromUtf8(snapshot.path);
    const QString name = QString::fromUtf8(snapshot.name);
    const bool changed =
        m_currentPath != path || m_currentName != name || m_dirty != snapshot.dirty;
    m_currentPath = path;
    m_currentName = name;
    m_dirty = snapshot.dirty;
    if (changed) {
        emit projectChanged();
    }
    return true;
}

void ProjectViewModel::applyImports(const rust::Vec<panta::ffi::ProjectImport>& imports) {
    QStringList nextImportedPartNames;
    QStringList nextPartIds;
    const QDir projectDirectory(QFileInfo(m_currentPath).absolutePath());
    for (std::size_t index = 0; index < imports.size(); ++index) {
        const auto& imported = imports[index];
        nextImportedPartNames.append(QString::fromUtf8(imported.display_name));
        nextPartIds.append(QString::fromUtf8(imported.id));
    }

    QString nextPartName;
    QString nextAssetPath;
    QString nextMeshType;
    QString nextUnits;
    QString nextDimensions;
    quint64 nextTriangleCount = 0;

    if (imports.size() > 0) {
        const auto& imported = imports[imports.size() - 1];
        nextPartName = QString::fromUtf8(imported.display_name);
        nextAssetPath = projectDirectory.filePath(QString::fromUtf8(imported.mesh_asset_path));
        nextMeshType = QString::fromUtf8(imported.mesh_type);
        nextUnits = QString::fromUtf8(imported.units);
        nextTriangleCount = imported.triangle_count;
        const QString unitLabel = nextUnits == QStringLiteral("millimeters") ? QStringLiteral("mm")
                                  : nextUnits == QStringLiteral("centimeters")
                                      ? QStringLiteral("cm")
                                      : QStringLiteral("in");
        nextDimensions = QString::fromUtf8(panta::ffi::format_project_dimensions(
            imported.size_x, imported.size_y, imported.size_z, unitLabel.toStdString()));
    }

    const bool changed =
        m_importedPartNames != nextImportedPartNames || m_importedPartIds != nextPartIds ||
        m_importedPartName != nextPartName || m_importedAssetPath != nextAssetPath ||
        m_importedMeshType != nextMeshType || m_importedUnits != nextUnits ||
        m_importedDimensions != nextDimensions || m_importedTriangleCount != nextTriangleCount;
    m_importedPartNames = std::move(nextImportedPartNames);
    m_importedPartIds = std::move(nextPartIds);
    m_importedPartName = nextPartName;
    m_importedAssetPath = nextAssetPath;
    m_importedMeshType = nextMeshType;
    m_importedUnits = nextUnits;
    m_importedDimensions = nextDimensions;
    m_importedTriangleCount = nextTriangleCount;
    if (changed) {
        emit importsChanged();
    }
}

bool ProjectViewModel::refreshImports() {
    const auto result = panta::ffi::project_service_imports(*m_service);
    if (!result.error.code.empty()) {
        return fail(result.error);
    }
    applyImports(result.value);
    refreshPlanSettings();
    // 网格内容只随文档（激活 / 快照变化）改变：由文档路径负责发射
    // meshChanged，避免与导入激活的发射重复。
    return true;
}

bool ProjectViewModel::toBoundaryText(const QString& text, std::string* out, QString* error) {
    return PathHost::toBoundaryUtf8(text, out, error);
}

QVariantList ProjectViewModel::openDocuments() const { return m_openDocuments; }

QString ProjectViewModel::activeDocumentId() const { return m_activeDocumentId; }

QString ProjectViewModel::activeDocumentTitle() const {
    for (const auto& item : m_openDocuments) {
        const auto document = item.toMap();
        if (document.value(QStringLiteral("id")).toString() == m_activeDocumentId) {
            return document.value(QStringLiteral("title")).toString();
        }
    }
    return QString{};
}

const QString& ProjectViewModel::variableSource() const { return m_variableSource; }

QVariantList ProjectViewModel::variableValues() const { return m_variableValues; }

QVariantList ProjectViewModel::variableDiagnostics() const { return m_variableDiagnostics; }

quint64 ProjectViewModel::variableRevision() const { return m_variableRevision; }

quint64 ProjectViewModel::variableGeneration() const { return m_variableGeneration; }

QStringList ProjectViewModel::importedPartIds() const { return m_importedPartIds; }

void ProjectViewModel::reset_documents() {
    panta::ffi::project_service_workspace_reset(*m_service, tr("Welcome").toStdString());
    refresh_workspace(true);
}

void ProjectViewModel::refresh_workspace(bool forceActiveNotification) {
    const auto snapshot = panta::ffi::project_service_workspace_snapshot(*m_service);
    QVariantList nextDocuments;
    nextDocuments.reserve(static_cast<qsizetype>(snapshot.documents.size()));
    for (const auto& document : snapshot.documents) {
        QVariantMap diagnostic;
        if (!document.diagnostic_code.empty()) {
            diagnostic = {
                {QStringLiteral("code"), QString::fromUtf8(document.diagnostic_code)},
                {QStringLiteral("category"), QString::fromUtf8(document.diagnostic_category)},
                {QStringLiteral("detail"), QString::fromUtf8(document.diagnostic_detail)}};
        }
        nextDocuments.append(
            QVariantMap{{QStringLiteral("id"), QString::fromUtf8(document.id)},
                        {QStringLiteral("kind"), QString::fromUtf8(document.kind)},
                        {QStringLiteral("state"), QString::fromUtf8(document.state)},
                        {QStringLiteral("title"), QString::fromUtf8(document.title)},
                        {QStringLiteral("message"), QString::fromUtf8(document.message_key)},
                        {QStringLiteral("diagnostic"), diagnostic},
                        {QStringLiteral("displayOrdinal"),
                         QVariant::fromValue<qulonglong>(document.display_ordinal)},
                        {QStringLiteral("displayCount"),
                         QVariant::fromValue<qulonglong>(document.display_count)}});
    }

    const bool documentsChangedValue = m_openDocuments != nextDocuments;
    const QString nextActiveId = QString::fromUtf8(snapshot.active_document_id);
    const bool activeChanged = forceActiveNotification || m_activeDocumentId != nextActiveId;
    m_openDocuments = std::move(nextDocuments);
    if (activeChanged) {
        m_activeDocumentId = nextActiveId;
        std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot> nextMesh;
        for (const auto& item : m_openDocuments) {
            const auto document = item.toMap();
            if (document.value(QStringLiteral("id")).toString() == nextActiveId &&
                document.value(QStringLiteral("kind")).toString() == QStringLiteral("import") &&
                document.value(QStringLiteral("state")).toString() == QStringLiteral("ready")) {
                nextMesh = pull_service_mesh(nextActiveId);
                break;
            }
        }
        m_activeMesh = std::move(nextMesh);
        emit activeDocumentChanged();
        emit meshChanged();
    }
    if (documentsChangedValue) {
        emit documentsChanged();
    }
    sync_activation_poll();
}

void ProjectViewModel::activateDocument(const QString& documentId) {
    panta::ffi::project_service_workspace_activate(*m_service, documentId.toStdString());
    refresh_workspace();
}

void ProjectViewModel::closeDocument(const QString& documentId) {
    panta::ffi::project_service_workspace_close(*m_service, documentId.toStdString());
    refresh_workspace();
}

void ProjectViewModel::openImportRecord(const QString& recordId) {
    const qsizetype index = m_importedPartIds.indexOf(recordId);
    if (index < 0) {
        return;
    }
    panta::ffi::project_service_workspace_open_import(
        *m_service, recordId.toStdString(), m_importedPartNames.value(index).toStdString());
    refresh_workspace();
}

void ProjectViewModel::moveDocument(int fromIndex, int toIndex) {
    panta::ffi::project_service_workspace_move(*m_service, static_cast<std::uint64_t>(fromIndex),
                                               static_cast<std::uint64_t>(toIndex));
    refresh_workspace();
}

void ProjectViewModel::drain_activations() {
    [[maybe_unused]] const auto outcomes =
        panta::ffi::project_service_drain_workspace_activations(*m_service);
    refresh_workspace();
}

void ProjectViewModel::sync_activation_poll() {
    const bool loading =
        std::any_of(m_openDocuments.cbegin(), m_openDocuments.cend(), [](const QVariant& item) {
            return item.toMap().value(QStringLiteral("state")) == QStringLiteral("loading");
        });
    if (loading && !m_activationPoll.isActive()) {
        m_activationPoll.start();
    } else if (!loading && m_activationPoll.isActive()) {
        m_activationPoll.stop();
    }
}

} // namespace panta::bridge
