//! 路径根 / 引用的 CXX 映射；包含检查与解析规则由 core 拥有。
use crate::bridge;

/// 路径服务的 FFI 包装（任务 023）：只做枚举/DTO 与错误文本映射，
/// 规则与包含检查全部在 panta-core 实现，不在边界复制。
pub struct PathService {
    service: panta_core::path::PathService,
}

pub(super) fn path_service_new() -> Box<PathService> {
    Box::new(PathService {
        service: panta_core::path::PathService::new(),
    })
}

pub(super) fn path_service_set_root(
    service: &mut PathService,
    kind: bridge::PathRootKind,
    utf8_root: String,
) -> Result<(), String> {
    let category = core_category(kind)?;
    service
        .service
        .set_root(category, std::path::Path::new(&utf8_root))
        .map_err(|error| error.to_string())
}

pub(super) fn path_ref_parse(reference: String) -> Result<bridge::PathRef, String> {
    let parsed = panta_core::path::ResourceRef::parse(&reference)?;
    Ok(bridge::PathRef {
        kind: bridge_kind(parsed.category),
        relative: parsed.relative,
    })
}

pub(super) fn path_ref_to_logical(reference: &bridge::PathRef) -> Result<String, String> {
    let category = core_category(reference.kind)?;
    Ok(panta_core::path::ResourceRef {
        category,
        relative: reference.relative.clone(),
    }
    .to_logical())
}

pub(super) fn path_service_resolve(
    service: &PathService,
    reference: &bridge::PathRef,
) -> Result<String, String> {
    resolve_with(service, reference, panta_core::path::PathService::resolve)
}

pub(super) fn path_service_resolve_existing(
    service: &PathService,
    reference: &bridge::PathRef,
) -> Result<String, String> {
    resolve_with(
        service,
        reference,
        panta_core::path::PathService::resolve_existing,
    )
}

pub(super) fn path_service_resolve_write_target(
    service: &PathService,
    reference: &bridge::PathRef,
) -> Result<String, String> {
    resolve_with(
        service,
        reference,
        panta_core::path::PathService::resolve_write_target,
    )
}

fn resolve_with(
    service: &PathService,
    reference: &bridge::PathRef,
    resolver: fn(
        &panta_core::path::PathService,
        &panta_core::path::ResourceRef,
    ) -> Result<std::path::PathBuf, panta_core::path::PathError>,
) -> Result<String, String> {
    let core_ref = panta_core::path::ResourceRef {
        category: core_category(reference.kind)?,
        relative: reference.relative.clone(),
    };
    resolver(&service.service, &core_ref)
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

fn core_category(kind: bridge::PathRootKind) -> Result<panta_core::path::RootCategory, String> {
    match kind {
        bridge::PathRootKind::Project => Ok(panta_core::path::RootCategory::Project),
        bridge::PathRootKind::UserConfig => Ok(panta_core::path::RootCategory::UserConfig),
        bridge::PathRootKind::AppData => Ok(panta_core::path::RootCategory::AppData),
        bridge::PathRootKind::Cache => Ok(panta_core::path::RootCategory::Cache),
        bridge::PathRootKind::Session => Ok(panta_core::path::RootCategory::Session),
        bridge::PathRootKind::Qrc => Ok(panta_core::path::RootCategory::Qrc),
        _ => Err("path.invalid_kind".to_owned()),
    }
}

fn bridge_kind(category: panta_core::path::RootCategory) -> bridge::PathRootKind {
    match category {
        panta_core::path::RootCategory::Project => bridge::PathRootKind::Project,
        panta_core::path::RootCategory::UserConfig => bridge::PathRootKind::UserConfig,
        panta_core::path::RootCategory::AppData => bridge::PathRootKind::AppData,
        panta_core::path::RootCategory::Cache => bridge::PathRootKind::Cache,
        panta_core::path::RootCategory::Session => bridge::PathRootKind::Session,
        panta_core::path::RootCategory::Qrc => bridge::PathRootKind::Qrc,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bridge, path_ref_parse, path_service_new, path_service_resolve,
        path_service_resolve_existing, path_service_resolve_write_target, path_service_set_root,
    };
    #[test]
    fn path_service_round_trips_references_and_rejects_escapes()
    -> Result<(), Box<dyn std::error::Error>> {
        let parsed = crate::path_ref_parse("project:/资产 齿轮/a.step".to_owned())?;
        assert!(matches!(parsed.kind, bridge::PathRootKind::Project));
        assert_eq!(parsed.relative, "资产 齿轮/a.step");
        let logical = match crate::path_ref_to_logical(&parsed) {
            Ok(logical) => logical,
            Err(error) => panic!("生成逻辑地址失败: {error}"),
        };
        assert_eq!(logical, "project:/资产 齿轮/a.step");
        match crate::path_ref_parse("workspace:/x".to_owned()) {
            Ok(parsed) => panic!("未知 scheme 被接受: {:?}", parsed.relative),
            Err(error) => assert_eq!(error, "path.unknown_scheme: workspace"),
        }

        let base =
            std::fs::canonicalize(std::env::temp_dir()).unwrap_or_else(|_| std::env::temp_dir());
        let root = base.join(format!("panta-ffi-path-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?; // mkdir 失败: {error}
        let mut service = crate::path_service_new();
        if let Err(error) = crate::path_service_set_root(
            &mut service,
            bridge::PathRootKind::Project,
            root.display().to_string(),
        ) {
            panic!("注入根失败: {error}");
        }
        let resolved = match crate::path_service_resolve(
            &service,
            &bridge::PathRef {
                kind: bridge::PathRootKind::Project,
                relative: "out/../a.pa".to_owned(),
            },
        ) {
            Ok(path) => path,
            Err(error) => panic!("合法引用被拒绝: {error}"),
        };
        assert_eq!(resolved, root.join("a.pa").display().to_string());

        for (relative, code) in [
            ("../escape", "path.parent_escape"),
            ("C:/win", "path.absolute_rejected"),
            ("COM1", "path.reserved_name"),
        ] {
            let error = match crate::path_service_resolve(
                &service,
                &bridge::PathRef {
                    kind: bridge::PathRootKind::Project,
                    relative: relative.to_owned(),
                },
            ) {
                Ok(path) => panic!("{relative} 意外通过: {path}"),
                Err(error) => error,
            };
            assert!(error.starts_with(code), "{relative} -> {error}");
        }
        match crate::path_service_resolve(
            &service,
            &bridge::PathRef {
                kind: bridge::PathRootKind::Qrc,
                relative: "icons/x.svg".to_owned(),
            },
        ) {
            Ok(path) => panic!("qrc 被解析为本机路径: {path}"),
            Err(error) => assert_eq!(error, "path.qrc_not_native"),
        }
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn path_service_covers_existing_write_target_and_all_kinds()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::path::Path;

        let base = std::fs::canonicalize(std::env::temp_dir())?;
        let root = base.join(format!("panta-ffi-path-kinds-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;
        let mut service = path_service_new();

        // 全类别注入(UserConfig/AppData/Cache/Session 各指临时子目录):
        // core_category 与 set_root 的全部分支被真实执行。
        for (kind, name) in [
            (bridge::PathRootKind::UserConfig, "user-config"),
            (bridge::PathRootKind::AppData, "app-data"),
            (bridge::PathRootKind::Cache, "cache"),
            (bridge::PathRootKind::Session, "session"),
        ] {
            let directory = root.join(name);
            std::fs::create_dir_all(&directory)?;
            path_service_set_root(&mut service, kind, directory.display().to_string())?;
        }
        let invalid_root = match path_service_set_root(
            &mut service,
            bridge::PathRootKind::Project,
            "relative-root".to_owned(),
        ) {
            Ok(()) => panic!("relative root unexpectedly accepted"),
            Err(error) => error,
        };
        assert!(invalid_root.starts_with("path.root_not_absolute:"));

        // 每个类别都可纯逻辑解析,且 scheme 往返经 bridge_kind 全分支。
        for scheme in ["user-config", "app-data", "cache", "session"] {
            let parsed = path_ref_parse(format!("{scheme}:/配置/x.pa"))?;
            let resolved = path_service_resolve(&service, &parsed)?;
            assert!(resolved.contains(scheme), "{scheme} -> {resolved}");
        }

        // 写目标(未创建)与读解析(现存)在 cache 类别走通全链。
        let reference = bridge::PathRef {
            kind: bridge::PathRootKind::Cache,
            relative: "out/新 口袋/pocket.pa".to_owned(),
        };
        let target = path_service_resolve_write_target(&service, &reference)?;
        let parent = Path::new(&target)
            .parent()
            .ok_or_else(|| format!("目标缺少父目录: {target}"))?;
        std::fs::create_dir_all(parent)?;
        std::fs::write(&target, b"pa")?;
        path_service_resolve_existing(&service, &reference)?;
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }
}
