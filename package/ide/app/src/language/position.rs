//! The one place that converts between Helix character offsets and language-server positions.

/// What:
///  `Rope` is Helix's text buffer,
///  indexed by Unicode scalar values ("characters"),
///       not by bytes or UTF-16 units.
/// Why:
///  Every offset the application stores is a character index into this type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// What:
///  `OffsetEncoding` names the column unit one server negotiated (bytes,
///  UTF-16 units,
///  or
///       characters);
///  `lsp` holds the protocol's data types;
///  the two `util` functions are Helix's
///       own converters.
/// Why:
///  Reusing Helix's converters keeps the column arithmetic identical to the client that
///      sends the document text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type OffsetEncoding, lsp, lspPosToPos, posToLspPos } from 'helix-lsp';
/// ```
use helix_lsp::{
    OffsetEncoding, lsp,
    util::{lsp_pos_to_pos, pos_to_lsp_pos},
};

/// What:
///  Convert a character offset into the line and column a server expects.
///  `&Rope` lends the
///       text read-only;
///  `usize` is the address-sized unsigned index Helix uses (siblings:
///  `u32`,
///       `u64`);
///  `Option<...>` is "a value or nothing",
///  Rust's replacement for `T | undefined`.
/// Why:
///  Helix's converter panics for an offset past the end of the text,
///  so the bound is checked
///      here and reported as "no position" instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toLspPosition(text: Rope, position: number, encoding: Encoding): Position | undefined {
///   if (position > text.length) return undefined;
///   return posToLspPos(text, position, encoding);
/// }
/// ```
pub fn to_lsp_position(
    text: &Rope,
    position: usize,
    encoding: OffsetEncoding,
) -> Option<lsp::Position> {
    if position > text.len_chars() {
        // What: `None` is the "nothing" variant of `Option`.
        // Why: A stale offset from an older revision must not crash the worker thread.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return undefined;
        // ```
        return None;
    }
    // What: `Some(...)` wraps the converted position in the "value present" variant.
    // Why: The caller sends exactly this position to the server.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return posToLspPos(text, position, encoding);
    // ```
    return Some(pos_to_lsp_pos(text, position, encoding));
}

/// Convert a character range,
///  in either direction,
///  into an ordered server range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toLspRange(text: Rope, start: number, end: number, encoding: Encoding): Range | undefined
/// ```
pub fn to_lsp_range(
    text: &Rope,
    start: usize,
    end: usize,
    encoding: OffsetEncoding,
) -> Option<lsp::Range> {
    // What: The trailing `?` returns `None` from this function at once when the call produced `None`.
    // Why: One out-of-bounds end makes the whole range unusable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first = toLspPosition(text, Math.min(start, end), encoding);
    // if (first === undefined) return undefined;
    // ```
    let first = to_lsp_position(text, start.min(end), encoding)?;
    let last = to_lsp_position(text, start.max(end), encoding)?;
    return Some(lsp::Range::new(first, last));
}

/// What:
///  Convert a server position into a character offset of `text`,
///  or nothing when the
///       position names a line the text does not have.
/// Why:
///  Helix's converter maps a line past the end to the end of the text;
///  anything the
///      application draws or navigates to must instead be rejected,
///  because such a position
///      belongs to another revision of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fromLspPosition(text: Rope, position: Position, encoding: Encoding): number | undefined {
///   if (position.line >= text.lineCount) return undefined;
///   return lspPosToPos(text, position, encoding);
/// }
/// ```
pub fn from_lsp_position(
    text: &Rope,
    position: lsp::Position,
    encoding: OffsetEncoding,
) -> Option<usize> {
    // What: `usize::try_from` converts the protocol's `u32` line number and returns a `Result`;
    //       `.ok()` turns that into an `Option`, and `?` returns `None` when it failed.
    // Why: The comparison with the line count needs both numbers in the same integer type.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const line: number = position.line;
    // ```
    let line = usize::try_from(position.line).ok()?;
    if line >= text.len_lines() {
        return None;
    }
    return lsp_pos_to_pos(text, position, encoding);
}

/// What:
///  Convert a server range into character offsets `(start, end)`.
///  The parentheses form a
///       tuple,
///  a fixed pair of values.
/// Why:
///  The start must be on an existing line;
///  an end past the last line means "through the end
///      of the text",
///  which the protocol uses to include a final line break.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fromLspRange(text: Rope, range: Range, encoding: Encoding): [number, number] | undefined
/// ```
pub fn from_lsp_range(
    text: &Rope,
    range: lsp::Range,
    encoding: OffsetEncoding,
) -> Option<(usize, usize)> {
    let start = from_lsp_position(text, range.start, encoding)?;
    // What: `match` picks the first arm whose pattern fits; here it unpacks the `Option`.
    // Why: A missing end line is clamped by this function, so Helix's own clamp (which logs a
    //      warning for every call) is never reached.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const end = fromLspPosition(text, range.end, encoding) ?? text.length;
    // ```
    let end = match from_lsp_position(text, range.end, encoding) {
        Some(found) => found,
        None => text.len_chars(),
    };
    // Servers that send an end before the start mean an empty range at the start.
    return Some((start, end.max(start)));
}

/// Encodings,
///  line endings,
///  combining marks,
///  and astral characters exercise every column unit.
#[cfg(test)]
#[path = "position_tests.rs"]
mod tests;
