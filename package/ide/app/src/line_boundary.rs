//! Home and End on the visual rows of a soft-wrapped line.
//!
//! The rules are IntelliJ Community's `EditorActionUtil.moveCaretToLineStart` and `moveCaretToLineEnd`
//!  with smart Home on (`platform/platform-impl`,
//! commit `d0936c36705bdaf87deca0f157a40c651345600c`).
//! Home goes to the start of the visual row;
//!  at the start of a continuation row it goes to the first non-blank character of the logical line's first row;
//!  on the first row it alternates between the line start and the first non-blank character.
//! End goes to the end of the visual row,
//! leaving trailing blanks out;
//!  a caret already there goes to the end of the logical line.
//! The rows are given as ranges of source characters,
//! so the rules need no shaping and a test can feed them directly.
//! Differences from IntelliJ:
//! the caret cannot stand in the indent of a continuation row or beyond the end of a line
//!  (there is no virtual space),
//! so the step into the indent and the smart-indent column of a blank line are left out.

/// Canonical source and the caret whose row the keys read.
use crate::document::{Document, ReadingAffinity};
/// The same shaper that paints the viewport breaks the rows.
use crate::shaped_text::TextShaper;
/// The rope view the rules read characters from.
use helix_core::RopeSlice;
/// Half-open ranges of source characters.
use std::ops::Range;

/// What:
///  A caret place:
/// the character position and the side of a wrap offset it is drawn on;
///  `usize` is the address-sized index ropes use (siblings `u32`,
/// `u64`).
/// Why:
///  A result at the end of an upper row and one at the start of the lower row are the same character position,
///  and only the side tells them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CaretPlace = { head: number; affinity: 'upstream' | 'downstream' };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaretPlace {
    /// Character position of the caret.
    pub head: usize,
    /// Side of the wrap offset when `head` is where a visual row breaks.
    pub affinity: ReadingAffinity,
}

/// A place on the default side of a wrap offset.
fn downstream(head: usize) -> CaretPlace {
    return CaretPlace {
        head,
        affinity: ReadingAffinity::Downstream,
    };
}

/// What:
///  Space,
/// tab,
/// and the line terminators:
/// the characters IntelliJ's `findFirstNonSpaceOffsetInRange` passes over.
/// Why:
///  Home looks for the first character that is none of them;
/// a row never holds a terminator,
/// so only blanks count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isBlank = (c: string) => c === ' ' || c === '\t';
/// ```
fn is_blank(character: char) -> bool {
    return character == ' ' || character == '\t' || character == '\n' || character == '\r';
}

/// What:
///  Index of the visual row that holds the caret at `head` on the side `affinity`.
///  `rows` are the ranges of all visual rows of one logical line,
/// in order,
/// ending at the end of its visible text.
/// Why:
///  A wrap offset ends the upper row and starts the lower one;
/// `Upstream` picks the upper row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rowContaining(rows: [number, number][], head: number, affinity: Affinity): number;
/// ```
pub fn row_containing(rows: &[Range<usize>], head: usize, affinity: ReadingAffinity) -> usize {
    let mut found = 0;
    for (index, row) in rows.iter().enumerate() {
        if row.start <= head {
            found = index;
        }
    }
    if affinity == ReadingAffinity::Upstream && found > 0 && rows[found].start == head {
        found -= 1;
    }
    return found;
}

/// Position of the first character of `row` that is not a blank,
/// or nothing when the row is all blanks.
fn first_non_blank(text: RopeSlice, row: &Range<usize>) -> Option<usize> {
    for index in row.start..row.end {
        if !is_blank(text.char(index)) {
            return Some(index);
        }
    }
    return None;
}

/// What:
///  Where Home moves the caret at `head` on `affinity`.
///  `text` is the document's rope;
///  `rows` are the ranges of the visual rows of the caret's logical line.
/// Why:
///  Past the start of a visual row,
/// Home goes to that start.
///  At the start of a continuation row it goes to the first non-blank character of the line's first row.
///  On the first row,
/// a caret at the line start goes to the first non-blank character,
///  one after that character or inside the indent goes to the line start,
///  and one further right goes to the first non-blank character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function home(text: RopeSlice, rows: [number, number][], head: number, affinity: Affinity): CaretPlace;
/// ```
pub fn home(
    text: RopeSlice,
    rows: &[Range<usize>],
    head: usize,
    affinity: ReadingAffinity,
) -> CaretPlace {
    let row = row_containing(rows, head, affinity);
    let line_start = rows[0].start;
    let indent_end = first_non_blank(text, &rows[0]);
    if row > 0 {
        if head > rows[row].start {
            return downstream(rows[row].start);
        }
        return downstream(indent_end.unwrap_or(line_start));
    }
    if head == line_start {
        return downstream(indent_end.unwrap_or(line_start));
    }
    return match indent_end {
        Some(indent) if indent < head => downstream(indent),
        _ => downstream(line_start),
    };
}

/// What:
///  Where End moves the caret at `head` on `affinity`,
/// under the same arguments as [`home`].
/// Why:
///  A caret that is not at the end of its visual row goes to that end,
/// trailing blanks left out,
///  and a caret at the blank-free end goes to the end including the blanks.
///  A caret at the end of a visual row that is not the last goes to the end of the logical line instead,
///  again left of its trailing blanks first.
///  The end of an upper row is drawn on that row,
/// so it is `Upstream`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function end(text: RopeSlice, rows: [number, number][], head: number, affinity: Affinity): CaretPlace;
/// ```
pub fn end(
    text: RopeSlice,
    rows: &[Range<usize>],
    head: usize,
    affinity: ReadingAffinity,
) -> CaretPlace {
    let last = rows.len() - 1;
    let mut row = row_containing(rows, head, affinity);
    // A caret already at the end of its visual row asks for the end of the logical line.
    if head == rows[row].end && row < last {
        row = last;
    }
    let offset = rows[row].end;
    let edge = CaretPlace {
        head: offset,
        affinity: if row < last {
            ReadingAffinity::Upstream
        } else {
            ReadingAffinity::Downstream
        },
    };
    let line_start = rows[0].start;
    // What: Walk left over the blanks that end the row; a row that starts inside them keeps its whole end.
    // Why: IntelliJ cancels the walk at a soft-wrap offset, so a row of blanks only is not trimmed to nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let i = offset - 1; i >= lineStart; i--) { if (startsRow(i)) { trimmed = offset; break; } ... }
    // ```
    let mut trimmed = offset;
    let mut index = offset;
    while index > line_start {
        index -= 1;
        // A row other than the first that starts here cancels the walk.
        let mut starts_row = false;
        for other in rows.iter().skip(1) {
            if other.start == index {
                starts_row = true;
                break;
            }
        }
        if starts_row {
            trimmed = offset;
            break;
        }
        if text.char(index) != ' ' && text.char(index) != '\t' {
            break;
        }
        trimmed = index;
    }
    if trimmed == offset || trimmed == head {
        return edge;
    }
    return downstream(trimmed);
}

/// What:
///  The two keys this module answers;
///  `enum` lists the allowed values as named variants.
/// Why:
///  The key handler names the key once and the same function serves the plain and the Shift form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Edge = 'home' | 'end';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// Home.
    Home,
    /// End.
    End,
}

/// What:
///  Where `edge` moves the document's caret,
/// with the line shaped at the wrap width `width`.
///  A width that is not positive means the line is not wrapped.
/// Why:
///  The visual rows of the caret's line are known only to the shaper,
/// and the line may be outside the
///  materialized rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveToEdge(shaper: TextShaper, document: Document, scale: number, width: number, edge: Edge): CaretPlace;
/// ```
pub fn moved_to_edge(
    shaper: &mut TextShaper,
    document: &Document,
    scale: f32,
    width: f32,
    edge: Edge,
) -> CaretPlace {
    let position = document.position();
    let text = document.text();
    let line = text.char_to_line(position.head.min(text.len_chars()));
    let rows = shaper
        .wrapped_row(document, line, scale, width)
        .source_rows();
    let place = if edge == Edge::Home {
        home(text.slice(..), &rows, position.head, position.affinity)
    } else {
        end(text.slice(..), &rows, position.head, position.affinity)
    };
    tracing::debug!(
        ?edge,
        line,
        rows = rows.len(),
        from = position.head,
        to = place.head,
        affinity = ?place.affinity,
        "moved the caret to a visual row edge"
    );
    return place;
}
