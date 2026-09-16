#![allow(clippy::expect_used)]

use panta_dsl_core::{emit_ts, parse, source_prefix};

const CATALOG: &str = include_str!("fixtures/panta-ui.pa");

#[test]
fn fixture_is_a_global_catalog() {
    let document = parse(CATALOG).expect("fixture should parse");
    assert_eq!(document.messages.len(), 3);
    assert_eq!(
        document.messages["Advance revision"].source,
        "Advance revision"
    );
}

#[test]
fn fixture_emits_locale_and_uses_longest_source() {
    let document = parse(CATALOG).expect("fixture should parse");
    let ts = emit_ts(&document, "cn").expect("locale should emit");
    assert!(ts.contains("language=\"zh_CN\""));
    assert_eq!(
        source_prefix(&document, "ok ok!").map(|(key, _)| key),
        Some("ok ok")
    );
}
