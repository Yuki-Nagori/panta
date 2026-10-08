//! 工程清单的单文件持久化。
use super::{ProjectError, ProjectService, ProjectState, gate_location, process_settings};

pub(super) fn write_manifest(state: &mut ProjectState) -> Result<(), ProjectError> {
    super::repository::WriteLease::acquire(state)?.commit(state)
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
        mut candidate: ProjectState,
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
                    write_manifest(&mut candidate)?;
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
