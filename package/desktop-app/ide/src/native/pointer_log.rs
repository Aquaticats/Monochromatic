//! The debug records of a pointer press,
//!  a drag move,
//!  and a release.
//!  Each names the point the handler received,
//!  the offsets of the view when it ran,
//!  and the line and character found for the point,
//!  so the source line a press selected at a moment can be read from the log without a screenshot.

/// The window the offsets are read from,
///  and the state that holds the document and the row map.
use super::{AppWindow, State};
/// The unit a press chose.
use ide_app::pointer_selection::Granularity;
/// Reads of the toolkit's window.
use slint::ComponentHandle;

/// What:
///  The point a pointer handler received and what the handler found for it.
/// Why:
///  `y` is already in the text space of the drawn rows (`drawn-y` in the markup),
///  so it is the pointer's place in the view plus the drawn offset (`view_y + drawn_offset`),
///  and the line the selection used must be the line drawn there, `y / row height`.
///  The record carries `view_y` (the pointer's distance below the top of the text area),
///  both offsets,
///  and the line,
///  so the line drawn under the pointer in the frame that was displayed,
///  `(view_y + that frame's drawn offset) / row height`,
///  can be compared with the line the press selected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pointer = { window: AppWindow; current: State; line: number; hit: number; y: number; x: number };
/// ```
pub(super) struct Pointer<'a> {
    /// The window whose offsets are read.
    pub(super) window: &'a AppWindow,
    /// The document and row map the point was found in.
    pub(super) current: &'a State,
    /// The line the point names, counted from 0.
    pub(super) line: usize,
    /// The document character offset the point names.
    pub(super) hit: usize,
    /// Logical pixels from the top of the text, as the drawn rows stand.
    pub(super) y: f32,
    /// Logical pixels from the start of the line's text.
    pub(super) x: f32,
}

/// What:
///  The selection a press or a drag move made.
/// Why:
///  A drag that reaches the callback with the wrong unit looks like a drag that never reached it on screen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Selection = { granularity: Granularity; anchor: number; head: number };
/// ```
pub(super) struct Selection {
    /// Character,
    ///  word,
    ///  or line.
    pub(super) granularity: Granularity,
    /// The fixed end.
    pub(super) anchor: usize,
    /// The moving end.
    pub(super) head: usize,
}

/// What:
///  The line counted from 1 and the column of the character at document offset `hit` on `line`.
/// Why:
///  The gutter counts lines from 1,
///  and a column is meaningful beside it where an offset into the whole document is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineAndColumn(lineStart: number, line: number, hit: number): [lineNumber: number, column: number];
/// ```
pub(super) fn line_and_column(line_start: usize, line: usize, hit: usize) -> (usize, usize) {
    return (line + 1, hit.saturating_sub(line_start));
}

/// What:
///  The line counted from 1 and the column of the character the point names.
/// Why:
///  A line the text does not have clamps to the last one,
///  as the row map does for a point below the text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function place(point: Pointer): [lineNumber: number, column: number];
/// ```
fn place(point: &Pointer) -> (usize, usize) {
    let text = point.current.document.text();
    let last = text.len_lines().saturating_sub(1);
    let line_start = text.line_to_char(point.line.min(last));
    return line_and_column(line_start, point.line, point.hit);
}

/// What:
///  Write the record of a press.
/// Why:
///  The line the press selected,
///  with the offsets of the view,
///  answers which source line a press at a moment selected.
///  The arguments are read only when the level is enabled,
///  so a build that does not log pays nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function logPress(point: Pointer, extend: boolean, selection: Selection): void;
/// ```
pub(super) fn log_press(point: &Pointer, extend: bool, selection: Selection) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    let (line_number, column) = place(point);
    tracing::debug!(
        y = point.y,
        x = point.x,
        view_y = point.y - point.window.get_drawn_offset(),
        offset = -point.window.get_scroll_y(),
        drawn_offset = point.window.get_drawn_offset(),
        held = point.window.get_held(),
        line = point.line,
        line_number,
        column,
        extend,
        granularity = ?selection.granularity,
        anchor = selection.anchor,
        head = selection.head,
        "source pointer press"
    );
}

/// What:
///  Write the record of a drag move;
///  the first move after a press is the drag start.
/// Why:
///  The start of a drag and its later moves are told apart by the message,
///  so the point where the drag began is one search.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function logDrag(point: Pointer, first: boolean, selection: Selection): void;
/// ```
pub(super) fn log_drag(point: &Pointer, first: bool, selection: Selection) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    let (line_number, column) = place(point);
    let message = if first {
        "source pointer drag start"
    } else {
        "source pointer drag"
    };
    tracing::debug!(
        y = point.y,
        x = point.x,
        view_y = point.y - point.window.get_drawn_offset(),
        offset = -point.window.get_scroll_y(),
        drawn_offset = point.window.get_drawn_offset(),
        held = point.window.get_held(),
        line = point.line,
        line_number,
        column,
        granularity = ?selection.granularity,
        anchor = selection.anchor,
        head = selection.head,
        "{message}"
    );
}

/// What:
///  Write the record of a release.
/// Why:
///  The line a drag ended on is read against the line drawn under the pointer,
///  as the press is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function logRelease(point: Pointer): void;
/// ```
pub(super) fn log_release(point: &Pointer) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    let (line_number, column) = place(point);
    tracing::debug!(
        y = point.y,
        x = point.x,
        view_y = point.y - point.window.get_drawn_offset(),
        offset = -point.window.get_scroll_y(),
        drawn_offset = point.window.get_drawn_offset(),
        held = point.window.get_held(),
        line = point.line,
        line_number,
        column,
        "source pointer release"
    );
}
