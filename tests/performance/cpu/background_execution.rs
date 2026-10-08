//! 手动 CPU 容量测量：通过公共工程预检入口加载合成大 STL，不设置 CI 时间门禁。
use panta_core::project::ProjectService;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

#[path = "../../support/rust/temp_directory.rs"]
mod temp_directory;

fn source(path: &std::path::Path, triangles: u32) -> Result<(), Box<dyn Error>> {
    let mut output = BufWriter::new(File::create(path)?);
    output.write_all(&[0; 80])?;
    output.write_all(&triangles.to_le_bytes())?;
    let mut triangle = Vec::new();
    for value in [
        0.0_f32, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ] {
        triangle.extend_from_slice(&value.to_le_bytes());
    }
    triangle.extend_from_slice(&0_u16.to_le_bytes());
    for _ in 0..triangles {
        output.write_all(&triangle)?;
    }
    output.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let fixture = temp_directory::Fixture::new()?;
    println!("triangles,requests,input_bytes,elapsed_ms,p50_ms,p95_ms");
    for triangles in [50_000_u32, 500_000] {
        let path = fixture.root.join("capacity.stl");
        source(&path, triangles)?;
        for count in [1, 4, 8, 64] {
            let started = Instant::now();
            let mut pending = Vec::new();
            for _ in 0..count {
                let mut service = ProjectService::new();
                let request = service.begin_stl_preview(&path)?;
                pending.push((service, request));
            }
            let mut latencies = Vec::new();
            while !pending.is_empty() {
                if started.elapsed() > Duration::from_secs(60) {
                    return Err("preview capacity run timed out".into());
                }
                for index in (0..pending.len()).rev() {
                    let (service, request) = &mut pending[index];
                    if let Some(preview) = service.finish_stl_preview(*request)? {
                        if preview.triangle_count != u64::from(triangles) {
                            return Err("incorrect preview".into());
                        }
                        latencies.push(started.elapsed().as_secs_f64() * 1000.0);
                        // 消费完就释放来源快照，测量在途执行而非人为保留全部结果。
                        pending.swap_remove(index);
                    }
                }
                if !pending.is_empty() {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            latencies.sort_by(f64::total_cmp);
            println!(
                "{triangles},{count},{},{:.3},{:.3},{:.3}",
                84 + u64::from(triangles) * 50,
                started.elapsed().as_secs_f64() * 1000.0,
                latencies[latencies.len().div_ceil(2) - 1],
                latencies[latencies.len() * 95 / 100]
            );
        }
    }
    Ok(())
}
