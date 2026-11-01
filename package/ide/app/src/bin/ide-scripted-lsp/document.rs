//! The server's own copy of each open document,
//!  edited by the client's change notifications.

/// Positions are interpreted in the unit the server announced.
use crate::script::Unit;
/// What:
///  `Value` is any JSON value.
/// Why:
///  Change events are read straight from the notification's parameters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::Value;

/// What:
///  Width of one character in the given column unit.
///  `char` is one Unicode scalar value;
///       `usize` is the address-sized unsigned integer (siblings:
///  `u32`,
///  `u64`).
/// Why:
///  The same character advances the column by a different amount in each unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const width = (character: string, unit: Unit) =>
///   unit === 'utf-8' ? Buffer.byteLength(character) : unit === 'utf-16' ? character.length : 1;
/// ```
fn width(character: char, unit: Unit) -> usize {
    return match unit {
        Unit::Utf8 => character.len_utf8(),
        Unit::Utf16 => character.len_utf16(),
        Unit::Utf32 => 1,
    };
}

/// What:
///  Convert a protocol position into a byte offset of `text`.
///  `&str` borrows the text;
///       `u64` holds the protocol's unsigned line and column numbers.
/// Why:
///  Rust strings are indexed by byte,
///  while the client sends columns in the announced unit.
///      Lines are split at line feeds only;
///  a carriage return stays part of its line,
///  and the
///      client never sends a column past it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function offsetAt(text: string, line: number, character: number, unit: Unit): number
/// ```
pub fn offset_at(text: &str, line: u64, character: u64, unit: Unit) -> usize {
    // `mut` allows the two counters to advance while walking the text.
    let mut offset = 0;
    let mut current_line = 0;
    while current_line < line {
        // What: `text[offset..]` borrows the text from `offset` to the end; `find` returns
        //       `Option<usize>`, the position of the next line feed if there is one.
        // Why: A line number past the end means the end of the text.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const next = text.indexOf('\n', offset); if (next < 0) return text.length;
        // ```
        let Some(next) = text[offset..].find('\n') else {
            return text.len();
        };
        offset += next + 1;
        current_line += 1;
    }
    let mut column: u64 = 0;
    // `chars()` walks the remaining text one Unicode scalar value at a time.
    for character_here in text[offset..].chars() {
        if character_here == '\n' || column >= character {
            break;
        }
        // What: `as u64` widens the address-sized width into the protocol's integer type.
        // Why: The comparison above needs both numbers in the same type; a width is at most 4.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // column += width(character, unit);
        // ```
        column += width(character_here, unit) as u64;
        offset += character_here.len_utf8();
    }
    return offset;
}

/// Read one coordinate of a position object,
///  treating a missing number as zero.
fn coordinate(position: &Value, name: &str) -> u64 {
    // `as_u64` returns `Option<u64>`; `unwrap_or(0)` substitutes zero for a missing or wrong field.
    return position[name].as_u64().unwrap_or(0);
}

/// What:
///  Apply one `contentChanges` entry to the stored text.
///  `&mut String` lends the text for
///       modification.
/// Why:
///  An entry without a range replaces everything (full synchronization);
///  one with a range
///      replaces that range (incremental synchronization).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyChange(text: string, change: ContentChange, unit: Unit): string {
///   if (!change.range) return change.text;
///   return text.slice(0, offsetAt(text, change.range.start)) + change.text
///        + text.slice(offsetAt(text, change.range.end));
/// }
/// ```
pub fn apply_change(text: &mut String, change: &Value, unit: Unit) {
    let replacement = change["text"].as_str().unwrap_or("");
    let range = &change["range"];
    if range.is_null() {
        // `to_string` copies the borrowed replacement into owned storage.
        *text = replacement.to_string();
        return;
    }
    let start = offset_at(
        text,
        coordinate(&range["start"], "line"),
        coordinate(&range["start"], "character"),
        unit,
    );
    let end = offset_at(
        text,
        coordinate(&range["end"], "line"),
        coordinate(&range["end"], "character"),
        unit,
    );
    // `replace_range` swaps the bytes between the two offsets for the replacement text.
    text.replace_range(start..end.max(start), replacement);
}

/// The line containing `offset`,
///  without its line break.
pub fn line_at(text: &str, offset: usize) -> &str {
    // What: `rfind` searches backwards and `find` forwards; both return `Option<usize>`.
    //       `map_or(a, f)` yields `a` for "nothing" and `f(value)` otherwise.
    // Why: The hover answer quotes the line so a test can see which text the server holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const start = text.lastIndexOf('\n', offset - 1) + 1;
    // ```
    let start = match text[..offset].rfind('\n') {
        Some(found) => found + 1,
        None => 0,
    };
    let end = match text[offset..].find('\n') {
        Some(found) => offset + found,
        None => text.len(),
    };
    return &text[start..end];
}
