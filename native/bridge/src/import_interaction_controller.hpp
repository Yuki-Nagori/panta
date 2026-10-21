/// 持有导入弹窗的交互状态与可恢复表单草稿；工程操作仍由 ProjectViewModel/Rust 服务裁决。
#pragma once

#include <QObject>
#include <QStateMachine>
#include <QString>
#include <QtQml/qqmlregistration.h>
#include <cstdint>

namespace panta::bridge {

class ProjectViewModel;

class ImportInteractionController final : public QObject {
    Q_OBJECT
    QML_ANONYMOUS
    Q_PROPERTY(QString stateName READ stateName NOTIFY stateChanged)
    Q_PROPERTY(QString sourcePath READ sourcePath NOTIFY formChanged)
    Q_PROPERTY(QString meshType READ meshType WRITE setMeshType NOTIFY formChanged)
    Q_PROPERTY(QString units READ units WRITE setUnits NOTIFY formChanged)
    Q_PROPERTY(bool showImportLog READ showImportLog WRITE setShowImportLog NOTIFY formChanged)
    Q_PROPERTY(bool canEdit READ canEdit NOTIFY stateChanged)
    Q_PROPERTY(bool canSubmit READ canSubmit NOTIFY stateChanged)

  public:
    enum class State : std::uint8_t {
        Closed,
        Editing,
        ChoosingFile,
        Previewing,
        Ready,
        Submitting,
        ProjectionRefreshFailed
    };
    Q_ENUM(State)

    explicit ImportInteractionController(ProjectViewModel* project, QObject* parent = nullptr);

    [[nodiscard]] QString stateName() const;
    [[nodiscard]] const QString& sourcePath() const { return m_sourcePath; }
    [[nodiscard]] const QString& meshType() const { return m_meshType; }
    [[nodiscard]] const QString& units() const { return m_units; }
    [[nodiscard]] bool showImportLog() const { return m_showImportLog; }
    [[nodiscard]] bool canEdit() const;
    [[nodiscard]] bool canSubmit() const;
    [[nodiscard]] State state() const { return m_state; }

    void setMeshType(const QString& value);
    void setUnits(const QString& value);
    void setShowImportLog(bool value);

    Q_INVOKABLE bool open();
    Q_INVOKABLE bool close();
    Q_INVOKABLE quint64 beginFileSelection();
    Q_INVOKABLE bool cancelFileSelection(quint64 selectionId);
    Q_INVOKABLE bool previewFile(const QString& path);
    Q_INVOKABLE bool previewFileForSelection(const QString& path, quint64 selectionId);
    Q_INVOKABLE void detachWindow();
    Q_INVOKABLE bool submit();

  signals:
    void stateChanged();
    void formChanged();
    void closed();

    void openRequested();
    void closeRequested();
    void chooseFileRequested();
    void fileSelectionAccepted();
    void fileSelectionCancelledToEditing();
    void fileSelectionCancelledToReady();
    void previewRequested();
    void previewSucceeded();
    void previewFailed();
    void submitRequested();
    void submitRejected();
    void importCommitted();
    void importFailed();
    void projectionRefreshFailed();

  private:
    void setState(State state);
    void finishStateEntry(State state);
    void startRequestedState();
    void onPreviewFinished(quint64 requestId, bool succeeded);
    void onImportFinished(quint64 requestId, bool operationSucceeded, bool projectionUpdated);

    ProjectViewModel* m_project;
    QStateMachine m_machine;
    State m_state = State::Closed;
    State m_stateBeforeFileSelection = State::Editing;
    QString m_sourcePath;
    QString m_meshType;
    QString m_units = QStringLiteral("millimeters");
    bool m_showImportLog = true;
    bool m_machineStarted = false;
    bool m_initialStateEntered = false;
    bool m_transitionPending = false;
    bool m_closeAfterTransition = false;
    bool m_closeTransitionPending = false;
    bool m_fileSelectionTransitionPending = false;
    bool m_fileSelectionDetached = false;
    quint64 m_sessionGeneration = 0;
    quint64 m_fileSelectionGeneration = 0;
    quint64 m_previewRequestId = 0;
    quint64 m_importRequestId = 0;
    quint64 m_importSessionGeneration = 0;
};

} // namespace panta::bridge
