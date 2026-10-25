//! Pointer selection never edits source.

/// Borrow the parent's state,
///  its single rendering boundary,
///  and the caret-following scroll.
use super::{
    AppWindow, State,
    pointer_log::{Pointer, Selection, log_drag, log_press, log_release},
    render,
    viewport::follow,
};
/// The side of a soft-wrap offset is part of a click.
use ide_app::document::ReadingAffinity;
/// Click counting and unit-wise drag extension.
use ide_app::pointer_selection::{ClickCounter, Granularity, extended, unit};
/// Weak window handles.
use slint::ComponentHandle;
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

/// What:
///  The pointer's vertical place in the text as the latest frame drew it,
///  given the place `y` the markup computed from the offsets of the moment the event is handled.
/// Why:
///  A press is handled after the frame it was made over,
///  and a busy window thread can handle it a hundred milliseconds later,
///  when a wheel ease has moved the toolkit's offset on by thousands of pixels.
///  The person pressed on the rows of the picture that was shown,
///  so the offset of that frame stands in for the offset of the moment.
///  Before any frame has begun nothing was shown,
///  and the moment's own place stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shownY(window: AppWindow, current: State, y: number): number;
/// ```
pub(super) fn shown_y(window: &AppWindow, current: &State, y: f32) -> f32 {
    let Some(shown) = current.gpu.shown_offset() else {
        return y;
    };
    return y - window.get_drawn_offset() + shown;
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
    owner.on_pointer_press(move |received, x, extend| {
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
        // The row under the pointer is the row of the picture the person sees: the one the latest frame drew.
        let y = shown_y(&window, &current, received);
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
                y: received,
                row_y: y,
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
    owner.on_pointer_hit(move |received, x, _extend| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let mut current = state.borrow_mut();
        // A drag over virtual rows extends to the line they belong to, like a press there.
        // The row under the pointer is the row of the picture the person sees: the one the latest frame drew.
        let y = shown_y(&window, &current, received);
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
                y: received,
                row_y: y,
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
    owner.on_pointer_release(move |received, x| {
        // The line and the character are found only to be written, so a build that does not log pays nothing.
        if !tracing::enabled!(tracing::Level::DEBUG) {
            return;
        }
        let Some(window) = release_window.upgrade() else {
            return;
        };
        let mut current = release_state.borrow_mut();
        // The row under the pointer is the row of the picture the person sees: the one the latest frame drew.
        let y = shown_y(&window, &current, received);
        let line = current.row_map.line_at(y);
        let (hit, _side) =
            pointer_position(&mut current, line, (x, y), window.window().scale_factor());
        log_release(&Pointer {
            window: &window,
            current: &current,
            line,
            hit,
            y: received,
            row_y: y,
            x,
        });
    });
}
