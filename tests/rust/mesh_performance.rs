//! 固定输入的 CPU 微基准；手动用 `cargo test -p panta-mesh --release --test mesh_performance -- --ignored --nocapture` 运行。
use panta_mesh::{
    SurfaceMesh, SurfaceTriangle, TetMesh, Tetrahedron, parse_stl, validate_tet_mesh,
};
use std::hint::black_box;
use std::time::{Duration, Instant};

const TRIANGLE_COUNT: usize = 100_000;
const SAMPLES: usize = 21;

fn binary_fixture() -> Vec<u8> {
    let mut bytes = vec![0_u8; 84 + TRIANGLE_COUNT * 50];
    bytes[80..84].copy_from_slice(&(TRIANGLE_COUNT as u32).to_le_bytes());
    for triangle in 0..TRIANGLE_COUNT {
        let base = 84 + triangle * 50 + 12;
        for (corner, point) in [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
            .into_iter()
            .enumerate()
        {
            for (axis, value) in point.into_iter().enumerate() {
                let offset = base + corner * 12 + axis * 4;
                bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
    }
    bytes
}

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn flatten(mesh: &SurfaceMesh) -> Vec<f64> {
    let mut coordinates = Vec::with_capacity(mesh.triangles.len() * 9);
    for triangle in &mesh.triangles {
        for point in triangle {
            coordinates.extend_from_slice(point);
        }
    }
    coordinates
}

#[test]
#[ignore = "CPU microbenchmark; run explicitly in release mode"]
fn parse_and_snapshot_copy_cost() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = binary_fixture();
    let mesh = parse_stl(&bytes)?;
    assert_eq!(mesh.summary().triangle_count, TRIANGLE_COUNT as u64);

    let mut parse_samples = Vec::with_capacity(SAMPLES);
    let mut copy_samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let parsed = parse_stl(black_box(&bytes))?;
        parse_samples.push(start.elapsed());
        let start = Instant::now();
        let coordinates = flatten(black_box(&parsed));
        copy_samples.push(start.elapsed());
        assert_eq!(coordinates.len(), TRIANGLE_COUNT * 9);
        black_box(coordinates);
    }
    println!(
        "triangles={TRIANGLE_COUNT} input_bytes={} mesh_bytes={} ffi_coordinates_bytes={} parse_median_ms={:.3} flatten_median_ms={:.3}",
        bytes.len(),
        mesh.triangles.len() * std::mem::size_of::<[[f64; 3]; 3]>(),
        TRIANGLE_COUNT * 9 * std::mem::size_of::<f64>(),
        median(parse_samples).as_secs_f64() * 1000.0,
        median(copy_samples).as_secs_f64() * 1000.0
    );
    Ok(())
}

#[test]
#[ignore = "CPU microbenchmark; run explicitly in release mode"]
fn tetrahedron_validation_cost() {
    const COUNT: usize = 10_000;
    let mut mesh = TetMesh {
        nodes: Vec::with_capacity(COUNT * 4),
        tets: Vec::with_capacity(COUNT),
        boundary: Vec::with_capacity(COUNT),
        region_count: 1,
        boundary_group_count: 1,
    };
    for index in 0..COUNT {
        let origin = index as f64 * 2.0;
        let node = (index * 4) as u32;
        mesh.nodes.extend_from_slice(&[
            [origin, 0.0, 0.0],
            [origin + 1.0, 0.0, 0.0],
            [origin, 1.0, 0.0],
            [origin, 0.0, 1.0],
        ]);
        mesh.tets.push(Tetrahedron {
            nodes: [node, node + 1, node + 2, node + 3],
            region: 0,
        });
        mesh.boundary.push(SurfaceTriangle {
            nodes: [node, node + 1, node + 2],
            group: 0,
        });
    }
    assert!(validate_tet_mesh(&mesh).is_ok());
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let report = validate_tet_mesh(black_box(&mesh));
        samples.push(start.elapsed());
        assert!(report.is_ok());
    }
    println!(
        "tetrahedra={COUNT} nodes={} boundary={} validation_median_ms={:.3}",
        mesh.nodes.len(),
        mesh.boundary.len(),
        median(samples).as_secs_f64() * 1000.0
    );
}
