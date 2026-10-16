//! 073 消费者 build.rs 的输入生命周期与失败清理回归。

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[path = "../../crates/panta-core/build.rs"]
mod build_script;

const FSM_TEMPLATE: &str = "version: 1\nkind: fsm\nname: {name}\ninitial: Idle\n\nstates:\n  Idle: active\n  Done: terminal\n\ntransitions:\n  go:\n    from: Idle\n    on: {event}\n    to: Done\n";

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Result<Self, Box<dyn Error>> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "panta-fsm-build-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn added_modified_and_deleted_inputs_rebuild_the_output_set() -> Result<(), Box<dyn Error>> {
    let root = TestDirectory::new("input-lifecycle")?;
    let inputs = root.path().join("fsm");
    let out = root.path().join("out");
    fs::create_dir(&inputs)?;
    write_fsm(&inputs, "first", "advance")?;
    write_fsm(&inputs, "second", "advance")?;

    build_script::generate_outputs(&inputs, &out)?;
    assert_eq!(generated_names(&out)?, ["first.rs", "second.rs"]);
    let second_path = out.join("fsm/second.rs");
    let original = fs::read_to_string(&second_path)?;

    write_fsm(&inputs, "third", "advance")?;
    write_fsm(&inputs, "second", "continue")?;
    build_script::generate_outputs(&inputs, &out)?;
    assert_eq!(
        generated_names(&out)?,
        ["first.rs", "second.rs", "third.rs"]
    );
    assert_ne!(fs::read_to_string(&second_path)?, original);

    fs::remove_file(inputs.join("first.pa"))?;
    build_script::generate_outputs(&inputs, &out)?;
    assert_eq!(generated_names(&out)?, ["second.rs", "third.rs"]);
    assert!(!out.join("fsm/first.rs").exists());
    Ok(())
}

#[test]
fn deleted_required_input_and_invalid_input_clear_stale_outputs() -> Result<(), Box<dyn Error>> {
    let root = TestDirectory::new("required-input")?;
    let inputs = root.path().join("fsm");
    let out = root.path().join("out");
    fs::create_dir(&inputs)?;
    write_fsm(&inputs, "required", "advance")?;

    build_script::generate_outputs(&inputs, &out)?;
    assert!(out.join("fsm/required.rs").exists());
    fs::remove_file(inputs.join("required.pa"))?;
    let error = require_error(build_script::generate_outputs(&inputs, &out))?;
    assert!(error.contains("no fsm inputs found"));
    assert!(generated_names(&out)?.is_empty());

    write_fsm(&inputs, "required", "advance")?;
    build_script::generate_outputs(&inputs, &out)?;
    fs::write(inputs.join("required.pa"), "version: 1\nkind: fsm\n")?;
    let error = require_error(build_script::generate_outputs(&inputs, &out))?;
    assert!(error.contains("invalid fsm"));
    assert!(generated_names(&out)?.is_empty());
    Ok(())
}

#[test]
fn rejects_filename_mismatch_and_output_name_collision() -> Result<(), Box<dyn Error>> {
    let root = TestDirectory::new("invalid-names")?;
    let inputs = root.path().join("fsm");
    let out = root.path().join("out");
    fs::create_dir(&inputs)?;
    fs::write(
        inputs.join("file-name.pa"),
        FSM_TEMPLATE
            .replace("{name}", "different-name")
            .replace("{event}", "advance"),
    )?;
    let error = require_error(build_script::generate_outputs(&inputs, &out))?;
    assert!(error.contains("must match fsm name"));
    assert!(generated_names(&out)?.is_empty());

    let mut names = Vec::new();
    build_script::register_output_name(&mut names, "duplicate_name.rs".to_owned())?;
    let error = require_error(build_script::register_output_name(
        &mut names,
        "duplicate_name.rs".to_owned(),
    ))?;
    assert!(error.contains("multiple fsm inputs map"));
    Ok(())
}

#[test]
#[cfg_attr(
    miri,
    ignore = "executes a Cargo subprocess and uses filesystem syscalls unsupported by Miri"
)]
fn cargo_rebuilds_changed_inputs_and_fails_when_required_input_is_deleted()
-> Result<(), Box<dyn Error>> {
    let root = TestDirectory::new("cargo-rebuild")?;
    let package = root.path().join("consumer");
    let inputs = package.join("fsm");
    let target = root.path().join("target");
    fs::create_dir_all(package.join("src"))?;
    fs::create_dir(&inputs)?;

    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = package_root.join("../..").canonicalize()?;
    let build_script_path = package_root.join("build.rs");
    fs::copy(build_script_path, package.join("build.rs"))?;
    let dsl_core_path = workspace.join("crates/panta-dsl-core").canonicalize()?;
    let dsl_core_path = serde_json::to_string(&dsl_core_path.to_string_lossy())?;
    fs::write(
        package.join("Cargo.toml"),
        format!(
            "[package]\nname = \"fsm-build-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\nbuild = \"build.rs\"\n\n[build-dependencies]\npanta-dsl-core = {{ path = {dsl_core_path} }}\n"
        ),
    )?;
    fs::copy(workspace.join("Cargo.lock"), package.join("Cargo.lock"))?;
    fs::write(
        package.join("src/lib.rs"),
        "mod generated { include!(concat!(env!(\"OUT_DIR\"), \"/fsm/open_saved_stl.rs\")); }\n",
    )?;
    write_fsm(&inputs, "open-saved-stl", "advance")?;

    // 仅首次锁定 fixture 包自身；所有 FSM 变更构建都使用该锁文件。
    assert_cargo_check_succeeds(&package, &target, false)?;
    let output_dir = fixture_output_dir(&target)?;
    assert!(output_dir.join("open_saved_stl.rs").is_file());

    thread::sleep(Duration::from_secs(1));
    write_fsm(&inputs, "added", "advance")?;
    assert_cargo_check_succeeds(&package, &target, true)?;
    assert!(output_dir.join("added.rs").is_file());

    thread::sleep(Duration::from_secs(1));
    write_fsm(&inputs, "added", "continue")?;
    assert_cargo_check_succeeds(&package, &target, true)?;
    let generated = fs::read_to_string(output_dir.join("added.rs"))?;
    assert!(generated.contains("Continue"));

    thread::sleep(Duration::from_secs(1));
    fs::remove_file(inputs.join("added.pa"))?;
    assert_cargo_check_succeeds(&package, &target, true)?;
    assert!(!output_dir.join("added.rs").exists());

    thread::sleep(Duration::from_secs(1));
    write_fsm(&inputs, "survivor", "advance")?;
    assert_cargo_check_succeeds(&package, &target, true)?;
    assert!(output_dir.join("survivor.rs").is_file());

    thread::sleep(Duration::from_secs(1));
    fs::remove_file(inputs.join("open-saved-stl.pa"))?;
    let output = run_fixture_cargo_check(&package, &target, true)?;
    assert!(!output.status.success(), "missing required input must fail");
    assert!(!output_dir.join("open_saved_stl.rs").exists());
    Ok(())
}

fn write_fsm(directory: &Path, name: &str, event: &str) -> Result<(), Box<dyn Error>> {
    fs::write(
        directory.join(format!("{name}.pa")),
        FSM_TEMPLATE
            .replace("{name}", name)
            .replace("{event}", event),
    )?;
    Ok(())
}

fn generated_names(out: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut names = fs::read_dir(out.join("fsm"))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .map_err(Into::into)
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    names.sort();
    Ok(names)
}

fn require_error(result: Result<(), String>) -> Result<String, Box<dyn Error>> {
    match result {
        Ok(()) => Err("expected build failure".into()),
        Err(error) => Ok(error),
    }
}

fn assert_cargo_check_succeeds(
    package: &Path,
    target: &Path,
    locked: bool,
) -> Result<(), Box<dyn Error>> {
    let output = run_fixture_cargo_check(package, target, locked)?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "fixture cargo check failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .into())
}

fn run_fixture_cargo_check(
    package: &Path,
    target: &Path,
    locked: bool,
) -> Result<Output, Box<dyn Error>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command
        .arg("check")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(package.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target);
    if locked {
        command.arg("--locked");
    }
    let output = command.output()?;
    Ok(output)
}

fn fixture_output_dir(target: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let build_dir = target.join("debug/build");
    for entry in fs::read_dir(build_dir)? {
        let path = entry?.path();
        if path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("fsm-build-fixture-"))
        {
            let output = path.join("out/fsm");
            if output.is_dir() {
                return Ok(output);
            }
        }
    }
    Err("Cargo fixture did not create an FSM output directory".into())
}
