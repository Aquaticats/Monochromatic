//! Caret and selection input never edit source.

/// Borrow the parent's state,
///  its single rendering boundary,
///  and the caret-following scroll.
use super::{
    AppWindow, State,
    pointer_log::{Pointer, Selection, log_drag, log_press, log_release},
    render,
    viewport::follow,
};
/// Geometry-free caret movement over source characters.
use ide_app::caret_motion::{Motion, moved};
/// Source offsets remain independent of rendered pixels,
/// and the side of a soft-wrap offset is part of a click.
use ide_app::document::{ReadingAffinity, ReadingPosition};
/// Home and End follow the visual rows of the caret's line.
use ide_app::line_boundary::{Edge, moved_to_edge};
/// Click counting and unit-wise drag extension.
use ide_app::pointer_selection::{ClickCounter, Granularity, extended, unit};
/// Line-wise movement by shaped pixel position with a remembered column.
use ide_app::vertical_motion::{PreferredColumn, vertical};
/// Toolkit key representations and weak window handles.
use slint::{ComponentHandle, SharedString, platform::Key};
/// What:
///  `Rc` shares one owner on this thread,
///  `RefCell` checks mutable borrows at run time,
/// and `Instant` is a point on a clock that never goes backwards.
/// Why:
///  Press and drag callbacks change the same pointer state,
///  and multi-clicks are defined by elapsed time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const drag = { current: createDrag() }; const now = performance.now();
/// ```
use std::{cell::RefCell, rc::Rc, time::Instant};

/// What:
///  The unit selected by the latest press;
///  `(usize, usize)` is a pair of character positions.
/// Why:
///  A drag extends from the pressed unit,
///  so a double-click drag keeps whole words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Drag = { clicks: ClickCounter; origin: [number, number]; granularity: Granularity };
/// ```
struct Drag {
    /// Time and place of the previous press.
    clicks: ClickCounter,
    /// Start and end of the unit selected by the press.
    origin: (usize, usize),
    /// Character,
    ///  word,
    ///  or line,
    ///  as chosen by the click count.
    granularity: Granularity,
    /// Whether a drag move has come since the latest press.
    dragged: bool,
}

/// What:
///  `&mut State` lends the source state mutably;
///  `usize` is an address-sized index (siblings `u32`,
///  `u64`);
/// `f32` is a 32-bit float (sibling `f64`).
///  The answer is the character position under the pointer and the side of a soft-wrap offset it was clicked on.
/// Why:
///  A drag can leave the materialized rows;
///  such a line is shaped on demand,
/// at the wrap width,
/// so the pointer still resolves by glyph geometry instead of falling back to the line start.
///  `y` is the logical distance below the top of the line's first visual row,
///  so a click on a continuation row resolves in that row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pointerPosition(current: State, line: number, x: number, y: number, scale: number): [number, Affinity];
/// ```
pub(super) fn pointer_position(
    current: &mut State,
    line: usize,
    point: (f32, f32),
    scale: f32,
) -> (usize, ReadingAffinity) {
    let (x, y) = point;
    let within = (y - current.row_map.code_top(line)).max(0.0);
    // What: Destructuring `&mut State` lends fields separately.
    // Why: The shaper must be mutable while the document and the shaped view are only read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const { document, shaper, shaped, width } = current;
    // ```
    let State {
        document,
        shaper,
        shaped,
        width,
        ..
    } = current;
    // What: `if let Some(view)` runs only when a viewport has been shaped.
    // Why: Materialized rows already hold the exact painted geometry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const row = shaped?.rows.find(candidate => candidate.row === line);
    // ```
    if let Some(view) = shaped {
        for candidate in &view.rows {
            if candidate.row == line {
                let hit = candidate.hit_at(x, within, view.viewport.scale);
                let side = candidate.click_affinity(hit, within, view.viewport.scale);
                return (hit, side);
            }
        }
    }
    let shaped_line = shaper.wrapped_row(document, line, scale, *width);
    let hit = shaped_line.hit_at(x, within, scale);
    return (hit, shaped_line.click_affinity(hit, within, scale));
}

/// Bind pointer selection without letting the UI mutate source text.
pub(super) fn bind_pointer(owner: &AppWindow, shared: &Rc<RefCell<State>>) {
    // What: `Rc::new(RefCell::new(...))` creates the shared owner of the pointer state.
    // Why: The press and the drag callback both outlive this binding function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const drag = { current: { clicks: new ClickCounter(), origin: [0, 0], granularity: 'character' } };
    // ```
    let drag = Rc::new(RefCell::new(Drag {
        clicks: ClickCounter::default(),
        origin: (0, 0),
        granularity: Granularity::Character,
        dragged: false,
    }));
    // What: clone shares Rc ownership, whereas cloning a String copies its bytes.
    // Why: The callback remains valid after this binding function returns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const callbackState = state;
    // ```
    let press_state = Rc::clone(shared);
    let press_drag = Rc::clone(&drag);
    let press_window = owner.as_weak();
    // What: move |...| transfers captured handles into a stored callback.
    // Why: Borrowed local variables would not outlive this function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.onPointerPress((y, x, extend) => { ... });
    // ```
    owner.on_pointer_press(move |y, x, extend| {
        // What: upgrade returns Some only while the window still exists; `let ... else` leaves otherwise.
        // Why: Closing the app must not keep a hidden window alive through callbacks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const window = weak.deref(); if (!window) return;
        // ```
        let Some(window) = press_window.upgrade() else {
            return;
        };
        let mut current = press_state.borrow_mut();
        // A press on a line's virtual rows acts on that line: the rows belong to the code row beneath them.
        let line = current.row_map.line_at(y);
        let (hit, side) =
            pointer_position(&mut current, line, (x, y), window.window().scale_factor());
        let mut position = current.document.position();
        let mut pressed = press_drag.borrow_mut();
        if extend {
            // Shift+click keeps the anchor and is never part of a double or triple click.
            pressed.clicks.reset();
            pressed.granularity = Granularity::Character;
            pressed.origin = (position.anchor, position.anchor);
            position.head = hit;
        } else {
            // A multi-click stays on one visual row: two quick presses on different rows of a wrapped line are
            // two single clicks, as the distance between them is a row. The row is counted over the whole text.
            let visual = current.row_map.visual_row_at(line, y);
            let row = line + current.row_map.line_rows().extra_before(line) + visual;
            let granularity = pressed.clicks.press(Instant::now(), row, x);
            let (start, end) = unit(current.document.text().slice(..), hit, granularity);
            pressed.granularity = granularity;
            pressed.origin = (start, end);
            position.anchor = start;
            position.head = end;
        }
        // What: The record names the line the press selected, with the offsets of the view at that moment.
        // Why: Which unit a press selected is the branch decision that explains a later drag, and which line it
        // selected, against the line drawn under the pointer, is what a press during a held view must get right.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.debug('source pointer press', { y, x, line, offset, drawnOffset, held, extend, granularity, head });
        // ```
        log_press(
            &Pointer {
                window: &window,
                current: &current,
                line,
                hit,
                y,
                x,
            },
            extend,
            Selection {
                granularity: pressed.granularity,
                anchor: position.anchor,
                head: position.head,
            },
        );
        pressed.dragged = false;
        drop(pressed);
        // A caret at a soft-wrap offset stays on the side that was clicked.
        position.affinity = if position.head == hit {
            side
        } else {
            ReadingAffinity::Downstream
        };
        current.document.select(position);
        drop(current);
        render(&window, &press_state);
    });
    let state = Rc::clone(shared);
    let weak = owner.as_weak();
    owner.on_pointer_hit(move |y, x, _extend| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let mut current = state.borrow_mut();
        // A drag over virtual rows extends to the line they belong to, like a press there.
        let line = current.row_map.line_at(y);
        let (hit, side) =
            pointer_position(&mut current, line, (x, y), window.window().scale_factor());
        let mut pressed = drag.borrow_mut();
        let granularity = pressed.granularity;
        let (anchor, head) = extended(
            current.document.text().slice(..),
            pressed.origin,
            granularity,
            hit,
        );
        // The first move after a press is where a drag starts.
        let first = !pressed.dragged;
        pressed.dragged = true;
        drop(pressed);
        let mut position = current.document.position();
        position.anchor = anchor;
        position.head = head;
        position.affinity = if head == hit {
            side
        } else {
            ReadingAffinity::Downstream
        };
        // What: The record carries the point the drag move names, with the offsets of the view, and says whether it is
        // the move that starts the drag.
        // Why: A drag that never reaches this callback and one that reaches it with the wrong unit look alike
        // on screen; the log tells them apart.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.debug('source pointer drag', { first, y, x, line, offset, drawnOffset, held, granularity, anchor, head });
        // ```
        log_drag(
            &Pointer {
                window: &window,
                current: &current,
                line,
                hit,
                y,
                x,
            },
            first,
            Selection {
                granularity,
                anchor,
                head,
            },
        );
        current.document.select(position);
        drop(current);
        // A drag past an edge of the view scrolls just far enough to show where the selection now ends.
        follow(&window, &state, 0.0);
    });
    let release_state = Rc::clone(shared);
    let release_window = owner.as_weak();
    // What: The release changes nothing; it writes the record of the point the button went up at.
    // Why: The line a drag ended on is read from the log against the line drawn under the pointer, as the press is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.onPointerRelease((y, x) => { logger.debug('source pointer release', { y, x, line, offset, drawnOffset }); });
    // ```
    owner.on_pointer_release(move |y, x| {
        // The line and the character are found only to be written, so a build that does not log pays nothing.
        if !tracing::enabled!(tracing::Level::DEBUG) {
            return;
        }
        let Some(window) = release_window.upgrade() else {
            return;
        };
        let mut current = release_state.borrow_mut();
        let line = current.row_map.line_at(y);
        let (hit, _side) =
            pointer_position(&mut current, line, (x, y), window.window().scale_factor());
        log_release(&Pointer {
            window: &window,
            current: &current,
            line,
            hit,
            y,
            x,
        });
    });
}

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
