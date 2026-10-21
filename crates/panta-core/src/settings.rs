//! 版本化的用户偏好快照；只包含跨平台业务值，不依赖 Qt 或持久化后端。

use crate::language::LanguageService;
use std::fmt;
use std::path::Path;

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_THEME_ID: &str = "light";
const SUPPORTED_THEME_IDS: [&str; 2] = ["light", "dark"];
pub const DEFAULT_WINDOW_WIDTH: u32 = 1280;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 720;
pub const MAX_WINDOW_DIMENSION: u32 = 16_384;
pub const MAX_RECENT_PROJECTS: usize = 10;
const MAX_PATH_BYTES: usize = 32 * 1024;

/// 可以在设置后端和 CXX 边界之间传递的完整用户偏好快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSettings {
    pub schema_version: u32,
    pub theme_id: String,
    pub language: String,
    pub window_width: u32,
    pub window_height: u32,
    pub window_maximized: bool,
    pub recent_project_paths: Vec<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            theme_id: DEFAULT_THEME_ID.to_owned(),
            language: LanguageService::default().current().as_str().to_owned(),
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
            window_maximized: true,
            recent_project_paths: Vec::new(),
        }
    }
}

/// 偏好快照中不符合当前 schema 的字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    UnsupportedVersion(u32),
    InvalidThemeId(String),
    InvalidLanguage(String),
    InvalidWindowWidth(u32),
    InvalidWindowHeight(u32),
    TooManyRecentProjects(usize),
    InvalidRecentProjectPath(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => {
                write!(formatter, "settings.unsupported_version: {version}")
            }
            Self::InvalidThemeId(id) => write!(formatter, "settings.invalid_theme_id: {id}"),
            Self::InvalidLanguage(language) => {
                write!(formatter, "settings.invalid_language: {language}")
            }
            Self::InvalidWindowWidth(width) => {
                write!(formatter, "settings.invalid_window_width: {width}")
            }
            Self::InvalidWindowHeight(height) => {
                write!(formatter, "settings.invalid_window_height: {height}")
            }
            Self::TooManyRecentProjects(count) => {
                write!(formatter, "settings.too_many_recent_projects: {count}")
            }
            Self::InvalidRecentProjectPath(path) => {
                write!(formatter, "settings.invalid_recent_project_path: {path}")
            }
        }
    }
}

impl std::error::Error for SettingsError {}

/// 先验证完整候选，再由 Qt adapter 持久化并提交，避免失败写入改变当前状态。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsService {
    current: UserSettings,
    language: LanguageService,
}

impl SettingsService {
    pub fn current(&self) -> &UserSettings {
        &self.current
    }

    pub fn validate(&self, candidate: UserSettings) -> Result<UserSettings, SettingsError> {
        validate_snapshot(candidate, &self.language)
    }

    pub fn commit(&mut self, candidate: UserSettings) -> Result<&UserSettings, SettingsError> {
        let candidate = self.validate(candidate)?;
        let locale = self
            .language
            .validate(&candidate.language)
            .map_err(|_| SettingsError::InvalidLanguage(candidate.language.clone()))?;
        self.current = candidate;
        self.language.commit(locale);
        Ok(&self.current)
    }

    pub fn candidate_recent_project(&self, path: &str) -> Result<UserSettings, SettingsError> {
        let mut candidate = self.current.clone();
        validate_recent_project_path(path)?;
        candidate
            .recent_project_paths
            .retain(|existing| existing != path);
        candidate.recent_project_paths.insert(0, path.to_owned());
        candidate.recent_project_paths.truncate(MAX_RECENT_PROJECTS);
        self.validate(candidate)
    }
}

fn validate_snapshot(
    candidate: UserSettings,
    language: &LanguageService,
) -> Result<UserSettings, SettingsError> {
    if candidate.schema_version != SETTINGS_SCHEMA_VERSION {
        return Err(SettingsError::UnsupportedVersion(candidate.schema_version));
    }
    if !SUPPORTED_THEME_IDS.contains(&candidate.theme_id.as_str()) {
        return Err(SettingsError::InvalidThemeId(candidate.theme_id));
    }
    if language.validate(&candidate.language).is_err() {
        return Err(SettingsError::InvalidLanguage(candidate.language));
    }
    if !(DEFAULT_WINDOW_WIDTH..=MAX_WINDOW_DIMENSION).contains(&candidate.window_width) {
        return Err(SettingsError::InvalidWindowWidth(candidate.window_width));
    }
    if !(DEFAULT_WINDOW_HEIGHT..=MAX_WINDOW_DIMENSION).contains(&candidate.window_height) {
        return Err(SettingsError::InvalidWindowHeight(candidate.window_height));
    }
    if candidate.recent_project_paths.len() > MAX_RECENT_PROJECTS {
        return Err(SettingsError::TooManyRecentProjects(
            candidate.recent_project_paths.len(),
        ));
    }
    for (index, path) in candidate.recent_project_paths.iter().enumerate() {
        validate_recent_project_path(path)?;
        if candidate.recent_project_paths[..index].contains(path) {
            return Err(SettingsError::InvalidRecentProjectPath(path.clone()));
        }
    }
    Ok(candidate)
}

fn validate_recent_project_path(value: &str) -> Result<(), SettingsError> {
    if value.is_empty()
        || value.len() > MAX_PATH_BYTES
        || value.contains('\0')
        || !Path::new(value).is_absolute()
    {
        return Err(SettingsError::InvalidRecentProjectPath(value.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_THEME_ID, MAX_RECENT_PROJECTS, MAX_WINDOW_DIMENSION, SETTINGS_SCHEMA_VERSION,
        SettingsError, SettingsService, UserSettings,
    };
    fn absolute_path(name: &str) -> String {
        std::env::temp_dir()
            .join(name)
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn defaults_are_valid_and_include_supported_locale() {
        let service = SettingsService::default();
        assert_eq!(service.current().theme_id, DEFAULT_THEME_ID);
        assert_eq!(service.current().schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(service.current().language, "en");
        assert!(service.current().window_maximized);
        assert!(service.current().recent_project_paths.is_empty());
        assert_eq!(
            service.validate(service.current().clone()),
            Ok(service.current().clone())
        );
    }

    #[test]
    fn invalid_loaded_snapshots_are_rejected_without_replacing_current_state() {
        let mut service = SettingsService::default();
        let original = service.current().clone();
        let invalid_candidates = [
            UserSettings {
                schema_version: 99,
                ..original.clone()
            },
            UserSettings {
                theme_id: "bad id".to_owned(),
                ..original.clone()
            },
            UserSettings {
                language: "zh_CN".to_owned(),
                ..original.clone()
            },
            UserSettings {
                window_width: 1279,
                ..original.clone()
            },
            UserSettings {
                window_height: MAX_WINDOW_DIMENSION + 1,
                ..original.clone()
            },
            UserSettings {
                recent_project_paths: vec!["relative.panta".to_owned()],
                ..original.clone()
            },
        ];

        for candidate in invalid_candidates {
            assert!(service.commit(candidate).is_err());
            assert_eq!(service.current(), &original);
        }
    }

    #[test]
    fn validated_preferences_commit_as_one_snapshot() -> Result<(), SettingsError> {
        let mut service = SettingsService::default();
        let mut candidate = service.current().clone();
        candidate.language = "zh-CN".to_owned();
        candidate.theme_id = "dark".to_owned();
        candidate.window_width = 1440;
        candidate.window_height = 900;
        candidate.window_maximized = false;
        candidate
            .recent_project_paths
            .push(absolute_path("one.panta"));
        service.commit(candidate.clone())?;
        assert_eq!(service.current(), &candidate);
        let mut invalid = candidate.clone();
        invalid.language = "fr".to_owned();
        assert_eq!(
            service.validate(invalid),
            Err(SettingsError::InvalidLanguage("fr".into()))
        );
        assert_eq!(service.current(), &candidate);
        Ok(())
    }

    #[test]
    fn recent_projects_are_deduplicated_ordered_and_bounded() -> Result<(), SettingsError> {
        let service = SettingsService::default();
        let paths: Vec<_> = (0..MAX_RECENT_PROJECTS)
            .map(|index| absolute_path(&format!("{index}.panta")))
            .collect();
        let mut candidate = service.current().clone();
        candidate.recent_project_paths = paths.clone();
        let mut service = SettingsService::default();
        service.commit(candidate)?;

        let new_path = absolute_path("new.panta");
        let promoted = service.candidate_recent_project(&new_path)?;
        assert_eq!(promoted.recent_project_paths.len(), MAX_RECENT_PROJECTS);
        assert_eq!(promoted.recent_project_paths[0], new_path);
        assert!(!promoted.recent_project_paths.contains(&paths[9]));
        assert_eq!(
            service.candidate_recent_project("relative.panta"),
            Err(SettingsError::InvalidRecentProjectPath(
                "relative.panta".into()
            ))
        );
        Ok(())
    }
}
