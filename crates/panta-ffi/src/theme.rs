//! 主题服务的 CXX 转发；解析、schema 校验与快照由 `panta-core` 拥有。

use panta_core::theme::{
    ThemeService as CoreThemeService, ThemeSnapshot as CoreThemeSnapshot,
    ThemeToken as CoreThemeToken, ThemeTokenValue,
};

/// Core 主题服务的 FFI 句柄；不持有 Qt 对象或平台设置。
pub struct ThemeService {
    service: CoreThemeService,
}

pub(super) fn theme_service_new(
    gui_source: String,
    theme_ids: Vec<String>,
    theme_sources: Vec<String>,
    default_theme_id: String,
) -> Result<Box<ThemeService>, String> {
    if theme_ids.len() != theme_sources.len() {
        return Err("theme.definition_count_mismatch: ids and sources differ".to_owned());
    }
    let themes: Vec<(String, String)> = theme_ids.into_iter().zip(theme_sources).collect();
    let service = CoreThemeService::new(&gui_source, themes, &default_theme_id)
        .map_err(|error| error.to_string())?;
    Ok(Box::new(ThemeService { service }))
}

pub(super) fn theme_service_theme_ids(service: &ThemeService) -> Vec<String> {
    service.service.theme_ids().map(str::to_owned).collect()
}

pub(super) fn theme_service_current(service: &ThemeService) -> crate::bridge::ThemeSnapshot {
    snapshot_dto(service.service.current())
}

pub(super) fn theme_service_select(
    service: &mut ThemeService,
    id: String,
) -> Result<crate::bridge::ThemeSnapshot, String> {
    service
        .service
        .select(&id)
        .map(snapshot_dto)
        .map_err(|error| error.to_string())
}

fn snapshot_dto(snapshot: &CoreThemeSnapshot) -> crate::bridge::ThemeSnapshot {
    crate::bridge::ThemeSnapshot {
        id: snapshot.id.clone(),
        tokens: snapshot.tokens.iter().map(token_dto).collect(),
    }
}

fn token_dto(token: &CoreThemeToken) -> crate::bridge::ThemeToken {
    let (value_type, value) = match &token.value {
        ThemeTokenValue::Color(value) => ("color", value.clone()),
        ThemeTokenValue::Integer(value) => ("integer", value.to_string()),
        ThemeTokenValue::Real(value) => ("real", value.to_string()),
        ThemeTokenValue::Boolean(value) => ("boolean", value.to_string()),
    };
    crate::bridge::ThemeToken {
        qml_name: token.qml_name.clone(),
        value_type: value_type.to_owned(),
        value,
    }
}

#[cfg(test)]
mod tests {
    const GUI: &str = include_str!("../../../qml/Themes/gui-default.pa");
    const LIGHT: &str = include_str!("../../../qml/Themes/light.pa");
    const DARK: &str = include_str!("../../../qml/Themes/dark.pa");

    #[test]
    fn theme_ffi_selects_and_exports_rust_validated_values() -> Result<(), String> {
        let mut service = crate::theme_service_new(
            GUI.to_owned(),
            vec!["light".to_owned(), "dark".to_owned()],
            vec![LIGHT.to_owned(), DARK.to_owned()],
            "light".to_owned(),
        )?;
        assert_eq!(crate::theme_service_current(&service).id, "light");
        let dark = crate::theme_service_select(&mut service, "dark".to_owned())?;
        assert_eq!(dark.id, "dark");
        assert!(dark.tokens.iter().any(|token| {
            token.qml_name == "controlHeight"
                && token.value_type == "integer"
                && token.value == "28"
        }));
        let before = crate::theme_service_current(&service);
        assert!(crate::theme_service_select(&mut service, "unknown".to_owned()).is_err());
        let current = crate::theme_service_current(&service);
        assert_eq!(current.id, before.id);
        assert_eq!(current.tokens.len(), before.tokens.len());
        assert!(
            current
                .tokens
                .iter()
                .zip(before.tokens.iter())
                .all(|(a, b)| {
                    a.qml_name == b.qml_name && a.value_type == b.value_type && a.value == b.value
                })
        );
        Ok(())
    }
}
