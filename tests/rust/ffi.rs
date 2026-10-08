//! CXX 服务适配与所有权的 Rust 单元回归。
use super::{
    FfiRequest, FfiResponse, MAX_LABEL_BYTES, activation_outcome_to_bridge, bridge,
    install_crash_handler, panic_probe, path_ref_parse, path_service_new, path_service_resolve,
    path_service_resolve_existing, path_service_resolve_write_target, path_service_set_root,
    process, project_service_create, project_service_current, project_service_execute,
    project_service_mesh_snapshot_for_import, project_service_new, project_service_open,
    project_service_resident_mesh_ids, project_service_save, session_close, session_create,
    session_label, session_live_count, task_service_cancel, task_service_drain, task_service_new,
    task_service_recent_logs, task_service_running, task_service_submit,
};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// 存活计数是进程级共享状态；触碰它的测试先取锁串行化，避免并行互扰。
fn session_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn round_trip_calls_cpp_and_preserves_unicode() -> Result<(), Box<dyn std::error::Error>> {
    let response = process(&FfiRequest {
        text: "界".to_owned(),
        repeat: 2,
    })?; // valid request failed: {error}
    assert_eq!(
        response,
        FfiResponse {
            value: "ffi:界界".to_owned(),
            repeat: 2,
        }
    );
    Ok(())
}

#[test]
fn empty_text_is_a_structured_error() {
    let error = match process(&FfiRequest {
        text: String::new(),
        repeat: 1,
    }) {
        Ok(response) => panic!("empty input unexpectedly succeeded: {response:?}"),
        Err(error) => error,
    };
    assert_eq!(error, "ffi.empty_input");
}

#[test]
fn repeat_bounds_are_rejected() {
    for repeat in [0, 9] {
        let error = match process(&FfiRequest {
            text: "x".to_owned(),
            repeat,
        }) {
            Ok(response) => panic!("invalid repeat unexpectedly succeeded: {response:?}"),
            Err(error) => error,
        };
        assert!(error.starts_with("ffi.invalid_repeat:"));
    }
}

#[test]
#[should_panic(expected = "ffi.panic_probe")]
fn panic_probe_panics_with_boundary_code() {
    panic_probe();
}

#[test]
fn sessions_create_use_and_release_balance() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = session_lock();
    assert_eq!(session_live_count(), 0);

    let alpha = match session_create("会话-α".to_owned()) {
        Ok(session) => session,
        Err(error) => panic!("valid label rejected: {error}"),
    };
    let beta = match session_create("会话-β".to_owned()) {
        Ok(session) => session,
        Err(error) => panic!("valid label rejected: {error}"),
    };
    assert_eq!(session_live_count(), 2);
    assert_eq!(session_label(&alpha), "会话-α");
    assert_eq!(session_label(&beta), "会话-β");

    // 逆序释放：证明各 Box 独立持有，释放顺序不影响计数平衡。
    assert_eq!(session_close(beta), 1);
    assert_eq!(session_close(alpha), 0);
    assert_eq!(session_live_count(), 0);
    Ok(())
}

#[test]
fn session_rejects_invalid_labels_without_leaking() {
    let _guard = session_lock();
    match session_create(String::new()) {
        Ok(_) => panic!("empty label unexpectedly accepted"),
        Err(error) => assert_eq!(error, "ffi.empty_label"),
    }
    match session_create("x".repeat(MAX_LABEL_BYTES + 1)) {
        Ok(_) => panic!("oversized label unexpectedly accepted"),
        Err(error) => assert!(error.starts_with("ffi.invalid_label:")),
    }
    assert_eq!(session_live_count(), 0);
}

#[test]
fn session_drop_releases_live_count() {
    let _guard = session_lock();
    let session = match session_create("drop".to_owned()) {
        Ok(session) => session,
        Err(error) => panic!("valid label rejected: {error}"),
    };
    assert_eq!(session_live_count(), 1);
    // 不经 session_close 的 Box 析构走同一 Drop 路径。
    drop(session);
    assert_eq!(session_live_count(), 0);
}

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

    let base = std::fs::canonicalize(std::env::temp_dir()).unwrap_or_else(|_| std::env::temp_dir());
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
fn task_service_drains_events_through_bridge() -> Result<(), Box<dyn std::error::Error>> {
    let service = task_service_new();
    let id = match task_service_submit(&service, "桥接-θ".to_owned(), 20, false) {
        Ok(id) => id,
        Err(error) => panic!("submit failed: {error}"),
    };

    // 轮询到终态；等待上界远大于任务时长，避免偶发失败。
    for _ in 0..400 {
        if task_service_running(&service) == 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(task_service_running(&service), 0);

    let events = task_service_drain(&service);
    let started = events
        .iter()
        .position(|event| matches!(event.kind, bridge::TaskEventKind::Started));
    let succeeded = events
        .iter()
        .position(|event| matches!(event.kind, bridge::TaskEventKind::Succeeded));
    match (started, succeeded) {
        (Some(started_index), Some(succeeded_index)) => {
            assert!(started_index < succeeded_index)
        }
        _ => panic!("expected started+succeeded, got started={started:?} succeeded={succeeded:?}"),
    }
    assert!(events.iter().all(|event| event.task_id == id));

    let logs = task_service_recent_logs(&service);
    assert!(
        logs.iter()
            .any(|line| line.task_id == id && line.message.contains("succeeded"))
    );

    // 无效输入映射为稳定错误码。
    match task_service_submit(&service, String::new(), 1, false) {
        Ok(_) => panic!("empty label accepted"),
        Err(error) => assert_eq!(error, "task.empty_label"),
    }
    match task_service_submit(&service, "x".to_owned(), 60_001, false) {
        Ok(_) => panic!("oversized duration accepted"),
        Err(error) => assert!(error.starts_with("task.invalid_duration:")),
    }
    Ok(())
}

#[test]
fn task_service_lifecycle_maps_cancel_failed_and_cancelled_events()
-> Result<(), Box<dyn std::error::Error>> {
    let service = task_service_new();
    // 失败任务:Failed 事件经桥接枚举映射。
    let failed_id = task_service_submit(&service, "失败任务".to_owned(), 10, true)?;
    for _ in 0..400 {
        if task_service_running(&service) == 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let events = task_service_drain(&service);
    assert!(
        events.iter().any(|event| {
            event.task_id == failed_id
                && matches!(event.kind, bridge::TaskEventKind::Failed)
                && !event.code.is_empty()
        }),
        "失败任务必须携带 Failed 事件与错误码"
    );
    // 取消长任务:Cancelled 事件经桥接枚举映射;cancel 返回 true。
    let long_id = task_service_submit(&service, "长任务".to_owned(), 5_000, false)?;
    assert!(
        task_service_cancel(&service, long_id),
        "运行中任务必须可取消"
    );
    for _ in 0..400 {
        if task_service_running(&service) == 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let events = task_service_drain(&service);
    assert!(
        events.iter().any(|event| {
            event.task_id == long_id && matches!(event.kind, bridge::TaskEventKind::Cancelled)
        }),
        "取消必须产生 Cancelled 事件"
    );
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

fn wait_confirmation(
    service: &mut super::ProjectService,
    finish: fn(&mut super::ProjectService) -> Result<bool, panta_core::project::ProjectError>,
) -> Result<(), String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !finish(service).map_err(|error| error.to_string())? {
        if std::time::Instant::now() >= deadline {
            return Err("metadata confirmation timed out".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    Ok(())
}

#[test]
fn language_bridge_validates_before_committing() -> Result<(), String> {
    let mut service = super::language_service_new();
    assert_eq!(super::language_service_supported_locales(), ["en", "zh-CN"]);
    assert_eq!(super::language_service_current(&service), "en");
    assert_eq!(
        super::language_service_validate(&service, "zh-CN".into())?,
        "zh-CN"
    );
    assert_eq!(super::language_service_current(&service), "en");
    assert!(super::language_service_validate(&service, "unknown".into()).is_err());
    assert!(super::language_service_commit(&mut service, "unknown".into()).is_err());
    assert_eq!(
        super::language_service_commit(&mut service, "zh-CN".into())?,
        "zh-CN"
    );
    assert_eq!(super::language_service_current(&service), "zh-CN");
    Ok(())
}

#[test]
fn plan_bridge_preserves_catalogs_confirmation_and_reopened_values()
-> Result<(), Box<dyn std::error::Error>> {
    use super::{
        project_service_begin_fill_settings_confirmation,
        project_service_begin_gate_location_settings_confirmation,
        project_service_begin_material_confirmation,
        project_service_finish_fill_settings_confirmation,
        project_service_finish_gate_location_settings_confirmation,
        project_service_finish_material_confirmation, project_service_plan_settings,
        project_service_set_analysis_sequence,
    };
    let root = std::env::temp_dir().join(format!("panta-ffi-plan-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let source = root.join("part.stl");
    fs::write(
        &source,
        b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendsolid\n",
    )?;
    assert_eq!(super::default_mesh_type(), "dual-domain");
    assert_eq!(super::mesh_type_catalog().len(), 3);
    assert_eq!(super::analysis_sequence_catalog().len(), 10);
    let material = super::default_material()?;
    assert!(!material.properties.is_empty());
    let mut service = project_service_new();
    let empty = project_service_plan_settings(&service, "");
    assert!(empty.import_id.is_empty());
    assert_eq!(empty.fill_settings.holding_profile.len(), 2);
    assert_eq!(empty.gate_location_settings.number_of_gates, 1);
    assert!(project_service_finish_material_confirmation(&mut service).is_err());
    assert!(project_service_finish_fill_settings_confirmation(&mut service).is_err());
    assert!(project_service_finish_gate_location_settings_confirmation(&mut service).is_err());
    let project = project_service_create(&mut service, root.display().to_string(), "Demo".into())?;
    let record = super::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".into(),
        "millimeters".into(),
        false,
    )?;
    let initial = project_service_plan_settings(&service, &record.id);
    assert_eq!(initial.mesh_type, "solid-3d");
    assert_eq!(initial.sequence_id, "fill");
    assert!(
        project_service_set_analysis_sequence(
            &mut service,
            &project.path,
            initial.revision,
            &record.id,
            "unknown"
        )
        .is_err()
    );
    assert!(
        project_service_begin_material_confirmation(
            &mut service,
            &project.path,
            initial.revision,
            &record.id,
            "unknown"
        )
        .is_err()
    );
    assert!(project_service_begin_material_confirmation(
        &mut service,
        &project.path,
        initial.revision,
        &record.id,
        &material.id
    )?);
    assert!(
        project_service_begin_fill_settings_confirmation(
            &mut service,
            &project.path,
            initial.revision,
            &record.id,
            initial.fill_settings
        )
        .is_err()
    );
    wait_confirmation(&mut service, project_service_finish_material_confirmation)?;
    let selected = project_service_plan_settings(&service, &record.id);
    assert_eq!(selected.material_id, material.id);
    assert_eq!(selected.material_source_text, material.source_text);
    assert!(!project_service_begin_material_confirmation(
        &mut service,
        &project.path,
        selected.revision,
        &record.id,
        &material.id
    )?);
    let mut fill = selected.fill_settings;
    fill.melt_temperature_celsius = 240.0;
    fill.holding_profile.push(bridge::HoldingProfilePoint {
        duration_seconds: 5.0,
        pressure_percent: 70.0,
    });
    assert!(project_service_begin_fill_settings_confirmation(
        &mut service,
        &project.path,
        selected.revision,
        &record.id,
        fill
    )?);
    wait_confirmation(
        &mut service,
        project_service_finish_fill_settings_confirmation,
    )?;
    let saved = project_service_plan_settings(&service, &record.id);
    assert!(saved.fill_settings_confirmed);
    assert_eq!(saved.fill_settings.melt_temperature_celsius, 240.0);
    assert_eq!(
        saved.fill_settings.holding_profile[2].pressure_percent,
        70.0
    );
    let revision = saved.revision;
    assert!(!project_service_begin_fill_settings_confirmation(
        &mut service,
        &project.path,
        revision,
        &record.id,
        saved.fill_settings
    )?);
    let mut invalid = project_service_plan_settings(&service, &record.id).fill_settings;
    invalid.flow_rate_cm3_per_second = -1.0;
    assert!(project_service_begin_fill_settings_confirmation(
        &mut service,
        &project.path,
        revision,
        &record.id,
        invalid
    )?);
    assert!(
        wait_confirmation(
            &mut service,
            project_service_finish_fill_settings_confirmation
        )
        .is_err()
    );
    assert_eq!(
        project_service_plan_settings(&service, &record.id).revision,
        revision
    );
    let changed = project_service_set_analysis_sequence(
        &mut service,
        &project.path,
        revision,
        &record.id,
        "gate-location",
    )?;
    let mut gate = project_service_plan_settings(&service, &record.id).gate_location_settings;
    gate.algorithm_id = "unknown".into();
    assert!(
        project_service_begin_gate_location_settings_confirmation(
            &mut service,
            &project.path,
            changed.revision,
            &record.id,
            gate
        )
        .is_err()
    );
    let mut gate = project_service_plan_settings(&service, &record.id).gate_location_settings;
    gate.number_of_gates = 3;
    assert!(project_service_begin_gate_location_settings_confirmation(
        &mut service,
        &project.path,
        changed.revision,
        &record.id,
        gate
    )?);
    wait_confirmation(
        &mut service,
        project_service_finish_gate_location_settings_confirmation,
    )?;
    let gate_saved = project_service_plan_settings(&service, &record.id);
    assert!(gate_saved.gate_location_settings_confirmed);
    assert_eq!(gate_saved.gate_location_settings.number_of_gates, 3);
    let mut reopened = project_service_new();
    project_service_open(&mut reopened, project.path)?;
    let snapshot = project_service_plan_settings(&reopened, &record.id);
    assert_eq!(snapshot.revision, gate_saved.revision);
    assert_eq!(snapshot.material_id, material.id);
    assert_eq!(snapshot.fill_settings.holding_profile.len(), 3);
    assert_eq!(snapshot.fill_settings.melt_temperature_celsius, 240.0);
    assert_eq!(snapshot.gate_location_settings.number_of_gates, 3);
    assert!(snapshot.fill_settings_confirmed && snapshot.gate_location_settings_confirmed);
    assert!(super::project_service_activate_mesh_document(
        &mut service,
        &record.id
    ));
    super::project_service_deactivate_mesh_document(&mut service);
    super::project_service_release_mesh_document(&mut service, &record.id);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn project_service_bridge_maps_snapshots_and_errors() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("panta-ffi-project-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;

    let mut service = project_service_new();
    let no_project = match project_service_current(&service) {
        Ok(snapshot) => panic!("empty service returned {}", snapshot.name),
        Err(error) => error,
    };
    assert_eq!(no_project.code(), "project.no_project");

    let create_error =
        match project_service_create(&mut service, "relative".to_owned(), "Demo".to_owned()) {
            Ok(snapshot) => panic!("relative project created at {}", snapshot.path),
            Err(error) => error,
        };
    assert_eq!(create_error.code(), "project.location_not_absolute");
    assert_eq!(create_error.detail(), "relative");
    let save_error = match project_service_save(&mut service) {
        Ok(snapshot) => panic!("empty service saved {}", snapshot.name),
        Err(error) => error,
    };
    assert_eq!(save_error.code(), "project.no_project");

    let created =
        project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
    assert_eq!(created.name, "Demo");
    assert!(!created.dirty);
    let current = project_service_current(&service)?;
    assert_eq!(current.path, created.path);
    assert_eq!(current.name, "Demo");

    let changed = project_service_execute(
        &mut service,
        bridge::ProjectCommand {
            kind: bridge::ProjectCommandKind::Rename,
            value: "Renamed".to_owned(),
        },
    )?;
    assert_eq!(changed.name, "Renamed");
    assert!(changed.dirty);

    let saved = project_service_save(&mut service)?;
    assert_eq!(saved.revision, 1);
    assert!(!saved.dirty);
    let unchanged = match project_service_execute(
        &mut service,
        bridge::ProjectCommand {
            kind: bridge::ProjectCommandKind::Rename,
            value: "Renamed".to_owned(),
        },
    ) {
        Ok(snapshot) => panic!(
            "unchanged rename succeeded at revision {}",
            snapshot.revision
        ),
        Err(error) => error,
    };
    assert_eq!(unchanged.code(), "project.command_invalid");
    assert_eq!(unchanged.detail(), "name is unchanged");

    let mut reopened = project_service_new();
    let opened = project_service_open(&mut reopened, created.path.clone())?;
    assert_eq!(opened.name, "Renamed");
    assert_eq!(opened.revision, 1);
    let invalid_file = PathBuf::from(&created.path).with_extension("json");
    fs::write(&invalid_file, br#"{}"#)?;
    let invalid_file_error =
        match project_service_open(&mut reopened, invalid_file.display().to_string()) {
            Ok(snapshot) => panic!("non-panta file opened as {}", snapshot.name),
            Err(error) => error,
        };
    assert_eq!(
        invalid_file_error.detail(),
        invalid_file.display().to_string()
    );

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn project_activation_bridge_admits_cancels_and_drains_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("panta-ffi-act-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let source = root.join("part.stl");
    let original = b"solid part
vertex 0 0 0
vertex 1 0 0
vertex 0 2 0
endsolid
";
    fs::write(&source, original)?;

    let mut service = project_service_new();
    let created =
        project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
    crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "millimeters".to_owned(),
        false,
    )?;

    // 未知记录在提交边界同步拒绝；未知 attempt 取消返回 false。
    let missing = match crate::project_service_begin_asset_activation(&mut service, "import-99") {
        Ok(attempt) => panic!("unknown record admitted as attempt {}", attempt.attempt),
        Err(error) => error,
    };
    assert_eq!(missing.code(), "project.import_record_missing");
    assert!(!crate::project_service_cancel_asset_activation(
        &mut service,
        u64::MAX
    ));

    let attempt = crate::project_service_begin_asset_activation(&mut service, "import-1")?;
    assert!(attempt.attempt > 0);
    assert!(crate::project_service_cancel_asset_activation(
        &mut service,
        attempt.attempt
    ));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let outcome = loop {
        let drained = crate::project_service_drain_asset_activations(&mut service);
        if let Some(outcome) = drained.into_iter().next() {
            break outcome;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "activation outcome did not arrive in time"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(outcome.attempt, attempt.attempt);
    assert_eq!(outcome.generation, attempt.generation);
    assert_eq!(outcome.import_id, "import-1");
    assert!(
        outcome.kind == bridge::ActivationOutcomeKind::Succeeded
            || outcome.kind == bridge::ActivationOutcomeKind::Cancelled,
        "小文件激活终态只可能是成功（先完成）或取消（先命中检查点）"
    );
    if outcome.kind == bridge::ActivationOutcomeKind::Succeeded {
        // 单三角形：9 个 f64 坐标必须完整过桥。
        assert_eq!(outcome.coordinates.len(), 9);
    } else {
        assert!(outcome.coordinates.is_empty(), "取消不得携带坐标负载");
    }
    let _ = created;
    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn project_stl_bridge_preserves_preview_commit_and_snapshot_state()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("panta-ffi-stl-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let source = root.join("part.stl");
    let original = b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 2 0\nendsolid\n";
    fs::write(&source, original)?;

    let mut service = project_service_new();
    let no_project = match crate::project_service_imports(&service) {
        Ok(imports) => panic!("empty project returned {} imports", imports.len()),
        Err(error) => error,
    };
    assert_eq!(no_project.code(), "project.no_project");
    let preview = crate::project_service_inspect_stl(&mut service, source.display().to_string())?;
    assert_eq!(preview.source_name, "part.stl");
    assert_eq!(preview.triangle_count, 1);
    assert_eq!(
        [preview.size_x, preview.size_y, preview.size_z],
        [1.0, 2.0, 0.0]
    );
    let import_without_project = match crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "millimeters".to_owned(),
        false,
    ) {
        Ok(imported) => panic!("import without a project succeeded as {}", imported.id),
        Err(error) => error,
    };
    assert_eq!(import_without_project.code(), "project.no_project");

    let created =
        project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
    assert!(project_service_resident_mesh_ids(&service).is_empty());
    assert!(crate::project_service_imports(&service)?.is_empty());

    let imported = crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "centimeters".to_owned(),
        true,
    )?;
    assert_eq!(imported.id, "import-1");
    assert_eq!(imported.source_name, "part.stl");
    assert_eq!(imported.asset, "assets/imports/0001-part.stl");
    assert_eq!(imported.mesh_type, "solid-3d");
    assert_eq!(imported.units, "centimeters");
    assert!(imported.show_import_log);
    assert_eq!(imported.triangle_count, 1);
    assert_eq!(
        [imported.size_x, imported.size_y, imported.size_z],
        [1.0, 2.0, 0.0]
    );

    let imports = crate::project_service_imports(&service)?;
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].parser_version, imported.parser_version);
    let snapshot = project_service_mesh_snapshot_for_import(&service, &imported.id)?;
    assert_eq!(snapshot.revision, 1);
    assert_eq!(
        snapshot.coordinates,
        [0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 20.0, 0.0]
    );

    let mut reopened = project_service_new();
    project_service_open(&mut reopened, created.path)?;
    // 打开工程仅读取清单；已保存网格须通过只读激活按需恢复（073/080）。
    assert!(project_service_resident_mesh_ids(&reopened).is_empty());
    let unloaded = match project_service_mesh_snapshot_for_import(&reopened, &imported.id) {
        Ok(_) => panic!("unloaded mesh returned a display snapshot"),
        Err(error) => error,
    };
    assert_eq!(unloaded.code, "project.mesh_not_resident");
    assert_eq!(unloaded.detail, imported.id);

    let step_source = root.join("part.step");
    fs::write(&step_source, b"not a supported STL source")?;
    let unsupported =
        match crate::project_service_inspect_stl(&mut service, step_source.display().to_string()) {
            Ok(preview) => panic!(
                "STEP file unexpectedly parsed as {} triangles",
                preview.triangle_count
            ),
            Err(error) => error,
        };
    assert_eq!(unsupported.code(), "project.import_invalid_file");

    crate::project_service_inspect_stl(&mut service, source.display().to_string())?;
    fs::write(&source, b"vertex 0 0 0\nvertex 3 0 0\nvertex 0 4 0\n")?;
    let changed = match crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "millimeters".to_owned(),
        false,
    ) {
        Ok(imported) => panic!("changed source was imported as {}", imported.id),
        Err(error) => error,
    };
    assert_eq!(changed.code(), "project.import_source_changed");
    assert_eq!(crate::project_service_imports(&service)?.len(), 1);

    fs::write(&source, original)?;
    let invalid_options = match crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "unknown".to_owned(),
        "millimeters".to_owned(),
        false,
    ) {
        Ok(imported) => panic!("unsupported mesh type was imported as {}", imported.id),
        Err(error) => error,
    };
    assert_eq!(
        invalid_options.code(),
        "project.import_unsupported_mesh_type"
    );
    assert_eq!(crate::project_service_imports(&service)?.len(), 1);

    let invalid_units = match crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "yards".to_owned(),
        false,
    ) {
        Ok(imported) => panic!("unsupported units were imported as {}", imported.id),
        Err(error) => error,
    };
    assert_eq!(invalid_units.code(), "project.import_unsupported_units");
    assert_eq!(crate::project_service_imports(&service)?.len(), 1);

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn failed_activation_transfers_structured_diagnostics_without_mesh_payload() {
    let mut service = project_service_new();
    let outcome = panta_core::project::Outcome {
        attempt: 7,
        generation: 3,
        import_id: "import-2".into(),
        kind: panta_core::project::OutcomeKind::Failed,
        code: "project.asset_read_failed".into(),
        category: "io",
        detail: "C:\\资产: read denied".into(),
        mesh: None,
    };
    let result = activation_outcome_to_bridge(&mut service.service, outcome);
    assert!(result.kind == bridge::ActivationOutcomeKind::Failed);
    assert_eq!(result.attempt, 7);
    assert_eq!(result.generation, 3);
    assert_eq!(result.import_id, "import-2");
    assert_eq!(result.error.code, "project.asset_read_failed");
    assert_eq!(result.error.category, "io");
    assert_eq!(result.error.detail, "C:\\资产: read denied");
    assert!(result.coordinates.is_empty());
}

#[test]
fn activation_without_a_current_attempt_does_not_cross_the_bridge() {
    let mut service = project_service_new();
    let outcome = |attempt, x| panta_core::project::Outcome {
        attempt,
        generation: 0,
        import_id: "import-1".to_owned(),
        kind: panta_core::project::OutcomeKind::Succeeded,
        code: String::new(),
        category: "",
        detail: String::new(),
        mesh: Some(panta_mesh::SurfaceMesh {
            triangles: vec![[[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]]],
        }),
    };

    let first = activation_outcome_to_bridge(&mut service.service, outcome(1, 1.0));
    let second = activation_outcome_to_bridge(&mut service.service, outcome(2, 10.0));

    assert!(first.kind == bridge::ActivationOutcomeKind::Expired);
    assert!(first.coordinates.is_empty());
    assert!(second.kind == bridge::ActivationOutcomeKind::Expired);
    assert!(second.coordinates.is_empty());
}

#[test]
fn tet_mesh_bridge_checks_layout_and_domain_data() {
    let mesh_data = || bridge::TetMeshData {
        nodes: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        tets: vec![0, 1, 2, 3],
        tet_regions: vec![0],
        boundary: vec![0, 1, 2],
        boundary_groups: vec![0],
        region_count: 1,
        boundary_group_count: 1,
    };

    let valid = crate::mesh_validate_tet(mesh_data());
    assert!(valid.issues.is_empty());
    assert_eq!(valid.volume_mm3, 1.0 / 6.0);

    let mut invalid_layout = mesh_data();
    invalid_layout.nodes.pop();
    let layout = crate::mesh_validate_tet(invalid_layout);
    assert_eq!(layout.issues, ["invalid mesh DTO layout"]);
    assert_eq!(layout.volume_mm3, 0.0);

    let mut invalid_mesh = mesh_data();
    invalid_mesh.tets[3] = 9;
    let domain = crate::mesh_validate_tet(invalid_mesh);
    assert!(!domain.issues.is_empty());
    assert_eq!(domain.volume_mm3, 0.0);
}

#[test]
fn crash_handler_rejects_a_file_used_as_log_directory() -> std::io::Result<()> {
    let path = std::env::temp_dir().join(format!("panta-ffi-crash-not-dir-{}", std::process::id()));
    std::fs::write(&path, b"not a directory")?;
    let text = path.to_str().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "临时路径不是 UTF-8")
    })?;
    let error = install_crash_handler(text)
        .err()
        .ok_or_else(|| std::io::Error::other("文件路径不应成为日志目录"))?;
    assert!(!error.is_empty());
    std::fs::remove_file(path)?;
    Ok(())
}
