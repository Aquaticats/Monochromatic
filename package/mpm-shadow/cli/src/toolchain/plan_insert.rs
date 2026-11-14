//! What:
//! Appending plan entries to `toolchain.plan.jsonc` as text:
//! the entry each declaration writes,
//! the place in the file it goes,
//! and every other byte of the file kept as it was.
//! Why:
//! People write this file,
//! so a command that rewrote it whole would lose the comments that say why each tool is there
//! and the layout they chose;
//! the repository's JSONC crate parses the text and reports where each value sits,
//! and this module splices the new entry into exactly that place:
//! after the last entry of `tools`,
//! before its own trailing comma,
//! so comments,
//! trailing commas and the entry order stay byte for byte.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // plan-insert.ts
//! export function planWithEntries(source: string, label: string, entries: Declaration[]): string  // throws on a broken plan
//! ```

/// Imports the JSON value types the entry is built from.
use serde_json::{Map, Value};

/// Imports the span-reporting parser and the tree it hands back beside the positions.
use monochromatic_jsonc_edit::{JsoncKind, JsoncSpanned, JsoncValue, parse_jsonc_with_spans, units_to_string};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports the declaration type and the entry encoder.
use super::model::Declaration;
use super::model_encode::{added_plan_entry_value, render_pretty_json};
/// Imports `parse_plan` from `super::plan`.
use super::plan::parse_plan;

/// The indentation the toolchain plan of this repository uses for one nesting level.
const ONE_LEVEL: &str = "  ";

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
struct ToolsRecord {
    /// The byte offset the new text goes in at:
    /// the end of the last member's value,
    /// or just after the `{` of an empty object.
    insert_at: usize,
    /// The indentation one entry of this object gets,
    /// taken from the entry that is there already.
    member_indent: String,
    /// The indentation of the line that holds the `{` of the object,
    /// written in front of the closing `}` of an empty object.
    closing_indent: String,
    /// Whether the object holds no entry yet.
    empty: bool,
    /// Whether an empty object's braces already sit on separate lines.
    empty_is_broken: bool,
}

/// A reader of the parser's two span lists,
/// in the order the parser recorded them.
struct SpanWalk<'a> {
    /// One span per value,
    /// the root first.
    values: &'a [monochromatic_jsonc_edit::JsoncSpan],
    /// One span per object member key,
    /// in source order.
    keys: &'a [monochromatic_jsonc_edit::JsoncSpan],
    /// The next value span to read.
    value_index: usize,
    /// The next key span to read.
    key_index: usize,
}

/// The methods of `SpanWalk`.
impl<'a> SpanWalk<'a> {
    /// Reads the next value span.
    fn value(&mut self) -> Option<monochromatic_jsonc_edit::JsoncSpan> {
        let span: monochromatic_jsonc_edit::JsoncSpan = *self.values.get(self.value_index)?;
        self.value_index += 1;
        return Some(span);
    }

    /// Reads the next key span.
    fn key(&mut self) -> Option<monochromatic_jsonc_edit::JsoncSpan> {
        let span: monochromatic_jsonc_edit::JsoncSpan = *self.keys.get(self.key_index)?;
        self.key_index += 1;
        return Some(span);
    }
}

/// The internal error for a span list that does not line up with the tree it came with:
/// the crate's own invariant,
/// so a person reading it knows the file was not written.
fn misaligned(label: &str) -> ToolchainError {
    return ToolchainError::config(format!(
        "{label}: the parser's positions do not line up with the values it returned, so no entry was added and the file is as it was"
    ));
}

/// Consumes the spans of the children of `value`,
/// whose own span the caller already read.
fn skip_children(walk: &mut SpanWalk, value: &JsoncValue, label: &str) -> Result<(), ToolchainError> {
    match &value.kind {
        JsoncKind::Record { entries } => {
            for entry in entries {
                walk.key().ok_or_else(|| return misaligned(label))?;
                walk.value().ok_or_else(|| return misaligned(label))?;
                skip_children(walk, &entry.value, label)?;
            }
        }
        JsoncKind::Array { elements } => {
            for element in elements {
                walk.value().ok_or_else(|| return misaligned(label))?;
                skip_children(walk, element, label)?;
            }
        }
        JsoncKind::Null | JsoncKind::Boolean { .. } | JsoncKind::Number { .. } | JsoncKind::Text { .. } => {}
    }
    return Ok(());
}

/// Consumes the spans of `value` and of everything below it.
fn skip_value(walk: &mut SpanWalk, value: &JsoncValue, label: &str) -> Result<(), ToolchainError> {
    walk.value().ok_or_else(|| return misaligned(label))?;
    return skip_children(walk, value, label);
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
fn line_indent(source: &str, offset: usize) -> Option<String> {
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
fn line_leading(source: &str, offset: usize) -> String {
    let start: usize = match source[..offset].rfind('\n') {
        Some(newline) => newline + 1,
        None => 0,
    };
    let line: &str = &source[start..];
    let leading: &str = &line[..line.len() - line.trim_start().len()];
    return leading.to_string();
}

/// The indentation of one entry in the object whose `{` is at `brace`, when no entry is there to copy it from.
fn indent_below(source: &str, brace: usize) -> String {
    return format!("{}{ONE_LEVEL}", line_leading(source, brace));
}

/// The text of one entry of `tools`:
/// the id as the key and the fields `added_plan_entry_value` writes,
/// expanded the way the plans are written,
/// with `indent` in front of every line.
fn entry_text(indent: &str, declaration: &Declaration) -> String {
    let mut members: Map<String, Value> = Map::new();
    members.insert(declaration.id.clone(), added_plan_entry_value(declaration));
    let rendered: String = render_pretty_json(&Value::Object(members));
    let lines: Vec<&str> = rendered.lines().collect();
    // The first and last lines are the braces of the object that holds the one member;
    // the plan writes its own, so only the member's own lines are kept.
    let mut text: String = String::new();
    for (index, line) in lines.iter().skip(1).take(lines.len().saturating_sub(2)).enumerate() {
        if index > 0 {
            text.push('\n');
        }
        text.push_str(indent);
        text.push_str(line.strip_prefix(ONE_LEVEL).unwrap_or(line));
    }
    return text;
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
fn tools_record(source: &str, label: &str) -> Result<ToolsRecord, ToolchainError> {
    let spanned: JsoncSpanned = match parse_jsonc_with_spans(source) {
        Ok(parsed) => parsed,
        Err(error) => return Err(ToolchainError::config(format!("{label} is not valid JSONC: {error}"))),
    };
    let root: &JsoncValue = &spanned.value;
    let JsoncKind::Record { entries } = &root.kind else {
        return Err(ToolchainError::config(format!("{label} must hold a JSON object at its top level")));
    };
    let mut walk: SpanWalk = SpanWalk { values: &spanned.spans.values, keys: &spanned.spans.keys, value_index: 0, key_index: 0 };
    walk.value().ok_or_else(|| return misaligned(label))?;
    for entry in entries {
        walk.key().ok_or_else(|| return misaligned(label))?;
        let key: String = match units_to_string(&entry.key.units) {
            Ok(text) => text,
            Err(error) => return Err(ToolchainError::config(format!("{label}: a key is not valid text: {error}"))),
        };
        if key != "tools" {
            skip_value(&mut walk, &entry.value, label)?;
            continue;
        }
        let brace_span: monochromatic_jsonc_edit::JsoncSpan = walk.value().ok_or_else(|| return misaligned(label))?;
        return tools_of(source, label, brace_span.start.offset, brace_span.end.offset, &entry.value, &mut walk);
    }
    return Err(ToolchainError::config(format!("{label} has no \"tools\" object")));
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
    if source.as_bytes().get(brace) != Some(&b'{') || end == 0 || source.as_bytes().get(end - 1) != Some(&b'}') {
        return Err(misaligned(label));
    }
    let closing_indent: String = line_leading(source, brace);
    let JsoncKind::Record { entries } = &value.kind else {
        return Err(ToolchainError::config(format!("{label}: \"tools\" must be an object keyed by tool id")));
    };
    if entries.is_empty() {
        return Ok(ToolsRecord {
            insert_at: brace + 1,
            member_indent: format!("{closing_indent}{ONE_LEVEL}"),
            closing_indent,
            empty: true,
            empty_is_broken: source[brace + 1..end - 1].contains('\n'),
        });
    }
    let mut last: Option<(usize, usize)> = None;
    for entry in entries {
        let key_span: monochromatic_jsonc_edit::JsoncSpan = walk.key().ok_or_else(|| return misaligned(label))?;
        let value_span: monochromatic_jsonc_edit::JsoncSpan = walk.value().ok_or_else(|| return misaligned(label))?;
        last = Some((key_span.start.offset, value_span.end.offset));
        skip_children(walk, &entry.value, label)?;
    }
    let Some((last_key, last_value_end)) = last else {
        return Err(misaligned(label));
    };
    let member_indent: String = line_indent(source, last_key).unwrap_or_else(|| return indent_below(source, brace));
    return Ok(ToolsRecord { insert_at: last_value_end, member_indent, closing_indent, empty: false, empty_is_broken: false });
}

/// What:
/// The plan's text with `declarations` appended to its `tools` object,
/// in the order given,
/// and every other byte as it was.
/// Why:
/// This is the one write into a file people edit,
/// so it is checked before the caller publishes it:
/// the text it returns must parse,
/// and a plan that would not read back refuses here rather than in the next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planWithEntries(source: string, label: string, declarations: Declaration[]): string
/// ```
pub fn plan_with_entries(source: &str, label: &str, declarations: &[Declaration]) -> Result<String, ToolchainError> {
    if declarations.is_empty() {
        return Ok(source.to_string());
    }
    let record: ToolsRecord = tools_record(source, label)?;
    let mut entries: Vec<String> = Vec::new();
    for declaration in declarations {
        entries.push(entry_text(&record.member_indent, declaration));
    }
    let joined: String = entries.join(",\n");
    let insertion: String = if record.empty && record.empty_is_broken {
        // The newline and the closing brace the person wrote already follow; only the entry is added.
        format!("\n{joined}")
    } else if record.empty {
        format!("\n{joined}\n{}", record.closing_indent)
    } else {
        // The comma the last entry already carries (or does not) stays where it was, after the new text.
        format!(",\n{joined}")
    };
    let mut text: String = String::with_capacity(source.len() + insertion.len());
    text.push_str(&source[..record.insert_at]);
    text.push_str(&insertion);
    text.push_str(&source[record.insert_at..]);
    let ids: Vec<&str> = declarations.iter().map(|declaration| return declaration.id.as_str()).collect();
    if let Err(error) = parse_plan(&text, label) {
        return Err(ToolchainError::config(format!(
            "{label}: the entry for {} cannot be added: {}",
            ids.join(", "),
            error.message
        )));
    }
    return Ok(text);
}
