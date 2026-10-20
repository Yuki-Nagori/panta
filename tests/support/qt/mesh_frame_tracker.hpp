// 将目标 Mesh 与实际报告的场景帧关联；空帧、旧帧不能完成目标模型的等待。
#pragma once

#include <cstddef>
#include <memory>
#include <panta/visualization/render_scene.hpp>

namespace panta::test {

/// 真实窗口报告 VTK 提交，离屏报告队列完成；两种边界均须匹配场景修订。
/// 只观察快照，不延长旧显示 DTO 的生命周期。
class MeshFrameTracker final {
  public:
    void record_scene(visualization::SceneRevision revision,
                      const std::shared_ptr<const visualization::SurfaceMeshSnapshot>& mesh) {
        m_sceneRevision = revision;
        m_mesh = mesh;
    }

    void record_frame(visualization::SceneRevision revision) {
        m_frameRevision = revision;
        ++m_frameCount;
    }

    [[nodiscard]] bool
    ready(const std::shared_ptr<const visualization::SurfaceMeshSnapshot>& mesh) const {
        return mesh != nullptr && m_mesh.lock() == mesh && scene_complete();
    }

    [[nodiscard]] bool scene_complete() const {
        return m_frameCount != 0 && m_frameRevision == m_sceneRevision;
    }

    [[nodiscard]] std::size_t frame_count() const { return m_frameCount; }

  private:
    std::weak_ptr<const visualization::SurfaceMeshSnapshot> m_mesh;
    visualization::SceneRevision m_sceneRevision = 0;
    visualization::SceneRevision m_frameRevision = 0;
    std::size_t m_frameCount = 0;
};

} // namespace panta::test
