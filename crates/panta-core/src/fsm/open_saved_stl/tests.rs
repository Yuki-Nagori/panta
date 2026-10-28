use super::*;

fn accept_activation_request(
    _: Arc<ActivationCoordinator>,
    _: u64,
    _: ActivationRequest,
    _: Arc<AtomicBool>,
) -> std::io::Result<()> {
    Ok(())
}

#[test]
fn sidecar_header_above_two_gib_is_accepted_by_format_policy()
-> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!(
        "panta-large-mesh-{}-{}.pmesh",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let mut header = [0_u8; 56];
    header[..8].copy_from_slice(b"PNTMESH\0");
    header[8..12].copy_from_slice(&panta_mesh::MESH_ASSET_FORMAT_VERSION.to_le_bytes());
    header[12] = 1;
    header[13] = 1;
    header[14] = 1; // Zstandard codec.
    header[16..24].copy_from_slice(&(2_u64 * 1024 * 1024 * 1024).to_le_bytes());
    let file = File::create(&path)?;
    use std::io::Write;
    (&file).write_all(&header)?;
    file.set_len(2_u64 * 1024 * 1024 * 1024 + header.len() as u64)?;
    drop(file);

    // Cancellation after header validation proves the read path accepts the size
    // without allocating or touching the sparse payload.
    let result = read_mesh_bytes(&path, || true);
    std::fs::remove_file(&path)?;
    assert!(matches!(result, Ok(None)));
    Ok(())
}

#[test]
fn activation_diagnostics_preserve_typed_categories_and_context() {
    let context = "参数: 中文 / C:\\零件.stl";
    let cases = [
        (
            ImportError::Missing(context.into()),
            "project.asset_missing",
            "missing",
            context,
        ),
        (
            ImportError::UnsupportedFormat(context.into()),
            "project.asset_unsupported_format",
            "format",
            context,
        ),
        (
            ImportError::Read(context.into()),
            "project.asset_read_failed",
            "io",
            context,
        ),
        (
            ImportError::SourceChanged(context.into()),
            "project.asset_source_changed",
            "conflict",
            context,
        ),
        (
            ImportError::UnsupportedUnits(context.into()),
            "project.import_unsupported_units",
            "format",
            context,
        ),
        (
            ImportError::UnsupportedMeshType(context.into()),
            "project.import_unsupported_mesh_type",
            "format",
            context,
        ),
        (
            ImportError::CoordinateOverflow,
            "project.asset_coordinate_overflow",
            "format",
            "millimeter coordinate overflow",
        ),
    ];
    for (error, code, category, detail) in cases {
        assert_eq!(activation_failure(&error), (code, category, detail.into()));
    }
    for error in [
        panta_mesh::StlError::Empty,
        panta_mesh::StlError::InvalidEncoding,
        panta_mesh::StlError::InvalidData,
        panta_mesh::StlError::NonFiniteCoordinate,
        panta_mesh::StlError::SizeOverflow,
    ] {
        let detail = error.to_string();
        assert_eq!(
            activation_failure(&ImportError::Parse(error)),
            ("project.asset_parse_failed", "format", detail)
        );
    }
}

/// 双向结构校验（073）：生成转移表 × 手写决策全对全比较。声明边在
/// guard 为真时必须 Enter(to)、为假时必须 GuardRejected；未声明组合
/// 在任意 guard 组合下必须 Illegal。
#[test]
fn generated_transitions_match_handwritten_decisions() {
    for &state in fsm::ALL_STATES {
        for &event in fsm::ALL_EVENTS {
            let declared: Vec<_> = fsm::TRANSITIONS
                .iter()
                .filter(|transition| transition.from == state && transition.on == event)
                .collect();
            assert!(
                declared.len() <= 1,
                "状态 {:?} 事件 {:?} 存在重复声明边",
                state.name(),
                event.name()
            );
            match declared.first() {
                None => {
                    for record_valid in [true, false] {
                        for session_current in [true, false] {
                            assert!(
                                matches!(
                                    decide(
                                        state,
                                        event,
                                        &GuardContext {
                                            record_valid,
                                            session_current,
                                        }
                                    ),
                                    Decision::Illegal
                                ),
                                "未声明组合 {:?} x {:?} 必须 Illegal",
                                state.name(),
                                event.name()
                            );
                        }
                    }
                }
                Some(transition) => match transition.guard {
                    None => {
                        for record_valid in [true, false] {
                            for session_current in [true, false] {
                                assert!(
                                    matches!(
                                        decide(
                                            state,
                                            event,
                                            &GuardContext {
                                                record_valid,
                                                session_current,
                                            }
                                        ),
                                        Decision::Enter(entered) if entered == transition.to
                                    ),
                                    "无边 guard 的 {:?} x {:?} 必须进入 {:?}",
                                    state.name(),
                                    event.name(),
                                    transition.to.name()
                                );
                            }
                        }
                    }
                    Some(fsm::Guard::RecordValid) => {
                        assert!(
                            matches!(
                                decide(
                                    state,
                                    event,
                                    &GuardContext {
                                        record_valid: true,
                                        session_current: false,
                                    }
                                ),
                                Decision::Enter(entered) if entered == transition.to
                            ),
                            "record-valid 为真必须进入 {:?}",
                            transition.to.name()
                        );
                        assert!(
                            matches!(
                                decide(
                                    state,
                                    event,
                                    &GuardContext {
                                        record_valid: false,
                                        session_current: true,
                                    }
                                ),
                                Decision::GuardRejected
                            ),
                            "record-valid 为假必须拒绝"
                        );
                    }
                    Some(fsm::Guard::SessionCurrent) => {
                        assert!(
                            matches!(
                                decide(
                                    state,
                                    event,
                                    &GuardContext {
                                        record_valid: false,
                                        session_current: true,
                                    }
                                ),
                                Decision::Enter(entered) if entered == transition.to
                            ),
                            "session-current 为真必须进入 {:?}",
                            transition.to.name()
                        );
                        assert!(
                            matches!(
                                decide(
                                    state,
                                    event,
                                    &GuardContext {
                                        record_valid: true,
                                        session_current: false,
                                    }
                                ),
                                Decision::GuardRejected
                            ),
                            "session-current 为假必须拒绝"
                        );
                    }
                },
            }
        }
    }
    // 声明完备性：状态、事件与声明边数量与 open-saved-stl 流程一致；
    // 增减声明而未更新手写决策时，上面的全对全比较会先失败。
    assert_eq!(fsm::ALL_STATES.len(), 7);
    assert_eq!(fsm::ALL_EVENTS.len(), 6);
    assert_eq!(fsm::TRANSITIONS.len(), 11);
}

fn coordinator_with_request() -> (Arc<ActivationCoordinator>, ActivationAttempt) {
    let coordinator = Arc::new(ActivationCoordinator::default());
    let attempt = coordinator
        .begin_with(
            "import-1",
            ActivationRequest {
                project_root: PathBuf::from("unused-project"),
                relative_path: "unused.pmesh".to_owned(),
                expected_digest: "sha256:test".to_owned(),
            },
            accept_activation_request,
        )
        .unwrap_or_else(|error| panic!("activation admission failed: {error}"));
    (coordinator, attempt)
}

fn run_path_request(root: PathBuf, relative_path: &str) -> Outcome {
    let (coordinator, attempt) = coordinator_with_request();
    run_attempt(
        Arc::clone(&coordinator),
        attempt.attempt,
        ActivationRequest {
            project_root: root,
            relative_path: relative_path.to_owned(),
            expected_digest: "sha256:test".to_owned(),
        },
        Arc::new(AtomicBool::new(false)),
    );
    let mut outcomes = coordinator.drain();
    assert_eq!(outcomes.len(), 1);
    outcomes.remove(0)
}

#[test]
fn worker_validates_relative_asset_paths_and_preserves_missing_diagnostics()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = crate::project::temp_directory::Fixture::new()?;
    let missing = run_path_request(fixture.root.clone(), "missing.pmesh");
    assert_eq!(missing.kind, OutcomeKind::Failed);
    assert_eq!(missing.code, "project.asset_missing");
    assert_eq!(missing.category, "missing");
    assert!(missing.mesh.is_none());

    for path in ["../outside.pmesh", "/outside.pmesh"] {
        let invalid = run_path_request(fixture.root.clone(), path);
        assert_eq!(invalid.kind, OutcomeKind::Failed);
        assert_eq!(invalid.code, "project.manifest_invalid");
        assert_eq!(invalid.category, "format");
        assert!(invalid.mesh.is_none());
    }
    Ok(())
}

#[test]
#[cfg(unix)]
fn worker_rejects_leaf_and_parent_symlinks_that_escape_the_project()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = crate::project::temp_directory::Fixture::new()?;
    let project_root = fixture.root.join("project");
    let external_root = fixture.root.join("external");
    std::fs::create_dir(&project_root)?;
    std::fs::create_dir(&external_root)?;
    let external_asset = external_root.join("asset.pmesh");
    std::fs::write(&external_asset, b"must not be read")?;
    std::os::unix::fs::symlink(&external_asset, project_root.join("leaf.pmesh"))?;
    std::os::unix::fs::symlink(&external_root, project_root.join("directory"))?;
    for path in ["leaf.pmesh", "directory/asset.pmesh"] {
        let invalid = run_path_request(project_root.clone(), path);
        assert_eq!(invalid.kind, OutcomeKind::Failed);
        assert_eq!(invalid.code, "project.manifest_invalid");
        assert_eq!(invalid.category, "format");
        assert!(invalid.mesh.is_none());
    }
    Ok(())
}

#[test]
fn success_publishes_snapshot_once_and_consumes_attempt() {
    let (coordinator, attempt) = coordinator_with_request();
    coordinator.complete(attempt.attempt, Event::AssetRead);
    coordinator.complete(
        attempt.attempt,
        Event::ParseSucceeded {
            mesh: SurfaceMesh {
                triangles: Vec::new(),
            },
        },
    );
    // 终态后重复完成是迟到结果：Illegal，不重复发布。
    coordinator.complete(
        attempt.attempt,
        Event::ParseSucceeded {
            mesh: SurfaceMesh {
                triangles: Vec::new(),
            },
        },
    );
    let outcomes = coordinator.drain();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].kind, OutcomeKind::Succeeded);
    assert_eq!(outcomes[0].import_id, "import-1");
    assert!(outcomes[0].mesh.is_some());
    assert!(coordinator.drain().is_empty(), "结果只发布一次");
    assert!(!coordinator.cancel(attempt.attempt), "终态后不可取消");
}

#[test]
fn fail_and_cancel_paths_publish_distinct_outcomes() {
    let (coordinator, attempt) = coordinator_with_request();
    coordinator.complete(
        attempt.attempt,
        Event::Fail {
            code: "project.asset_missing",
            category: "missing",
            detail: "part.stl".to_owned(),
        },
    );
    let outcomes = coordinator.drain();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].kind, OutcomeKind::Failed);
    assert_eq!(outcomes[0].code, "project.asset_missing");
    assert_eq!(outcomes[0].category, "missing");

    let (coordinator, attempt) = coordinator_with_request();
    coordinator.complete(attempt.attempt, Event::CancelAcknowledged);
    let outcomes = coordinator.drain();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].kind, OutcomeKind::Cancelled);
}

#[test]
fn generation_invalidation_drops_late_snapshot() {
    let (coordinator, attempt) = coordinator_with_request();
    coordinator.advance_generation();
    // 旧代次结果（含无负载的 Expired）不得跨会话发布：drain 为空，
    // 且 attempt 已终态化（不可再取消）。
    assert!(coordinator.drain().is_empty());
    assert!(!coordinator.cancel(attempt.attempt));
    // 迟到的解析成功在 Expired 之后到达：Illegal，快照就地释放。
    coordinator.complete(
        attempt.attempt,
        Event::ParseSucceeded {
            mesh: SurfaceMesh {
                triangles: Vec::new(),
            },
        },
    );
    assert!(coordinator.drain().is_empty(), "迟到成功不得发布");
}

#[test]
fn generation_advance_without_attempts_is_a_noop() {
    let coordinator = Arc::new(ActivationCoordinator::default());
    assert_eq!(coordinator.advance_generation(), 1);
    assert!(coordinator.drain().is_empty());
    assert_eq!(coordinator.advance_generation(), 2);
    assert!(coordinator.drain().is_empty());
    let diagnostic = format!("{coordinator:?}");
    assert!(diagnostic.contains("generation: 2"));
    assert!(!diagnostic.contains("outcomes"));
}

#[test]
fn drain_drops_results_queued_for_stale_generations() {
    let (coordinator, attempt) = coordinator_with_request();
    coordinator.complete(attempt.attempt, Event::AssetRead);
    coordinator.complete(
        attempt.attempt,
        Event::ParseSucceeded {
            mesh: SurfaceMesh {
                triangles: Vec::new(),
            },
        },
    );
    // 成功结果已入队；随后代次推进（工程切换）：旧代次结果在 drain
    // 边界被过滤，坐标（含网格负载）就地释放，不跨会话发布。
    coordinator.advance_generation();
    assert!(coordinator.drain().is_empty());
}

#[test]
fn duplicate_begin_reuses_in_flight_attempt() {
    let coordinator = Arc::new(ActivationCoordinator::default());
    let request = || ActivationRequest {
        project_root: PathBuf::from("unused-project"),
        relative_path: "unused.pmesh".to_owned(),
        expected_digest: "sha256:test".to_owned(),
    };
    let first = coordinator
        .begin_with("import-1", request(), accept_activation_request)
        .unwrap_or_else(|error| panic!("activation admission failed: {error}"));
    let second = coordinator
        .begin_with("import-1", request(), accept_activation_request)
        .unwrap_or_else(|error| panic!("activation admission failed: {error}"));
    assert_eq!(first.attempt, second.attempt);
    assert_eq!(first.generation, second.generation);
    // 不同记录仍创建独立 attempt。
    let other = coordinator
        .begin_with("import-2", request(), accept_activation_request)
        .unwrap_or_else(|error| panic!("activation admission failed: {error}"));
    assert_ne!(first.attempt, other.attempt);
}

#[test]
fn spawn_failure_terminates_with_declared_edge() {
    let coordinator = Arc::new(ActivationCoordinator::default());
    let attempt = coordinator
        .begin_with(
            "import-1",
            ActivationRequest {
                project_root: PathBuf::from("unused-project"),
                relative_path: "unused.pmesh".to_owned(),
                expected_digest: "sha256:test".to_owned(),
            },
            |_coordinator, _attempt, _request, _cancel| {
                Err(std::io::Error::other("thread pool exhausted"))
            },
        )
        .unwrap_or_else(|error| panic!("activation admission failed: {error}"));
    let outcomes = coordinator.drain();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].kind, OutcomeKind::Failed);
    assert_eq!(outcomes[0].code, "project.activation_spawn_failed");
    assert_eq!(outcomes[0].category, "resource");
    assert_eq!(outcomes[0].attempt, attempt.attempt);
}

#[test]
fn worker_panic_publishes_a_structured_failure() {
    let (coordinator, attempt) = coordinator_with_request();

    worker_panicked(&coordinator, attempt.attempt);

    let outcomes = coordinator.drain();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].kind, OutcomeKind::Failed);
    assert_eq!(outcomes[0].code, "project.activation_worker_failed");
    assert_eq!(outcomes[0].category, "internal");
    assert_eq!(outcomes[0].detail, "asset activation worker panicked");
    assert_eq!(outcomes[0].attempt, attempt.attempt);
}

#[test]
fn asset_read_advances_and_parse_failure_terminates() {
    let (coordinator, attempt) = coordinator_with_request();
    coordinator.complete(attempt.attempt, Event::AssetRead);
    // 资产读取后的中间态没有可观察结果。
    assert!(coordinator.drain().is_empty());
    coordinator.complete(
        attempt.attempt,
        Event::Fail {
            code: "project.asset_parse_failed",
            category: "format",
            detail: "invalid stl".to_owned(),
        },
    );
    let outcomes = coordinator.drain();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].kind, OutcomeKind::Failed);
    assert_eq!(outcomes[0].code, "project.asset_parse_failed");
    assert_eq!(outcomes[0].category, "format");
}

#[test]
fn activation_backlog_and_identifiers_are_bounded_and_reclaimed() {
    let coordinator = Arc::new(ActivationCoordinator::default());
    let request = || ActivationRequest {
        project_root: PathBuf::from("unused-project"),
        relative_path: "unused.pmesh".to_owned(),
        expected_digest: "sha256:test".into(),
    };
    for id in 0..OUTCOME_CAPACITY {
        let attempt = coordinator
            .begin_with(
                &format!("import-{id}"),
                request(),
                accept_activation_request,
            )
            .unwrap_or_else(|error| panic!("activation admission failed: {error}"));
        coordinator.complete(
            attempt.attempt,
            Event::Fail {
                code: "test.failure",
                category: "internal",
                detail: String::new(),
            },
        );
    }
    assert_eq!(
        coordinator
            .begin_with("overflow", request(), accept_activation_request)
            .err(),
        Some("asset activation backlog full")
    );
    assert_eq!(coordinator.drain().len(), OUTCOME_CAPACITY);
    coordinator.lock().next_attempt = u64::MAX;
    assert_eq!(
        coordinator
            .begin_with("exhausted", request(), accept_activation_request)
            .err(),
        Some("activation attempt exhausted")
    );
    assert!(coordinator.lock().attempts.is_empty());
}

#[test]
fn generation_change_cancels_queued_reads_and_releases_old_outcomes() {
    let coordinator = Arc::new(ActivationCoordinator::default());
    let cancelled = Arc::new(Mutex::new(None));
    let saved = Arc::clone(&cancelled);
    coordinator
        .begin_with(
            "queued",
            ActivationRequest {
                project_root: PathBuf::from("unused-project"),
                relative_path: "unused.pmesh".to_owned(),
                expected_digest: "sha256:test".into(),
            },
            move |_, _, _, token| {
                *saved
                    .lock()
                    .unwrap_or_else(|error| panic!("token lock failed: {error}")) = Some(token);
                Ok(())
            },
        )
        .unwrap_or_else(|error| panic!("activation admission failed: {error}"));
    coordinator.advance_generation();
    assert!(
        cancelled
            .lock()
            .unwrap_or_else(|error| panic!("token lock failed: {error}"))
            .as_ref()
            .unwrap_or_else(|| panic!("cancellation token missing"))
            .load(Ordering::SeqCst)
    );
    assert!(coordinator.lock().attempts.is_empty());
    assert!(coordinator.lock().outcomes.is_empty());
}
