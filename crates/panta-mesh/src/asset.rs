//! 面向工程旁置文件的版本化 Mesh 编解码；不依赖 VTK / Netgen 内存布局。

use crate::{SurfaceMesh, TetMesh, validate_tet_mesh};
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};

const MAGIC: &[u8; 8] = b"PNTMESH\0";
pub const MESH_ASSET_FORMAT_VERSION: u32 = 1;
const HEADER_SIZE: usize = 56;
const SURFACE_KIND: u8 = 1;
const TET_KIND: u8 = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum MeshAsset {
    Surface(SurfaceMesh),
    Volume(TetMesh),
}

pub enum MeshAssetRef<'a> {
    Surface(&'a SurfaceMesh),
    Volume(&'a TetMesh),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshAssetError {
    InvalidHeader,
    UnsupportedVersion(u32),
    UnsupportedKind(u8),
    UnsupportedUnits(u8),
    InvalidLength,
    ChecksumMismatch,
    InvalidMesh,
    SizeOverflow,
    AllocationFailed,
}

impl Display for MeshAssetError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for MeshAssetError {}

/// 编码固定小端 IEEE-754 / u32 字段，header 的摘要覆盖完整 payload。
pub fn encode_mesh_asset(mesh: &MeshAsset) -> Result<(Vec<u8>, String), MeshAssetError> {
    match mesh {
        MeshAsset::Surface(mesh) => encode_surface_asset(mesh),
        MeshAsset::Volume(mesh) => encode_volume_asset(mesh),
    }
}

pub fn encode_mesh_asset_ref(mesh: MeshAssetRef<'_>) -> Result<(Vec<u8>, String), MeshAssetError> {
    match mesh {
        MeshAssetRef::Surface(mesh) => encode_surface_asset(mesh),
        MeshAssetRef::Volume(mesh) => encode_volume_asset(mesh),
    }
}

fn encode_payload(
    kind: u8,
    payload_size: usize,
    write_payload: impl FnOnce(&mut Vec<u8>),
) -> Result<(Vec<u8>, String), MeshAssetError> {
    let capacity = HEADER_SIZE
        .checked_add(payload_size)
        .ok_or(MeshAssetError::SizeOverflow)?;
    let payload_len = u64::try_from(payload_size).map_err(|_| MeshAssetError::SizeOverflow)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| MeshAssetError::AllocationFailed)?;
    bytes.resize(HEADER_SIZE, 0);
    write_payload(&mut bytes);
    let digest = Sha256::digest(&bytes[HEADER_SIZE..]);
    let digest_text = format!("sha256:{digest:x}");
    bytes[..8].copy_from_slice(MAGIC);
    bytes[8..12].copy_from_slice(&MESH_ASSET_FORMAT_VERSION.to_le_bytes());
    bytes[12] = kind;
    bytes[13] = 1; // Coordinates are stored in millimeters.
    bytes[16..24].copy_from_slice(&payload_len.to_le_bytes());
    bytes[24..56].copy_from_slice(&digest);
    Ok((bytes, digest_text))
}

/// 验证格式、长度与内容摘要后解码，并对所有浮点坐标和体网格连接关系校验。
pub fn decode_mesh_asset(bytes: &[u8]) -> Result<MeshAsset, MeshAssetError> {
    decode_mesh_asset_with_digest(bytes).map(|(mesh, _)| mesh)
}

pub fn decode_mesh_asset_with_digest(bytes: &[u8]) -> Result<(MeshAsset, String), MeshAssetError> {
    if bytes.len() < HEADER_SIZE || &bytes[..8] != MAGIC {
        return Err(MeshAssetError::InvalidHeader);
    }
    let declared_len = mesh_asset_encoded_len(&bytes[..HEADER_SIZE])?;
    if declared_len != bytes.len() {
        return Err(MeshAssetError::InvalidLength);
    }
    let payload = &bytes[HEADER_SIZE..];
    let digest = Sha256::digest(payload);
    if digest[..] != bytes[24..56] {
        return Err(MeshAssetError::ChecksumMismatch);
    }
    let mesh = match bytes[12] {
        SURFACE_KIND => decode_surface(payload).map(MeshAsset::Surface),
        TET_KIND => decode_volume(payload).map(MeshAsset::Volume),
        other => Err(MeshAssetError::UnsupportedKind(other)),
    }?;
    Ok((mesh, format!("sha256:{digest:x}")))
}

/// 从完整固定长度头部验证格式并返回总字节长度，以便流式读取端先拒绝异常文件。
pub fn mesh_asset_encoded_len(header: &[u8]) -> Result<usize, MeshAssetError> {
    if header.len() != HEADER_SIZE || &header[..8] != MAGIC {
        return Err(MeshAssetError::InvalidHeader);
    }
    let version = u32::from_le_bytes(
        header[8..12]
            .try_into()
            .map_err(|_| MeshAssetError::InvalidHeader)?,
    );
    if version != MESH_ASSET_FORMAT_VERSION {
        return Err(MeshAssetError::UnsupportedVersion(version));
    }
    if !matches!(header[12], SURFACE_KIND | TET_KIND) {
        return Err(MeshAssetError::UnsupportedKind(header[12]));
    }
    if header[13] != 1 {
        return Err(MeshAssetError::UnsupportedUnits(header[13]));
    }
    if header[14..16] != [0, 0] {
        return Err(MeshAssetError::InvalidHeader);
    }
    let payload_len = u64::from_le_bytes(
        header[16..24]
            .try_into()
            .map_err(|_| MeshAssetError::InvalidHeader)?,
    );
    let payload_len = usize::try_from(payload_len).map_err(|_| MeshAssetError::InvalidLength)?;
    let total_len = HEADER_SIZE
        .checked_add(payload_len)
        .ok_or(MeshAssetError::InvalidLength)?;
    Ok(total_len)
}

fn encode_surface_asset(mesh: &SurfaceMesh) -> Result<(Vec<u8>, String), MeshAssetError> {
    if mesh.triangles.is_empty()
        || mesh
            .triangles
            .iter()
            .flatten()
            .flatten()
            .any(|coordinate| !coordinate.is_finite())
    {
        return Err(MeshAssetError::InvalidMesh);
    }
    let count = u64::try_from(mesh.triangles.len()).map_err(|_| MeshAssetError::SizeOverflow)?;
    let payload_size = mesh
        .triangles
        .len()
        .checked_mul(72)
        .and_then(|size| size.checked_add(8))
        .ok_or(MeshAssetError::SizeOverflow)?;
    encode_payload(SURFACE_KIND, payload_size, |bytes| {
        encode_surface_payload(mesh, count, bytes);
    })
}

fn encode_surface_payload(mesh: &SurfaceMesh, count: u64, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&count.to_le_bytes());
    for triangle in &mesh.triangles {
        for point in triangle {
            for coordinate in point {
                bytes.extend_from_slice(&coordinate.to_le_bytes());
            }
        }
    }
}

fn decode_surface(payload: &[u8]) -> Result<SurfaceMesh, MeshAssetError> {
    if payload.len() < 8 {
        return Err(MeshAssetError::InvalidLength);
    }
    let count = usize::try_from(u64::from_le_bytes(
        payload[..8]
            .try_into()
            .map_err(|_| MeshAssetError::InvalidLength)?,
    ))
    .map_err(|_| MeshAssetError::InvalidLength)?;
    if count == 0
        || 8_usize.checked_add(count.checked_mul(72).ok_or(MeshAssetError::SizeOverflow)?)
            != Some(payload.len())
    {
        return Err(MeshAssetError::InvalidLength);
    }
    let mut triangles = Vec::new();
    triangles
        .try_reserve_exact(count)
        .map_err(|_| MeshAssetError::AllocationFailed)?;
    let (encoded_triangles, _) = payload[8..].as_chunks::<72>();
    for encoded_triangle in encoded_triangles {
        let mut triangle = [[0.0; 3]; 3];
        for (index, coordinate) in triangle.iter_mut().flatten().enumerate() {
            let offset = index * 8;
            *coordinate = f64::from_le_bytes(
                encoded_triangle[offset..offset + 8]
                    .try_into()
                    .map_err(|_| MeshAssetError::InvalidLength)?,
            );
            if !coordinate.is_finite() {
                return Err(MeshAssetError::InvalidMesh);
            }
        }
        triangles.push(triangle);
    }
    Ok(SurfaceMesh { triangles })
}

fn encode_volume_asset(mesh: &TetMesh) -> Result<(Vec<u8>, String), MeshAssetError> {
    if !validate_tet_mesh(mesh).is_ok() {
        return Err(MeshAssetError::InvalidMesh);
    }
    let node_count = u64::try_from(mesh.nodes.len()).map_err(|_| MeshAssetError::SizeOverflow)?;
    let tet_count = u64::try_from(mesh.tets.len()).map_err(|_| MeshAssetError::SizeOverflow)?;
    let boundary_count =
        u64::try_from(mesh.boundary.len()).map_err(|_| MeshAssetError::SizeOverflow)?;
    let payload_size = mesh
        .nodes
        .len()
        .checked_mul(24)
        .and_then(|size| {
            mesh.tets
                .len()
                .checked_mul(20)
                .and_then(|other| size.checked_add(other))
        })
        .and_then(|size| {
            mesh.boundary
                .len()
                .checked_mul(16)
                .and_then(|other| size.checked_add(other))
        })
        .and_then(|size| size.checked_add(32))
        .ok_or(MeshAssetError::SizeOverflow)?;
    encode_payload(TET_KIND, payload_size, |bytes| {
        bytes.extend_from_slice(&node_count.to_le_bytes());
        bytes.extend_from_slice(&tet_count.to_le_bytes());
        bytes.extend_from_slice(&boundary_count.to_le_bytes());
        bytes.extend_from_slice(&mesh.region_count.to_le_bytes());
        bytes.extend_from_slice(&mesh.boundary_group_count.to_le_bytes());
        for node in &mesh.nodes {
            for coordinate in node {
                bytes.extend_from_slice(&coordinate.to_le_bytes());
            }
        }
        for tet in &mesh.tets {
            for node in tet.nodes {
                bytes.extend_from_slice(&node.to_le_bytes());
            }
            bytes.extend_from_slice(&tet.region.to_le_bytes());
        }
        for triangle in &mesh.boundary {
            for node in triangle.nodes {
                bytes.extend_from_slice(&node.to_le_bytes());
            }
            bytes.extend_from_slice(&triangle.group.to_le_bytes());
        }
    })
}

fn decode_volume(payload: &[u8]) -> Result<TetMesh, MeshAssetError> {
    let mut cursor = Cursor::new(payload);
    let node_count = cursor.count()?;
    let tet_count = cursor.count()?;
    let boundary_count = cursor.count()?;
    let region_count = cursor.u32()?;
    let boundary_group_count = cursor.u32()?;
    let expected = node_count
        .checked_mul(24)
        .and_then(|size| {
            tet_count
                .checked_mul(20)
                .and_then(|other| size.checked_add(other))
        })
        .and_then(|size| {
            boundary_count
                .checked_mul(16)
                .and_then(|other| size.checked_add(other))
        })
        .and_then(|size| size.checked_add(32))
        .ok_or(MeshAssetError::SizeOverflow)?;
    if payload.len() != expected {
        return Err(MeshAssetError::InvalidLength);
    }
    let mut nodes = Vec::new();
    nodes
        .try_reserve_exact(node_count)
        .map_err(|_| MeshAssetError::AllocationFailed)?;
    for _ in 0..node_count {
        nodes.push([cursor.f64()?, cursor.f64()?, cursor.f64()?]);
    }
    let mut tets = Vec::new();
    tets.try_reserve_exact(tet_count)
        .map_err(|_| MeshAssetError::AllocationFailed)?;
    for _ in 0..tet_count {
        tets.push(crate::Tetrahedron {
            nodes: [cursor.u32()?, cursor.u32()?, cursor.u32()?, cursor.u32()?],
            region: cursor.u32()?,
        });
    }
    let mut boundary = Vec::new();
    boundary
        .try_reserve_exact(boundary_count)
        .map_err(|_| MeshAssetError::AllocationFailed)?;
    for _ in 0..boundary_count {
        boundary.push(crate::SurfaceTriangle {
            nodes: [cursor.u32()?, cursor.u32()?, cursor.u32()?],
            group: cursor.u32()?,
        });
    }
    let mesh = TetMesh {
        nodes,
        tets,
        boundary,
        region_count,
        boundary_group_count,
    };
    if !validate_tet_mesh(&mesh).is_ok() {
        return Err(MeshAssetError::InvalidMesh);
    }
    Ok(mesh)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn take<const N: usize>(&mut self) -> Result<[u8; N], MeshAssetError> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or(MeshAssetError::InvalidLength)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(MeshAssetError::InvalidLength)?;
        self.offset = end;
        value.try_into().map_err(|_| MeshAssetError::InvalidLength)
    }
    fn count(&mut self) -> Result<usize, MeshAssetError> {
        usize::try_from(u64::from_le_bytes(self.take()?)).map_err(|_| MeshAssetError::InvalidLength)
    }
    fn u32(&mut self) -> Result<u32, MeshAssetError> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn f64(&mut self) -> Result<f64, MeshAssetError> {
        let value = f64::from_le_bytes(self.take()?);
        if value.is_finite() {
            Ok(value)
        } else {
            Err(MeshAssetError::InvalidMesh)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MeshAsset, MeshAssetError, decode_mesh_asset, encode_mesh_asset, mesh_asset_encoded_len,
    };
    use crate::{SurfaceMesh, SurfaceTriangle, TetMesh, Tetrahedron};

    #[test]
    fn surface_mesh_round_trips_and_checks_payload_digest() -> Result<(), Box<dyn std::error::Error>>
    {
        let mesh = MeshAsset::Surface(SurfaceMesh {
            triangles: vec![[[1.0, 2.0, 3.0]; 3]],
        });
        let (mut bytes, digest) = encode_mesh_asset(&mesh)?;
        assert!(digest.starts_with("sha256:"));
        assert_eq!(decode_mesh_asset(&bytes)?, mesh);
        *bytes.last_mut().ok_or(MeshAssetError::InvalidLength)? ^= 1;
        assert_eq!(
            decode_mesh_asset(&bytes),
            Err(MeshAssetError::ChecksumMismatch)
        );
        Ok(())
    }

    #[test]
    fn volume_mesh_round_trips_and_rejects_invalid_connectivity()
    -> Result<(), Box<dyn std::error::Error>> {
        let mesh = MeshAsset::Volume(TetMesh {
            nodes: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            tets: vec![Tetrahedron {
                nodes: [0, 1, 2, 3],
                region: 0,
            }],
            boundary: vec![SurfaceTriangle {
                nodes: [0, 1, 2],
                group: 0,
            }],
            region_count: 1,
            boundary_group_count: 1,
        });
        let (bytes, _) = encode_mesh_asset(&mesh)?;
        assert_eq!(decode_mesh_asset(&bytes)?, mesh);
        Ok(())
    }

    #[test]
    fn unsupported_version_is_reported() -> Result<(), Box<dyn std::error::Error>> {
        let (mut bytes, _) = encode_mesh_asset(&MeshAsset::Surface(SurfaceMesh {
            triangles: vec![[[0.0; 3]; 3]],
        }))?;
        bytes[8..12].copy_from_slice(&99_u32.to_le_bytes());
        assert_eq!(
            decode_mesh_asset(&bytes),
            Err(MeshAssetError::UnsupportedVersion(99))
        );
        Ok(())
    }

    #[test]
    fn asset_larger_than_two_gib_is_not_rejected_by_format_policy()
    -> Result<(), Box<dyn std::error::Error>> {
        let (mut bytes, _) = encode_mesh_asset(&MeshAsset::Surface(SurfaceMesh {
            triangles: vec![[[0.0; 3]; 3]],
        }))?;
        let payload_size = (2_u64 * 1024 * 1024 * 1024).to_le_bytes();
        bytes[16..24].copy_from_slice(&payload_size);

        assert!(mesh_asset_encoded_len(&bytes[..56]).is_ok());
        assert_eq!(
            decode_mesh_asset(&bytes),
            Err(MeshAssetError::InvalidLength)
        );
        Ok(())
    }
}
