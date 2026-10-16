use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use panta_core::project::{ProjectCommand, ProjectService, ProjectSnapshot};

const EXIT_COMMAND_FAILED: u8 = 2;
const USAGE: &str = "用法：panta-cli project <inspect <工程文件.panta> | create <父目录> <工程名>>";

#[cfg(test)]
#[path = "../../../tests/support/rust/temp_directory.rs"]
mod temp_directory;

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    match run(&arguments) {
        Ok(project) => {
            println!("Project: {}", project.name);
            println!("Path: {}", project.path.display());
            println!("Revision: {}", project.revision);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("panta-cli: {message}");
            ExitCode::from(EXIT_COMMAND_FAILED)
        }
    }
}

fn run(arguments: &[OsString]) -> Result<ProjectSnapshot, String> {
    let current_directory =
        std::env::current_dir().map_err(|error| format!("无法读取当前目录：{error}"))?;
    let mut service = ProjectService::new();
    match arguments {
        [command, operation, path] if command == "project" && operation == "inspect" => {
            let path = resolve_project_path(PathBuf::from(path), &current_directory);
            service
                .execute(ProjectCommand::Open { path })
                .map_err(|error| error.to_string())
        }
        [command, operation, location, name] if command == "project" && operation == "create" => {
            let location = resolve_project_path(PathBuf::from(location), &current_directory);
            let name = name
                .to_str()
                .ok_or_else(|| "工程名必须是有效 UTF-8".to_owned())?;
            service
                .create(&location, name)
                .map_err(|error| error.to_string())
        }
        _ => Err(USAGE.to_owned()),
    }
}

fn resolve_project_path(path: PathBuf, current_directory: &Path) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    current_directory.join(path)
}

#[cfg(test)]
mod tests {
    use super::resolve_project_path;
    use super::run;
    use super::temp_directory;
    use panta_core::project::ProjectService;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    #[test]
    fn inspect_uses_the_shared_project_open_command() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let mut creator = ProjectService::new();
        let created = creator.create(&fixture.root, "CLI inspection")?;
        let arguments = [
            OsString::from("project"),
            OsString::from("inspect"),
            created.path.as_os_str().to_owned(),
        ];

        let inspected = run(&arguments)?;

        assert_eq!(inspected.path, created.path);
        assert_eq!(inspected.name, "CLI inspection");
        assert_eq!(inspected.revision, created.revision);
        assert!(!inspected.dirty);
        Ok(())
    }

    #[test]
    fn inspect_reports_invalid_project_files_with_core_diagnostics()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let invalid = fixture.root.join("broken.panta");
        std::fs::write(&invalid, b"not a Panta project")?;
        let arguments = [
            OsString::from("project"),
            OsString::from("inspect"),
            invalid.into_os_string(),
        ];

        let error = run(&arguments)
            .err()
            .ok_or("invalid project was accepted")?;

        assert!(error.contains("project.manifest_invalid"));
        Ok(())
    }

    #[test]
    fn relative_path_is_resolved_against_the_cli_working_directory() {
        let current = Path::new("/workspace");
        let path = resolve_project_path(PathBuf::from("projects/demo.panta"), current);
        assert_eq!(path, current.join("projects/demo.panta"));
    }

    #[test]
    fn absolute_paths_are_preserved() -> Result<(), Box<dyn std::error::Error>> {
        let current = std::env::current_dir()?;
        let path = current.join("data/projects/demo");
        assert_eq!(resolve_project_path(path.clone(), &current), path);
        Ok(())
    }
}
