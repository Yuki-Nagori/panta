//! 工程清单的单文件持久化。
use super::*;

pub(super) fn write_manifest(state: &ProjectState) -> Result<(), ProjectError> {
    let manifest = ProjectManifest {
        schema: PROJECT_SCHEMA_VERSION,
        name: state.name.clone(),
        revision: state.revision,
        imports: state.imports.clone(),
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
