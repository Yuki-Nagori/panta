#pragma once

#include <vtkSmartPointer.h>

class vtkPolyData;

namespace panta::visualization {

/// 创建居中的封闭 panta 字形及欢迎场景顶点色；不包含求解结果或 GPU 对象。
vtkSmartPointer<vtkPolyData> create_default_wordmark();

} // namespace panta::visualization
