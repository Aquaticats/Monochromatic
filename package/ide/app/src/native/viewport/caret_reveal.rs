//! The caret and a revealed range kept in view:
//!  the visual rows a position is drawn on,
//!  and the smallest scroll that shows them,
//!  or centers them when they are hidden.

/// The window,
///  the source state,
///  the render that follows a move,
///  and the row rules a reveal applies.
use super::super::{
    AppWindow,
    State,
    render,
    rows,
};
/// The one placement every caret move and reveal ends in.
use super::place;
/// The side of a soft-wrap offset a position is read on:
///  upstream or downstream of an ambiguous offset,
///  and a revealed range is not a caret,
///  so it is read downstream.
use ide_app::document::ReadingAffinity;
/// Height of one code row:
///  the least a line can take,
///  which bounds how many lines a view can show.
use ide_app::row_map::CODE_ROW;
/// A wrap width must be a positive number of pixels.
use ide_app::wrap::has_wrap_width;
/// Checked shared ownership remains confined to the UI thread.
use std::{
    cell::RefCell,
    rc::Rc,
};

/// What:
///  How many visual rows below the first row of its line the character `position` is drawn,
///  on the side of a soft wrap `affinity` names.
///  A line outside the materialized rows is shaped on the spot.
/// Why:
///  Keeping the caret in view means keeping its visual row in view,
/// not only its line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rowsDown(current: State, position: number, affinity: ReadingAffinity, scale: number): number;
/// ```
fn rows_down(
    current: &mut State,
    position: usize,
    affinity: ReadingAffinity,
    scale: f32,
) -> usize {
    let line = current
        .document
        .text()
        .char_to_line(position.min(current.document.text().len_chars()));
    if let Some(view) = &current.shaped {
        for row in &view.rows {
            if row.row == line {
                return row.caret_at(position, affinity, view.viewport.scale).row;
            }
        }
    }
    if !has_wrap_width(current.width) {
        return 0;
    }
    let State {
        document,
        shaper,
        width,
        ..
    } = current;
    return shaper
        .wrapped_row(document, line, scale, *width)
        .caret_at(position, affinity, scale)
        .row;
}

/// Render after a keyboard caret movement and keep the caret visible with the smallest possible scroll.
///
/// `paged` first moves the view by that many logical pixels,
///  so a page key keeps the caret's place on screen.
/// A caret above the view becomes the top row,
///  one below the bottom row;
///  a visible caret keeps the offset.
/// Offsets are assigned directly,
///  like other programmatic reveals;
///  native wheel easing is unaffected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function follow(window: AppWindow, state: Shared<State>, paged: number): void;
/// ```
pub(in crate::native) fn follow(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    paged: f32,
) {
    let height = window.get_viewport_height();
    let width = window.get_viewport_width();
    let before = -window.get_scroll_y();
    let scale = window.window().scale_factor();
    let mut current = state.borrow_mut();
    let caret = current.document.position();
    let head = caret.head;
    let row = current.document.text().char_to_line(head);
    let down = rows_down(&mut current, head, caret.affinity, scale);
    let limit = rows::limit(&current.row_map, height);
    let paged_offset = (before + paged).clamp(0.0, limit);
    let mut offset =
        rows::showing(&current.row_map, row, down, paged_offset, height).clamp(0.0, limit);
    // A fractional offset left by smooth scrolling is kept while the caret stays in view.
    if offset != before {
        offset = offset.round();
    }
    place(&mut current, offset, width, height);
    drop(current);
    if offset != before {
        window.set_scroll_y(-offset);
    }
    render(window, state);
    tracing::debug!(row, vertical = offset, "followed source caret");
}

/// Scroll a source character range into view and render it.
/// A visible range keeps its offset;
///  a hidden row is centered.
/// Offsets are assigned directly,
///  like other programmatic reveals;
///  native wheel easing is unaffected.
pub(in crate::native) fn reveal(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    start: usize,
    end: usize,
) {
    let height = window.get_viewport_height();
    let width = window.get_viewport_width();
    let before = -window.get_scroll_y();
    let scale = window.window().scale_factor();
    let mut offset = before;
    let mut current = state.borrow_mut();
    let text = current.document.text();
    let bounded = start.min(text.len_chars());
    let row = text.char_to_line(bounded);
    // A range to reveal is not the caret, so its start stands on the default side of a wrap offset.
    let down = rows_down(&mut current, bounded, ReadingAffinity::Downstream, scale);
    // What: The target line's own visual rows go into the map before the offset is limited by its extent.
    // Why: A line the background measurement has not reached counts as one row, so a match on a late row of the
    //      last line would be clamped out of the view, and nothing moves the view again once the rows are known.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // absorb(state, measureLines(row, row + 1)); settle(state);
    // ```
    if down > 0 && has_wrap_width(current.width) {
        let wrap = current.width;
        let measured = {
            let State {
                shaper,
                document,
                styles,
                ..
            } = &mut *current;
            shaper.measure_lines(document, row..row + 1, scale, Some(wrap), styles)
        };
        if rows::absorb(&mut current, measured.lines) {
            tracing::debug!(
                row,
                down,
                "measured the revealed line before limiting the offset"
            );
            rows::settle(&mut current, scale, (before, width, height), true);
        }
    }
    let top = current.row_map.code_top(row) + down as f32 * CODE_ROW;
    if top < offset || top + CODE_ROW > offset + height {
        let limit = rows::limit(&current.row_map, height);
        offset = (top - ((height - CODE_ROW) / 2.0).max(0.0))
            .clamp(0.0, limit)
            .round();
    }
    place(&mut current, offset, width, height);
    drop(current);
    if offset != before {
        window.set_scroll_y(-offset);
    }
    render(window, state);
    tracing::debug!(row, vertical = offset, end, "revealed source range");
}
