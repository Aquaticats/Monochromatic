//! Up,
//!  Down,
//!  PageUp,
//!  and PageDown by visual row and shaped pixel position instead of character columns.
//!
//! A soft-wrapped line is several visual rows,
//! and each step moves one of them.
//! The caret keeps aiming for the horizontal position where a run of vertical movements started
//!  (IntelliJ's `CaretImpl.myDesiredX`),
//! so passing through a short row or an empty line does not pull it to the left for the rows after it.
//! Target lines are shaped on demand at the wrap width,
//!  because a page movement usually leaves the materialized viewport.

/// Canonical source,
/// the current caret,
/// and the side of a wrap offset it stands on.
use crate::document::{Document, ReadingAffinity};
/// A target row is hit in its middle.
use crate::row_map::CODE_ROW;
/// The same shaper that paints the viewport supplies the advances for hit testing.
use crate::shaped_text::TextShaper;

/// What:
///  A copyable record;
///  `usize` is an address-sized character index (siblings `u32`,
///  `u64`),
/// `f32` a 32-bit float of logical pixels (sibling `f64`).
/// Why:
///  The remembered x is only meaningful for the caret position and side it was produced for.
/// Storing them with it makes every other caret change (pointer,
///  find,
///  reload,
///  Left,
///  Right)
/// invalidate the memory without those code paths knowing about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreferredColumn = { head: number; affinity: Affinity; x: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreferredColumn {
    /// Caret position this memory belongs to.
    pub head: usize,
    /// Side of the wrap offset the caret stood on at that position.
    pub affinity: ReadingAffinity,
    /// Logical x the caret aims for on every further vertical movement.
    pub x: f32,
}

/// Move the caret `rows` visual rows down (negative:
///  up) and return the new position with its remembered x.
///
/// `remembered` is used only when it belongs to the current caret position and side;
///  otherwise the caret's own x is taken.
/// `width` is the wrap width in logical pixels,
/// and a width that is not positive means no wrapping.
/// Moving up from the first row goes to the start of the text,
///  and down from the last row to its end,
/// as an ordinary text view does.
///  A larger step than the remaining rows stops on the first or last row.
/// A caret that lands at the end of an upper row stays on that row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function vertical(shaper: TextShaper, document: Document, scale: number, width: number, rows: number,
///   remembered?: PreferredColumn): PreferredColumn;
/// ```
pub fn vertical(
    shaper: &mut TextShaper,
    document: &Document,
    scale: f32,
    width: f32,
    rows: isize,
    remembered: Option<PreferredColumn>,
) -> PreferredColumn {
    let text = document.text();
    let position = document.position();
    let head = position.head;
    let affinity = position.affinity;
    let line = text.char_to_line(head);
    let current = shaper.wrapped_row(document, line, scale, width);
    let caret = current.caret_at(head, affinity, scale);
    // What: `if let Some(column)` runs only when a memory exists; `&& column.head == head` checks it is current;
    // the whole `if`/`else` is an expression whose branch value initializes `aim`.
    // Why: A memory for another caret position describes a movement that has since been interrupted,
    // and the current line is shaped only when no usable memory exists.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const aim = remembered?.head === head && remembered.affinity === affinity ? remembered.x : caret.x;
    // ```
    let aim = if let Some(column) = remembered
        && column.head == head
        && column.affinity == affinity
    {
        column.x
    } else {
        caret.x
    };
    let last = text.len_lines().saturating_sub(1);
    let first_row = line == 0 && caret.row == 0;
    let last_row = line == last && caret.row + 1 >= current.visual_rows();
    if rows < 0 && first_row {
        return PreferredColumn {
            head: 0,
            affinity: ReadingAffinity::Downstream,
            x: aim,
        };
    }
    if rows > 0 && last_row {
        return PreferredColumn {
            head: text.len_chars(),
            affinity: ReadingAffinity::Downstream,
            x: aim,
        };
    }
    // What: The row index is signed while walking, because a step up leaves it below zero until a line above is added.
    // Why: Each line passed is shaped at the wrap width, so its true count of rows decides where the step ends,
    // whatever the vertical map still counts for a line nobody has measured.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let row = caret.row + rows; while (row < 0 && line > 0) { line--; row += rowsOf(line); }
    // while (row >= rowsOf(line) && line < last) { row -= rowsOf(line); line++; }
    // ```
    let mut target_line = line;
    let mut shaped = current;
    let mut row = caret.row as isize + rows;
    while row < 0 && target_line > 0 {
        target_line -= 1;
        shaped = shaper.wrapped_row(document, target_line, scale, width);
        row += shaped.visual_rows() as isize;
    }
    while row >= shaped.visual_rows() as isize && target_line < last {
        row -= shaped.visual_rows() as isize;
        target_line += 1;
        shaped = shaper.wrapped_row(document, target_line, scale, width);
    }
    let landing = row.clamp(0, shaped.visual_rows() as isize - 1) as usize;
    let middle = shaped.visual_row_offset(landing, scale) + CODE_ROW / 2.0;
    let landed = shaped.hit_at(aim, middle, scale);
    tracing::debug!(
        line,
        target_line,
        landing,
        rows,
        aim,
        landed,
        "moved the caret by visual rows"
    );
    return PreferredColumn {
        head: landed,
        affinity: shaped.click_affinity(landed, middle, scale),
        x: aim,
    };
}
