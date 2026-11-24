//! What:
//!  Reading one binding record's bytes into [`BindingRecord`],
//!  or refusing them by line and field.
//! Why:
//!  A record decides which dataset a destruction may target,
//!  so a file this module cannot read refuses rather than being skipped,
//!  and a field the record's phase has not proved refuses too:
//!  a phase ahead of the live facts is damage or another hand,
//!  never a lag the reader may guess past
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "A resumed add" and "What no reader may conclude").
//!  The reader checks size before it parses,
//!  then the header,
//!  then each line against the grammar's order,
//!  which catches an unknown key and a repeated one with one rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = parseBinding(bytes); // { ok: BindingRecord } | { refused: message }
//! ```

use super::binding_bound::{BINDING_MAXIMUM_BYTES, BINDING_MAXIMUM_LINE_BYTES};
use super::record_refusal::RecordRefusal;
use super::binding_encoding::path_as_hexadecimal;
use super::binding_field_set::Fields;
use super::binding_record::BindingRecord;
use super::binding_words::{BINDING_HEADER, BINDING_KEYS_IN_ORDER};


/// What:
///  Read one binding record's bytes.
/// Why:
///  This is the one reader every command uses,
///  so size,
///  header,
///  order and each field's encoding are decided once.
///
/// # Errors
/// A refusal naming the line or the field,
/// which the caller prints and acts on by changing nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = parseBinding(bytes); // BindingRecord or a refusal message
/// ```
pub fn parse_binding(bytes: &[u8]) -> Result<BindingRecord, RecordRefusal> {
    let length: u64 = bytes.len() as u64;
    if length > BINDING_MAXIMUM_BYTES {
        return Err(RecordRefusal::TooLarge { found: length, bound: BINDING_MAXIMUM_BYTES });
    }
    let text: &str = std::str::from_utf8(bytes).map_err(|_| return RecordRefusal::NotUtf8 { line: 1 })?;
    let mut lines = text.split_terminator('\n');
    let header: &str = lines.next().ok_or_else(|| return RecordRefusal::Header { found: String::new() })?;
    if header != BINDING_HEADER {
        return Err(RecordRefusal::Header { found: String::from(header) });
    }
    let mut fields: Fields = Fields::empty();
    let mut last_order: Option<usize> = None;
    let mut line_number: usize = 1;
    for line in lines {
        line_number += 1;
        let bytes_in_line: u64 = line.len() as u64 + 1;
        if bytes_in_line > BINDING_MAXIMUM_LINE_BYTES {
            return Err(RecordRefusal::LineTooLarge { line: line_number, found: bytes_in_line, bound: BINDING_MAXIMUM_LINE_BYTES });
        }
        let Some((key, value)): Option<(&str, &str)> = line.split_once('=') else {
            return Err(RecordRefusal::NotAField { line: line_number, found: String::from(line) });
        };
        let Some(order) = BINDING_KEYS_IN_ORDER.iter().position(|known| return known == &key) else {
            return Err(RecordRefusal::UnknownKey { line: line_number, key: String::from(key) });
        };
        if last_order.is_some_and(|seen| return order <= seen) {
            let expected: &str = BINDING_KEYS_IN_ORDER.get(last_order.unwrap_or(0) + 1).copied().unwrap_or("-");
            return Err(RecordRefusal::KeyOutOfOrder { line: line_number, key: String::from(key), expected: String::from(expected) });
        }
        last_order = Some(order);
        fields.set(key, value, line_number)?;
    }
    return fields.into_record();
}

/// A slot path as the record writes it,
/// for the tests and the writers that build one.
pub fn slot_hexadecimal(slot: &std::path::Path) -> String {
    return path_as_hexadecimal(slot);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "binding_parse_tests.rs"]
mod tests;
