//! Lossless source mapping for the rendered representation of one code line.

/// What:
///  Helix's borrowed rope view and its grapheme walk.
/// Why:
///  The shaping engine lets a pointer land between the characters of one multi-character cluster,
/// such as a letter and its combining mark;
///  the caret may only stand on the boundaries Helix reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RopeSlice, nextGraphemeBoundary } from 'helix-core';
/// ```
use helix_core::{Rope, RopeSlice, graphemes::next_grapheme_boundary};

/// What:
///  Longest line,
/// in bytes,
/// whose caret stops are found by walking the line as one flat string.
/// Why:
///  A string viewed as a rope slice is one chunk,
/// and finding a character in a chunk scans the chunk,
///  so a walk over a long line would scan it once for every grapheme:
/// work that grows with the square of the length.
///  A line past this length is copied into a rope,
/// whose chunks are about a kilobyte,
///  so each lookup scans one chunk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FLAT_LINE_BYTES = 1024;
/// ```
const FLAT_LINE_BYTES: usize = 1024;

/// What:
///  Own display text and maps between its bytes and source characters;
/// `Vec<usize>` is a growable list of address-sized indices (siblings:
///  `u32`,
///  `u64`).
/// Why:
///  A tab is shaped as one space whose advance the shaper widens to the next tab stop,
/// while copying and language requests keep the source tab and its offset.
/// `usize` is what string and rope indexing take,
///  so no conversion is needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Projection = { text: string; sourceToByte: number[]; byteToSource: number[]; tabs: number[] };
/// ```
#[derive(Clone)]
pub struct Projection {
    /// Text passed to the shaper,
    ///  with each tab replaced by one space and terminators excluded.
    pub text: String,
    /// Display byte position for each source character boundary.
    pub source_to_byte: Vec<usize>,
    /// Source character position for each display byte boundary.
    pub byte_to_source: Vec<usize>,
    /// Display byte position of the space standing in for each source tab,
    ///  in reading order.
    pub tabs: Vec<usize>,
    /// Source character positions a caret may take on this line,
    ///  in ascending order:
    /// the grapheme boundaries from the line start to the end of its visible text.
    pub stops: Vec<usize>,
}

/// Grapheme boundaries of `source` up to `visible` characters,
///  the line's text without its terminator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function caretStops(source: string, visible: number): number[];
/// ```
fn caret_stops(source: &str, visible: usize) -> Vec<usize> {
    return caret_stops_visiting(source, visible, IgnoreVisit);
}

/// What:
///  What a caller may do before each grapheme lookup of the walk.
///  A type that implements this trait receives the rope view and the position of the lookup.
/// Why:
///  The structural test counts the bytes each lookup scans,
///  and a named type carries the counter without an anonymous function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface StopVisitor { visit(slice: RopeSlice, position: number): void }
/// ```
trait StopVisitor {
    /// Look at the rope view and the position before the lookup.
    fn visit(&mut self, slice: RopeSlice<'_>, position: usize);
}

/// The visitor that looks at nothing,
///  for callers that only want the stops.
struct IgnoreVisit;

/// Looking at nothing is the whole job of this visitor.
impl StopVisitor for IgnoreVisit {
    /// Ignore the lookup.
    fn visit(&mut self, _slice: RopeSlice<'_>, _position: usize) {}
}

/// What:
///  The caret stops of `source`,
/// calling `visit` with the rope view and the position before each lookup.
///  The production caller passes a visitor that does nothing.
/// Why:
///  The structural test counts the bytes each lookup has to scan through this hook,
///  and so proves the walk's work is bounded without timing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function caretStopsVisiting(source: string, visible: number, visitor: StopVisitor): number[];
/// ```
fn caret_stops_visiting(source: &str, visible: usize, mut visitor: impl StopVisitor) -> Vec<usize> {
    // What: `RopeSlice::from` views the borrowed string as rope text without copying it; a long line is copied
    //       into a rope first, which keeps it in chunks.
    // Why: Helix's grapheme functions take rope views, and the same functions move the keyboard caret.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const slice = source.length <= FLAT_LINE_BYTES ? RopeSlice.from(source) : Rope.from(source).slice();
    // ```
    let rope;
    let slice = if source.len() <= FLAT_LINE_BYTES {
        RopeSlice::from(source)
    } else {
        rope = Rope::from_str(source);
        rope.slice(..)
    };
    let mut stops = vec![0];
    let mut position = 0;
    while position < visible {
        visitor.visit(slice, position);
        // A grapheme never spans the visible text and the terminator, so the walk ends exactly at `visible`.
        position = next_grapheme_boundary(slice, position).min(visible);
        stops.push(position);
    }
    return stops;
}

/// Build the projection in one pass;
///  input is one logical source line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function projectLine(source: string): Projection;
/// ```
pub fn project_line(source: &str) -> Projection {
    // Owned buffers grow linearly; no repeated prefix-string reconstruction.
    let mut text = String::new();
    let mut source_to_byte = vec![0];
    let mut byte_to_source = vec![0];
    let mut tabs = Vec::new();
    // enumerate supplies each scalar index without a separate mutable counter.
    for (source_index, character) in source.chars().enumerate() {
        if character == '\n' || character == '\r' {
            break;
        }
        if character == '\t' {
            // One display byte per tab: the shaper decides its pixel width, never a column count here.
            tabs.push(text.len());
            text.push(' ');
            byte_to_source.push(source_index + 1);
        } else {
            text.push(character);
            for index in 0..character.len_utf8() {
                // Bytes inside a multi-byte character belong to the character they are part of.
                if index + 1 == character.len_utf8() {
                    byte_to_source.push(source_index + 1);
                } else {
                    byte_to_source.push(source_index);
                }
            }
        }
        source_to_byte.push(text.len());
    }
    let stops = caret_stops(source, source_to_byte.len() - 1);
    return Projection {
        text,
        source_to_byte,
        byte_to_source,
        tabs,
        stops,
    };
}

/// Unit tests of the caret stops,
///  including the work of finding them,
///  live in a sidecar file.
#[cfg(test)]
#[path = "text_projection_tests.rs"]
mod tests;
