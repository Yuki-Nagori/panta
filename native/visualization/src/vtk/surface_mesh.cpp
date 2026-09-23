#include "surface_mesh.hpp"

#include "mesh_source.hpp"
#include <cstddef>
#include <vtkCellArray.h>
#include <vtkPoints.h>
#include <vtkPolyData.h>
#include <vtkSmartPointer.h>
#include <vtkType.h>

namespace panta::visualization {

vtkSmartPointer<vtkPolyData> make_surface_poly_data(const SurfaceMeshSnapshot& mesh) {
    auto points = vtkSmartPointer<vtkPoints>::New();
    points->SetDataTypeToDouble();
    auto cells = vtkSmartPointer<vtkCellArray>::New();
    for (std::size_t vertex = 0; vertex + 2 < mesh.vertices.size(); vertex += 3) {
        vtkIdType ids[3]{};
        for (std::size_t corner = 0; corner < 3; ++corner) {
            ids[corner] = points->InsertNextPoint(mesh.vertices[vertex + corner].data());
        }
        cells->InsertNextCell(3, ids);
    }
    auto data = vtkSmartPointer<vtkPolyData>::New();
    data->SetPoints(points);
    data->SetPolys(cells);
    return data;
}

} // namespace panta::visualization
