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
};

class MeshSource : public QObject {
    Q_OBJECT

  public:
    using QObject::QObject;
    ~MeshSource() override = default;
    [[nodiscard]] virtual std::shared_ptr<const SurfaceMeshSnapshot> mesh_snapshot() const = 0;
    /// 无网格时是否显示占位字样（Welcome 场景）。返回 false 表示空白视口，
    /// 后端不得用占位内容填充；文档视图据此区分 Welcome 与全关闭。
    [[nodiscard]] virtual bool placeholder_visible() const { return true; }

  signals:
    void meshChanged();
};

} // namespace panta::visualization
