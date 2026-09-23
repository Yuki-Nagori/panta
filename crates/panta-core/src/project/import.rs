//! STL 导入的工程事务；读取、预览和解析由 panta-import 负责。
use super::*;
use panta_import::{ImportError, ImportOptions};
use std::io::Write;

impl ProjectService {
    /// 校验、复制并持久化当前工程包中的一次 STL 导入。
    /// 返回成功前先写入清单，重新打开工程时无需依赖后续显式保存即可恢复。
    pub fn import_stl(
        &mut self,
        source: &Path,
        mesh_type: &str,
        units: &str,
        show_import_log: bool,
    ) -> Result<ImportRecord, ProjectError> {
        if self.current.is_none() {
            return Err(ProjectError::NoProject);
        }
        let mut prepared = self
            .import_session
            .prepare(source)
            .map_err(map_import_error)?;
        let options =
            ImportOptions::new(mesh_type, units, show_import_log).map_err(map_import_error)?;
        let import_number = self
            .current
            .as_ref()
            .ok_or(ProjectError::NoProject)?
            .imports
            .len()
            + 1;
        let record = prepared.record(import_number, options);
        prepared.scale_mesh_mm(options).map_err(map_import_error)?;
        let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
        let project_root = state
            .path
            .parent()
            .ok_or_else(|| ProjectError::Io("project has no package directory".to_owned()))?;
        let asset_dir = project_root.join("assets").join("imports");
        fs::create_dir_all(&asset_dir).map_err(|error| {
            ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_dir.display()))
        })?;
        let asset_name = prepared.asset_name(import_number);
        let asset_path = asset_dir.join(&asset_name);
        let mut asset = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&asset_path)
            .map_err(|error| {
                ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_path.display()))
            })?;
        let written = asset
            .write_all(&prepared.bytes)
            .and_then(|()| asset.sync_all());
        drop(asset);
        if let Err(error) = written {
            let _ = fs::remove_file(&asset_path);
            return Err(ProjectError::ImportAssetCopyFailed(format!(
                "{}: {error}",
                asset_path.display()
            )));
        }

        let previous_revision = state.revision;
        let previous_dirty = state.dirty;
        state.imports.push(record.clone());
        state.revision = state.revision.saturating_add(1);
        state.dirty = false;
        if let Err(error) = write_manifest(state) {
            state.imports.pop();
            state.revision = previous_revision;
            state.dirty = previous_dirty;
            let _ = fs::remove_file(&asset_path);
            return Err(error);
        }
        self.current_mesh = Some(prepared.mesh);
        Ok(record)
    }

    /// Latest committed mesh; copied only when crossing the CXX boundary.
    pub fn current_mesh(&self) -> Option<&SurfaceMesh> {
        self.current_mesh.as_ref()
    }

    /// Parse STL metadata without requiring an open project or mutating disk.
    pub fn inspect_stl(&mut self, source: &Path) -> Result<StlImportPreview, ProjectError> {
        self.import_session
            .preview(source)
            .map_err(map_import_error)
    }
}

pub(super) fn map_import_error(error: ImportError) -> ProjectError {
    match error {
        ImportError::Missing(path) => ProjectError::ImportFileMissing(path),
        ImportError::UnsupportedFormat(path) => ProjectError::ImportInvalidFile(path),
        ImportError::Read(detail) => ProjectError::ImportParseFailed(detail),
        ImportError::Parse(detail) => ProjectError::ImportParseFailed(detail.to_string()),
        ImportError::SourceChanged(path) => ProjectError::ImportSourceChanged(path),
        ImportError::UnsupportedMeshType(kind) => ProjectError::ImportUnsupportedMeshType(kind),
        ImportError::UnsupportedUnits(units) => ProjectError::ImportUnsupportedUnits(units),
        ImportError::CoordinateOverflow => {
            ProjectError::ImportParseFailed("millimeter coordinate overflow".to_owned())
        }
    }
}
