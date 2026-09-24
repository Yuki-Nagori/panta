/// RenderScene：后端中立的视口场景 CPU 状态（任务 007，
/// architecture/visualization.md 的 RenderScene 契约最小实现）。
///
/// 当前在 GUI 线程提交与消费，后端合并同一事件轮的状态。第三方渲染类型
/// （VTK 对象等）不得出现在本头文件——后端私有状态归各适配器所有。
/// 应用可恢复状态保存为 CPU 值，场景节点销毁重建时按状态重建渲染对象，
/// 不依赖上一轮 GPU 资源。
#pragma once

#include <QColor>
#include <cstdint>
#include <memory>
#include <panta/visualization/mesh_source.hpp>

namespace panta::visualization {

/// 场景修订号：GUI 侧每次状态提交自增；后端回调携带旧值即视为迟到更新。
using SceneRevision = std::uint64_t;

struct RenderScene {
    /// 状态版本：后端已应用的修订低于此值时必须重新应用。
    SceneRevision revision = 0;
    /// 视口背景基色（演示阶段的可恢复状态之一；VTK 以轻微渐变呈现）。
    QColor background{248, 249, 250};
    /// 默认欢迎字样可见性；为空 mesh 时由 VTK 适配器显示欢迎几何。
    bool primitive_visible = true;
    /// Rust 工程服务已校验的表面网格快照；为空时显示默认欢迎图形。
    std::shared_ptr<const SurfaceMeshSnapshot> mesh;
};

} // namespace panta::visualization
