//! What:
//!  Remove trailing periods/colons from a heading's final text node.
//! Why:
//!  The edit must preserve inline markup and consume complete source escapes/entities,
//!  not only decoded character counts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Find the decoded punctuation suffix, then map each suffix character back to its authored bytes.
//! ```

/// Import the existing diagnostic and localized edit boundaries.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the edit type that a punctuation repair is written as.
use crate::edits::Edit;
/// Import the finding constructors for punctuation problems and unreadable documents.
use crate::markdown_finding::{finding, structure_failure};
/// Import the parsed Markdown source whose punctuation is inspected.
use crate::markdown_source::MarkdownSource;
/// Import native text decoding and heading kinds.
use satteri_ast::mdast::{MdastNodeType, decode_string_ref_data};

/// Count only the configured ASCII punctuation suffix.
fn trailing_count(value: &str) -> usize {
    let mut count: usize = 0;
    for character in value.chars().rev() {
        if character != '.' && character != ':' {
            break;
        }
        count += 1;
    }
    return count;
}

/// Decode only entity spellings for the punctuation this rule removes.
fn punctuation_entity(entity: &str) -> bool {
    if entity == "period" || entity == "colon" {
        return true;
    }
    let parsed: Result<u32, std::num::ParseIntError>;
    if let Some(hex) = entity.strip_prefix("#x") {
        parsed = u32::from_str_radix(hex, 16);
    } else if let Some(hex) = entity.strip_prefix("#X") {
        parsed = u32::from_str_radix(hex, 16);
    } else if let Some(decimal) = entity.strip_prefix('#') {
        parsed = decimal.parse::<u32>();
    } else {
        return false;
    }
    if let Ok(value) = parsed {
        return value == 46 || value == 58;
    }
    return false;
}

/// Locate the authored suffix without leaving a dangling escape or half an entity.
/// Each successful step consumes the suffix it scanned;
///  the first failed scan returns immediately.
/// Therefore source bytes are scanned a bounded number of times,
///  including backslash runs.
fn suffix_start(written: &str, count: usize) -> Option<usize> {
    let bytes: &[u8] = written.as_bytes();
    let mut end: usize = bytes.len();
    for _ in 0..count {
        let last: u8 = *bytes.get(end.checked_sub(1)?)?;
        if last == b'.' || last == b':' {
            let mut start: usize = end - 1;
            // Each backslash directly before the punctuation flips whether that punctuation is escaped.
            // A flag, not a run length, holds the parity; walking backwards stops at the text's first byte by itself.
            let mut escaped: bool = false;
            // What: `bytes[..start].iter().rev()` borrows the bytes before the punctuation and visits them
            // from the nearest one backwards; `&u8` items are read through `*byte`.
            // Why: The reversed walk has no hand-stepped index, so no mutation of a step can make it spin.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // for (const byte of bytes.slice(0, start).reverse()) { if (byte !== BACKSLASH) break; escaped = !escaped; }
            // ```
            for byte in bytes[..start].iter().rev() {
                if *byte != b'\\' {
                    break;
                }
                escaped = !escaped;
            }
            // An escaped punctuation character is spelled with its backslash, which leaves with it.
            if escaped {
                start -= 1;
            }
            end = start;
            continue;
        }
        if last != b';' {
            return None;
        }
        let start: usize = written[..end].rfind('&')?;
        if !punctuation_entity(&written[start + 1..end - 1]) {
            return None;
        }
        end = start;
    }
    return Some(end);
}

/// What:
///  Build one optional deletion after finding its complete authored suffix.
/// Why:
///  Unmappable spellings keep their diagnostic but never receive a guessed edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function punctuationEdit(written, count, start, end): Edit | undefined;
/// ```
fn punctuation_edit(written: &str, count: usize, start: usize, end: usize) -> Option<Edit> {
    // What: The question mark returns None immediately when the suffix has no verified mapping.
    // Why: Successful mappings can construct their edit without a capturing callback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const relative = suffixStart(written, count); if (relative === undefined) return undefined;
    // ```
    let relative: usize = suffix_start(written, count)?;
    // Return an owned edit only for a complete mapping; the empty string requests deletion.
    return Some(Edit {
        start: start + relative,
        end,
        replacement: String::new(),
    });
}

/// Report heading punctuation while keeping the original heading and inline delimiters intact.
pub fn no_trailing_punctuation(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Heading {
            continue;
        }
        // What: `match` unwraps `Ok(nodes)`, the heading's text nodes from the bounded descendant walk, or
        // handles `Err(error)`, which means the document's child index has a cycle.
        // Why: A heading whose text nodes cannot be listed was not checked, so the rule reports one
        // processing failure that names it and stops.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let textNodes: number[];
        // try { textNodes = context.textNodes(id); }
        // catch (error) { findings.push(structureFailure(context, rule, error)); return findings; }
        // ```
        let text_nodes: Vec<u32> = match context.text_nodes(*id) {
            Ok(nodes) => nodes,
            Err(error) => {
                findings.push(structure_failure(
                    context,
                    "markdown/no-trailing-punctuation",
                    error,
                ));
                return findings;
            }
        };
        let Some(last): Option<&u32> = text_nodes.last() else {
            continue;
        };
        let reference = decode_string_ref_data(context.data(*last));
        let count: usize = trailing_count(context.text(reference));
        if count == 0 {
            continue;
        }
        let (start, end): (usize, usize) = context.offsets(*last);
        let written: &str = context.slice(*last);
        let edit: Option<Edit> = punctuation_edit(written, count, start, end);
        findings.push(finding(
            context,
            *id,
            "markdown/no-trailing-punctuation",
            severity,
            String::from("Heading ends with punctuation; remove the trailing punctuation."),
            edit,
        ));
    }
    return findings;
}

/// Source-spelling controls are not included in release artifacts.
#[cfg(test)]
#[path = "markdown_punctuation_tests.rs"]
mod tests;
