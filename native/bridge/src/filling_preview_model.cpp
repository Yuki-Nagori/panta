#include "filling_preview_model.hpp"

#include "panta_ffi.h"
#include "rust/cxx.h"
#include <QObject>
#include <QString>
#include <QtCore/qtmetamacros.h>
#include <algorithm>
#include <cmath>
#include <cstddef>
#include <memory>
#include <panta/visualization/mesh_source.hpp>
#include <string_view>
#include <utility>

namespace panta::bridge {
namespace {
QString qt_string(const rust::String& text) {
    return QString::fromUtf8(std::string_view(text.data(), text.size()));
}
} // namespace

FillingPreviewModel::FillingPreviewModel(QObject* parent)
    : MeshSource(parent), m_service(panta::ffi::preview_service_new()), m_pollTimer(this),
      m_playTimer(this) {
    m_pollTimer.setInterval(100);
    connect(&m_pollTimer, &QTimer::timeout, this, &FillingPreviewModel::poll);
    m_playTimer.setInterval(50);
    connect(&m_playTimer, &QTimer::timeout, this, [this] {
        const double delta = static_cast<double>(m_playClock.restart()) / 10000.0 * m_duration;
        setPlaybackTime(m_playbackTime + delta);
    });
}
FillingPreviewModel::~FillingPreviewModel() = default;

int FillingPreviewModel::triangles() const {
    return m_mesh ? static_cast<int>(m_mesh->vertices.size() / 3) : 0;
}

void FillingPreviewModel::advance() {
    if (m_busy || m_step == 3) {
        return;
    }
    try {
        if (m_step == 0) {
            panta::ffi::preview_load(*m_service);
        } else if (m_step == 1) {
            panta::ffi::preview_remesh(*m_service);
        } else {
            panta::ffi::preview_fill(*m_service);
        }
        m_busy = true;
        m_error.clear();
        m_outputDir.clear();
        m_progress = -1.0;
        m_elapsed = 0.0;
        m_message = tr("Working with the default settings…");
        m_clock.start();
        m_pollTimer.start();
    } catch (const rust::Error& error) {
        m_error = QString::fromUtf8(error.what());
    }
    emit stateChanged();
}

void FillingPreviewModel::cancel() {
    if (m_busy) {
        panta::ffi::preview_cancel(*m_service);
    }
}

void FillingPreviewModel::reset() {
    if (m_busy) {
        return;
    }
    m_service = panta::ffi::preview_service_new();
    m_playTimer.stop();
    m_mesh.reset();
    m_step = 0;
    m_message.clear();
    m_error.clear();
    m_outputDir.clear();
    m_progress = -1.0;
    m_elapsed = 0;
    m_duration = 0;
    m_peakPressure = 0;
    m_playbackTime = 0;
    emit meshChanged();
    emit stateChanged();
    emit playbackChanged();
}

void FillingPreviewModel::poll() {
    m_elapsed = static_cast<double>(m_clock.elapsed()) / 1000.0;
    for (const auto& event : panta::ffi::preview_drain(*m_service)) {
        m_message = qt_string(event.message);
        if (!event.output_dir.empty()) {
            m_outputDir = qt_string(event.output_dir);
        }
        if (event.progress >= 0) {
            m_progress = event.progress;
        }
        if (event.kind == panta::ffi::PreviewEventKind::Progress) {
            continue;
        }
        m_busy = false;
        m_pollTimer.stop();
        m_elapsed = event.elapsed;
        if (event.kind == panta::ffi::PreviewEventKind::Failed) {
            m_error = m_message;
        } else if (event.kind != panta::ffi::PreviewEventKind::Cancelled) {
            auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
            for (std::size_t i = 0; i + 2 < event.coordinates.size(); i += 3) {
                mesh->vertices.push_back(
                    {event.coordinates[i], event.coordinates[i + 1], event.coordinates[i + 2]});
            }
            mesh->fill_times.assign(event.fill_times.begin(), event.fill_times.end());
            mesh->z_up = true;
            mesh->show_edges = event.kind == panta::ffi::PreviewEventKind::MeshReady;
            mesh->fill_duration = event.fill_time;
            m_mesh = std::move(mesh);
            if (event.kind == panta::ffi::PreviewEventKind::ModelReady) {
                m_step = 1;
            } else if (event.kind == panta::ffi::PreviewEventKind::MeshReady) {
                m_step = 2;
            } else {
                m_step = 3;
                m_duration = event.fill_time;
                m_peakPressure = event.peak_pressure;
                m_playbackTime = m_duration;
                emit playbackChanged();
            }
            emit meshChanged();
        }
    }
    emit stateChanged();
}

void FillingPreviewModel::togglePlayback() {
    if (m_step != 3) {
        return;
    }
    if (playing()) {
        m_playTimer.stop();
    } else {
        if (m_playbackTime >= m_duration) {
            m_playbackTime = 0;
        }
        m_playClock.start();
        m_playTimer.start();
    }
    emit playbackChanged();
}

void FillingPreviewModel::setPlaybackTime(double seconds) {
    if (!std::isfinite(seconds)) {
        return;
    }
    m_playbackTime = std::clamp(seconds, 0.0, m_duration);
    if (m_playbackTime >= m_duration) {
        m_playTimer.stop();
    }
    emit playbackChanged();
}
} // namespace panta::bridge
