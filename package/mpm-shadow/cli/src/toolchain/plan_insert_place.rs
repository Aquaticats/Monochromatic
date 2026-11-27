//! What:
//! Where in a plan's text one new entry goes:
//! the `tools` object's own span,
//! the indentation its entries use,
//! the byte after its last entry's value,
//! and what has to be written around that entry's comma and comment.
//! Why:
//! The comma,
//! the whitespace and any comment beside the last entry are the person's,
//! so the place is read from the text itself rather than assumed from the file's shape,
//! and the same reading carries the indentation the new entry is written with.
//! Only `plan_insert` reads this module,
//! which is why every name here is `pub(super)`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // plan-insert-place.ts
//! type ToolsRecord = { place: Place; memberIndent: string; closingIndent: string };
//! function toolsRecord(source: string, label: string): ToolsRecord  // throws
//! ```

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::plan_insert_spans`.
use super::plan_insert_spans::{
    SpanWalk,
    misaligned,
    skip_children,
    skip_value,
};
/// Imports the span-reporting parser and the tree it hands back beside the positions.
use monochromatic_jsonc_edit::{
    JsoncKind,
    JsoncSpanned,
    JsoncValue,
    parse_jsonc_with_spans,
    units_to_string,
};

/// The indentation the toolchain plan of this repository uses for one nesting level.
pub(super) const ONE_LEVEL: &str = "  ";

/// What:
/// The places in a plan this module writes at,
/// read once from the parsed text.
/// Why:
/// One value carries every offset and indentation the splice needs,
/// so the splice itself is three `push_str` calls and cannot recompute one of them from a different read of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolsRecord = { insertAt: number; memberIndent: string; closingIndent: string; empty: boolean };
/// ```
pub(super) struct ToolsRecord {
    /// Where the new text goes,
    /// read from the text that follows the last entry,
    /// or from an object that holds none.
    pub(super) place: Place,
    /// The indentation one entry of this object gets,
    /// taken from the entry that is there already.
    pub(super) member_indent: String,
    /// The indentation of the line that holds the `{` of the object,
    /// written in front of the closing `}` of an empty object.
    pub(super) closing_indent: String,
}

/// What:
/// Where the new entry goes and what has to be written around it.
/// Why:
/// A comment beside the last entry belongs to that entry:
/// the comma that separates the new entry from it goes in front of the comment,
/// and the new entry goes after it,
/// so the explanation never ends up above the wrong tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Place =
///   | { kind: 'atLastValue'; valueEnd: number }
///   | { kind: 'afterComment'; valueEnd: number; commaPresent: boolean; afterComment: number }
///   | { kind: 'empty'; brace: number; bracesBroken: boolean };
/// ```
pub(super) enum Place {
    /// Nothing but whitespace (and the entry's own comma) follows the last value.
    AtLastValue {
        /// The byte just after the last member's value.
        value_end: usize,
    },
    /// A comment follows the last value on the same line.
    AfterComment {
        /// The byte just after the last member's value,
        /// where the comma goes when none is written yet.
        value_end: usize,
        /// Whether the entry already carries the comma after its value.
        comma_present: bool,
        /// The byte just after the comment,
        /// where the new entry goes.
        after_comment: usize,
    },
    /// The object holds no entry.
    Empty {
        /// The byte just after the object's `{`.
        brace: usize,
        /// Whether the object's braces already sit on separate lines.
        braces_broken: bool,
    },
}

/// What:
/// The indentation in front of the token at `offset`,
/// when that token is the first thing on its line.
/// Why:
/// The plan's own indentation is the one to keep;
/// a line that holds something before the token has no indentation to copy,
/// and the caller falls back to one level below the object's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineIndent(source: string, offset: number): string | undefined
/// ```
fn line_indent(
    source: &str,
    offset: usize,
) -> Option<String> {
    let start: usize = source[..offset].rfind('\n')? + 1;
    let prefix: &str = &source[start..offset];
    if prefix.trim().is_empty() {
        return Some(prefix.to_string());
    }
    return None;
}

/// What:
/// The whitespace the line holding `offset` starts with,
/// whatever follows it on that line.
/// Why:
/// An object written on the key's own line (`"tools": {}`) has no token to copy an indentation from,
/// and the line it sits on is still the one its entries are one level below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineLeading(source: string, offset: number): string
/// ```
fn line_leading(
    source: &str,
    offset: usize,
) -> String {
    let start: usize = match source[..offset].rfind('\n') {
        Some(newline) => newline + 1,
        None => 0,
    };
    let line: &str = &source[start..];
    let leading: &str = &line[..line.len() - line.trim_start().len()];
    return leading.to_string();
}

/// The indentation of one entry in the object whose `{` is at `brace`,
/// when no entry is there to copy it from.
fn indent_below(
    source: &str,
    brace: usize,
) -> String {
    return format!("{}{ONE_LEVEL}", line_leading(source, brace));
}

/// What:
/// Where the new entry goes,
/// read from the text between the last member's value and the object's closing brace.
/// Why:
/// The comma,
/// the whitespace and any comment beside that entry are the person's,
/// and the only question is where the new text goes around them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function placeAfter(source: string, valueEnd: number, objectEnd: number): Place
/// ```
fn place_after(
    source: &str,
    value_end: usize,
    object_end: usize,
) -> Place {
    let limit: usize = object_end.saturating_sub(1);
    let mut index: usize = value_end;
    let skip_blanks = |from: usize| -> usize {
        let mut cursor: usize = from;
        while cursor < limit
            && (source.as_bytes()[cursor] == b' ' || source.as_bytes()[cursor] == b'\t')
        {
            cursor += 1;
        }
        return cursor;
    };
    index = skip_blanks(index);
    let comma_present: bool = index < limit && source.as_bytes()[index] == b',';
    if comma_present {
        index += 1;
        index = skip_blanks(index);
    }
    if source[index.min(source.len())..].starts_with("//") {
        let after_comment: usize = match source[index..].find('\n') {
            Some(newline) => index + newline,
            None => source.len(),
        };
        return Place::AfterComment {
            value_end,
            comma_present,
            after_comment,
        };
    }
    if source[index.min(source.len())..].starts_with("/*") {
        let after_comment: usize = match source[index..].find("*/") {
            Some(close) => index + close + 2,
            None => source.len(),
        };
        return Place::AfterComment {
            value_end,
            comma_present,
            after_comment,
        };
    }
    return Place::AtLastValue {
        value_end,
    };
}

/// What:
/// Reads the `tools` object of a plan:
/// its span,
/// the end of its last entry's value,
/// and the indentation its entries use.
/// Why:
/// That is the one place the plan grows,
/// and reading it from the parsed text keeps the write from depending on the file's shape beyond it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toolsRecord(source: string, label: string): ToolsRecord
/// ```
pub(super) fn tools_record(
    source: &str,
    label: &str,
) -> Result<ToolsRecord, ToolchainError> {
    let spanned: JsoncSpanned = match parse_jsonc_with_spans(source) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Err(ToolchainError::config(format!(
                "{label} is not valid JSONC: {error}"
            )));
        }
    };
    let root: &JsoncValue = &spanned.value;
    let JsoncKind::Record {
        entries,
    } = &root.kind
    else {
        return Err(ToolchainError::config(format!(
            "{label} must hold a JSON object at its top level"
        )));
    };
    let mut walk: SpanWalk = SpanWalk {
        values: &spanned.spans.values,
        keys: &spanned.spans.keys,
        value_index: 0,
        key_index: 0,
    };
    walk.value().ok_or_else(|| return misaligned(label))?;
    for entry in entries {
        walk.key().ok_or_else(|| return misaligned(label))?;
        let key: String = match units_to_string(&entry.key.units) {
            Ok(text) => text,
            Err(error) => {
                return Err(ToolchainError::config(format!(
                    "{label}: a key is not valid text: {error}"
                )));
            }
        };
        if key != "tools" {
            skip_value(&mut walk, &entry.value, label)?;
            continue;
        }
        let brace_span: monochromatic_jsonc_edit::JsoncSpan =
            walk.value().ok_or_else(|| return misaligned(label))?;
        return tools_of(
            source,
            label,
            brace_span.start.offset,
            brace_span.end.offset,
            &entry.value,
            &mut walk,
        );
    }
    return Err(ToolchainError::config(format!(
        "{label} has no \"tools\" object"
    )));
}

/// Reads the members of the `tools` object at the span `[start, end)`.
fn tools_of(
    source: &str,
    label: &str,
    brace: usize,
    end: usize,
    value: &JsoncValue,
    walk: &mut SpanWalk,
) -> Result<ToolsRecord, ToolchainError> {
    if source.as_bytes().get(brace) != Some(&b'{')
        || end == 0
        || source.as_bytes().get(end - 1) != Some(&b'}')
    {
        return Err(misaligned(label));
    }
    let closing_indent: String = line_leading(source, brace);
    let JsoncKind::Record {
        entries,
    } = &value.kind
    else {
        return Err(ToolchainError::config(format!(
            "{label}: \"tools\" must be an object keyed by tool id"
        )));
    };
    if entries.is_empty() {
        return Ok(ToolsRecord {
            place: Place::Empty {
                brace: brace + 1,
                braces_broken: source[brace + 1..end - 1].contains('\n'),
            },
            member_indent: format!("{closing_indent}{ONE_LEVEL}"),
            closing_indent,
        });
    }
    let mut last: Option<(usize, usize)> = None;
    for entry in entries {
        let key_span: monochromatic_jsonc_edit::JsoncSpan =
            walk.key().ok_or_else(|| return misaligned(label))?;
        let value_span: monochromatic_jsonc_edit::JsoncSpan =
            walk.value().ok_or_else(|| return misaligned(label))?;
        last = Some((key_span.start.offset, value_span.end.offset));
        skip_children(walk, &entry.value, label)?;
    }
    let Some((last_key, last_value_end)) = last else {
        return Err(misaligned(label));
    };
    let member_indent: String =
        line_indent(source, last_key).unwrap_or_else(|| return indent_below(source, brace));
    return Ok(ToolsRecord {
        place: place_after(source, last_value_end, end),
        member_indent,
        closing_indent,
    });
}
