#include "project_view_model.hpp"

#include "panta/visualization/mesh_source.hpp"
#include "panta_ffi.h"
#include "path_host.hpp"
#include <QDir>
#include <QFileInfo>
#include <QLatin1Char>
#include <QObject>
#include <QString>
#include <QUrl>
#include <QVariantMap>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qtmetamacros.h>
#include <cstddef>
#include <memory>
#include <panta/qt_adapter/standard_paths.hpp>
#include <qlogging.h>
#include <rust/cxx.h>
#include <string>
#include <utility>

namespace panta::bridge {

namespace {

constexpr auto kWelcomeDocumentId = "welcome";

QString default_project_location() {
    const QString documents =
        panta::qt_adapter::standard_location(panta::qt_adapter::StandardLocation::Documents);
    return documents.isEmpty() ? QString{} : QDir(documents).filePath(QStringLiteral("panta"));
}

QString format_dimensions(double sizeX, double sizeY, double sizeZ, const QString& unit = {}) {
    const QString values = QStringLiteral("%1 × %2 × %3")
                               .arg(QString::number(sizeX, 'f', 2), QString::number(sizeY, 'f', 2),
                                    QString::number(sizeZ, 'f', 2));
    return unit.isEmpty() ? values : values + QLatin1Char(' ') + unit;
}

// Rust 错误可能带冒号后的细节；用户消息只按稳定错误码选择。
QString activation_message_for(const QString& code) {
    const QString errorCode = code.section(QLatin1Char(':'), 0, 0);
    if (errorCode == QStringLiteral("project.asset_missing")) {
        return QStringLiteral("The saved STL asset is missing from the project package.");
    }
    if (errorCode == QStringLiteral("project.asset_unsupported_format")) {
        return QStringLiteral("The saved STL asset format is not supported.");
    }
    if (errorCode == QStringLiteral("project.import_unsupported_units")) {
        return QStringLiteral("The saved STL asset uses unsupported units.");
    }
    if (errorCode == QStringLiteral("project.asset_read_failed") ||
        errorCode == QStringLiteral("project.asset_parse_failed")) {
        return QStringLiteral("The saved STL asset could not be read.");
    }
    return QStringLiteral("The saved STL asset could not be opened.");
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

} // namespace

ProjectViewModel::ProjectViewModel(QObject* parent)
    : MeshSource(parent), m_defaultLocation(default_project_location()),
      m_analysisSequences(choice_catalog(panta::ffi::analysis_sequence_catalog())),
      m_meshTypes(choice_catalog(panta::ffi::mesh_type_catalog())),
      m_defaultMeshType(QString::fromUtf8(panta::ffi::default_mesh_type())),
      m_service(panta::ffi::project_service_new()) {
    try {
        m_defaultMaterial = material_definition(panta::ffi::default_material());
    } catch (const rust::Error& failure) {
        fail(QString::fromUtf8(failure.what()));
    }
    connect(this, &ProjectViewModel::activeDocumentChanged, this,
            &ProjectViewModel::refreshPlanSettings);
    connect(this, &ProjectViewModel::projectChanged, this, &ProjectViewModel::refreshPlanSettings);

    // 激活结果为拉取式队列（073）；10ms 轮询与 TaskHost 节奏一致，仅在
    // 存在 Loading 文档时运转。
    m_activationPoll.setInterval(10);
    connect(&m_activationPoll, &QTimer::timeout, this, &ProjectViewModel::drain_activations);
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
    for (const auto& document : m_documents) {
        if (document.id == m_activeDocumentId) {
            return document.kind == QStringLiteral("welcome");
        }
    }
    return false;
}

std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
ProjectViewModel::pull_service_mesh(const QString& importId) const {
    try {
        const auto snapshot = panta::ffi::project_service_mesh_snapshot_for_import(
            *m_service, importId.toStdString());
        if (snapshot.coordinates.empty() || snapshot.coordinates.size() % 9 != 0) {
            return nullptr;
        }
        auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        mesh->project_revision = snapshot.revision;
        mesh->vertices.reserve(snapshot.coordinates.size() / 3);
        for (std::size_t offset = 0; offset < snapshot.coordinates.size(); offset += 3) {
            mesh->vertices.push_back({snapshot.coordinates[offset],
                                      snapshot.coordinates[offset + 1],
                                      snapshot.coordinates[offset + 2]});
        }
        return mesh;
    } catch (const rust::Error& error) {
        qWarning("ProjectViewModel mesh snapshot failed: %s", error.what());
        return nullptr;
    }
}

const QString& ProjectViewModel::defaultLocation() const { return m_defaultLocation; }

QUrl ProjectViewModel::defaultLocationUrl() const {
    return m_defaultLocation.isEmpty() ? QUrl{} : QUrl::fromLocalFile(m_defaultLocation);
}

const QString& ProjectViewModel::lastCreatedPath() const { return m_lastCreatedPath; }

const QString& ProjectViewModel::error() const { return m_error; }

const QString& ProjectViewModel::errorCode() const { return m_errorCode; }

const QString& ProjectViewModel::currentPath() const { return m_currentPath; }

const QString& ProjectViewModel::currentName() const { return m_currentName; }

bool ProjectViewModel::dirty() const { return m_dirty; }

const QStringList& ProjectViewModel::importedPartNames() const { return m_importedPartNames; }

const QString& ProjectViewModel::importedPartName() const { return m_importedPartName; }

const QString& ProjectViewModel::importedAssetPath() const { return m_importedAssetPath; }

QVariantList ProjectViewModel::meshTypes() const { return m_meshTypes; }

QString ProjectViewModel::defaultMeshType() const { return m_defaultMeshType; }

QVariantMap ProjectViewModel::defaultMaterial() const { return m_defaultMaterial; }

QVariantList ProjectViewModel::analysisSequences() const { return m_analysisSequences; }

QVariantMap ProjectViewModel::planSettings() const { return m_planSettings; }

void ProjectViewModel::refreshPlanSettings() {
    const auto settings =
        panta::ffi::project_service_plan_settings(*m_service, m_activeDocumentId.toStdString());
    const QVariantMap next{
        {QStringLiteral("projectPath"), QString::fromUtf8(settings.project_path)},
        {QStringLiteral("revision"), QVariant::fromValue(settings.revision)},
        {QStringLiteral("importId"), QString::fromUtf8(settings.import_id)},
        {QStringLiteral("meshType"), QString::fromUtf8(settings.mesh_type)},
        {QStringLiteral("sequenceId"), QString::fromUtf8(settings.sequence_id)},
        {QStringLiteral("sequenceSourceText"), QString::fromUtf8(settings.sequence_source_text)},
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
    try {
        applySnapshot(panta::ffi::project_service_set_analysis_sequence(*m_service, path, revision,
                                                                        id, sequence));
        refreshPlanSettings();
        clearError();
        return true;
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
}

bool ProjectViewModel::setMaterial(const QString& projectPath, quint64 revision,
                                   const QString& importId, const QString& materialId) {
    std::string path;
    std::string id;
    std::string material;
    QString conversionError;
    if (!toBoundaryText(projectPath, &path, &conversionError) ||
        !toBoundaryText(importId, &id, &conversionError) ||
        !toBoundaryText(materialId, &material, &conversionError)) {
        return fail(conversionError);
    }
    try {
        applySnapshot(
            panta::ffi::project_service_set_material(*m_service, path, revision, id, material));
        // 工程摘要不包含方案字段；即使 projectChanged 未发出，也要刷新已提交的配置。
        refreshPlanSettings();
        clearError();
        return true;
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
}

const QString& ProjectViewModel::importedMeshType() const { return m_importedMeshType; }

const QString& ProjectViewModel::importedUnits() const { return m_importedUnits; }

const QString& ProjectViewModel::importedDimensions() const { return m_importedDimensions; }

quint64 ProjectViewModel::importedTriangleCount() const { return m_importedTriangleCount; }

bool ProjectViewModel::importPreviewReady() const { return m_importPreviewReady; }

const QString& ProjectViewModel::importPreviewName() const { return m_importPreviewName; }

const QString& ProjectViewModel::importPreviewDimensions() const {
    return m_importPreviewDimensions;
}

quint64 ProjectViewModel::importPreviewTriangleCount() const {
    return m_importPreviewTriangleCount;
}

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
    try {
        const auto snapshot = panta::ffi::project_service_create(*m_service, location, name);
        if (!applySnapshot(snapshot)) {
            return false;
        }
        if (!refreshImports()) {
            return false;
        }
        reset_documents();
        m_lastCreatedPath = m_currentPath;
        emit projectCreated(m_lastCreatedPath);
        return true;
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
}

void ProjectViewModel::clearError() {
    if (m_error.isEmpty() && m_errorCode.isEmpty()) {
        return;
    }
    m_error.clear();
    m_errorCode.clear();
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
    try {
        const auto snapshot = panta::ffi::project_service_open(*m_service, boundaryPath);
        if (!applySnapshot(snapshot)) {
            return false;
        }
        if (!refreshImports()) {
            return false;
        }
        reset_documents();
        emit projectOpened(m_currentPath);
        return true;
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
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
    try {
        const auto snapshot = panta::ffi::project_service_save(*m_service);
        if (!applySnapshot(snapshot)) {
            return false;
        }
        emit projectSaved(m_currentPath);
        return true;
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
}

bool ProjectViewModel::renameProject(const QString& name) {
    clearError();
    std::string boundaryName;
    QString conversionError;
    if (!toBoundaryText(name.trimmed(), &boundaryName, &conversionError)) {
        return fail(conversionError);
    }
    try {
        const auto snapshot = panta::ffi::project_service_execute(
            *m_service,
            panta::ffi::ProjectCommand{panta::ffi::ProjectCommandKind::Rename, boundaryName});
        return applySnapshot(snapshot);
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
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
    try {
        const auto imported = panta::ffi::project_service_import_stl(*m_service, path, meshType,
                                                                     units, showImportLog);
        const auto snapshot = panta::ffi::project_service_current(*m_service);
        if (!applySnapshot(snapshot) || !refreshImports()) {
            return false;
        }
        const QString recordId = QString::fromUtf8(imported.id);
        int index = document_index(recordId);
        bool documentsChangedEmitted = false;
        if (index < 0) {
            index = static_cast<int>(m_documents.size());
            m_documents.append({recordId, QStringLiteral("import"), QStringLiteral("ready"),
                                QString::fromUtf8(imported.source_name), QString{}});
            documentsChangedEmitted = true;
        }
        documentsChangedEmitted = activate_ready_document(index) || documentsChangedEmitted;
        documentsChangedEmitted = sync_mesh_residency() || documentsChangedEmitted;
        if (documentsChangedEmitted) {
            emit documentsChanged();
        }
        emit projectImported(m_importedAssetPath);
        return true;
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
}

bool ProjectViewModel::inspectStl(const QString& rawPath) {
    clearError();
    if (m_importPreviewReady) {
        m_importPreviewReady = false;
        m_importPreviewName.clear();
        m_importPreviewDimensions.clear();
        m_importPreviewTriangleCount = 0;
        emit importPreviewChanged();
    }
    std::string path;
    QString conversionError;
    if (!toBoundaryText(QDir::cleanPath(QDir::fromNativeSeparators(rawPath.trimmed())), &path,
                        &conversionError)) {
        return fail(conversionError);
    }
    try {
        const auto preview = panta::ffi::project_service_inspect_stl(*m_service, path);
        const QString name = QString::fromUtf8(preview.source_name);
        const QString dimensions =
            format_dimensions(preview.size_x, preview.size_y, preview.size_z);
        const bool changed = !m_importPreviewReady || m_importPreviewName != name ||
                             m_importPreviewDimensions != dimensions ||
                             m_importPreviewTriangleCount != preview.triangle_count;
        m_importPreviewReady = true;
        m_importPreviewName = name;
        m_importPreviewDimensions = dimensions;
        m_importPreviewTriangleCount = preview.triangle_count;
        if (changed) {
            emit importPreviewChanged();
        }
        return true;
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
}

bool ProjectViewModel::fail(const QString& boundaryError) {
    const auto separator = boundaryError.indexOf(QLatin1Char(':'));
    const QString code = separator < 0 ? boundaryError : boundaryError.left(separator);
    const QString message = userMessageFor(code);
    if (m_errorCode != code || m_error != message) {
        m_errorCode = code;
        m_error = message;
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
        nextImportedPartNames.append(QString::fromUtf8(imported.source_name));
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
        nextPartName = QString::fromUtf8(imported.source_name);
        nextAssetPath = projectDirectory.filePath(QString::fromUtf8(imported.asset));
        nextMeshType = QString::fromUtf8(imported.mesh_type);
        nextUnits = QString::fromUtf8(imported.units);
        nextTriangleCount = imported.triangle_count;
        const QString unitLabel = nextUnits == QStringLiteral("millimeters") ? QStringLiteral("mm")
                                  : nextUnits == QStringLiteral("centimeters")
                                      ? QStringLiteral("cm")
                                      : QStringLiteral("in");
        nextDimensions =
            format_dimensions(imported.size_x, imported.size_y, imported.size_z, unitLabel);
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
    try {
        applyImports(panta::ffi::project_service_imports(*m_service));
        refreshPlanSettings();
        // 网格内容只随文档（激活 / 快照变化）改变：由文档路径负责发射
        // meshChanged，避免与导入激活的发射重复。
        return true;
    } catch (const rust::Error& failure) {
        return fail(QString::fromUtf8(failure.what()));
    }
}

QString ProjectViewModel::userMessageFor(const QString& errorCode) {
    if (errorCode == QStringLiteral("project.empty_name") ||
        errorCode == QStringLiteral("project.invalid_name")) {
        return QStringLiteral("Enter a valid project name.");
    }
    if (errorCode == QStringLiteral("project.location_empty") ||
        errorCode == QStringLiteral("project.location_not_absolute")) {
        return QStringLiteral("Choose an absolute project location.");
    }
    if (errorCode == QStringLiteral("project.location_create_failed")) {
        return QStringLiteral("The project location could not be created.");
    }
    if (errorCode == QStringLiteral("project.file_missing")) {
        return QStringLiteral("The selected project file does not exist.");
    }
    if (errorCode == QStringLiteral("project.invalid_file")) {
        return QStringLiteral("Choose a .panta project file.");
    }
    if (errorCode == QStringLiteral("project.file_not_local")) {
        return QStringLiteral("Choose a project file on this device.");
    }
    if (errorCode == QStringLiteral("project.already_exists")) {
        return QStringLiteral("A project with this name already exists.");
    }
    if (errorCode == QStringLiteral("project.manifest_invalid") ||
        errorCode == QStringLiteral("project.unsupported_schema")) {
        return QStringLiteral("The selected project file is not supported.");
    }
    if (errorCode == QStringLiteral("project.no_project")) {
        return QStringLiteral("Open or create a project first.");
    }
    if (errorCode == QStringLiteral("project.command_invalid")) {
        return QStringLiteral("The project change is not valid.");
    }
    if (errorCode == QStringLiteral("project.import_file_missing")) {
        return QStringLiteral("The selected STL file does not exist.");
    }
    if (errorCode == QStringLiteral("project.import_invalid_file")) {
        return QStringLiteral("Choose an STL file to import.");
    }
    if (errorCode == QStringLiteral("project.import_unsupported_mesh_type") ||
        errorCode == QStringLiteral("project.import_unsupported_units")) {
        return QStringLiteral("Choose valid STL import options.");
    }
    if (errorCode == QStringLiteral("project.import_source_changed")) {
        return QStringLiteral("The STL file changed after preview. Select it again.");
    }
    if (errorCode == QStringLiteral("project.import_parse_failed")) {
        return QStringLiteral("The selected STL file could not be read.");
    }
    if (errorCode == QStringLiteral("project.import_asset_copy_failed")) {
        return QStringLiteral("The STL file could not be copied into the project.");
    }
    return QStringLiteral("The project operation could not be completed.");
}

bool ProjectViewModel::toBoundaryText(const QString& text, std::string* out, QString* error) {
    return PathHost::toBoundaryUtf8(text, out, error);
}

QVariantList ProjectViewModel::openDocuments() const {
    QVariantList documents;
    documents.reserve(m_documents.size());
    for (const auto& document : m_documents) {
        documents.append(QVariantMap{{QStringLiteral("id"), document.id},
                                     {QStringLiteral("kind"), document.kind},
                                     {QStringLiteral("state"), document.state},
                                     {QStringLiteral("title"), document.title},
                                     {QStringLiteral("message"), document.message}});
    }
    return documents;
}

QString ProjectViewModel::activeDocumentId() const { return m_activeDocumentId; }

QString ProjectViewModel::activeDocumentTitle() const {
    for (const auto& document : m_documents) {
        if (document.id == m_activeDocumentId) {
            return document.title;
        }
    }
    return QString{};
}

QStringList ProjectViewModel::importedPartIds() const { return m_importedPartIds; }

int ProjectViewModel::document_index(const QString& documentId) const {
    for (int index = 0; index < m_documents.size(); ++index) {
        if (m_documents[index].id == documentId) {
            return index;
        }
    }
    return -1;
}

bool ProjectViewModel::activate_ready_document(
    int index, std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot> loadedMesh) {
    if (index < 0 || index >= m_documents.size()) {
        return false;
    }
    const auto& document = m_documents[index];
    if (document.state != QStringLiteral("ready")) {
        return false;
    }
    const QString documentId = document.id;
    const bool isImport = document.kind == QStringLiteral("import");
    if (isImport &&
        !panta::ffi::project_service_activate_mesh_document(*m_service, documentId.toStdString())) {
        m_documents[index].state = QStringLiteral("unloaded");
        m_documents[index].message.clear();
        return true;
    }

    if (!isImport) {
        panta::ffi::project_service_deactivate_mesh_document(*m_service);
    }

    const bool activeChanged = m_activeDocumentId != documentId;
    if (activeChanged) {
        auto nextMesh = isImport
                            ? (loadedMesh ? std::move(loadedMesh) : pull_service_mesh(documentId))
                            : nullptr;
        if (isImport && !nextMesh) {
            // 保留旧视口；恢复与 UI 活动文档一致的 Rust pin。
            const int previous = document_index(m_activeDocumentId);
            if (previous >= 0 && m_documents[previous].kind == QStringLiteral("import")) {
                panta::ffi::project_service_activate_mesh_document(
                    *m_service, m_activeDocumentId.toStdString());
            } else {
                panta::ffi::project_service_deactivate_mesh_document(*m_service);
            }
            return false;
        }
        m_activeDocumentId = documentId;
        m_activeMesh = std::move(nextMesh);
    }

    const bool residencyChanged = sync_mesh_residency();
    if (activeChanged) {
        emit activeDocumentChanged();
        emit meshChanged();
    }
    return residencyChanged || activeChanged;
}

bool ProjectViewModel::sync_mesh_residency() {
    const auto residentIds = panta::ffi::project_service_resident_mesh_ids(*m_service);
    QSet<QString> residents;
    residents.reserve(static_cast<qsizetype>(residentIds.size()));
    for (std::size_t index = 0; index < residentIds.size(); ++index) {
        residents.insert(QString::fromUtf8(residentIds[index]));
    }

    bool changed = false;
    for (auto& document : m_documents) {
        if (document.kind != QStringLiteral("import") ||
            document.state == QStringLiteral("loading") ||
            document.state == QStringLiteral("failed")) {
            continue;
        }
        const QString state =
            residents.contains(document.id) ? QStringLiteral("ready") : QStringLiteral("unloaded");
        if (document.state != state) {
            document.state = state;
            document.message.clear();
            changed = true;
        }
    }
    return changed;
}

void ProjectViewModel::reset_documents() {
    m_documents.clear();
    m_activeMesh.reset();
    m_activationAttempts.clear();
    m_activationPoll.stop();
    m_documents.append({QString::fromLatin1(kWelcomeDocumentId), QStringLiteral("welcome"),
                        QStringLiteral("ready"), tr("Welcome"), QString{}});
    m_activeDocumentId = QString::fromLatin1(kWelcomeDocumentId);
    emit documentsChanged();
    emit activeDocumentChanged();
    emit meshChanged();
}

void ProjectViewModel::activateDocument(const QString& documentId) {
    const int index = document_index(documentId);
    if (index < 0) {
        return;
    }
    if (m_documents[index].state == QStringLiteral("unloaded")) {
        openImportRecord(documentId);
        return;
    }
    if (activate_ready_document(index)) {
        if (m_documents[index].state == QStringLiteral("unloaded")) {
            openImportRecord(documentId);
            return;
        }
        emit documentsChanged();
    }
}

void ProjectViewModel::closeDocument(const QString& documentId) {
    const int index = document_index(documentId);
    if (index < 0) {
        return;
    }
    // 先取消加载，再清理簿记；旧 attempt 的结果不能命中新开的同 ID 页签。
    if (m_documents[index].state == QStringLiteral("loading")) {
        if (const auto attempt = m_activationAttempts.constFind(documentId);
            attempt != m_activationAttempts.cend()) {
            panta::ffi::project_service_cancel_asset_activation(*m_service, attempt.value());
        }
    }
    m_activationAttempts.remove(documentId);
    if (m_documents[index].kind == QStringLiteral("import")) {
        panta::ffi::project_service_release_mesh_document(*m_service, documentId.toStdString());
    }
    if (m_activeDocumentId == documentId) {
        m_activeMesh.reset();
    }
    m_documents.remove(index);
    sync_mesh_residency();

    // 被关的是活动文档：按右邻优先找第一个就绪页签，否则回到 Welcome；
    // 全部关闭后活动 ID 为空，视口留白。
    if (m_activeDocumentId == documentId) {
        m_activeDocumentId.clear();
        int fallback = -1;
        for (int probe = index; probe < m_documents.size() && fallback < 0; ++probe) {
            if (m_documents[probe].state == QStringLiteral("ready")) {
                fallback = probe;
            }
        }
        for (int probe = index - 1; probe >= 0 && fallback < 0; --probe) {
            if (m_documents[probe].state == QStringLiteral("ready")) {
                fallback = probe;
            }
        }
        if (fallback >= 0) {
            activate_ready_document(fallback);
            if (m_documents[fallback].state == QStringLiteral("unloaded")) {
                m_documents[fallback].state = QStringLiteral("loading");
                m_documents[fallback].message.clear();
                begin_import_activation(m_documents[fallback].id);
            }
        } else {
            panta::ffi::project_service_deactivate_mesh_document(*m_service);
            emit activeDocumentChanged();
            emit meshChanged();
            // 若其他页签都因预算被逐出，关闭活动页签后异步恢复最近的邻页。
            int reload = -1;
            for (int probe = index; probe < m_documents.size() && reload < 0; ++probe) {
                if (m_documents[probe].state == QStringLiteral("unloaded")) {
                    reload = probe;
                }
            }
            for (int probe = index - 1; probe >= 0 && reload < 0; --probe) {
                if (m_documents[probe].state == QStringLiteral("unloaded")) {
                    reload = probe;
                }
            }
            if (reload >= 0) {
                m_documents[reload].state = QStringLiteral("loading");
                m_documents[reload].message.clear();
                begin_import_activation(m_documents[reload].id);
            }
        }
    }
    sync_activation_poll();
    emit documentsChanged();
}

void ProjectViewModel::openImportRecord(const QString& recordId) {
    const int index = document_index(recordId);
    if (index >= 0) {
        // 已打开：就绪直接激活；加载中不重复请求；失败页签重新发起加载。
        if (m_documents[index].state == QStringLiteral("ready")) {
            const bool changed = activate_ready_document(index);
            if (m_documents[index].state == QStringLiteral("unloaded")) {
                m_documents[index].state = QStringLiteral("loading");
                m_documents[index].message.clear();
                emit documentsChanged();
                begin_import_activation(recordId);
            } else if (changed) {
                emit documentsChanged();
            }
        } else if (m_documents[index].state == QStringLiteral("failed") ||
                   m_documents[index].state == QStringLiteral("unloaded")) {
            m_documents[index].state = QStringLiteral("loading");
            m_documents[index].message.clear();
            emit documentsChanged();
            begin_import_activation(recordId);
        }
        return;
    }
    const QString title = m_importedPartNames.value(m_importedPartIds.indexOf(recordId));
    if (title.isEmpty()) {
        return;
    }
    m_documents.append(
        {recordId, QStringLiteral("import"), QStringLiteral("loading"), title, QString{}});
    emit documentsChanged();
    begin_import_activation(recordId);
}

void ProjectViewModel::begin_import_activation(const QString& recordId) {
    try {
        const auto attempt =
            panta::ffi::project_service_begin_asset_activation(*m_service, recordId.toStdString());
        m_activationAttempts.insert(recordId, attempt.attempt);
        sync_activation_poll();
    } catch (const rust::Error& failure) {
        qWarning("ProjectViewModel activation begin failed: %s", failure.what());
        const int index = document_index(recordId);
        if (index >= 0) {
            auto& document = m_documents[index];
            document.state = QStringLiteral("failed");
            document.message = activation_message_for(QString::fromUtf8(failure.what()));
            emit documentsChanged();
        }
    }
}

void ProjectViewModel::moveDocument(int fromIndex, int toIndex) {
    if (fromIndex < 0 || fromIndex >= m_documents.size() || toIndex < 0 ||
        toIndex >= m_documents.size() || fromIndex == toIndex) {
        return;
    }
    m_documents.move(fromIndex, toIndex);
    emit documentsChanged();
}

void ProjectViewModel::drain_activations() {
    try {
        const auto outcomes = panta::ffi::project_service_drain_asset_activations(*m_service);
        bool documentsChangedEmitted = false;
        const bool residencyMayHaveChanged = !outcomes.empty();
        for (const auto& outcome : outcomes) {
            const QString recordId = QString::fromUtf8(outcome.import_id);
            // 同一记录可在关闭后重新打开；旧 attempt 的迟到结果不能
            // 覆盖新页签，即使它们的 ImportRecord.id 相同。
            const auto attempt = m_activationAttempts.constFind(recordId);
            if (attempt == m_activationAttempts.cend() || attempt.value() != outcome.attempt) {
                const int index = document_index(recordId);
                if (index < 0) {
                    // 只有页签确已关闭才按 ID 释放；同 ID 的重开页签可能已有新 attempt 缓存。
                    panta::ffi::project_service_release_mesh_document(*m_service,
                                                                      recordId.toStdString());
                }
                continue;
            }
            const int index = document_index(recordId);
            if (index < 0) {
                panta::ffi::project_service_release_mesh_document(*m_service,
                                                                  recordId.toStdString());
                continue; // 页签已关闭：结果（含快照）就地丢弃。
            }
            if (outcome.kind == panta::ffi::ActivationOutcomeKind::Cancelled) {
                // 关闭页签引发的取消：页签已不在；此处兜底移除残留。
                m_activationAttempts.remove(recordId);
                m_documents.remove(index);
                documentsChangedEmitted = true;
                continue;
            }
            if (outcome.kind == panta::ffi::ActivationOutcomeKind::Expired) {
                continue; // 过期结果不驱动任何 UI 状态。
            }
            auto& document = m_documents[index];
            if (outcome.kind == panta::ffi::ActivationOutcomeKind::Succeeded) {
                auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
                mesh->vertices.reserve(outcome.coordinates.size() / 3);
                for (std::size_t offset = 0; offset + 2 < outcome.coordinates.size(); offset += 3) {
                    mesh->vertices.push_back({outcome.coordinates[offset],
                                              outcome.coordinates[offset + 1],
                                              outcome.coordinates[offset + 2]});
                }
                document.state = QStringLiteral("ready");
                document.message.clear();
                m_activationAttempts.remove(recordId);
                documentsChangedEmitted = true;
                // 激活仍由活动文档语义决定：成功后自动切到该文档。
                activate_ready_document(index, std::move(mesh));
            } else if (outcome.kind == panta::ffi::ActivationOutcomeKind::Failed) {
                document.state = QStringLiteral("failed");
                document.message = activation_message_for(QString::fromUtf8(outcome.code));
                m_activationAttempts.remove(recordId);
                documentsChangedEmitted = true;
            }
        }
        if (residencyMayHaveChanged) {
            documentsChangedEmitted = sync_mesh_residency() || documentsChangedEmitted;
        }
        if (documentsChangedEmitted) {
            emit documentsChanged();
        }
        sync_activation_poll();
    } catch (const rust::Error& failure) {
        qWarning("ProjectViewModel activation drain failed: %s", failure.what());
    }
}

void ProjectViewModel::sync_activation_poll() {
    bool loading = false;
    for (const auto& document : m_documents) {
        loading = loading || document.state == QStringLiteral("loading");
    }
    if (loading && !m_activationPoll.isActive()) {
        m_activationPoll.start();
    } else if (!loading && m_activationPoll.isActive()) {
        m_activationPoll.stop();
    }
}

} // namespace panta::bridge
