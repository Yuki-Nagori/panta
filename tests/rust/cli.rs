//! panta-cli 公共行为黑盒测试；由 panta-cli manifest 显式注册。
use std::process::Command;

use panta_core::project::ProjectService;

#[path = "../support/rust/temp_directory.rs"]
mod temp_directory;

fn panta_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_panta-cli"))
}

#[test]
fn create_writes_a_project_that_inspect_can_read() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let relative_parent = std::path::Path::new("nested").join("projects");
    let parent = fixture.root.join(&relative_parent);
    let created = panta_cli()
        .current_dir(&fixture.root)
        .args(["project", "create"])
        .arg(&relative_parent)
        .arg("CLI created project")
        .output()?;

    assert_eq!(created.status.code(), Some(0), "{created:?}");
    let created_stdout = String::from_utf8(created.stdout)?;
    assert!(
        created_stdout.contains("Project: CLI created project"),
        "{created_stdout}"
    );
    assert!(created_stdout.contains("Revision: 0"), "{created_stdout}");

    let project_path = parent
        .join("CLI created project")
        .join("CLI created project.panta");
    assert!(project_path.is_file());
    let inspected = panta_cli()
        .current_dir(&fixture.root)
        .args(["project", "inspect"])
        .arg(&project_path)
        .output()?;

    assert_eq!(inspected.status.code(), Some(0), "{inspected:?}");
    assert!(String::from_utf8(inspected.stdout)?.contains("Project: CLI created project"));
    Ok(())
}

#[test]
fn create_rejects_an_existing_project_without_overwriting_it()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let parent = fixture.root.join("projects");
    let first = panta_cli()
        .args(["project", "create"])
        .arg(&parent)
        .arg("Existing project")
        .output()?;
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let manifest = parent
        .join("Existing project")
        .join("Existing project.panta");
    let original_contents = std::fs::read(&manifest)?;

    let conflict = panta_cli()
        .args(["project", "create"])
        .arg(&parent)
        .arg("Existing project")
        .output()?;

    assert_eq!(conflict.status.code(), Some(2), "{conflict:?}");
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("project.already_exists"));
    assert!(conflict.stdout.is_empty());
    assert_eq!(std::fs::read(manifest)?, original_contents);
    Ok(())
}

#[test]
fn create_rejects_invalid_names_with_a_domain_diagnostic() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = temp_directory::Fixture::new()?;
    let output = panta_cli()
        .args(["project", "create"])
        .arg(&fixture.root)
        .arg("../escape")
        .output()?;

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("project.invalid_name"));
    assert!(output.stdout.is_empty());
    assert!(!fixture.root.join("escape").exists());
    Ok(())
}

#[test]
fn create_reports_an_unusable_parent_directory() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let parent_file = fixture.root.join("not-a-directory");
    std::fs::write(&parent_file, b"occupied")?;
    let output = panta_cli()
        .args(["project", "create"])
        .arg(&parent_file)
        .arg("Unwritable project")
        .output()?;

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("project.location_create_failed"));
    assert!(output.stdout.is_empty());
    Ok(())
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
fn stl_inspect_prints_the_shared_preview_for_a_path_with_spaces_and_unicode()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let source = fixture.root.join("零件 sample.stl");
    let original = b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 2 0\nendsolid\n";
    std::fs::write(&source, original)?;

    let output = panta_cli().args(["stl", "inspect"]).arg(&source).output()?;

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("STL: 零件 sample.stl"), "{stdout}");
    assert!(stdout.contains("Triangles: 1"), "{stdout}");
    assert!(stdout.contains("Dimensions: 1 × 2 × 0"), "{stdout}");
    assert!(stdout.contains("STL does not encode units"), "{stdout}");
    assert!(stdout.contains("does not assess mesh quality"), "{stdout}");
    assert_eq!(std::fs::read(&source)?, original);
    assert_eq!(std::fs::read_dir(&fixture.root)?.count(), 1);
    Ok(())
}

#[test]
fn stl_inspect_rejects_missing_and_malformed_inputs_with_diagnostics()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let missing = fixture.root.join("missing.stl");
    let missing_output = panta_cli().args(["stl", "inspect"]).arg(missing).output()?;
    assert_eq!(missing_output.status.code(), Some(2), "{missing_output:?}");
    assert!(
        String::from_utf8_lossy(&missing_output.stderr).contains("project.import_file_missing")
    );
    assert!(missing_output.stdout.is_empty());

    let malformed = fixture.root.join("malformed.stl");
    std::fs::write(&malformed, b"not an STL")?;
    let malformed_output = panta_cli()
        .args(["stl", "inspect"])
        .arg(malformed)
        .output()?;
    assert_eq!(
        malformed_output.status.code(),
        Some(2),
        "{malformed_output:?}"
    );
    assert!(
        String::from_utf8_lossy(&malformed_output.stderr).contains("project.import_parse_failed")
    );
    assert!(malformed_output.stdout.is_empty());

    let empty = fixture.root.join("empty.stl");
    std::fs::write(&empty, [])?;
    let empty_output = panta_cli().args(["stl", "inspect"]).arg(empty).output()?;
    assert_eq!(empty_output.status.code(), Some(2), "{empty_output:?}");
    assert!(String::from_utf8_lossy(&empty_output.stderr).contains("project.import_parse_failed"));
    assert!(empty_output.stdout.is_empty());

    let unsupported = fixture.root.join("model.obj");
    std::fs::write(&unsupported, b"triangle data")?;
    let unsupported_output = panta_cli()
        .args(["stl", "inspect"])
        .arg(unsupported)
        .output()?;
    assert_eq!(
        unsupported_output.status.code(),
        Some(2),
        "{unsupported_output:?}"
    );
    assert!(
        String::from_utf8_lossy(&unsupported_output.stderr).contains("project.import_invalid_file")
    );
    assert!(unsupported_output.stdout.is_empty());
    Ok(())
}

#[test]
fn stl_inspect_accepts_a_non_utf8_unix_parent_path() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;

        let fixture = temp_directory::Fixture::new()?;
        let parent = fixture
            .root
            .join(std::ffi::OsString::from_vec(b"input-\xff".to_vec()));
        match std::fs::create_dir(&parent) {
            Ok(()) => {}
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied
                    || error.kind() == std::io::ErrorKind::InvalidInput
                    || (cfg!(target_os = "macos") && error.raw_os_error() == Some(92)) =>
            {
                eprintln!("skipping non-UTF-8 path fixture: {error}");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        }
        let source = parent.join(std::ffi::OsString::from_vec(b"part-\xff.stl".to_vec()));
        match std::fs::write(
            &source,
            b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendsolid\n",
        ) {
            Ok(()) => {}
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied
                    || error.kind() == std::io::ErrorKind::InvalidInput
                    || (cfg!(target_os = "macos") && error.raw_os_error() == Some(92)) =>
            {
                eprintln!("skipping non-UTF-8 file-name fixture: {error}");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        }

        let output = panta_cli().args(["stl", "inspect"]).arg(&source).output()?;

        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let stdout = String::from_utf8(output.stdout)?;
        assert!(stdout.contains("STL: part-\u{fffd}.stl"), "{stdout}");
        assert!(stdout.contains("Triangles: 1"), "{stdout}");
    }
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
    assert!(String::from_utf8_lossy(&output.stderr).contains("用法：panta-cli project"));
    assert!(output.stdout.is_empty());
    Ok(())
}
