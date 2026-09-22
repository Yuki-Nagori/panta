/// RenderScene：后端中立的视口场景 CPU 状态（任务 007，
/// architecture/visualization.md 的 RenderScene 契约最小实现）。
///
/// 当前在 GUI 线程提交与消费，后端合并同一事件轮的状态。第三方渲染类型
/// （VTK 对象等）不得出现在本头文件——后端私有状态归各适配器所有。
/// 应用可恢复状态保存为 CPU 值，场景节点销毁重建时按状态重建渲染对象，
/// 不依赖上一轮 GPU 资源。
#pragma once

#include <QColor>
#include <QString>
#include <cstdint>

namespace panta::visualization {

/// 场景修订号：GUI 侧每次状态提交自增；后端回调携带旧值即视为迟到更新。
using SceneRevision = std::uint64_t;

struct RenderScene {
    /// 状态版本：后端已应用的修订低于此值时必须重新应用。
    SceneRevision revision = 0;
    /// 视口背景色（演示阶段的可恢复状态之一）。
    QColor background{232, 238, 247};
    /// 临时欢迎图形可见性；053 的最终字样设计仍为规划。
    bool primitive_visible = true;
    /// 工程包内的可选 STL 资产；为空时显示默认欢迎图形，非空时替换场景网格。
    QString mesh_path;
};

} // namespace panta::visualization
