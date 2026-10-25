//! Keyboard caret movement and select-all;
//!  none of it edits source.

/// Borrow the parent's state,
///  its single rendering boundary,
///  and the caret-following scroll.
use super::{AppWindow, State, render, viewport::follow};
/// Geometry-free caret movement over source characters.
use ide_app::caret_motion::{Motion, moved};
/// Source offsets remain independent of rendered pixels,
/// and the side of a soft-wrap offset is part of a click.
use ide_app::document::{ReadingAffinity, ReadingPosition};
/// Home and End follow the visual rows of the caret's line.
use ide_app::line_boundary::{Edge, moved_to_edge};
/// Line-wise movement by shaped pixel position with a remembered column.
use ide_app::vertical_motion::{PreferredColumn, vertical};
/// Toolkit key representations and weak window handles.
use slint::{ComponentHandle, SharedString, platform::Key};
/// Shared ownership of the window state on this thread.
use std::{cell::RefCell, rc::Rc};

/// What:
///  `Option<Motion>` is a movement or nothing (TypeScript's `Motion | undefined`);
/// `&SharedString` borrows the toolkit's encoded key text.
/// Why:
///  Slint represents special keys as encoded strings,
///  not enum values,
///  and most keys are not movements.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function horizontal(key: string, control: boolean): Motion | undefined;
/// ```
fn horizontal(key: &SharedString, control: bool) -> Option<Motion> {
    let mut plain = None;
    let mut modified = None;
    if *key == SharedString::from(Key::LeftArrow) {
        plain = Some(Motion::Left);
        modified = Some(Motion::WordLeft);
    } else if *key == SharedString::from(Key::RightArrow) {
        plain = Some(Motion::Right);
        modified = Some(Motion::WordRight);
    } else if *key == SharedString::from(Key::Home) {
        modified = Some(Motion::DocumentStart);
    } else if *key == SharedString::from(Key::End) {
        modified = Some(Motion::DocumentEnd);
    }
    if control {
        return modified;
    }
    return plain;
}

/// What:
///  `Option<Edge>` is Home or End without Control,
/// or nothing.
/// Why:
///  Plain Home and End follow the visual rows of a wrapped line,
/// which only the shaped line knows;
///  with Control they go to the ends of the document instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function edge(key: string, control: boolean): Edge | undefined;
/// ```
fn edge(key: &SharedString, control: bool) -> Option<Edge> {
    if control {
        return None;
    }
    if *key == SharedString::from(Key::Home) {
        return Some(Edge::Home);
    }
    if *key == SharedString::from(Key::End) {
        return Some(Edge::End);
    }
    return None;
}

/// Signed visual row count for Up,
///  Down,
///  PageUp,
///  and PageDown,
///  and whether the view pages along with the caret.
/// `pages` pairs the whole visual rows one page holds above the top row with those from the top row down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function vertical_rows(key: string, pages: [up: number, down: number]): [rows: number, paging: boolean] | undefined;
/// ```
fn vertical_rows(key: &SharedString, pages: (isize, isize)) -> Option<(isize, bool)> {
    let (up, down) = pages;
    if *key == SharedString::from(Key::UpArrow) {
        return Some((-1, false));
    }
    if *key == SharedString::from(Key::DownArrow) {
        return Some((1, false));
    }
    if *key == SharedString::from(Key::PageUp) {
        return Some((-up, true));
    }
    if *key == SharedString::from(Key::PageDown) {
        return Some((down, true));
    }
    return None;
}

/// Bind native keyboard caret movement and select-all.
pub(super) fn bind_keys(owner: &AppWindow, shared: &Rc<RefCell<State>>) {
    let select_state = Rc::clone(shared);
    let select_window = owner.as_weak();
    owner.on_select_all_request(move || {
        let mut current = select_state.borrow_mut();
        let end = current.document.text().len_chars();
        current.document.select(ReadingPosition {
            anchor: 0,
            head: end,
            viewport: 0,
            affinity: ReadingAffinity::Downstream,
        });
        drop(current);
        if let Some(window) = select_window.upgrade() {
            render(&window, &select_state);
        }
    });
    let state = Rc::clone(shared);
    let weak = owner.as_weak();
    // The remembered column names the caret it belongs to, so any other caret change invalidates it.
    let mut column: Option<PreferredColumn> = None;
    owner.on_key_input(move |key, control, shift, alt| {
        // Alt combinations are not reading keys; leaving them alone keeps them free for later commands.
        if alt {
            return;
        }
        let Some(window) = weak.upgrade() else {
            return;
        };
        let height = window.get_viewport_height();
        let offset = -window.get_scroll_y();
        let scale = window.window().scale_factor();
        let mut current = state.borrow_mut();
        let mut position = current.document.position();
        let start = position.anchor.min(position.head);
        let end = position.anchor.max(position.head);
        // A page is the whole visual rows that fit in the view, at least one, and the pixels exactly those rows take:
        // counted from the top row down for PageDown and upwards from it for PageUp.
        let (down, down_pixels) = current.row_map.page(offset, height, true);
        let (up, up_pixels) = current.row_map.page(offset, height, false);
        let mut scrolled = 0.0;
        // Only Home, End, and the vertical keys choose a side of a wrap offset; every other move takes the default.
        let mut side = ReadingAffinity::Downstream;
        let edge_key = edge(&key, control);
        let motion_key = horizontal(&key, control);
        if edge_key.is_some() || motion_key.is_some() {
            // A horizontal key names a new aim even when the caret cannot move, such as Home at a line start.
            column = None;
            if let Some(which) = edge_key {
                // Home and End follow the visual row of the caret, on the shaped line.
                let State {
                    document,
                    shaper,
                    width,
                    ..
                } = &mut *current;
                let place = moved_to_edge(shaper, document, scale, *width, which);
                position.head = place.head;
                side = place.affinity;
            } else if let Some(motion) = motion_key {
                // Without Shift, Left and Right first collapse a selection to the side they point at.
                if !shift && start != end && motion == Motion::Left {
                    position.head = start;
                } else if !shift && start != end && motion == Motion::Right {
                    position.head = end;
                } else {
                    position.head = moved(current.document.text().slice(..), position.head, motion);
                }
            }
        } else if let Some((rows, paging)) = vertical_rows(&key, (up as isize, down as isize))
            && !control
        {
            // Destructure the mutable borrow so the shaper can work while the document is lent read-only.
            let State {
                document,
                shaper,
                width,
                ..
            } = &mut *current;
            let landed = vertical(shaper, document, scale, *width, rows, column);
            column = Some(landed);
            position.head = landed.head;
            side = landed.affinity;
            if paging && rows > 0 {
                scrolled = down_pixels;
            } else if paging {
                scrolled = -up_pixels;
            }
        } else {
            return;
        }
        position.affinity = side;
        if !shift {
            position.anchor = position.head;
        }
        tracing::debug!(
            control,
            shift,
            anchor = position.anchor,
            head = position.head,
            affinity = ?position.affinity,
            paged = scrolled,
            "source caret key"
        );
        current.document.select(position);
        drop(current);
        follow(&window, &state, scrolled);
    });
}
