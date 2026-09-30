/// 自有表面网格快照到 VTK 显示数据的边界转换。
#pragma once

#include <panta/visualization/mesh_source.hpp>
#include <vtkSmartPointer.h>

class vtkPolyData;

namespace panta::visualization {

/// 所选浇口的球形显示标记；不改变求解网格或边界条件。
vtkSmartPointer<vtkPolyData> make_gate_poly_data(const SurfaceMeshSnapshot& mesh);

vtkSmartPointer<vtkPolyData> make_surface_poly_data(const SurfaceMeshSnapshot& mesh);

/// 用线性插值的到达时间切分每个三角面；灰色为尚未充填，颜色为到达时间。
/// 保留完整模型轮廓，回放更新不会改变相机适配边界。
vtkSmartPointer<vtkPolyData> make_filling_poly_data(const SurfaceMeshSnapshot& mesh, double time);

} // namespace panta::visualization
