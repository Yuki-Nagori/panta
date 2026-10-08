//! 网格 DTO 布局转换及显示缓冲扁平化；领域校验转交 panta-mesh。
use crate::bridge;

pub(super) fn mesh_validate_tet(data: bridge::TetMeshData) -> bridge::TetMeshValidation {
    if !data.nodes.len().is_multiple_of(3)
        || !data.tets.len().is_multiple_of(4)
        || !data.boundary.len().is_multiple_of(3)
        || data.tet_regions.len() != data.tets.len() / 4
        || data.boundary_groups.len() != data.boundary.len() / 3
    {
        return bridge::TetMeshValidation {
            issues: vec!["invalid mesh DTO layout".to_owned()],
            volume_mm3: 0.0,
        };
    }
    let mut nodes = Vec::new();
    let mut tets = Vec::new();
    let mut boundary = Vec::new();
    if let Err(error) = nodes.try_reserve_exact(data.nodes.len() / 3) {
        return bridge::TetMeshValidation {
            issues: vec![format!("cannot allocate mesh nodes: {error}")],
            volume_mm3: 0.0,
        };
    }
    if let Err(error) = tets.try_reserve_exact(data.tets.len() / 4) {
        return bridge::TetMeshValidation {
            issues: vec![format!("cannot allocate tetrahedra: {error}")],
            volume_mm3: 0.0,
        };
    }
    if let Err(error) = boundary.try_reserve_exact(data.boundary.len() / 3) {
        return bridge::TetMeshValidation {
            issues: vec![format!("cannot allocate boundary triangles: {error}")],
            volume_mm3: 0.0,
        };
    }
    nodes.extend(
        data.nodes
            .as_chunks::<3>()
            .0
            .iter()
            .map(|point| [point[0], point[1], point[2]]),
    );
    tets.extend(
        data.tets
            .as_chunks::<4>()
            .0
            .iter()
            .zip(data.tet_regions)
            .map(|(nodes, region)| panta_mesh::Tetrahedron {
                nodes: [nodes[0], nodes[1], nodes[2], nodes[3]],
                region,
            }),
    );
    boundary.extend(
        data.boundary
            .as_chunks::<3>()
            .0
            .iter()
            .zip(data.boundary_groups)
            .map(|(nodes, group)| panta_mesh::SurfaceTriangle {
                nodes: [nodes[0], nodes[1], nodes[2]],
                group,
            }),
    );
    let mesh = panta_mesh::TetMesh {
        nodes,
        tets,
        boundary,
        region_count: data.region_count,
        boundary_group_count: data.boundary_group_count,
    };
    let report = panta_mesh::validate_tet_mesh(&mesh);
    bridge::TetMeshValidation {
        issues: report.issues,
        volume_mm3: report.volume_mm3,
    }
}

/// SurfaceMesh → 扁平坐标缓冲；分配失败的上下文由调用方配上稳定诊断码。
pub(super) fn mesh_coordinates(mesh: &panta_mesh::SurfaceMesh) -> Result<Vec<f64>, String> {
    let mut coordinates = Vec::new();
    let values = mesh
        .triangles
        .len()
        .checked_mul(9)
        .ok_or("mesh snapshot coordinate count overflow")?;
    coordinates
        .try_reserve_exact(values)
        .map_err(|error| format!("mesh snapshot allocation failed: {error}"))?;
    for triangle in &mesh.triangles {
        for point in triangle {
            coordinates.extend_from_slice(point);
        }
    }
    Ok(coordinates)
}
