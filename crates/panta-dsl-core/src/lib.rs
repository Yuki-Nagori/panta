//! Shared parser and artifact aggregation for the human-readable `.pa` DSL.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use pest::Parser as PestParser;
use pest::error::{Error as PestError, InputLocation};
use pest::iterators::Pair;
use pest_derive::Parser;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PaParser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Language,
    Theme,
    Variables,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    Bool,
    Int,
    Real,
    String,
    Resource,
}

impl ValueType {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "bool" => Some(Self::Bool),
            "int" => Some(Self::Int),
            "real" => Some(Self::Real),
            "string" => Some(Self::String),
            "resource" => Some(Self::Resource),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value_type: ValueType,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub source: String,
    pub translations: BTreeMap<String, String>,
    pub source_note: Option<String>,
    pub translation_notes: BTreeMap<String, String>,
    pub unfinished: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    pub kind: Kind,
    pub catalog: Option<String>,
    pub fallback: Option<String>,
    pub messages: BTreeMap<String, Message>,
    pub values: Vec<Variable>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    fn one(code: &str, message: impl Into<String>, offset: usize, source: &str) -> Self {
        let (line, column) = source_position(source, offset);
        Self {
            diagnostics: vec![Diagnostic {
                code: code.to_owned(),
                message: message.into(),
                offset,
                line,
                column,
            }],
        }
    }
}

impl Display for Diagnostics {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            write!(
                formatter,
                "{} at {}:{}: {}",
                diagnostic.code, diagnostic.line, diagnostic.column, diagnostic.message
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

const MAX_SOURCE_BYTES: usize = 1_048_576;
const MAX_DECLARATIONS: usize = 16_384;

pub fn parse(source: &str) -> Result<Document, Diagnostics> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(Diagnostics::one(
            "pa.source_too_large",
            format!("source exceeds {MAX_SOURCE_BYTES} bytes"),
            0,
            source,
        ));
    }

    let Some(document_pair) = PaParser::parse(Rule::document, source)
        .map_err(|error| pest_diagnostic(error, source))?
        .next()
    else {
        return Err(Diagnostics::one(
            "pa.syntax",
            "document is empty",
            0,
            source,
        ));
    };

    let mut result = Document {
        version: 0,
        kind: Kind::Variables,
        catalog: None,
        fallback: None,
        messages: BTreeMap::new(),
        values: Vec::new(),
    };
    let mut seen_version = false;
    let mut seen_kind = false;
    let mut section: Option<String> = None;
    let mut current_message: Option<(String, Message)> = None;
    let mut declaration_count = 0usize;
    let mut diagnostics = Vec::new();

    for line in document_pair.into_inner() {
        let Some(body) = line.into_inner().next() else {
            continue;
        };
        let body_offset = body.as_span().start();
        match body.as_rule() {
            Rule::blank | Rule::comment => {}
            Rule::version => {
                if seen_version {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.duplicate_header",
                        "version is repeated",
                        body_offset,
                        source,
                    );
                } else {
                    seen_version = true;
                    result.version = body
                        .into_inner()
                        .find(|pair| pair.as_rule() == Rule::integer)
                        .and_then(|pair| pair.as_str().parse::<u32>().ok())
                        .unwrap_or_default();
                }
            }
            Rule::kind => {
                if seen_kind {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.duplicate_header",
                        "kind is repeated",
                        body_offset,
                        source,
                    );
                } else {
                    seen_kind = true;
                    if let Some(pair) = body
                        .into_inner()
                        .find(|pair| pair.as_rule() == Rule::kind_name)
                    {
                        result.kind = match pair.as_str() {
                            "language" => Kind::Language,
                            "theme" => Kind::Theme,
                            "variables" => Kind::Variables,
                            _ => Kind::Variables,
                        };
                    }
                }
            }
            Rule::catalog => {
                set_header(
                    &mut result.catalog,
                    body,
                    "catalog",
                    source,
                    &mut diagnostics,
                    Rule::catalog_name,
                );
            }
            Rule::fallback => {
                set_header(
                    &mut result.fallback,
                    body,
                    "fallback",
                    source,
                    &mut diagnostics,
                    Rule::locale_name,
                );
                if let Some(fallback) = result.fallback.as_mut() {
                    *fallback = normalize_locale(fallback);
                }
            }
            Rule::message => {
                flush_message(&mut current_message, &mut result);
                let key = body
                    .into_inner()
                    .find(|pair| pair.as_rule() == Rule::source_key)
                    .map(|pair| decode_scalar(pair.as_str(), source, &mut diagnostics))
                    .unwrap_or_default();
                if key.is_empty() {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.empty_source",
                        "language entry requires a source text",
                        body_offset,
                        source,
                    );
                    continue;
                }
                if result.messages.contains_key(&key) {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.duplicate_source",
                        format!("source '{key}' is repeated"),
                        body_offset,
                        source,
                    );
                    current_message = None;
                } else {
                    current_message = Some((
                        key.clone(),
                        Message {
                            source: key,
                            translations: BTreeMap::new(),
                            source_note: None,
                            translation_notes: BTreeMap::new(),
                            unfinished: BTreeSet::new(),
                        },
                    ));
                }
            }
            Rule::translation => {
                let Some((key, message)) = current_message.as_mut() else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.translation_without_source",
                        "translation must follow a source entry",
                        body_offset,
                        source,
                    );
                    continue;
                };
                let mut parts = body.into_inner();
                let Some(locale) = parts
                    .find(|pair| pair.as_rule() == Rule::locale_name)
                    .map(|pair| normalize_locale(pair.as_str()))
                else {
                    continue;
                };
                let value = parts
                    .find(|pair| pair.as_rule() == Rule::scalar)
                    .map(|pair| decode_scalar(pair.as_str(), source, &mut diagnostics))
                    .unwrap_or_default();
                if message.translations.insert(locale.clone(), value).is_some() {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.duplicate_translation",
                        format!("translation locale '{locale}' for '{key}' is repeated"),
                        body_offset,
                        source,
                    );
                }
            }
            Rule::directive => {
                let Some((key, message)) = current_message.as_mut() else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.directive_without_source",
                        "directive must follow a source entry",
                        body_offset,
                        source,
                    );
                    continue;
                };
                let Some(directive) = body.into_inner().next() else {
                    continue;
                };
                match directive.as_rule() {
                    Rule::unfinished => {
                        if let Some(locale) = directive
                            .into_inner()
                            .find(|pair| pair.as_rule() == Rule::locale_name)
                        {
                            message.unfinished.insert(normalize_locale(locale.as_str()));
                        }
                    }
                    Rule::source_note => {
                        if message.source_note.is_some() {
                            push_diagnostic(
                                &mut diagnostics,
                                "pa.duplicate_source_note",
                                format!("source note for '{key}' is repeated"),
                                directive.as_span().start(),
                                source,
                            );
                        } else {
                            message.source_note = directive
                                .into_inner()
                                .find(|pair| pair.as_rule() == Rule::scalar)
                                .map(|pair| decode_scalar(pair.as_str(), source, &mut diagnostics));
                        }
                    }
                    Rule::translation_note => {
                        let mut parts = directive.into_inner();
                        let Some(locale) = parts
                            .find(|pair| pair.as_rule() == Rule::locale_name)
                            .map(|pair| normalize_locale(pair.as_str()))
                        else {
                            continue;
                        };
                        let note = parts
                            .find(|pair| pair.as_rule() == Rule::scalar)
                            .map(|pair| decode_scalar(pair.as_str(), source, &mut diagnostics))
                            .unwrap_or_default();
                        if message
                            .translation_notes
                            .insert(locale.clone(), note)
                            .is_some()
                        {
                            push_diagnostic(
                                &mut diagnostics,
                                "pa.duplicate_translation_note",
                                format!(
                                    "translation note locale '{locale}' for '{key}' is repeated"
                                ),
                                body_offset,
                                source,
                            );
                        }
                    }
                    _ => {}
                }
            }
            Rule::section => {
                section = body
                    .into_inner()
                    .find(|pair| pair.as_rule() == Rule::section_name)
                    .map(|pair| pair.as_str().to_owned());
            }
            Rule::entry => {
                declaration_count += 1;
                if declaration_count > MAX_DECLARATIONS {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.too_many_declarations",
                        format!("more than {MAX_DECLARATIONS} declarations"),
                        body_offset,
                        source,
                    );
                    continue;
                }
                let mut parts = body.into_inner();
                let Some(key) = parts.next().map(|pair| pair.as_str().to_owned()) else {
                    continue;
                };
                let Some(value_pair) = parts.next() else {
                    continue;
                };
                if section.as_deref() != Some("values") {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.entry_outside_section",
                        "value entry must be inside values",
                        body_offset,
                        source,
                    );
                    continue;
                }
                let (value_type, expression) = typed_entry(value_pair, source, &mut diagnostics);
                if result.values.iter().any(|item| item.name == key) {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.duplicate_key",
                        format!("value key '{key}' is repeated"),
                        body_offset,
                        source,
                    );
                } else if let Some(value_type) = value_type {
                    result.values.push(Variable {
                        name: key,
                        value_type,
                        expression,
                    });
                }
            }
            _ => {}
        }
    }
    flush_message(&mut current_message, &mut result);

    if !seen_version {
        push_diagnostic(
            &mut diagnostics,
            "pa.missing_version",
            "version is required",
            0,
            source,
        );
    } else if result.version != 1 {
        push_diagnostic(
            &mut diagnostics,
            "pa.unsupported_version",
            "only version 1 is supported",
            0,
            source,
        );
    }
    if !seen_kind {
        push_diagnostic(
            &mut diagnostics,
            "pa.missing_kind",
            "kind is required",
            0,
            source,
        );
    }
    match result.kind {
        Kind::Language => {
            if result.catalog.is_none() {
                push_diagnostic(
                    &mut diagnostics,
                    "pa.missing_catalog",
                    "language requires catalog",
                    0,
                    source,
                );
            }
            if !result.values.is_empty() {
                push_diagnostic(
                    &mut diagnostics,
                    "pa.unexpected_values",
                    "values are only valid for theme or variables",
                    0,
                    source,
                );
            }
            if result.messages.is_empty() {
                push_diagnostic(
                    &mut diagnostics,
                    "pa.empty_language",
                    "language catalog has no messages",
                    0,
                    source,
                );
            }
        }
        Kind::Theme | Kind::Variables => {
            if result.catalog.is_some() || result.fallback.is_some() || !result.messages.is_empty()
            {
                push_diagnostic(
                    &mut diagnostics,
                    "pa.unexpected_language_field",
                    "catalog, fallback and messages are only valid for language",
                    0,
                    source,
                );
            }
            if result.values.is_empty() {
                push_diagnostic(
                    &mut diagnostics,
                    "pa.empty_values",
                    "theme or variables requires values",
                    0,
                    source,
                );
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(result)
    } else {
        Err(Diagnostics { diagnostics })
    }
}

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
    if locale.is_empty()
        || !locale.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        return Err(Diagnostics::one(
            "pa.ts_locale",
            "invalid output locale",
            0,
            "",
        ));
    }

    let mut diagnostics = Vec::new();
    for (key, message) in &document.messages {
        if let Some(translation) = message.translations.get(&locale)
            && !message.unfinished.contains(&locale)
            && placeholders(&message.source) != placeholders(translation)
        {
            diagnostics.push(Diagnostic {
                code: "pa.placeholder_mismatch".to_owned(),
                message: format!("message '{key}' has different placeholders"),
                offset: 0,
                line: 1,
                column: 1,
            });
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
    ts.push_attribute(("sourcelanguage", "en"));
    writer
        .write_event(Event::Start(ts))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    writer
        .write_event(Event::Start(BytesStart::new("context")))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    write_text_element(&mut writer, "name", "Panta")?;
    for (key, message) in &document.messages {
        let translation = message.translations.get(&locale).unwrap_or(&message.source);
        let mut message_node = BytesStart::new("message");
        message_node.push_attribute(("id", key.as_str()));
        if message.source.contains("%n") {
            message_node.push_attribute(("numerus", "yes"));
        }
        writer
            .write_event(Event::Start(message_node))
            .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
        if let Some(note) = &message.source_note {
            write_text_element(&mut writer, "extracomment", note)?;
        }
        write_text_element(&mut writer, "source", &message.source)?;
        let unfinished =
            !message.translations.contains_key(&locale) || message.unfinished.contains(&locale);
        let mut translation_node = BytesStart::new("translation");
        if unfinished {
            translation_node.push_attribute(("type", "unfinished"));
        }
        writer
            .write_event(Event::Start(translation_node))
            .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
        writer
            .write_event(Event::Text(BytesText::new(translation)))
            .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
        writer
            .write_event(Event::End(BytesEnd::new("translation")))
            .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
        writer
            .write_event(Event::End(BytesEnd::new("message")))
            .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("context")))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    writer
        .write_event(Event::End(BytesEnd::new("TS")))
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))?;
    String::from_utf8(writer.into_inner())
        .map_err(|error| Diagnostics::one("pa.ts_write", error.to_string(), 0, ""))
}

/// Returns the first source match at the start of `input`, preferring the longest source.
/// Exact source lookup is the normal QML/C++ path; this helper is for text adapters.
pub fn source_prefix<'a>(document: &'a Document, input: &str) -> Option<(&'a str, &'a Message)> {
    document
        .messages
        .iter()
        .filter(|(_, message)| input.starts_with(&message.source))
        .max_by_key(|(_, message)| message.source.chars().count())
        .map(|(key, message)| (key.as_str(), message))
}

fn normalize_locale(locale: &str) -> String {
    if locale.eq_ignore_ascii_case("cn")
        || locale.eq_ignore_ascii_case("zh-cn")
        || locale.eq_ignore_ascii_case("zh_cn")
    {
        "zh-CN".to_owned()
    } else {
        locale.to_owned()
    }
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

fn set_header(
    target: &mut Option<String>,
    body: Pair<'_, Rule>,
    name: &str,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
    value_rule: Rule,
) {
    if target.is_some() {
        push_diagnostic(
            diagnostics,
            "pa.duplicate_header",
            format!("{name} is repeated"),
            body.as_span().start(),
            source,
        );
        return;
    }
    *target = body
        .into_inner()
        .find(|pair| pair.as_rule() == value_rule)
        .map(|pair| pair.as_str().to_owned());
}

fn flush_message(current: &mut Option<(String, Message)>, result: &mut Document) {
    let Some((key, message)) = current.take() else {
        return;
    };
    result.messages.insert(key, message);
}

fn typed_entry(
    value_pair: Pair<'_, Rule>,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Option<ValueType>, String) {
    let typed = if value_pair.as_rule() == Rule::typed_value {
        value_pair
    } else {
        push_diagnostic(
            diagnostics,
            "pa.value_type",
            "values require type = expression",
            0,
            source,
        );
        return (None, String::new());
    };
    let mut parts = typed.into_inner();
    let value_type = parts
        .find(|pair| pair.as_rule() == Rule::type_name)
        .and_then(|pair| ValueType::parse(pair.as_str()));
    let expression = parts
        .find(|pair| pair.as_rule() == Rule::scalar)
        .map(|pair| decode_scalar(pair.as_str(), source, diagnostics))
        .unwrap_or_default();
    (value_type, expression)
}

fn decode_scalar(raw: &str, source: &str, diagnostics: &mut Vec<Diagnostic>) -> String {
    let raw = raw.trim_end_matches([' ', '\t']);
    let mut result = String::with_capacity(raw.len());
    let mut escaped = false;
    for character in raw.chars() {
        if escaped {
            match character {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                ':' | '=' | ' ' | '\\' => result.push(character),
                _ => {
                    push_diagnostic(
                        diagnostics,
                        "pa.invalid_escape",
                        format!("unsupported escape \\{character}"),
                        0,
                        source,
                    );
                    result.push(character);
                }
            }
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            result.push(character);
        }
    }
    if escaped {
        push_diagnostic(
            diagnostics,
            "pa.invalid_escape",
            "trailing escape",
            raw.len().saturating_sub(1),
            source,
        );
    }
    result
}

fn placeholders(value: &str) -> Vec<String> {
    let bytes = value.as_bytes();
    let mut result = Vec::new();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'%' {
            if bytes[index + 1] == b'n' {
                result.push("%n".to_owned());
                index += 2;
                continue;
            }
            if bytes[index + 1].is_ascii_digit() {
                let start = index;
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                result.push(value[start..index].to_owned());
                continue;
            }
        }
        index += 1;
    }
    result.sort();
    result
}

fn pest_diagnostic(error: PestError<Rule>, source: &str) -> Diagnostics {
    let offset = match error.location {
        InputLocation::Pos(position) => position,
        InputLocation::Span((start, _)) => start,
    };
    Diagnostics::one("pa.syntax", error.to_string(), offset, source)
}

fn push_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    code: &str,
    message: impl Into<String>,
    offset: usize,
    source: &str,
) {
    let (line, column) = source_position(source, offset);
    diagnostics.push(Diagnostic {
        code: code.to_owned(),
        message: message.into(),
        offset,
        line,
        column,
    });
}

fn source_position(source: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(source.len());
    let before = &source[..clamped];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map_or(1, |last| last.chars().count() + 1);
    (line, column)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    const CATALOG: &str = "version: 1\nkind: language\ncatalog: panta-ui\nfallback: en\n\nok:\n  translation cn: 好的\n\nok ok:\n  translation cn: 好好的\n";

    #[test]
    fn parses_global_catalog_without_quotes() {
        let document = parse(CATALOG).expect("valid catalog");
        assert_eq!(document.kind, Kind::Language);
        assert_eq!(document.catalog.as_deref(), Some("panta-ui"));
        assert_eq!(document.messages["ok"].source, "ok");
        assert_eq!(document.messages["ok"].translations["zh-CN"], "好的");
    }

    #[test]
    fn parses_source_text_with_escaped_colon_and_cn_alias() {
        let document = parse(
            "version: 1\nkind: language\ncatalog: panta-ui\n\nRevision\\: %1:\n  translation cn: 修订 %1\n",
        )
        .expect("source text should parse");
        assert_eq!(
            document.messages["Revision: %1"].translations["zh-CN"],
            "修订 %1"
        );
    }

    #[test]
    fn parses_typed_values_and_escapes() {
        let source = "version: 1\nkind: variables\n\nvalues:\n  label: string = Inlet\\: A\n  count: int = 24\n";
        let document = parse(source).expect("valid document");
        assert_eq!(document.values.len(), 2);
        assert_eq!(document.values[0].expression, "Inlet: A");
        assert_eq!(document.values[1].value_type, ValueType::Int);
    }

    #[test]
    fn enforces_kebab_case_for_business_keys() {
        let error = parse("version: 1\nkind: variables\n\nvalues:\n  spacing_small: real = 8\n")
            .expect_err("underscore is not a business key");
        assert_eq!(error.diagnostics[0].code, "pa.syntax");
    }

    #[test]
    fn language_rejects_variable_values() {
        let error = parse(
            "version: 1\nkind: language\ncatalog: panta-ui\n\nvalues:\n  spacing: real = 8\n",
        )
        .expect_err("language cannot contain values");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|item| item.code == "pa.unexpected_values")
        );
    }

    #[test]
    fn rejects_duplicate_source_and_unknown_version() {
        let error = parse(
            "version: 2\nkind: language\ncatalog: panta\nok:\n  translation cn: 好的\nok:\n  translation cn: 好好\n",
        )
        .expect_err("invalid document");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|item| item.code == "pa.duplicate_source")
        );
        assert!(
            error
                .diagnostics
                .iter()
                .any(|item| item.code == "pa.unsupported_version")
        );
    }

    #[test]
    fn emits_qt_ts_per_locale() {
        let document = parse(CATALOG).expect("catalog");
        let ts = emit_ts(&document, "cn").expect("TS");
        assert!(ts.contains("id=\"ok ok\""));
        assert!(ts.contains("好的"));
        assert!(ts.contains("language=\"zh_CN\""));
        assert!(ts.contains("<source>ok</source>"));
    }

    #[test]
    fn source_prefix_prefers_longer_match() {
        let document = parse(CATALOG).expect("catalog");
        let (key, _) = source_prefix(&document, "ok ok!").expect("match");
        assert_eq!(key, "ok ok");
    }
}
