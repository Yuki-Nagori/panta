#![allow(clippy::expect_used)]

use panta_dsl_core::{emit_ts, parse, source_prefix};

const CATALOG: &str = include_str!("fixtures/panta-cn.pa");

#[test]
fn fixture_is_a_locale_catalog() {
    let document = parse(CATALOG).expect("fixture should parse");
    assert_eq!(document.messages.len(), 3);
    assert_eq!(
        document
            .messages
            .values()
            .find(|message| message.context == "FileMenu" && message.id == "open")
            .map(|message| message.source.as_str()),
        Some("Open")
    );
}

#[test]
fn fixture_emits_locale_and_uses_longest_source() {
    let document = parse(CATALOG).expect("fixture should parse");
    let ts = emit_ts(&document, "cn").expect("locale should emit");
    assert!(ts.contains("language=\"zh_CN\""));
    assert_eq!(
        source_prefix(&document, "ok ok!").map(|(key, _)| key),
        Some("ok-pair")
    );
}

#[test]
fn artifact_outputs_match_the_pre_refactor_baseline() -> Result<(), panta_dsl_core::Diagnostics> {
    assert_eq!(format!("{:?}", panta_dsl_core::Rule::document), "document");
    let source = include_str!("../../../tests/fixtures/dsl/artifacts.pa");
    let canonical = include_str!("../../../tests/fixtures/dsl/artifacts.formatted.pa");
    let expected_ts = include_str!("../../../tests/fixtures/dsl/artifacts.ts");
    let document = parse(source)?;
    assert_eq!(panta_dsl_core::format_document(&document), canonical);
    assert_eq!(panta_dsl_core::format_source(canonical)?, canonical);
    assert_eq!(parse(canonical)?, document);
    assert_eq!(emit_ts(&document, "cn")?, expected_ts);
    assert_eq!(emit_ts(&document, "zh_CN")?, expected_ts);
    Ok(())
}
