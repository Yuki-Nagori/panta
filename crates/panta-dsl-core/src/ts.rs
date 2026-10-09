//! 确定性 Qt TS 输出；公共 AST 可手工修改，输出前复核目标 locale 与占位符。

use std::collections::BTreeMap;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

use crate::validation::{normalize_locale, placeholders, valid_locale};
use crate::{Diagnostic, Diagnostics, Document, Kind, Message, Status};

/// 为 language 文档生成目标 locale 的 TS；拒绝非法 locale 与占位符不一致。
pub fn emit_ts(document: &Document, locale: &str) -> Result<String, Diagnostics> {
    if document.kind != Kind::Language {
        return Err(Diagnostics::one(
            "pa.ts_kind",
            "TS output requires kind language",
            0,
            "",
        ));
    }
    let locale = normalize_locale(locale);
    if !valid_locale(&locale) {
        return Err(Diagnostics::one(
            "pa.ts_locale",
            "invalid output locale",
            0,
            "",
        ));
    }
    let mut diagnostics = Vec::new();
    for message in document.messages.values() {
        if let Some(translation) = message.translations.get(&locale)
            && message.status != Some(Status::Unfinished)
            && message.status != Some(Status::Vanished)
        {
            for form in &translation.forms {
                if placeholders(&message.source) != placeholders(form) {
                    diagnostics.push(Diagnostic {
                        code: "pa.placeholder_mismatch".to_owned(),
                        message: format!("message '{}' has different placeholders", message.id),
                        offset: 0,
                        line: 1,
                        column: 1,
                    });
                    break;
                }
            }
        }
    }
    if !diagnostics.is_empty() {
        return Err(Diagnostics { diagnostics });
    }
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    let mut ts = BytesStart::new("TS");
    ts.push_attribute(("version", "2.1"));
    ts.push_attribute(("language", locale.replace('-', "_").as_str()));
    ts.push_attribute(("sourcelanguage", document.source_language.as_str()));
    writer
        .write_event(Event::Start(ts))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    let mut contexts: BTreeMap<&str, Vec<&Message>> = BTreeMap::new();
    for message in document.messages.values() {
        contexts
            .entry(message.context.as_str())
            .or_default()
            .push(message);
    }
    for (context, messages) in contexts {
        writer
            .write_event(Event::Start(BytesStart::new("context")))
            .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
        write_text_element(&mut writer, "name", context)?;
        for message in messages {
            emit_message(&mut writer, message, &locale)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("context")))
            .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("TS")))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    String::from_utf8(writer.into_inner())
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))
}

fn emit_message(
    writer: &mut Writer<Vec<u8>>,
    message: &Message,
    locale: &str,
) -> Result<(), Diagnostics> {
    let mut node = BytesStart::new("message");
    // Qt 的 ID 在整个 TS 中唯一，.pa 的 ID 只在 context 内唯一。
    // 为 qtTrId 限定命名空间，同时保持 context + source 查找语义。
    let qualified_id = format!("{}.{}", message.context, message.id);
    node.push_attribute(("id", qualified_id.as_str()));
    if message.numerus {
        node.push_attribute(("numerus", "yes"));
    }
    writer
        .write_event(Event::Start(node))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    if let Some(comment) = &message.comment {
        write_text_element(writer, "comment", comment)?;
    }
    if let Some(extra) = &message.extra {
        write_text_element(writer, "extracomment", extra)?;
    }
    if let Some(old_source) = &message.old_source {
        write_text_element(writer, "oldsource", old_source)?;
    }
    write_text_element(writer, "source", &message.source)?;
    let translation = message.translations.get(locale);
    let mut translation_node = BytesStart::new("translation");
    if message.status == Some(Status::Vanished) {
        translation_node.push_attribute(("type", "vanished"));
    } else if message.status == Some(Status::Unfinished) || translation.is_none() {
        translation_node.push_attribute(("type", "unfinished"));
    }
    writer
        .write_event(Event::Start(translation_node))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    let forms = translation
        .map(|value| value.forms.as_slice())
        .unwrap_or(&[]);
    if message.numerus {
        for form in forms.iter().take(2) {
            write_text_element(writer, "numerusform", form)?;
        }
        if forms.is_empty() {
            write_text_element(writer, "numerusform", &message.source)?;
        }
    } else {
        writer
            .write_event(Event::Text(BytesText::new(
                forms.first().unwrap_or(&message.source),
            )))
            .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("translation")))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    writer
        .write_event(Event::End(BytesEnd::new("message")))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    Ok(())
}

fn write_text_element(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: &str,
) -> Result<(), Diagnostics> {
    writer
        .write_event(Event::Start(BytesStart::new(name)))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    Ok(())
}
