/// 工作区网格与两阶段分析的 Qt 投影。Rust 拥有作业，VTK 消费本地显示快照。
#pragma once

#include "panta_ffi.h"
#include "project_view_model.hpp"
#include <QElapsedTimer>
#include <QPointer>
#include <QTimer>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>
#include <memory>
#include <panta/visualization/mesh_source.hpp>

namespace panta::bridge {
class AnalysisModel : public panta::visualization::MeshSource {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(panta::bridge::ProjectViewModel* projectModel READ projectModel WRITE setProjectModel
                   NOTIFY contextChanged)
    Q_PROPERTY(QString operation READ operation NOTIFY stateChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY stateChanged)
    Q_PROPERTY(bool meshReady READ meshReady NOTIFY stateChanged)
    Q_PROPERTY(bool gateReady READ gateReady NOTIFY stateChanged)
    Q_PROPERTY(bool animationAvailable READ animationAvailable NOTIFY stateChanged)
    Q_PROPERTY(bool resultReady READ resultReady NOTIFY stateChanged)
    Q_PROPERTY(QString error READ error NOTIFY stateChanged)
    Q_PROPERTY(QVariantList logRuns READ logRuns NOTIFY stateChanged)
    Q_PROPERTY(QVariantList studies READ studies NOTIFY stateChanged)
    Q_PROPERTY(QVariantList openDocuments READ openDocuments NOTIFY documentsChanged)
    Q_PROPERTY(QString activeDocumentId READ activeDocumentId NOTIFY documentsChanged)
    Q_PROPERTY(QString activeDocumentTitle READ activeDocumentTitle NOTIFY documentsChanged)
    Q_PROPERTY(double duration READ duration NOTIFY stateChanged)
    Q_PROPERTY(double playbackTime READ playbackTime WRITE setPlaybackTime NOTIFY playbackChanged)
    Q_PROPERTY(bool playing READ playing NOTIFY playbackChanged)

  public:
    explicit AnalysisModel(QObject* parent = nullptr);
    [[nodiscard]] ProjectViewModel* projectModel() const { return m_project; }
    void setProjectModel(ProjectViewModel* project);
    [[nodiscard]] QString operation() const { return m_operation; }
    [[nodiscard]] bool busy() const { return m_busy; }
    [[nodiscard]] bool meshReady() const { return m_meshReady; }
    [[nodiscard]] bool gateReady() const { return m_gateReady; }
    [[nodiscard]] bool animationAvailable() const {
        return m_resultReady && !m_busy && m_mesh && m_mesh->fields_visible &&
               !m_mesh->pressure_visible;
    }
    [[nodiscard]] bool resultReady() const { return m_resultReady; }
    [[nodiscard]] QString error() const { return m_error; }
    [[nodiscard]] QVariantList logRuns() const { return m_logRuns; }
    [[nodiscard]] QVariantList studies() const { return m_studies; }
    [[nodiscard]] QVariantList openDocuments() const;
    [[nodiscard]] QString activeDocumentId() const;
    [[nodiscard]] QString activeDocumentTitle() const;
    [[nodiscard]] double duration() const { return m_duration; }
    [[nodiscard]] double playbackTime() const { return m_playbackTime; }
    [[nodiscard]] bool playing() const { return m_playTimer.isActive(); }
    [[nodiscard]] std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
    mesh_snapshot() const override;
    [[nodiscard]] bool placeholder_visible() const override;
    /// 针对当前零件冻结已确认参数；单作业执行，失败可重试。
    Q_INVOKABLE void generateMesh(double edgeLength);
    Q_INVOKABLE void analyze();
    Q_INVOKABLE void cancel();
    /// resultId 为 fill-time 或 pressure；只选择已有真实字段。
    Q_INVOKABLE void selectResult(const QString& resultId);
    Q_INVOKABLE void play();
    Q_INVOKABLE void pause();
    Q_INVOKABLE void stop();
    void setPlaybackTime(double seconds);
    Q_INVOKABLE void activateDocument(const QString& documentId);
    Q_INVOKABLE void closeDocument(const QString& documentId);
    Q_INVOKABLE void moveDocument(int fromIndex, int toIndex);
    Q_INVOKABLE void openImportRecord(const QString& recordId);

  signals:
    void stateChanged();
    void playbackChanged();
    void contextChanged();
    void documentsChanged();
    void runStarted();

  private:
    void syncContext();
    void start(const QString& operation, double edgeLength);
    void poll();
    void updateRun(const QString& text);
    QPointer<ProjectViewModel> m_project;
    QString m_context;
    QString m_importId;
    QString m_activeStudy;
    QString m_operation;
    QString m_phase;
    QString m_error;
    QString m_console;
    QString m_summary;
    QString m_outputDir;
    QVariantList m_logRuns;
    QVariantList m_studies;
    bool m_busy = false;
    bool m_meshReady = false;
    bool m_gateReady = false;
    bool m_resultReady = false;
    double m_edgeLength = 12.0;
    double m_progress = -1;
    double m_duration = 0;
    double m_playbackTime = 0;
    QElapsedTimer m_clock;
    QElapsedTimer m_playClock;
    QTimer m_pollTimer;
    QTimer m_playTimer;
    std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot> m_mesh;
    rust::Box<panta::ffi::AnalysisService> m_service;
};
} // namespace panta::bridge
