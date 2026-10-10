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
        self.execution
            .writes
            .submit(move || {
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
                // 宿主已关闭也不能吞掉提交失败；成功以磁盘清单为权威。
                if let Err(std::sync::mpsc::SendError(Err(error))) = sender.send(result) {
                    eprintln!(
                        "panta: metadata confirmation failed after service teardown: {error}"
                    );
                }
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

#[cfg(test)]
mod tests {
    use super::super::ProjectCommand;
    use super::super::temp_directory;
    use super::{MetadataWriteKind, PendingMetadataWrite, ProjectService};
    use crate::execution::Executor;
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    #[test]
    fn accepted_write_finishes_after_service_drop_without_a_gui_join()
    -> Result<(), Box<dyn std::error::Error>> {
        for block_staging in [false, true] {
            let fixture = temp_directory::Fixture::new()?;
            let executor = Executor::new("write-drop-test", 1, 3);
            let (started, ready) = mpsc::channel();
            let (release, gate) = mpsc::channel();
            executor.submit(move || {
                let _ = started.send(());
                let _ = gate.recv();
            })?;
            ready.recv_timeout(Duration::from_secs(2))?;
            let mut service = ProjectService::new();
            let project = service.create(&fixture.root, "WriteDrop")?;
            service.execution.writes = Arc::clone(&executor);
            if block_staging {
                let staging = project.path.with_extension("panta.write");
                std::fs::remove_dir(&staging)?;
                std::fs::write(staging, "blocked")?;
            }
            let mut candidate = service.current.as_ref().ok_or("no project")?.clone();
            candidate.name = "Committed".into();
            candidate.revision += 1;
            service.start_metadata_write(candidate, MetadataWriteKind::Material)?;
            drop(service);
            release.send(())?;
            let (finished, done) = mpsc::channel();
            executor.submit(move || {
                let _ = finished.send(());
            })?;
            done.recv_timeout(Duration::from_secs(2))?;
            let mut reopened = ProjectService::new();
            reopened.open(&project.path)?;
            assert_eq!(
                reopened.current()?.name,
                if block_staging {
                    "WriteDrop"
                } else {
                    "Committed"
                }
            );
            assert_eq!(
                reopened.current()?.revision,
                if block_staging { 0 } else { 1 }
            );
        }
        Ok(())
    }

    #[test]
    fn full_write_channel_preserves_state_and_does_not_publish_a_pending_candidate()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let executor = Executor::new("write-capacity-test", 1, 1);
        let (started, ready) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        executor.submit(move || {
            let _ = started.send(());
            let _ = gate.recv();
        })?;
        ready.recv_timeout(Duration::from_secs(2))?;
        let mut service = ProjectService::new();
        let project = service.create(&fixture.root, "Capacity")?;
        let before = service.current()?;
        let manifest = std::fs::read(&project.path)?;
        service.execution.writes = Arc::clone(&executor);
        let mut candidate = service.current.as_ref().ok_or("no project")?.clone();
        candidate.name = "Rejected".into();
        candidate.revision += 1;
        let error = service
            .start_metadata_write(candidate, MetadataWriteKind::Material)
            .err()
            .ok_or("full channel accepted a write")?;
        assert_eq!(error.code(), "project.io");
        assert!(error.detail().contains("background capacity exhausted"));
        assert!(service.pending_metadata.is_none());
        assert_eq!(service.current()?, before);
        assert_eq!(std::fs::read(&project.path)?, manifest);
        release.send(())?;
        executor.close_and_wait();
        Ok(())
    }

    #[test]
    fn disconnected_metadata_channel_clears_pending_and_preserves_current()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Disconnected worker")?;
        let before = service.current()?;
        let (sender, receiver) = mpsc::channel();
        drop(sender);
        service.pending_metadata = Some(PendingMetadataWrite {
            kind: MetadataWriteKind::Material,
            receiver,
        });

        let error = service
            .finish_material_confirmation()
            .err()
            .ok_or("disconnected metadata worker unexpectedly succeeded")?;

        assert_eq!(error.code(), "project.io");
        assert_eq!(error.detail(), "metadata confirmation worker disconnected");
        assert!(service.pending_metadata.is_none());
        assert_eq!(service.current()?, before);
        Ok(())
    }

    #[test]
    fn pending_metadata_confirmation_rejects_commands_without_changing_project()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let mut service = ProjectService::new();
        let project = service.create(&fixture.root, "Pending metadata")?;
        let before = service.current()?;
        let manifest = std::fs::read(&project.path)?;
        let (_sender, receiver) = mpsc::channel();
        service.pending_metadata = Some(PendingMetadataWrite {
            kind: MetadataWriteKind::Material,
            receiver,
        });

        let error = service
            .execute(ProjectCommand::Rename {
                name: "Rejected rename".to_owned(),
            })
            .err()
            .ok_or("command was accepted while metadata confirmation was pending")?;

        assert_eq!(error.code(), "project.command_invalid");
        assert_eq!(error.detail(), "metadata confirmation pending");
        assert!(service.pending_metadata.is_some());
        assert_eq!(service.current()?, before);
        assert_eq!(std::fs::read(&project.path)?, manifest);
        Ok(())
    }
}
