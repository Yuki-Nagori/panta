//! 唯一 Pest grammar、入口识别与语法错误转换；FSM 与数据文档共用。

use pest::error::{Error as PestError, InputLocation};
use pest_derive::Parser;

use crate::{Diagnostics, SourceKind};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub(crate) struct PaParser;

/// 扫描头部块（首个空行之前）的 `kind:` 行；无法判定时返回 `None`。
pub fn source_kind(source: &str) -> Option<SourceKind> {
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            break;
        }
        if trimmed.starts_with("//") {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("kind:") {
            return match value.trim() {
                "language" => Some(SourceKind::Language),
                "theme" => Some(SourceKind::Theme),
                "variables" => Some(SourceKind::Variables),
                "fsm" => Some(SourceKind::Fsm),
                _ => None,
            };
        }
    }
    None
}

pub(crate) const MAX_SOURCE_BYTES: usize = 1_048_576;
pub(crate) const MAX_DECLARATIONS: usize = 16_384;

pub(crate) fn pest_diagnostic(error: PestError<Rule>, source: &str) -> Diagnostics {
    let offset = match error.location {
        InputLocation::Pos(position) => position,
        InputLocation::Span((start, _)) => start,
    };
    Diagnostics::one("pa.syntax", error.to_string(), offset, source)
}
