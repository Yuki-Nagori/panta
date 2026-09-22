#include "path_host.hpp"

#include "panta_ffi.h"
#include "rust/cxx.h"
#include <QByteArray>
#include <QDir>
#include <QLatin1String>
#include <QStandardPaths>
#include <QTemporaryFile>
#include <QUrl>
#include <QtCore/qnamespace.h>
#include <cstddef>
#include <functional>
#include <iterator>
#include <memory>
#include <string>
#include <vector>

namespace panta::bridge {

std::unique_ptr<PathHost> PathHost::create(QString* error) {
    struct Location {
        QStandardPaths::StandardLocation location;
        panta::ffi::PathRootKind kind;
        const char* label;
    };
    const Location locations[] = {
        {QStandardPaths::AppConfigLocation, panta::ffi::PathRootKind::UserConfig, "user-config"},
        {QStandardPaths::AppDataLocation, panta::ffi::PathRootKind::AppData, "app-data"},
        {QStandardPaths::CacheLocation, panta::ffi::PathRootKind::Cache, "cache"},
        {QStandardPaths::TempLocation, panta::ffi::PathRootKind::Session, "session"},
    };
    std::vector<StandardRoot> roots;
    roots.reserve(std::size(locations));
    for (const auto& entry : locations) {
        roots.push_back(StandardRoot{entry.kind, QStandardPaths::writableLocation(entry.location),
                                     QLatin1String(entry.label)});
    }
    return createWithStandardRoots(roots, error);
}

std::unique_ptr<PathHost> PathHost::createWithStandardRoots(const std::vector<StandardRoot>& roots,
                                                            QString* error) {
    std::unique_ptr<PathHost> host(new PathHost());
    for (const auto& root : roots) {
        if (root.directory.isEmpty() || !QDir::isAbsolutePath(root.directory)) {
            if (error != nullptr) {
                *error = QStringLiteral("path.standard_dir_unavailable: %1").arg(root.label);
            }
            return nullptr;
        }
        // 标准目录由宿主在注入时确保可用：缺失即创建（首次启动建立布局），
        // 写探针验证真实可写；探针临时文件析构即清理，不残留。
        if (!QDir(root.directory).exists() && !QDir().mkpath(root.directory)) {
            if (error != nullptr) {
                *error = QStringLiteral("path.standard_dir_unavailable: %1").arg(root.label);
            }
            return nullptr;
        }
        QTemporaryFile probe(root.directory + QStringLiteral("/.panta-write-probe-XXXXXX"));
        if (!probe.open()) {
            if (error != nullptr) {
                *error = QStringLiteral("path.standard_dir_unwritable: %1").arg(root.label);
            }
            return nullptr;
        }

        std::string utf8;
        if (!toBoundaryUtf8(root.directory, &utf8, error)) {
            return nullptr;
        }
        try {
            panta::ffi::path_service_set_root(*host->m_service, root.kind, utf8);
        } catch (const rust::Error& failure) {
            if (error != nullptr) {
                *error = QString::fromUtf8(failure.what());
            }
            return nullptr;
        }
    }
    return host;
}

bool PathHost::setProjectRoot(const QString& root, QString* error) {
    std::string utf8;
    if (!toBoundaryUtf8(root, &utf8, error)) {
        return false;
    }
    try {
        panta::ffi::path_service_set_root(*m_service, panta::ffi::PathRootKind::Project, utf8);
    } catch (const rust::Error& failure) {
        if (error != nullptr) {
            *error = QString::fromUtf8(failure.what());
        }
        return false;
    }
    return true;
}

QString PathHost::callResolve(const QString& reference,
                              const std::function<QString(const panta::ffi::PathRef&)>& resolver,
                              QString* error) const {
    std::string utf8;
    if (!toBoundaryUtf8(reference, &utf8, error)) {
        return {};
    }
    try {
        const panta::ffi::PathRef parsed = panta::ffi::path_ref_parse(utf8);
        return resolver(parsed);
    } catch (const rust::Error& failure) {
        if (error != nullptr) {
            *error = QString::fromUtf8(failure.what());
        }
        return {};
    }
}

QString PathHost::resolve(const QString& reference, QString* error) const {
    return callResolve(
        reference,
        [this](const panta::ffi::PathRef& parsed) {
            return QString::fromUtf8(panta::ffi::path_service_resolve(*m_service, parsed));
        },
        error);
}

QString PathHost::resolveExisting(const QString& reference, QString* error) const {
    return callResolve(
        reference,
        [this](const panta::ffi::PathRef& parsed) {
            return QString::fromUtf8(panta::ffi::path_service_resolve_existing(*m_service, parsed));
        },
        error);
}

QString PathHost::resolveWriteTarget(const QString& reference, QString* error) const {
    return callResolve(
        reference,
        [this](const panta::ffi::PathRef& parsed) {
            return QString::fromUtf8(
                panta::ffi::path_service_resolve_write_target(*m_service, parsed));
        },
        error);
}

QString PathHost::fileUrlToPath(const QUrl& url, QString* error) {
    if (!url.isValid() || url.scheme().compare(QStringLiteral("file"), Qt::CaseInsensitive) != 0) {
        if (error != nullptr) {
            *error = QStringLiteral("path.not_file_url: %1").arg(url.toString());
        }
        return {};
    }
    // toLocalFile 恰好解码一次；qrc/其它 scheme 不经此入口成为本机路径。
    QString path = url.toLocalFile();
    if (path.isEmpty()) {
        if (error != nullptr) {
            *error = QStringLiteral("path.file_url_without_local_path: %1").arg(url.toString());
        }
        return {};
    }
    return path;
}

bool PathHost::toBoundaryUtf8(const QString& text, std::string* out, QString* error) {
    const QByteArray utf8 = text.toUtf8();
    // 有损转换（未配对代理项等）在往返比较中暴露：当前契约只支持可往返
    // Unicode 路径，拒绝而不是替换字符后访问其它文件。
    if (QString::fromUtf8(utf8) != text) {
        if (error != nullptr) {
            *error = QStringLiteral("path.non_unicode");
        }
        return false;
    }
    if (out != nullptr) {
        *out = std::string(utf8.constData(), static_cast<std::size_t>(utf8.size()));
    }
    return true;
}

} // namespace panta::bridge
