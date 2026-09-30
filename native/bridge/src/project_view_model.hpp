/// 工程服务的 Qt 适配层与运行期视口文档状态。
///
/// Rust application service 持有工程清单、导入记录和异步资产激活；
/// 本类型持有当前打开的页签、活动 ID 与显示快照，并向 QML/视口投影状态。
#pragma once

#include "panta_ffi.h"
#include <QMap>
#include <QSet>
#include <QStringList>
#include <QTimer>
#include <QUrl>
#include <QVariantList>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>
#include <cstdint>
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
    Q_PROPERTY(QStringList importedPartNames READ importedPartNames NOTIFY importsChanged)
    Q_PROPERTY(QStringList importedPartIds READ importedPartIds NOTIFY importsChanged)
    Q_PROPERTY(QString importedPartName READ importedPartName NOTIFY importsChanged)
    Q_PROPERTY(QString importedAssetPath READ importedAssetPath NOTIFY importsChanged)
    Q_PROPERTY(QString importedMeshType READ importedMeshType NOTIFY importsChanged)
    Q_PROPERTY(QString importedUnits READ importedUnits NOTIFY importsChanged)
    Q_PROPERTY(QString importedDimensions READ importedDimensions NOTIFY importsChanged)
    Q_PROPERTY(quint64 importedTriangleCount READ importedTriangleCount NOTIFY importsChanged)
    Q_PROPERTY(QVariantList meshTypes READ meshTypes CONSTANT)
    Q_PROPERTY(QString defaultMeshType READ defaultMeshType CONSTANT)
    Q_PROPERTY(QVariantMap defaultMaterial READ defaultMaterial CONSTANT)
    Q_PROPERTY(bool materialConfirmationPending READ materialConfirmationPending NOTIFY
                   materialConfirmationPendingChanged)
    Q_PROPERTY(QVariantList analysisSequences READ analysisSequences CONSTANT)
    Q_PROPERTY(bool fillSettingsConfirmationPending READ fillSettingsConfirmationPending NOTIFY
                   fillSettingsConfirmationPendingChanged)
    Q_PROPERTY(QVariantMap planSettings READ planSettings NOTIFY planSettingsChanged)
    Q_PROPERTY(bool importPreviewReady READ importPreviewReady NOTIFY importPreviewChanged)
    Q_PROPERTY(QString importPreviewName READ importPreviewName NOTIFY importPreviewChanged)
    Q_PROPERTY(
        QString importPreviewDimensions READ importPreviewDimensions NOTIFY importPreviewChanged)
    Q_PROPERTY(quint64 importPreviewTriangleCount READ importPreviewTriangleCount NOTIFY
                   importPreviewChanged)
    Q_PROPERTY(QVariantList openDocuments READ openDocuments NOTIFY documentsChanged)
    Q_PROPERTY(QString activeDocumentId READ activeDocumentId NOTIFY activeDocumentChanged)
    Q_PROPERTY(QString activeDocumentTitle READ activeDocumentTitle NOTIFY activeDocumentChanged)

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
    /// 名称按 Rust 工程导入记录顺序排列；QML 只负责呈现，不派生领域状态。
    [[nodiscard]] const QStringList& importedPartNames() const;
    [[nodiscard]] QStringList importedPartIds() const;
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

    [[nodiscard]] QVariantList meshTypes() const;
    [[nodiscard]] QString defaultMeshType() const;
    [[nodiscard]] QVariantMap defaultMaterial() const;
    [[nodiscard]] bool materialConfirmationPending() const;
    [[nodiscard]] bool fillSettingsConfirmationPending() const;
    [[nodiscard]] QVariantList analysisSequences() const; // Rust 目录：ID 与英文源文案。
    [[nodiscard]] QVariantMap planSettings() const;       // Rust 当前方案快照。

    [[nodiscard]] QVariantList openDocuments() const; // 视口文档的只读投影。
    [[nodiscard]] QString activeDocumentId() const;
    [[nodiscard]] QString activeDocumentTitle() const;
    [[nodiscard]] bool placeholder_visible() const override;

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

    /// 确认打开弹窗时的方案选择；Rust 校验工程身份与修订并事务保存。
    /// 失败保留已确认值，错误由 error/errorCode 提供。
    Q_INVOKABLE bool setAnalysisSequence(const QString& projectPath, quint64 revision,
                                         const QString& importId, const QString& sequenceId);

    /// 启动 Rust 后台材料确认；返回值表示请求被接受，不表示已保存。
    /// 完成由 materialConfirmationFinished 通知；失败保留旧配置。
    Q_INVOKABLE bool setMaterial(const QString& projectPath, quint64 revision,
                                 const QString& importId, const QString& materialId);

    /// 启动 Rust 后台工艺确认；返回请求是否被接受，完成由信号通知。
    /// 校验或写入失败保留旧设置，目标为弹窗打开时的工程修订和导入记录。
    Q_INVOKABLE bool setFillSettings(const QString& projectPath, quint64 revision,
                                     const QString& importId, const QVariantMap& settings);

    /// 激活一个就绪文档（Welcome 或已就绪导入页签）；Loading/Failed 文档
    /// 不可激活。同步 UI 操作，不经 Rust Flow。
    Q_INVOKABLE void activateDocument(const QString& documentId);

    /// 关闭文档：仅结束本次运行期视图，不删 ImportRecord / 资产、不设
    /// dirty。关闭 Loading 文档会先请求取消对应 attempt。
    Q_INVOKABLE void closeDocument(const QString& documentId);

    /// 工程树点击入口：已就绪记录直接激活；未打开记录创建 Loading 页签并
    /// 经 073 只读激活异步加载，成功后自动激活；失败保留 Failed 页签。
    Q_INVOKABLE void openImportRecord(const QString& recordId);

    /// 拖拽重排的提交入口；from/to 为 openDocuments 下标。
    Q_INVOKABLE void moveDocument(int fromIndex, int toIndex);

  signals:
    void errorChanged();
    void projectChanged();
    void projectCreated(const QString& path);
    void projectOpened(const QString& path);
    void projectSaved(const QString& path);
    void importsChanged();
    void planSettingsChanged();
    void materialConfirmationPendingChanged();
    void materialConfirmationFinished(bool succeeded);
    void fillSettingsConfirmationPendingChanged();
    void fillSettingsConfirmationFinished(bool succeeded);
    void projectImported(const QString& path);
    void importPreviewChanged();
    void documentsChanged();
    void activeDocumentChanged();

  private:
    /// 打开的视口文档条目；Welcome 与导入记录共用一套生命周期。
    struct DocumentEntry {
        QString id;
        QString kind;  // "welcome" | "import"
        QString state; // "ready" | "loading" | "failed" | "unloaded"
        QString title;
        QString message; // Failed 态的用户可读原因
    };

    bool fail(const QString& boundaryError);
    bool applySnapshot(const panta::ffi::ProjectSnapshot& snapshot);
    void applyImports(const rust::Vec<panta::ffi::ProjectImport>& imports);
    bool refreshImports();
    void refreshPlanSettings();
    enum class ConfirmationKind : std::uint8_t { None, Material, FillSettings };
    void finish_metadata_confirmation();
    void set_pending_confirmation(ConfirmationKind kind);
    static QString userMessageFor(const QString& errorCode);
    static bool toBoundaryText(const QString& text, std::string* out, QString* error);

    std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
    pull_service_mesh(const QString& importId) const;
    int document_index(const QString& documentId) const;
    bool activate_ready_document(
        int index,
        std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot> loadedMesh = nullptr);
    bool sync_mesh_residency();
    void reset_documents();
    void begin_import_activation(const QString& recordId);
    void drain_activations();
    void sync_activation_poll();

    QString m_defaultLocation;
    const QVariantList m_analysisSequences;
    const QVariantList m_meshTypes;
    const QString m_defaultMeshType;
    QVariantMap m_defaultMaterial;
    QString m_error;
    QString m_errorCode;
    QString m_currentPath;
    QString m_currentName;
    QString m_lastCreatedPath;
    bool m_dirty = false;
    QStringList m_importedPartNames;
    QStringList m_importedPartIds;
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
    QVector<DocumentEntry> m_documents;
    QString m_activeDocumentId;
    /// Rust 当前方案快照的 Qt 投影，不拥有独立领域选择。
    QVariantMap m_planSettings;
    /// 当前活动文档的 C++ 显示 DTO；Rust ProjectService 拥有所有驻留 Mesh 与缓存策略。
    std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot> m_activeMesh;
    QMap<QString, quint64> m_activationAttempts;
    QTimer m_activationPoll;
    QTimer m_metadataConfirmationPoll;
    ConfirmationKind m_pendingConfirmation = ConfirmationKind::None;
    rust::Box<panta::ffi::ProjectService> m_service;
};

} // namespace panta::bridge
