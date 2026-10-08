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
