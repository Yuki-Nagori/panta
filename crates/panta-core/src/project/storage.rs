//! 工程清单的单文件持久化。
use super::{
    PROJECT_SCHEMA_VERSION, ProjectError, ProjectManifest, ProjectService, ProjectState,
    gate_location, process_settings,
};
use std::fs;

pub(super) fn write_manifest(state: &ProjectState) -> Result<(), ProjectError> {
    let manifest = ProjectManifest {
        schema: PROJECT_SCHEMA_VERSION,
        name: state.name.clone(),
        revision: state.revision,
        imports: state.imports.clone(),
        analysis_sequences: state.analysis_sequences.clone(),
        materials: state.materials.clone(),
        fill_settings: state.fill_settings.clone(),
        gate_location_settings: state.gate_location_settings.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| ProjectError::Io(format!("serialize manifest: {error}")))?;
    let target = &state.path;
    let temporary = target.with_extension("panta.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| ProjectError::Io(format!("{}: {error}", temporary.display())))?;

    // rename 替换目标；失败时保留旧清单，不先删除已提交的工程文件。
    if let Err(error) = fs::rename(&temporary, target) {
        let _ = fs::remove_file(&temporary);
        return Err(ProjectError::Io(format!("{}: {error}", target.display())));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MetadataWriteKind {
    Material,
    FillSettings,
    GateLocationSettings,
}

#[derive(Debug)]
pub(super) struct PendingMetadataWrite {
    kind: MetadataWriteKind,
    receiver: std::sync::mpsc::Receiver<Result<ProjectState, ProjectError>>,
}

impl ProjectService {
    pub(super) fn start_metadata_write(
        &mut self,
        candidate: ProjectState,
        kind: MetadataWriteKind,
    ) -> Result<bool, ProjectError> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let _worker = std::thread::Builder::new()
            .name("panta-metadata-confirmation".to_owned())
            .spawn(move || {
                let result = (|| {
                    if kind == MetadataWriteKind::FillSettings {
                        process_settings::validate_settings(
                            &candidate.imports,
                            &candidate.fill_settings,
                        )?;
                    }
                    if kind == MetadataWriteKind::GateLocationSettings {
                        gate_location::validate_settings(
                            &candidate.imports,
                            &candidate.gate_location_settings,
                        )?;
                    }
                    write_manifest(&candidate)?;
                    Ok(candidate)
                })();
                // 工作线程只拥有候选值；服务销毁后无需向 Qt 或旧接收端交付。
                let _ = sender.send(result);
            })
            .map_err(|error| ProjectError::Io(format!("start metadata confirmation: {error}")))?;
        self.pending_metadata = Some(PendingMetadataWrite { kind, receiver });
        Ok(true)
    }

    pub(super) fn finish_metadata_write(
        &mut self,
        kind: MetadataWriteKind,
    ) -> Result<bool, ProjectError> {
        use std::sync::mpsc::TryRecvError;
        let pending = self
            .pending_metadata
            .as_ref()
            .filter(|pending| pending.kind == kind)
            .ok_or_else(|| {
                ProjectError::CommandInvalid("no matching metadata confirmation pending".to_owned())
            })?;
        let result = match pending.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return Ok(false),
            Err(TryRecvError::Disconnected) => Err(ProjectError::Io(
                "metadata confirmation worker disconnected".to_owned(),
            )),
        };
        self.pending_metadata = None;
        // 候选写入期间其他元数据命令均被拒绝，不会覆盖更新的工程修订。
        self.current = Some(result?);
        Ok(true)
    }
}
