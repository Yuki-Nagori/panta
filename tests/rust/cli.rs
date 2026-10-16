//! panta-cli 公共行为黑盒测试；由 panta-cli manifest 显式注册。
use std::process::Command;

use panta_core::project::ProjectService;

#[path = "../support/rust/temp_directory.rs"]
mod temp_directory;

fn panta_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_panta-cli"))
}

#[test]
fn inspect_accepts_relative_paths_and_prints_the_project_summary()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let mut creator = ProjectService::new();
    let created = creator.create(&fixture.root, "CLI relative inspection")?;
    let relative_path = created.path.strip_prefix(&fixture.root)?;

    let output = panta_cli()
        .current_dir(&fixture.root)
        .args(["project", "inspect"])
        .arg(relative_path)
        .output()?;

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("Project: CLI relative inspection"),
        "{stdout}"
    );
    assert!(
        stdout.contains(&created.path.display().to_string()),
        "{stdout}"
    );
    assert!(stdout.contains("Revision: 0"), "{stdout}");
    Ok(())
}

#[test]
fn invalid_project_returns_diagnostic_on_stderr_and_exit_code_two()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let invalid = fixture.root.join("invalid.panta");
    std::fs::write(&invalid, b"not a project")?;

    let output = panta_cli()
        .args(["project", "inspect"])
        .arg(&invalid)
        .output()?;

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("project.manifest_invalid"));
    assert!(output.stdout.is_empty());
    Ok(())
}

#[test]
fn unsupported_arguments_print_usage_and_exit_two() -> Result<(), Box<dyn std::error::Error>> {
    let output = panta_cli().output()?;

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("用法：panta-cli project inspect"));
    assert!(output.stdout.is_empty());
    Ok(())
}
