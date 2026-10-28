use super::*;

#[test]
fn reverse_dependency_closure_is_transitive() {
    let reverse = BTreeMap::from([
        ("base".into(), BTreeSet::from(["middle".into()])),
        ("middle".into(), BTreeSet::from(["app".into()])),
    ]);
    assert_eq!(
        dependency_closure(BTreeSet::from(["base".into()]), &reverse),
        BTreeSet::from(["base".into(), "middle".into(), "app".into()])
    );
}

#[test]
fn clean_local_scope_is_a_noop_but_explicit_empty_ci_scope_fails_closed()
-> Result<(), Box<dyn Error>> {
    assert!(is_empty_local_change_set(&[], false));
    assert!(!is_empty_local_change_set(&[], true));
    assert!(!is_empty_local_change_set(
        &["crates/panta-core/src/lib.rs".into()],
        false
    ));
    let root = repository_root()?;
    assert!(make_plan(&[], &root)?.full);
    Ok(())
}

#[test]
fn local_scope_escalates_to_full_suite_when_impact_is_unknown() -> Result<(), Box<dyn Error>> {
    let plan = Plan {
        packages_l2: BTreeSet::from(["panta-core".into(), "panta-tests".into()]),
        full: true,
        ..Plan::default()
    };
    for scope in ["l1", "l2", "all"] {
        assert_eq!(
            selected_packages(scope, &plan)?,
            plan.packages(),
            "{scope} must not become a false-green partial run"
        );
        assert!(scope_runs_native(scope, &plan));
    }
    Ok(())
}

#[test]
fn documents_are_recognized_only_outside_code_and_test_roots() {
    assert!(is_document_path("ai-docs/task/120.md"));
    assert!(is_document_path("README.md"));
    assert!(!is_document_path("tests/fixtures/readme.md"));
    assert!(!is_document_path("native/foundation/README.md"));
    assert!(!is_document_path("ai-docs/unregistered-input.js"));
}

#[test]
fn prototype_input_runs_qml_tests_and_unknown_document_inputs_fail_closed()
-> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let prototype = make_plan(&["prototypes/shell/shell.js".into()], &root)?;
    assert!(prototype.code && prototype.native && prototype.qml);
    assert!(prototype.native_labels.contains("qml"));
    assert!(!prototype.full);
    let unknown = make_plan(&["ai-docs/unregistered-input.js".into()], &root)?;
    assert!(unknown.full && unknown.qml_lint && unknown.cpp_lint_all);
    Ok(())
}

#[test]
fn project_boundary_modules_and_shared_qt_support_select_their_consumers()
-> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    for input in [
        "crates/panta-core/src/project/command_job.rs",
        "crates/panta-core/src/project/activation_result.rs",
        "crates/panta-ffi/src/project_operation.rs",
        "crates/panta-ffi/src/display.rs",
    ] {
        let plan = make_plan(&[input.into()], &root)?;
        assert!(plan.rust && plan.native, "{input}");
        assert!(plan.native_labels.contains("bridge"), "{input}");
        assert!(plan.native_labels.contains("qml"), "{input}");
        assert!(plan.packages_l2.contains("panta-launcher"), "{input}");
    }
    for input in [
        "tests/support/qt/project_operation_helpers.hpp",
        "tests/support/qt/mesh_frame_tracker.hpp",
    ] {
        let support = make_plan(&[input.into()], &root)?;
        assert!(support.full && support.qml, "{input}");
        assert!(support.native_labels.contains("bridge"), "{input}");
        assert!(support.native_labels.contains("visualization"), "{input}");
    }
    Ok(())
}

#[test]
fn native_analysis_follows_build_and_generated_header_inputs() {
    for input in [
        "qml/CMakeLists.txt",
        "native/cmake/sdk-provision.cmake",
        "native/removed_header.hpp",
        "crates/panta-ffi/src/lib.rs",
        "crates/panta-ffi/Cargo.toml",
        "crates/launcher/build.rs",
        "crates/panta-build/src/assets.rs",
    ] {
        let scope = native_scope_for_paths(&[input.into()]);
        if is_cmake_path(input) {
            assert!(
                scope == NativeLintScope::All,
                "CMake changes must lint every native translation unit: {input}"
            );
        } else {
            assert_eq!(scope, NativeLintScope::All, "{input}");
        }
    }
    let source = "native/app/window_chrome_metrics_mac.mm";
    assert_eq!(
        native_scope_for_paths(&[source.into()]),
        NativeLintScope::Sources(BTreeSet::from([source.into()]))
    );
    assert!(native_scope_for_paths(&["crates/panta-core/src/project.rs".into()]).is_empty());
    assert!(native_scope_for_paths(&["qml/App.qml".into()]).is_empty());
}

#[test]
fn label_regex_is_anchored_and_deterministic() {
    let labels = BTreeSet::from(["qml".into(), "bridge".into()]);
    assert_eq!(label_regex(&labels), "^(bridge|qml)$");
}

#[test]
fn repository_crate_change_selects_direct_package_and_consumers() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let plan = make_plan(&["crates/panta-foundation/Cargo.toml".into()], &root)?;
    assert!(plan.packages_l1.contains("panta-foundation"));
    assert!(!plan.packages_l2.is_empty());
    assert!(plan.rust_lint_packages.contains("panta-foundation"));
    assert!(plan.rust_lint_packages.contains("panta-ffi"));
    assert_eq!(
        plan.rust_format_packages,
        BTreeSet::from(["panta-foundation".into()])
    );
    assert!(plan.native_labels.contains("ffi"));
    assert!(!plan.full);
    Ok(())
}

#[test]
fn cli_changes_select_the_cli_package_and_core_changes_select_cli_consumer()
-> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let cli = make_plan(&["crates/panta-cli/src/main.rs".into()], &root)?;
    assert!(cli.packages_l1.contains("panta-cli"));
    assert!(cli.packages_l2.is_empty());
    assert!(!cli.native);

    let cli_test = make_plan(&["tests/rust/cli.rs".into()], &root)?;
    assert!(cli_test.packages_l1.contains("panta-cli"), "{cli_test:?}");
    assert!(cli_test.packages_l2.is_empty());

    let core = make_plan(&["crates/panta-core/src/project/service.rs".into()], &root)?;
    assert!(core.packages_l1.contains("panta-core"));
    assert!(core.packages_l2.contains("panta-cli"));
    assert!(!core.full);
    Ok(())
}

#[test]
fn criterion_benches_select_their_registered_cargo_packages() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    for (path, package) in [
        (
            "tests/performance/cpu/dsl_core_criterion.rs",
            "panta-dsl-core",
        ),
        (
            "tests/performance/cpu/core_workspace_criterion.rs",
            "panta-core",
        ),
    ] {
        assert!(is_rust_package_path(path), "{path}");
        let plan = make_plan(&[path.into()], &root)?;
        assert!(plan.rust, "{path}");
        assert!(plan.packages_l1.contains(package), "{plan:?}");
        assert!(!plan.full, "{path} should stay package-scoped");
    }
    assert!(!is_rust_package_path("tests/performance/cpu/example.cpp"));
    Ok(())
}

#[test]
fn ffi_changes_select_mesh_consumers() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    for path in [
        "crates/panta-ffi/Cargo.toml",
        "crates/panta-ffi/src/ffi_support.cc",
        "crates/panta-ffi/include/panta/ffi.hpp",
    ] {
        let plan = make_plan(&[path.into()], &root)?;
        assert!(plan.packages_l1.contains("panta-ffi"), "{path}");
        assert!(plan.native, "{path}");
        assert!(plan.native_labels.contains("mesh"), "{path}");
    }
    Ok(())
}

#[test]
fn mesh_adapter_changes_select_geometry_and_mesh_tests() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let plan = make_plan(&["native/mesh/src/netgen/netgen_mesher.cpp".into()], &root)?;
    assert!(plan.native_labels.contains("geometry"), "{plan:?}");
    assert!(plan.native_labels.contains("mesh"), "{plan:?}");
    assert!(!plan.full, "{plan:?}");
    Ok(())
}

#[test]
fn dsl_build_dependency_selects_native_translation_tests() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let plan = make_plan(&["crates/panta-dsl-core/Cargo.toml".into()], &root)?;
    assert!(plan.packages_l1.contains("panta-dsl-core"));
    assert!(plan.packages_l2.contains("panta-launcher"));
    assert!(plan.native_labels.contains("i18n"));
    assert!(plan.native_labels.contains("qt-adapter"));
    Ok(())
}

#[test]
fn theme_sources_select_core_consumers_and_qml_tests() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    for path in [
        "qml/Themes/gui-default.pa",
        "qml/Themes/dark.pa",
        "qml/Themes/light.pa",
    ] {
        let plan = make_plan(&[path.into()], &root)?;
        assert!(plan.rust, "{plan:?}");
        assert!(plan.packages_l1.contains("panta-core"), "{plan:?}");
        assert!(plan.packages_l2.contains("panta-ffi"), "{plan:?}");
        assert!(plan.native && plan.qml, "{plan:?}");
        assert!(plan.native_labels.contains("bridge"), "{plan:?}");
        assert!(plan.native_labels.contains("qml"), "{plan:?}");
        assert!(!plan.full, "{plan:?}");
    }
    Ok(())
}

#[test]
fn full_fallback_paths_schedule_every_domain() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    for path in [
        Some("native/CMakeLists.txt"),
        Some("unregistered/new-area/file.rs"),
        Some("crates/launcher/build.rs"),
        None,
    ] {
        let paths = path.map(|path| vec![path.into()]).unwrap_or_default();
        let plan = make_plan(&paths, &root)?;
        assert!(plan.full, "{path:?} must select the full suite");
        assert!(plan.code, "{path:?} must schedule code checks");
        assert!(plan.cargo_deps, "{path:?} must schedule dependency checks");
        assert!(
            plan.rust && plan.native && plan.qml,
            "{path:?} must schedule all domains"
        );
        assert_eq!(plan.packages_l1.len(), 0);
        assert_eq!(plan.packages_l2.len(), cargo_packages(&root)?.len());
        assert_eq!(plan.native_labels, all_native_labels());
    }
    Ok(())
}

#[test]
fn public_rust_build_inputs_expand_quality_together_with_tests() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    for path in [
        "crates/launcher/build.rs",
        "crates/panta-build/src/assets.rs",
    ] {
        let plan = make_plan(&[path.into()], &root)?;
        assert!(plan.full, "{path}");
    }
    Ok(())
}

#[test]
fn native_module_change_selects_registered_label_closure() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let plan = make_plan(&["native/geometry/src/adapter.cpp".into()], &root)?;
    assert!(!plan.full);
    assert_eq!(
        plan.native_labels,
        BTreeSet::from([
            "geometry".into(),
            "mesh".into(),
            "ffi".into(),
            "bridge".into(),
            "app".into(),
            "qml".into(),
        ])
    );
    Ok(())
}

#[test]
fn tool_configs_expand_only_their_own_quality_scope() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let clang_format = make_plan(&[".clang-format".into()], &root)?;
    assert!(!clang_format.full);
    assert!(clang_format.cpp_format_all);
    assert!(!clang_format.cpp_lint && !clang_format.qml_lint);

    let cmake = make_plan(&["native/CMakeLists.txt".into()], &root)?;
    assert!(cmake.cmake && cmake.layering);
    assert!(cmake.cpp_lint && cmake.cpp_lint_all);
    assert!(!cmake.qml_lint);

    for path in ["qml/CMakeLists.txt", "native/cmake/sdk-provision.cmake"] {
        let plan = make_plan(&[path.into()], &root)?;
        assert!(plan.cmake && plan.layering, "{path}: {plan:?}");
        assert!(plan.cpp_lint && plan.cpp_lint_all, "{path}: {plan:?}");
    }

    for path in [".clang-tidy", "tests/cpp/geometry/.clang-tidy"] {
        let plan = make_plan(&[path.into()], &root)?;
        assert!(plan.cpp_lint && plan.cpp_lint_all, "{path}: {plan:?}");
        assert!(!plan.full && !plan.qml_lint, "{path}: {plan:?}");
    }

    let cmake_format = make_plan(&[".cmake-format.py".into()], &root)?;
    assert!(cmake_format.cmake_lint_all && cmake_format.cmake_format_all);
    assert!(cmake_format.layering);
    assert!(!cmake_format.cpp_lint && !cmake_format.cpp_format_all);
    assert!(!cmake_format.full);

    let cargo_config = make_plan(&["Cargo.toml".into()], &root)?;
    assert!(cargo_config.full && cargo_config.cpp_lint_all && cargo_config.qml_lint);
    Ok(())
}

#[test]
fn runner_build_and_layering_inputs_select_only_their_required_checks() -> Result<(), Box<dyn Error>>
{
    let root = repository_root()?;

    let build = make_plan(&["tests/src/build.rs".into()], &root)?;
    assert!(build.runner_tests && build.native, "{build:?}");
    assert_eq!(build.packages_l1, BTreeSet::from(["panta-tests".into()]));
    assert!(build.packages_l2.is_empty());
    assert_eq!(build.native_labels, all_native_labels());
    assert!(!build.full, "{build:?}");

    let layering = make_plan(&["tests/src/layering.rs".into()], &root)?;
    assert!(layering.runner_tests && layering.layering, "{layering:?}");
    assert_eq!(layering.packages_l1, BTreeSet::from(["panta-tests".into()]));
    assert!(layering.packages_l2.is_empty());
    assert!(!layering.native && !layering.full, "{layering:?}");
    Ok(())
}

#[test]
fn runner_and_ci_router_changes_do_not_expand_to_application_tests() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    for path in ["tests/src/test_scope.rs", "tests/src/main.rs"] {
        let plan = make_plan(&[path.into()], &root)?;
        assert!(plan.rust && plan.runner_tests, "{path}: {plan:?}");
        assert!(plan.packages_l1.contains("panta-tests"), "{path}: {plan:?}");
        assert!(plan.packages_l2.is_empty(), "{path}: {plan:?}");
        assert!(!plan.full && !plan.native && !plan.qml, "{path}: {plan:?}");
        assert_eq!(
            plan.rust_format_packages,
            BTreeSet::from(["panta-tests".into()]),
            "{path}: {plan:?}"
        );
    }
    for path in [
        ".github/workflows/ci.yml",
        ".github/actions/changed-paths/action.yml",
    ] {
        let plan = make_plan(&[path.into()], &root)?;
        assert!(plan.runner_tests, "{path}: {plan:?}");
        assert!(!plan.code && !plan.rust && !plan.full, "{path}: {plan:?}");
        assert!(plan.rust_lint_packages.is_empty(), "{path}: {plan:?}");
    }
    Ok(())
}

#[test]
fn qml_lint_follows_qml_and_bridge_inputs_only() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    for path in [
        "qml/App.qml",
        "tests/qml/theme_component_test.qml",
        "prototypes/shell/shell.js",
        "crates/panta-ffi/src/lib.rs",
        "crates/panta-core/src/theme_schema.rs",
        "native/app/theme_view_model.hpp",
        "native/app/edit_command_router.hpp",
        "native/bridge/src/project_view_model.hpp",
        "native/bridge/src/settings.hpp",
        "native/bridge/src/import_interaction_controller.hpp",
        "native/visualization/include/panta/visualization/cae_viewport.hpp",
    ] {
        assert!(make_plan(&[path.into()], &root)?.qml_lint, "{path}");
    }
    for path in [
        "crates/panta-core/src/project/service.rs",
        "native/i18n/qm_catalog.cpp",
        "qml/Themes/dark.pa",
        "native/app/document_path_adapter.hpp",
    ] {
        assert!(!make_plan(&[path.into()], &root)?.qml_lint, "{path}");
    }
    Ok(())
}

#[test]
fn empty_and_unknown_inputs_request_complete_quality_and_test_ranges() -> Result<(), Box<dyn Error>>
{
    let root = repository_root()?;
    for paths in [vec![], vec!["new-area/unknown.rs".into()]] {
        let plan = make_plan(&paths, &root)?;
        assert!(plan.full && plan.code, "{paths:?}: {plan:?}");
        assert!(plan.rust && plan.native && plan.qml, "{paths:?}: {plan:?}");
        assert!(
            plan.cpp_lint_all
                && plan.cpp_format_all
                && plan.cmake_lint_all
                && plan.cmake_format_all
                && plan.qml_lint
                && plan.layering,
            "{paths:?}: {plan:?}"
        );
        assert_eq!(
            plan.rust_format_packages.len(),
            cargo_packages(&root)?.len()
        );
        assert_eq!(plan.rust_lint_packages.len(), cargo_packages(&root)?.len());
    }
    Ok(())
}
