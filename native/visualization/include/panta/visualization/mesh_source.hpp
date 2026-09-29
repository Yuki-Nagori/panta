/// 工程服务向视口提交的后端中立网格快照；没有第三方渲染类型。
#pragma once

#include <QObject>
#include <array>
#include <cstdint>
#include <memory>
#include <vector>

namespace panta::visualization {

struct SurfaceMeshSnapshot {
    std::vector<std::array<double, 3>> vertices;
    std::uint64_t project_revision = 0;
    /// 为空或与展开顶点一一对应；单位 s，NaN 表示未充填。
    std::vector<double> fill_times;
    double fill_duration = 0.0;
    bool show_edges = false;
    bool z_up = false;
};

class MeshSource : public QObject {
    Q_OBJECT

  public:
    using QObject::QObject;
    ~MeshSource() override = default;
    /// 返回已准备好的本地显示 DTO。视口刷新会读取此值；实现不得在此调用
    /// Rust / FFI，网格跨界复制只发生在导入、激活等粗粒度状态变化中。
    [[nodiscard]] virtual std::shared_ptr<const SurfaceMeshSnapshot> mesh_snapshot() const = 0;
    /// 无网格时是否显示占位字样（Welcome 场景）。返回 false 表示空白视口，
    /// 后端不得用占位内容填充；文档视图据此区分 Welcome 与全关闭。
    [[nodiscard]] virtual bool placeholder_visible() const { return true; }

  signals:
    void meshChanged();
};

} // namespace panta::visualization
