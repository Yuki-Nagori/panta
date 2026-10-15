//! Moldfill 求解服务的 CXX DTO 适配层。
use crate::bridge;

/// Rust 持有外部求解器作业服务；析构由服务负责取消和回收。
pub struct AnalysisService(panta_solver::AnalysisService);

pub(super) fn analysis_service_new() -> Box<AnalysisService> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Box::new(AnalysisService(panta_solver::AnalysisService::new(
        root.canonicalize().unwrap_or(root),
    )))
}

pub(super) fn analysis_bind(service: &mut AnalysisService, output: &str) -> bridge::AnalysisCache {
    let view = service.0.bind(std::path::Path::new(output));
    bridge::AnalysisCache {
        root: view.root,
        has_mesh: view.has_mesh,
        has_gate: view.has_gate,
        has_fill: view.has_fill,
        coordinates: view.coordinates,
        fill_times: view.fill_times,
        pressures: view.pressures,
        gate_points: view.gate_points,
        fill_time: view.fill_time,
        peak_pressure: view.peak_pressure,
    }
}

pub(super) fn analysis_start(
    service: &mut AnalysisService,
    operation: &str,
    settings: &str,
) -> Result<(), String> {
    service.0.start(operation, settings)
}

pub(super) fn analysis_cancel(service: &AnalysisService) {
    service.0.cancel();
}

pub(super) fn analysis_drain(service: &mut AnalysisService) -> Vec<bridge::AnalysisEvent> {
    service
        .0
        .drain()
        .into_iter()
        .map(|event| bridge::AnalysisEvent {
            kind: match event.kind {
                panta_solver::EventKind::Progress => bridge::AnalysisEventKind::Progress,
                panta_solver::EventKind::Log => bridge::AnalysisEventKind::Log,
                panta_solver::EventKind::GateReady => bridge::AnalysisEventKind::GateReady,
                panta_solver::EventKind::MeshReady => bridge::AnalysisEventKind::MeshReady,
                panta_solver::EventKind::Completed => bridge::AnalysisEventKind::Completed,
                panta_solver::EventKind::Failed => bridge::AnalysisEventKind::Failed,
                panta_solver::EventKind::Cancelled => bridge::AnalysisEventKind::Cancelled,
                panta_solver::EventKind::Restored => bridge::AnalysisEventKind::Restored,
            },
            message: event.message,
            progress: event.progress,
            elapsed: event.elapsed,
            fill_time: event.fill_time,
            peak_pressure: event.peak_pressure,
            output_dir: event.output_dir,
            coordinates: event.mesh.coordinates,
            fill_times: event.mesh.fill_times,
            pressures: event.mesh.pressures,
            gate_points: event.mesh.gate_points,
            scope: event.scope,
            from_cache: event.from_cache,
            has_mesh: event.has_mesh,
            has_gate: event.has_gate,
            has_fill: event.has_fill,
        })
        .collect()
}
