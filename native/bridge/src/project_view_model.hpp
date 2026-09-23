/// 工程创建、打开和导入流程的 Qt 适配层。
///
/// 该类型只负责平台目录发现、Qt 字符串/URL 适配和用户可读错误；工程清单、
/// 工程模型及创建/打开/保存命令由 Rust application service 持有。
#pragma once

#include "panta_ffi.h"
#include <QUrl>
#include <QtQml/qqmlregistration.h>
#include <panta/visualization/mesh_source.hpp>
#include <rust/cxx.h>

namespace panta::bridge {

class ProjectViewModel : public panta::visualization::MeshSource {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString defaultLocation READ defaultLocation CONSTANT)
    Q_PROPERTY(QUrl defaultLocationUrl READ defaultLocationUrl CONSTANT)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    Q_PROPERTY(QString errorCode READ errorCode NOTIFY errorChanged)
    Q_PROPERTY(QString currentPath READ currentPath NOTIFY projectChanged)
    Q_PROPERTY(QString currentName READ currentName NOTIFY projectChanged)
    Q_PROPERTY(bool dirty READ dirty NOTIFY projectChanged)
    Q_PROPERTY(QString lastCreatedPath READ lastCreatedPath NOTIFY projectCreated)
    Q_PROPERTY(bool hasImportedPart READ hasImportedPart NOTIFY importsChanged)
    Q_PROPERTY(QString importedPartName READ importedPartName NOTIFY importsChanged)
    Q_PROPERTY(QString importedAssetPath READ importedAssetPath NOTIFY importsChanged)
    Q_PROPERTY(QString importedMeshType READ importedMeshType NOTIFY importsChanged)
    Q_PROPERTY(QString importedUnits READ importedUnits NOTIFY importsChanged)
    Q_PROPERTY(QString importedDimensions READ importedDimensions NOTIFY importsChanged)
    Q_PROPERTY(quint64 importedTriangleCount READ importedTriangleCount NOTIFY importsChanged)
    Q_PROPERTY(bool importPreviewReady READ importPreviewReady NOTIFY importPreviewChanged)
    Q_PROPERTY(QString importPreviewName READ importPreviewName NOTIFY importPreviewChanged)
    Q_PROPERTY(
        QString importPreviewDimensions READ importPreviewDimensions NOTIFY importPreviewChanged)
    Q_PROPERTY(quint64 importPreviewTriangleCount READ importPreviewTriangleCount NOTIFY
                   importPreviewChanged)

  public:
    explicit ProjectViewModel(QObject* parent = nullptr);
    [[nodiscard]] std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
    mesh_snapshot() const override;

    /// 默认工程目录为平台 Documents 目录下的 panta 子目录；不会在构造时写盘。
    [[nodiscard]] const QString& defaultLocation() const;
    [[nodiscard]] QUrl defaultLocationUrl() const;

    /// 最近一次创建成功的工程目录；失败时保留上一成功路径。
    [[nodiscard]] const QString& lastCreatedPath() const;

    /// 最近一次可恢复失败的用户可读摘要；空串表示无错误。
    [[nodiscard]] const QString& error() const;
    [[nodiscard]] const QString& errorCode() const;

    [[nodiscard]] const QString& currentPath() const;
    [[nodiscard]] const QString& currentName() const;
    [[nodiscard]] bool dirty() const;
    [[nodiscard]] bool hasImportedPart() const;
    [[nodiscard]] const QString& importedPartName() const;
    [[nodiscard]] const QString& importedAssetPath() const;
    [[nodiscard]] const QString& importedMeshType() const;
    [[nodiscard]] const QString& importedUnits() const;
    [[nodiscard]] const QString& importedDimensions() const;
    [[nodiscard]] quint64 importedTriangleCount() const;
    [[nodiscard]] bool importPreviewReady() const;
    [[nodiscard]] const QString& importPreviewName() const;
    [[nodiscard]] const QString& importPreviewDimensions() const;
    [[nodiscard]] quint64 importPreviewTriangleCount() const;

    /// 通过 Rust 工程服务创建目录和初始清单；工程资产由后续命令负责。
    ///
    /// 成功时发出 projectCreated(path) 并返回 true；失败时保留输入目录，
    /// 设置 error 并返回 false。调用发生在 GUI 线程，操作只包含短小的目录
    /// 创建和元数据检查，不执行工程解析或大型文件读写。
    Q_INVOKABLE bool createProject(const QString& name, const QString& location);

    /// 清除当前错误，供对话框每次打开时重置显示状态。
    Q_INVOKABLE void clearError();

    /// 打开已有工程并读取 Rust 清单。
    Q_INVOKABLE bool openProject(const QString& path);

    /// 将 Qt 文件 URL 转成本地路径并打开；QML 不自行解析 URL 字符串。
    Q_INVOKABLE bool openProjectUrl(const QUrl& url);

    /// FolderDialog/FileDialog 共用的 URL 适配；非本地 URL 返回空串。
    Q_INVOKABLE QString localPath(const QUrl& url) const;

    /// 保存当前 Rust 工程模型；无当前工程时返回可恢复错误。
    Q_INVOKABLE bool saveProject();

    /// 示例模型命令：修改显示名称并将工程标记为 dirty。
    Q_INVOKABLE bool renameProject(const QString& name);

    /// 读取、复制并记录 STL；源文件的绝对路径只作为一次性输入，不写入清单。
    Q_INVOKABLE bool importStl(const QString& path, const QString& meshType, const QString& units,
                               bool showImportLog);

    /// 预检 STL 元数据供导入对话框展示，不改变当前工程。
    Q_INVOKABLE bool inspectStl(const QString& path);

  signals:
    void errorChanged();
    void projectChanged();
    void projectCreated(const QString& path);
    void projectOpened(const QString& path);
    void projectSaved(const QString& path);
    void importsChanged();
    void projectImported(const QString& path);
    void importPreviewChanged();

  private:
    bool fail(const QString& boundaryError);
    bool applySnapshot(const panta::ffi::ProjectSnapshot& snapshot);
    // 首期 QML 只展示最新导入项；Rust 工程服务仍保留完整记录列表。
    void applyImports(const rust::Vec<panta::ffi::ProjectImport>& imports);
    bool refreshImports();
    static QString userMessageFor(const QString& errorCode);
    static bool toBoundaryText(const QString& text, std::string* out, QString* error);

    QString m_defaultLocation;
    QString m_error;
    QString m_errorCode;
    QString m_currentPath;
    QString m_currentName;
    QString m_lastCreatedPath;
    bool m_dirty = false;
    bool m_hasImportedPart = false;
    QString m_importedPartName;
    QString m_importedAssetPath;
    QString m_importedMeshType;
    QString m_importedUnits;
    QString m_importedDimensions;
    quint64 m_importedTriangleCount = 0;
    bool m_importPreviewReady = false;
    QString m_importPreviewName;
    QString m_importPreviewDimensions;
    quint64 m_importPreviewTriangleCount = 0;
    rust::Box<panta::ffi::ProjectService> m_service;
};

} // namespace panta::bridge
