/// 固定充填演示的 Qt 显示状态。仅 GUI 线程调用；Rust 持有作业和外部进程。
#pragma once

#include "panta_ffi.h"
#include <QElapsedTimer>
#include <QString>
#include <QTimer>
#include <QtQml/qqmlregistration.h>
#include <memory>
#include <panta/visualization/mesh_source.hpp>

namespace panta::bridge {

class FillingPreviewModel : public panta::visualization::MeshSource {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(int step READ step NOTIFY stateChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY stateChanged)
    Q_PROPERTY(QString message READ message NOTIFY stateChanged)
    Q_PROPERTY(QString error READ error NOTIFY stateChanged)
    Q_PROPERTY(QString outputDir READ outputDir NOTIFY stateChanged)
    Q_PROPERTY(double progress READ progress NOTIFY stateChanged)
    Q_PROPERTY(double elapsed READ elapsed NOTIFY stateChanged)
    Q_PROPERTY(double duration READ duration NOTIFY stateChanged)
    Q_PROPERTY(double peakPressure READ peakPressure NOTIFY stateChanged)
    Q_PROPERTY(int triangles READ triangles NOTIFY stateChanged)
    Q_PROPERTY(double playbackTime READ playbackTime WRITE setPlaybackTime NOTIFY playbackChanged)
    Q_PROPERTY(bool playing READ playing NOTIFY playbackChanged)

  public:
    explicit FillingPreviewModel(QObject* parent = nullptr);
    ~FillingPreviewModel() override;
    [[nodiscard]] int step() const { return m_step; }
    [[nodiscard]] bool busy() const { return m_busy; }
    [[nodiscard]] QString message() const { return m_message; }
    [[nodiscard]] QString error() const { return m_error; }
    [[nodiscard]] QString outputDir() const { return m_outputDir; }
    [[nodiscard]] double progress() const { return m_progress; }
    [[nodiscard]] double elapsed() const { return m_elapsed; }
    [[nodiscard]] double duration() const { return m_duration; }
    [[nodiscard]] double peakPressure() const { return m_peakPressure; }
    [[nodiscard]] int triangles() const;
    [[nodiscard]] double playbackTime() const { return m_playbackTime; }
    [[nodiscard]] bool playing() const { return m_playTimer.isActive(); }
    [[nodiscard]] std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot>
    mesh_snapshot() const override {
        return m_mesh;
    }
    [[nodiscard]] bool placeholder_visible() const override { return false; }

    /// 前进一步或重试失败步骤；忙时忽略。重置只在空闲时释放本地快照。
    Q_INVOKABLE void advance();
    Q_INVOKABLE void cancel();
    Q_INVOKABLE void reset();
    /// 本地回放控制，物理时间 s；整个结果按 10 秒墙钟时间播放。
    Q_INVOKABLE void togglePlayback();
    void setPlaybackTime(double seconds);

  signals:
    void stateChanged();
    void playbackChanged();

  private:
    void poll();
    rust::Box<panta::ffi::PreviewService> m_service;
    QTimer m_pollTimer;
    QTimer m_playTimer;
    QElapsedTimer m_clock;
    QElapsedTimer m_playClock;
    std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot> m_mesh;
    int m_step = 0;
    bool m_busy = false;
    QString m_message;
    QString m_error;
    QString m_outputDir;
    double m_progress = -1.0;
    double m_elapsed = 0.0;
    double m_duration = 0.0;
    double m_peakPressure = 0.0;
    double m_playbackTime = 0.0;
};

} // namespace panta::bridge
