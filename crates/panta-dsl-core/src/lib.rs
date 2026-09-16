//! Shared parser and artifact aggregation for the human-readable `.pa` DSL.

use std::collections::BTreeMap;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Finished,
    Unfinished,
    Vanished,
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
pub struct Translation {
    pub forms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub context: String,
    pub id: String,
    pub source: String,
    pub old_source: Option<String>,
    pub translations: BTreeMap<String, Translation>,
    pub status: Option<Status>,
    pub comment: Option<String>,
    pub extra: Option<String>,
    pub numerus: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    pub kind: Kind,
    pub catalog: Option<String>,
    pub language: Option<String>,
    pub source_language: String,
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
        language: None,
        source_language: "en".to_owned(),
        messages: BTreeMap::new(),
        values: Vec::new(),
    };
    let mut seen_version = false;
    let mut seen_kind = false;
    let mut seen_language = false;
    let mut seen_source_language = false;
    let mut section: Option<String> = None;
    let mut current_context = "Panta".to_owned();
    let mut current_message: Option<(String, Message)> = None;
    let mut declaration_count = 0usize;
    let mut diagnostics = Vec::new();

    for line in document_pair.into_inner() {
        let Some(body) = line.into_inner().next() else {
            continue;
        };
        let body_offset = body.as_span().start();
        match body.as_rule() {
            Rule::blank | Rule::comment_line => {}
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
                    result.version = find_text(body, Rule::integer)
                        .and_then(|value| value.parse::<u32>().ok())
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
                    if let Some(kind) = find_text(body, Rule::kind_name) {
                        result.kind = match kind.as_str() {
                            "language" => Kind::Language,
                            "theme" => Kind::Theme,
                            "variables" => Kind::Variables,
                            _ => Kind::Variables,
                        };
                    }
                }
            }
            Rule::catalog => set_header(
                &mut result.catalog,
                body,
                "catalog",
                source,
                &mut diagnostics,
                Rule::catalog_name,
            ),
            Rule::language => {
                if seen_language {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.duplicate_header",
                        "language is repeated",
                        body_offset,
                        source,
                    );
                } else {
                    seen_language = true;
                    result.language =
                        find_text(body, Rule::locale_name).map(|value| normalize_locale(&value));
                }
            }
            Rule::sourcelanguage => {
                if seen_source_language {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.duplicate_header",
                        "sourcelanguage is repeated",
                        body_offset,
                        source,
                    );
                } else {
                    seen_source_language = true;
                    result.source_language = find_text(body, Rule::locale_name)
                        .map(|value| normalize_locale(&value))
                        .unwrap_or_else(|| "en".to_owned());
                }
            }
            Rule::context => {
                flush_message(&mut current_message, &mut result, &mut diagnostics, source);
                current_context = find_text(body, Rule::context_name)
                    .map(|value| value.trim().to_owned())
                    .unwrap_or_else(|| "Panta".to_owned());
                section = None;
            }
            Rule::section => {
                section = find_text(body, Rule::section_name);
            }
            Rule::message => {
                flush_message(&mut current_message, &mut result, &mut diagnostics, source);
                let id = find_message_key(body, source, &mut diagnostics).unwrap_or_default();
                if id.is_empty() {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.empty_id",
                        "language entry requires an id",
                        body_offset,
                        source,
                    );
                    continue;
                }
                current_message = Some((
                    message_key(&current_context, &id),
                    Message {
                        context: current_context.clone(),
                        id,
                        source: String::new(),
                        old_source: None,
                        translations: BTreeMap::new(),
                        status: None,
                        comment: None,
                        extra: None,
                        numerus: false,
                    },
                ));
            }
            Rule::src => {
                if let Some((_, message)) = current_message.as_mut() {
                    let value = field_value(body, source, &mut diagnostics);
                    if message.source.is_empty() {
                        message.source = value;
                    } else {
                        duplicate_field(&mut diagnostics, "src", body_offset, source);
                    }
                } else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.field_without_message",
                        "src must follow a message id",
                        body_offset,
                        source,
                    );
                }
            }
            Rule::oldsrc => {
                if let Some((_, message)) = current_message.as_mut() {
                    let value = field_value(body, source, &mut diagnostics);
                    if message.old_source.is_none() {
                        message.old_source = Some(value);
                    } else {
                        duplicate_field(&mut diagnostics, "oldsrc", body_offset, source);
                    }
                } else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.field_without_message",
                        "oldsrc must follow a message id",
                        body_offset,
                        source,
                    );
                }
            }
            Rule::translation => {
                if let Some((_, message)) = current_message.as_mut() {
                    let forms = translation_value(body, source, &mut diagnostics);
                    let locale = result
                        .language
                        .clone()
                        .unwrap_or_else(|| result.source_language.clone());
                    if message
                        .translations
                        .insert(locale, Translation { forms })
                        .is_some()
                    {
                        duplicate_field(&mut diagnostics, "tr", body_offset, source);
                    }
                } else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.field_without_message",
                        "tr must follow a message id",
                        body_offset,
                        source,
                    );
                }
            }
            Rule::status => {
                if let Some((_, message)) = current_message.as_mut() {
                    if message.status.is_some() {
                        duplicate_field(&mut diagnostics, "st", body_offset, source);
                    }
                    if let Some(value) = find_text(body, Rule::status_name) {
                        message.status = match value.as_str() {
                            "finished" => Some(Status::Finished),
                            "unfinished" => Some(Status::Unfinished),
                            "vanished" => Some(Status::Vanished),
                            _ => None,
                        };
                    }
                } else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.field_without_message",
                        "st must follow a message id",
                        body_offset,
                        source,
                    );
                }
            }
            Rule::comment_field => {
                if let Some((_, message)) = current_message.as_mut() {
                    if message.comment.is_some() {
                        duplicate_field(&mut diagnostics, "comment", body_offset, source);
                    }
                    message.comment = Some(field_value(body, source, &mut diagnostics));
                } else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.field_without_message",
                        "comment must follow a message id",
                        body_offset,
                        source,
                    );
                }
            }
            Rule::extra_field => {
                if let Some((_, message)) = current_message.as_mut() {
                    if message.extra.is_some() {
                        duplicate_field(&mut diagnostics, "extra", body_offset, source);
                    }
                    message.extra = Some(field_value(body, source, &mut diagnostics));
                } else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.field_without_message",
                        "extra must follow a message id",
                        body_offset,
                        source,
                    );
                }
            }
            Rule::numerus => {
                if let Some((_, message)) = current_message.as_mut() {
                    if let Some(value) = find_text(body, Rule::boolean) {
                        message.numerus = value == "true";
                    }
                } else {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.field_without_message",
                        "numerus must follow a message id",
                        body_offset,
                        source,
                    );
                }
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
                if key.contains('_') {
                    push_diagnostic(
                        &mut diagnostics,
                        "pa.invalid_key",
                        format!("value key '{key}' must use kebab-case"),
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
    flush_message(&mut current_message, &mut result, &mut diagnostics, source);

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
    if let Some(language) = result.language.as_deref()
        && !valid_locale(language)
    {
        push_diagnostic(
            &mut diagnostics,
            "pa.invalid_locale",
            format!("invalid language locale '{language}'"),
            0,
            source,
        );
    }
    if !valid_locale(&result.source_language) {
        push_diagnostic(
            &mut diagnostics,
            "pa.invalid_locale",
            format!("invalid source locale '{}'", result.source_language),
            0,
            source,
        );
    }
    match result.kind {
        Kind::Language => {
            if result.language.is_none() {
                push_diagnostic(
                    &mut diagnostics,
                    "pa.missing_language",
                    "language requires a target locale",
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
            if !result.values.is_empty() {
                push_diagnostic(
                    &mut diagnostics,
                    "pa.unexpected_values",
                    "values are only valid for theme or variables",
                    0,
                    source,
                );
            }
        }
        Kind::Theme | Kind::Variables => {
            if result.catalog.is_some() || result.language.is_some() || !result.messages.is_empty()
            {
                push_diagnostic(
                    &mut diagnostics,
                    "pa.unexpected_language_field",
                    "language fields are only valid for language",
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
    validate_messages(&result, source, &mut diagnostics);
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
    node.push_attribute(("id", message.id.as_str()));
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

/// Returns the first source match at the start of `input`, preferring the longest source.
pub fn source_prefix<'a>(document: &'a Document, input: &str) -> Option<(&'a str, &'a Message)> {
    document
        .messages
        .values()
        .filter(|message| input.starts_with(&message.source))
        .max_by_key(|message| message.source.chars().count())
        .map(|message| (message.id.as_str(), message))
}

fn duplicate_field(diagnostics: &mut Vec<Diagnostic>, field: &str, offset: usize, source: &str) {
    push_diagnostic(
        diagnostics,
        "pa.duplicate_field",
        format!("{field} is repeated"),
        offset,
        source,
    );
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
    *target = find_text(body, value_rule);
}

fn flush_message(
    current: &mut Option<(String, Message)>,
    result: &mut Document,
    diagnostics: &mut Vec<Diagnostic>,
    source: &str,
) {
    let Some((key, message)) = current.take() else {
        return;
    };
    if message.source.is_empty() {
        push_diagnostic(
            diagnostics,
            "pa.missing_src",
            format!("message '{}' requires src", message.id),
            0,
            source,
        );
    }
    if result.messages.insert(key, message).is_some() {
        push_diagnostic(
            diagnostics,
            "pa.duplicate_message",
            "context and id are repeated",
            0,
            source,
        );
    }
}

fn find_text(body: Pair<'_, Rule>, rule: Rule) -> Option<String> {
    if body.as_rule() == rule {
        return Some(body.as_str().to_owned());
    }
    body.into_inner().find_map(|pair| find_text(pair, rule))
}

fn find_message_key(
    body: Pair<'_, Rule>,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let key = find_text(body.clone(), Rule::bare_key)
        .or_else(|| find_text(body, Rule::quoted).map(|value| value.trim_matches('"').to_owned()));
    key.map(|value| decode_scalar(&value, source, diagnostics))
}

fn field_value(body: Pair<'_, Rule>, source: &str, diagnostics: &mut Vec<Diagnostic>) -> String {
    find_text(body.clone(), Rule::quoted)
        .map(|value| decode_scalar(value.trim_matches('"'), source, diagnostics))
        .or_else(|| {
            find_text(body, Rule::scalar).map(|value| decode_scalar(&value, source, diagnostics))
        })
        .unwrap_or_default()
}

fn translation_value(
    body: Pair<'_, Rule>,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<String> {
    if let Some(array) = find_pair(body.clone(), Rule::array) {
        return array
            .into_inner()
            .filter(|pair| pair.as_rule() == Rule::quoted)
            .map(|pair| decode_scalar(pair.as_str().trim_matches('"'), source, diagnostics))
            .collect();
    }
    vec![field_value(body, source, diagnostics)]
}

fn find_pair(body: Pair<'_, Rule>, rule: Rule) -> Option<Pair<'_, Rule>> {
    if body.as_rule() == rule {
        Some(body)
    } else {
        body.into_inner().find_map(|pair| find_pair(pair, rule))
    }
}

fn message_key(context: &str, id: &str) -> String {
    format!("{context}\u{1f}{id}")
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

fn valid_locale(locale: &str) -> bool {
    let mut chars = locale.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    let mut previous_separator = false;
    for character in chars {
        if character.is_ascii_alphanumeric() {
            previous_separator = false;
        } else if character == '-' || character == '_' {
            if previous_separator {
                return false;
            }
            previous_separator = true;
        } else {
            return false;
        }
    }
    !previous_separator
}

fn typed_entry(
    value_pair: Pair<'_, Rule>,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Option<ValueType>, String) {
    if value_pair.as_rule() != Rule::typed_value {
        push_diagnostic(
            diagnostics,
            "pa.value_type",
            "values require type = expression",
            0,
            source,
        );
        return (None, String::new());
    }
    let value_type =
        find_text(value_pair.clone(), Rule::type_name).and_then(|value| ValueType::parse(&value));
    let expression = find_text(value_pair, Rule::scalar)
        .map(|value| decode_scalar(&value, source, diagnostics))
        .unwrap_or_default();
    (value_type, expression)
}

fn validate_messages(document: &Document, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut source_lengths: BTreeMap<(String, usize), String> = BTreeMap::new();
    for message in document.messages.values() {
        let length_key = (message.context.clone(), message.source.chars().count());
        if let Some(previous_id) = source_lengths.insert(length_key, message.id.clone())
            && previous_id != message.id
        {
            push_diagnostic(
                diagnostics,
                "pa.source_length_conflict",
                format!(
                    "context '{}' contains messages '{}' and '{}' with the same source length",
                    message.context, previous_id, message.id
                ),
                0,
                source,
            );
        }
        if message.numerus
            && message
                .translations
                .values()
                .any(|translation| translation.forms.is_empty())
        {
            push_diagnostic(
                diagnostics,
                "pa.empty_plural",
                format!("message '{}' has no plural forms", message.id),
                0,
                source,
            );
        }
        if !message.numerus
            && message
                .translations
                .values()
                .any(|translation| translation.forms.len() > 1)
        {
            push_diagnostic(
                diagnostics,
                "pa.unexpected_plural",
                format!("message '{}' is not numerus", message.id),
                0,
                source,
            );
        }
    }
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

    const CATALOG: &str = "version: 1\nkind: language\nlanguage: zh_CN\nsourcelanguage: en\n\n[FileMenu]\nopen:\n  src: Open\n  tr: 打开\n\nok:\n  src: ok\n  tr: 好的\n\nok-pair:\n  src: ok ok\n  tr: 好好的\n";

    #[test]
    fn parses_ts_shaped_catalog() {
        let document = parse(CATALOG).expect("valid catalog");
        assert_eq!(document.language.as_deref(), Some("zh-CN"));
        assert_eq!(document.messages.len(), 3);
        assert_eq!(
            document.messages[&message_key("FileMenu", "open")].source,
            "Open"
        );
    }

    #[test]
    fn parses_status_plural_and_comments() {
        let document = parse("version: 1\nkind: language\nlanguage: cn\n\n[FileMenu]\nfiles-selected:\n  src: \"%n file(s) selected\"\n  numerus: true\n  tr: [\"已选择 %n 个文件\"]\n  comment: 菜单项\n  extra: 翻译提示\n  st: unfinished\n").expect("valid plural");
        let message = &document.messages[&message_key("FileMenu", "files-selected")];
        assert!(message.numerus);
        assert_eq!(message.translations["zh-CN"].forms.len(), 1);
        assert_eq!(message.status, Some(Status::Unfinished));
    }

    #[test]
    fn parses_variables_and_rejects_underscored_key() {
        let document = parse("version: 1\nkind: variables\n\nvalues:\n  spacing-small: real = 8\n")
            .expect("valid variables");
        assert_eq!(document.values[0].name, "spacing-small");
        assert!(
            parse("version: 1\nkind: variables\n\nvalues:\n  spacing_small: real = 8\n").is_err()
        );
    }

    #[test]
    fn rejects_duplicate_context_id_and_missing_source() {
        let error = parse("version: 1\nkind: language\nlanguage: cn\n\n[FileMenu]\nopen:\n  src: Open\nopen:\n  src: Open again\n").expect_err("duplicate id");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|item| item.code == "pa.duplicate_message")
        );
    }

    #[test]
    fn longest_source_and_ts_output_work() {
        let document = parse(CATALOG).expect("catalog");
        assert_eq!(
            source_prefix(&document, "ok ok!").map(|(id, _)| id),
            Some("ok-pair")
        );
        let ts = emit_ts(&document, "cn").expect("TS");
        assert!(ts.contains("language=\"zh_CN\""));
        assert!(ts.contains("<name>FileMenu</name>"));
        assert!(ts.contains("<source>ok ok</source>"));
        assert!(!ts.contains("<numerusform>好的</numerusform>"));
    }

    #[test]
    fn rejects_source_length_conflicts_and_invalid_locale() {
        let error = parse(
            "version: 1\nkind: language\nlanguage: en--US\n\n[Menu]\na:\n  src: One\n  tr: Uno\nb:\n  src: Two\n  tr: Dos\n",
        )
        .expect_err("invalid locale and equal source lengths");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|item| item.code == "pa.invalid_locale")
        );
        assert!(
            error
                .diagnostics
                .iter()
                .any(|item| item.code == "pa.source_length_conflict")
        );
    }
}
