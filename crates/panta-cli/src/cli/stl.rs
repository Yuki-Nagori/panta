//! CLI STL 命令；预览和提交均委托给 `ProjectService`。

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use panta_core::project::{
    DEFAULT_MESH_TYPE, ImportRecord, ProjectCommand, ProjectService, StlImportPreview,
};

const EXPANDED_MESH_WARNING_BYTES: u128 = 2 * 1024 * 1024 * 1024;
const BYTES_PER_EXPANDED_TRIANGLE: u128 = 72;
const SIDECAR_HEADER_ESTIMATE_BYTES: u128 = 64;

#[derive(Debug, PartialEq, Eq)]
pub struct ImportOptions {
    mesh_type: String,
    units: String,
}

pub fn inspect(service: &mut ProjectService, path: PathBuf) -> Result<StlImportPreview, String> {
    service
        .inspect_stl(&path)
        .map_err(|error| error.to_string())
}

pub fn import(
    service: &mut ProjectService,
    project_path: PathBuf,
    source_path: PathBuf,
    options: ImportOptions,
) -> Result<ImportRecord, String> {
    service
        .execute(ProjectCommand::Open { path: project_path })
        .map_err(|error| error.to_string())?;
    let preview = service
        .inspect_stl(&source_path)
        .map_err(|error| error.to_string())?;
    if expanded_mesh_estimate_bytes(preview.triangle_count) > EXPANDED_MESH_WARNING_BYTES {
        eprintln!(
            "Warning: expanded mesh data is estimated to exceed 2 GiB. Import will continue, but it may require substantial memory and disk space."
        );
    }
    service
        .import_stl(&source_path, &options.mesh_type, &options.units, false)
        .map_err(|error| error.to_string())
}

pub(super) fn parse_import_options(arguments: &[OsString]) -> Result<ImportOptions, String> {
    let mut options = ImportOptions {
        mesh_type: DEFAULT_MESH_TYPE.to_owned(),
        units: "millimeters".to_owned(),
    };
    let mut mesh_type_seen = false;
    let mut units_seen = false;
    let mut arguments = arguments.iter();
    while let Some(option) = arguments.next() {
        let value = arguments
            .next()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("缺少有效的 UTF-8 参数值：{}", option.to_string_lossy()))?;
        if option == OsStr::new("--mesh-type") && !mesh_type_seen {
            options.mesh_type = value.to_owned();
            mesh_type_seen = true;
        } else if option == OsStr::new("--units") && !units_seen {
            options.units = value.to_owned();
            units_seen = true;
        } else {
            return Err(format!(
                "未知或重复的导入选项：{}",
                option.to_string_lossy()
            ));
        }
    }
    Ok(options)
}

fn expanded_mesh_estimate_bytes(triangle_count: u64) -> u128 {
    u128::from(triangle_count) * BYTES_PER_EXPANDED_TRIANGLE + SIDECAR_HEADER_ESTIMATE_BYTES
}

#[cfg(test)]
mod tests {
    use super::{
        BYTES_PER_EXPANDED_TRIANGLE, DEFAULT_MESH_TYPE, EXPANDED_MESH_WARNING_BYTES, ImportOptions,
        expanded_mesh_estimate_bytes, parse_import_options,
    };
    use std::ffi::OsString;

    #[test]
    fn import_options_reuse_the_domain_default_values() {
        assert_eq!(
            parse_import_options(&[]),
            Ok(ImportOptions {
                mesh_type: DEFAULT_MESH_TYPE.to_owned(),
                units: "millimeters".to_owned(),
            })
        );
    }

    #[test]
    fn import_options_accept_explicit_values_and_reject_bad_syntax() {
        assert_eq!(
            parse_import_options(&[
                OsString::from("--units"),
                OsString::from("inches"),
                OsString::from("--mesh-type"),
                OsString::from("midplane"),
            ]),
            Ok(ImportOptions {
                mesh_type: "midplane".to_owned(),
                units: "inches".to_owned(),
            })
        );
        assert!(parse_import_options(&[OsString::from("--unknown"), OsString::from("x")]).is_err());
        assert!(parse_import_options(&[OsString::from("--units")]).is_err());
    }

    #[test]
    fn expanded_mesh_estimate_matches_the_sidecar_ir_size_and_warning_boundary() {
        assert_eq!(
            expanded_mesh_estimate_bytes(1),
            BYTES_PER_EXPANDED_TRIANGLE + 64
        );
        assert!(expanded_mesh_estimate_bytes(29_826_161) > EXPANDED_MESH_WARNING_BYTES);
        assert!(expanded_mesh_estimate_bytes(29_826_160) < EXPANDED_MESH_WARNING_BYTES);
    }
}
