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
use serde_json::{
    Map,
    Value,
};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports the declaration type the entry is built from.
use super::model::Declaration;
/// Imports the encoder of one plan entry and the renderer the entry's lines come from.
use super::model_encode::{
    added_plan_entry_value,
    render_pretty_json,
};
/// Imports `parse_plan` from `super::plan`.
use super::plan::parse_plan;
/// Imports what the parent module uses from `super::plan_insert_place`.
use super::plan_insert_place::{
    ONE_LEVEL,
    Place,
    ToolsRecord,
    tools_record,
};

/// The text of one entry of `tools`:
/// the id as the key and the fields `added_plan_entry_value` writes,
/// expanded the way the plans are written,
/// with `indent` in front of every line.
fn entry_text(
    indent: &str,
    declaration: &Declaration,
) -> String {
    let mut members: Map<String, Value> = Map::new();
    members.insert(declaration.id.clone(), added_plan_entry_value(declaration));
    let rendered: String = render_pretty_json(&Value::Object(members));
    let lines: Vec<&str> = rendered.lines().collect();
    // The first and last lines are the braces of the object that holds the one member;
    // the plan writes its own, so only the member's own lines are kept.
    let mut text: String = String::new();
    for (index, line) in lines
        .iter()
        .skip(1)
        .take(lines.len().saturating_sub(2))
        .enumerate()
    {
        if index > 0 {
            text.push('\n');
        }
        text.push_str(indent);
        text.push_str(line.strip_prefix(ONE_LEVEL).unwrap_or(line));
    }
    return text;
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
pub fn plan_with_entries(
    source: &str,
    label: &str,
    declarations: &[Declaration],
) -> Result<String, ToolchainError> {
    if declarations.is_empty() {
        return Ok(source.to_string());
    }
    let record: ToolsRecord = tools_record(source, label)?;
    let mut entries: Vec<String> = Vec::new();
    for declaration in declarations {
        entries.push(entry_text(&record.member_indent, declaration));
    }
    let joined: String = entries.join(",\n");
    let insertions: Vec<(usize, String)> = match record.place {
        Place::AtLastValue {
            value_end,
        } => vec![(value_end, format!(",\n{joined}"))],
        Place::AfterComment {
            value_end,
            comma_present,
            after_comment,
        } => {
            let mut parts: Vec<(usize, String)> = Vec::new();
            if !comma_present {
                parts.push((value_end, ",".to_string()));
            }
            parts.push((after_comment, format!("\n{joined}")));
            parts
        }
        Place::Empty {
            brace,
            braces_broken: true,
        } => {
            // The newline and the closing brace the person wrote already follow; only the entry is added.
            vec![(brace, format!("\n{joined}"))]
        }
        Place::Empty {
            brace,
            braces_broken: false,
        } => {
            vec![(brace, format!("\n{joined}\n{}", record.closing_indent))]
        }
    };
    let mut text: String = source.to_string();
    // From the last edit to the first, so an earlier offset still names the byte it was read at.
    let mut ordered: Vec<(usize, String)> = insertions;
    ordered.sort_by_key(|(offset, _)| return std::cmp::Reverse(*offset));
    for (offset, part) in ordered {
        text.insert_str(offset, &part);
    }
    let ids: Vec<&str> = declarations
        .iter()
        .map(|declaration| return declaration.id.as_str())
        .collect();
    if let Err(error) = parse_plan(&text, label) {
        return Err(ToolchainError::config(format!(
            "{label}: the entry for {} cannot be added: {}",
            ids.join(", "),
            error.message
        )));
    }
    return Ok(text);
}
