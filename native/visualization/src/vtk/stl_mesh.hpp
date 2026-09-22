// STL 读取适配：VTK SDK 未提供 vtkSTLReader 时，先把三角面转换为 vtkPolyData。
#pragma once

#include <QString>
#include <vtkSmartPointer.h>

class vtkPolyData;

namespace panta::visualization {

vtkSmartPointer<vtkPolyData> load_stl_mesh(const QString& path, QString* error);

} // namespace panta::visualization
