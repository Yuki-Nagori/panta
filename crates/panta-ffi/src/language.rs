//! 语言服务的 CXX 转发；locale 校验与选择由 core 拥有。

/// Rust locale 服务的 CXX 封装；边界只传递 locale 字符串值。
pub struct LanguageService {
    service: panta_core::language::LanguageService,
}

pub(super) fn language_service_new() -> Box<LanguageService> {
    Box::new(LanguageService {
        service: panta_core::language::LanguageService::default(),
    })
}

pub(super) fn language_service_supported_locales() -> Vec<String> {
    panta_core::language::LanguageService::supported_locales()
        .iter()
        .map(|locale| (*locale).to_owned())
        .collect()
}

pub(super) fn language_service_current(service: &LanguageService) -> String {
    service.service.current().as_str().to_owned()
}

pub(super) fn language_service_validate(
    service: &LanguageService,
    candidate: String,
) -> Result<String, String> {
    service
        .service
        .validate(&candidate)
        .map(|locale| locale.as_str().to_owned())
        .map_err(|error| error.to_string())
}

pub(super) fn language_service_commit(
    service: &mut LanguageService,
    candidate: String,
) -> Result<String, String> {
    let locale = service
        .service
        .validate(&candidate)
        .map_err(|error| error.to_string())?;
    service.service.commit(locale);
    Ok(service.service.current().as_str().to_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn language_bridge_validates_before_committing() -> Result<(), String> {
        let mut service = crate::language_service_new();
        assert_eq!(crate::language_service_supported_locales(), ["en", "zh-CN"]);
        assert_eq!(crate::language_service_current(&service), "en");
        assert_eq!(
            crate::language_service_validate(&service, "zh-CN".into())?,
            "zh-CN"
        );
        assert_eq!(crate::language_service_current(&service), "en");
        assert!(crate::language_service_validate(&service, "unknown".into()).is_err());
        assert!(crate::language_service_commit(&mut service, "unknown".into()).is_err());
        assert_eq!(
            crate::language_service_commit(&mut service, "zh-CN".into())?,
            "zh-CN"
        );
        assert_eq!(crate::language_service_current(&service), "zh-CN");
        Ok(())
    }
}
