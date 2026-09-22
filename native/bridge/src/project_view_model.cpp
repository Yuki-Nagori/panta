#include "project_view_model.hpp"

#include "panta_ffi.h"
#include "path_host.hpp"
#include <QDir>
#include <QLatin1Char>
#include <QObject>
#include <QStandardPaths>
#include <QString>
#include <QUrl>
#include <QtCore/qtmetamacros.h>
#include <rust/cxx.h>
#include <string>

namespace panta::bridge {

namespace {

QString default_project_location() {
    const QString documents = QStandardPaths::writableLocation(QStandardPaths::DocumentsLocation);
    return documents.isEmpty() ? QString{} : QDir(documents).filePath(QStringLiteral("panta"));
}

} // namespace

ProjectViewModel::ProjectViewModel(QObject* parent)
    : QObject(parent), m_defaultLocation(default_project_location()),
      m_service(panta::ffi::project_service_new()) {}

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
        return QStringLiteral("Only .panta project files can be opened.");
    }
    if (errorCode == QStringLiteral("project.file_not_local")) {
        return QStringLiteral("Choose a local .panta project file.");
    }
    if (errorCode == QStringLiteral("project.already_exists")) {
        return QStringLiteral("A project with this name already exists.");
    }
    if (errorCode == QStringLiteral("project.manifest_invalid") ||
        errorCode == QStringLiteral("project.unsupported_schema")) {
        return QStringLiteral("The selected folder is not a supported panta project.");
    }
    if (errorCode == QStringLiteral("project.no_project")) {
        return QStringLiteral("Open or create a project first.");
    }
    if (errorCode == QStringLiteral("project.command_invalid")) {
        return QStringLiteral("The project change is not valid.");
    }
    return QStringLiteral("The project operation could not be completed.");
}

bool ProjectViewModel::toBoundaryText(const QString& text, std::string* out, QString* error) {
    return PathHost::toBoundaryUtf8(text, out, error);
}

} // namespace panta::bridge
