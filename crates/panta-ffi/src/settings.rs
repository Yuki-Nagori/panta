//! 用户设置服务的 CXX 转发；Rust 校验完整快照，Qt adapter 执行持久化。

use panta_core::settings::{SettingsService, UserSettings};

pub struct UserSettingsService {
    service: SettingsService,
}

pub(super) fn settings_service_new() -> Box<UserSettingsService> {
    Box::new(UserSettingsService {
        service: SettingsService::default(),
    })
}

pub(super) fn settings_service_current(
    service: &UserSettingsService,
) -> crate::bridge::UserSettingsSnapshot {
    to_snapshot(service.service.current())
}

pub(super) fn settings_service_supported_languages() -> Vec<String> {
    panta_core::language::LanguageService::supported_locales()
        .iter()
        .map(|locale| (*locale).to_owned())
        .collect()
}

pub(super) fn settings_service_candidate_recent_project(
    service: &UserSettingsService,
    path: String,
) -> Result<crate::bridge::UserSettingsSnapshot, String> {
    service
        .service
        .candidate_recent_project(&path)
        .map(|settings| to_snapshot(&settings))
        .map_err(|error| error.to_string())
}

pub(super) fn settings_service_validate(
    service: &UserSettingsService,
    candidate: &crate::bridge::UserSettingsSnapshot,
) -> Result<crate::bridge::UserSettingsSnapshot, String> {
    service
        .service
        .validate(from_snapshot(candidate))
        .map(|settings| to_snapshot(&settings))
        .map_err(|error| error.to_string())
}

pub(super) fn settings_service_commit(
    service: &mut UserSettingsService,
    candidate: &crate::bridge::UserSettingsSnapshot,
) -> Result<crate::bridge::UserSettingsSnapshot, String> {
    service
        .service
        .commit(from_snapshot(candidate))
        .map(to_snapshot)
        .map_err(|error| error.to_string())
}

fn from_snapshot(snapshot: &crate::bridge::UserSettingsSnapshot) -> UserSettings {
    UserSettings {
        schema_version: snapshot.schema_version,
        theme_id: snapshot.theme_id.clone(),
        language: snapshot.language.clone(),
        window_width: snapshot.window_width,
        window_height: snapshot.window_height,
        window_maximized: snapshot.window_maximized,
        recent_project_paths: snapshot.recent_project_paths.clone(),
    }
}

fn to_snapshot(settings: &UserSettings) -> crate::bridge::UserSettingsSnapshot {
    crate::bridge::UserSettingsSnapshot {
        schema_version: settings.schema_version,
        theme_id: settings.theme_id.clone(),
        language: settings.language.clone(),
        window_width: settings.window_width,
        window_height: settings.window_height,
        window_maximized: settings.window_maximized,
        recent_project_paths: settings.recent_project_paths.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::bridge::UserSettingsSnapshot;

    #[test]
    fn settings_ffi_validates_then_commits_complete_snapshots() -> Result<(), String> {
        let mut service = crate::settings_service_new();
        let original = crate::settings_service_current(&service);
        assert_eq!(original.theme_id, "light");
        assert_eq!(original.language, "en");
        assert!(crate::settings_service_supported_languages().contains(&"zh-CN".to_owned()));

        let mut candidate = original.clone();
        candidate.theme_id = "dark".to_owned();
        candidate.language = "zh-CN".to_owned();
        candidate.window_width = 1440;
        candidate.window_height = 900;
        candidate.window_maximized = false;
        let validated = crate::settings_service_validate(&service, &candidate)?;
        assert_eq!(crate::settings_service_current(&service), original);
        assert_eq!(validated, candidate);

        assert_eq!(
            crate::settings_service_commit(&mut service, &validated)?,
            candidate
        );
        assert_eq!(crate::settings_service_current(&service), candidate);

        let invalid = UserSettingsSnapshot {
            language: "fr".to_owned(),
            ..candidate.clone()
        };
        assert!(crate::settings_service_validate(&service, &invalid).is_err());
        assert!(crate::settings_service_commit(&mut service, &invalid).is_err());
        assert_eq!(crate::settings_service_current(&service), candidate);
        Ok(())
    }
}
