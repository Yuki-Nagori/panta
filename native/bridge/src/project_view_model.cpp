#include "project_view_model.hpp"

#include "panta/visualization/mesh_source.hpp"
#include "panta_ffi.h"
#include "path_host.hpp"
#include <QDir>
#include <QFileInfo>
#include <QLatin1Char>
#include <QObject>
#include <QStandardPaths>
#include <QString>
#include <QUrl>
#include <QtCore/qtmetamacros.h>
#include <cstddef>
#include <memory>
#include <qlogging.h>
#include <rust/cxx.h>
#include <string>

namespace panta::bridge {

namespace {

QString default_project_location() {
    const QString documents = QStandardPaths::writableLocation(QStandardPaths::DocumentsLocation);
    return documents.isEmpty() ? QString{} : QDir(documents).filePath(QStringLiteral("panta"));
}

QString format_dimensions(double sizeX, double sizeY, double sizeZ, const QString& unit = {}) {
    const QString values = QStringLiteral("%1 × %2 × %3")
                               .arg(QString::number(sizeX, 'f', 2), QString::number(sizeY, 'f', 2),
                                    QString::number(sizeZ, 'f', 2));
    return unit.isEmpty() ? values : values + QLatin1Char(' ') + unit;
}

} // namespace

ProjectViewModel::ProjectViewModel(QObject* parent)
    : MeshSource(parent), m_defaultLocation(default_project_location()),
      m_service(panta::ffi::project_service_new()) {}

std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
ProjectViewModel::mesh_snapshot() const {
    if (m_currentPath.isEmpty()) {
        return nullptr;
    }
    try {
        const auto snapshot = panta::ffi::project_service_mesh_snapshot(*m_service);
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
        panta::ffi::project_service_import_stl(*m_service, path, meshType, units, showImportLog);
        const auto snapshot = panta::ffi::project_service_current(*m_service);
        if (!applySnapshot(snapshot) || !refreshImports()) {
            return false;
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
    const QDir projectDirectory(QFileInfo(m_currentPath).absolutePath());
    for (std::size_t index = 0; index < imports.size(); ++index) {
        const auto& imported = imports[index];
        nextImportedPartNames.append(QString::fromUtf8(imported.source_name));
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
        m_importedPartNames != nextImportedPartNames || m_importedPartName != nextPartName ||
        m_importedAssetPath != nextAssetPath || m_importedMeshType != nextMeshType ||
        m_importedUnits != nextUnits || m_importedDimensions != nextDimensions ||
        m_importedTriangleCount != nextTriangleCount;
    m_importedPartNames = std::move(nextImportedPartNames);
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
        emit meshChanged();
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

} // namespace panta::bridge
