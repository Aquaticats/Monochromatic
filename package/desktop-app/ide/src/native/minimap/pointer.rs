//! The minimap as a scroll control:
//!  a press,
//!  a drag,
//!  a page step,
//!  and a resize become scroll offsets,
//!  a caret move,
//!  and a note of the new size.

/// The press that is mapped again once the layout changes.
use crate::native::minimap::aim::{self, Press};
/// A resize changes what a picture would be valid for.
use crate::native::minimap::request::note;
/// The drag that a press starts.
use crate::native::minimap::state::Drag;
/// The window,
///  its state,
///  and the mapping of lines to pixels.
use crate::native::{AppWindow, State, render::render, rows, viewport};
/// The geometry of the minimap.
use ide_app::minimap::Geometry;
/// Weak handles of the window.
use slint::ComponentHandle;
/// Shared state and its cell.
use std::{cell::RefCell, rc::Rc};

/// What:
///  Scroll the view to `offset`,
/// held to the scroll range,
/// on the window thread's synchronous path:
///  the tile for the new place is built before the next frame,
/// as for a caret key or a revealed range.
/// Why:
///  A drag of the box over a long text moves the view by several view heights at a step;
/// a tile built by the
///  worker would be out of date when it arrived,
/// and the text would stay blank while the pointer moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scrollTo(window: AppWindow, state: Shared<State>, offset: number): void;
/// ```
fn scroll_to(window: &AppWindow, state: &Rc<RefCell<State>>, offset: f32) {
    let height = window.get_viewport_height();
    let width = window.get_viewport_width();
    let before = -window.get_scroll_y();
    let mut current = state.borrow_mut();
    let wanted = offset
        .clamp(0.0, rows::limit(&current.row_map, height))
        .round();
    if wanted == before {
        return;
    }
    viewport::place(&mut current, wanted, width, height);
    drop(current);
    window.set_scroll_y(-wanted);
    render(window, state);
}

/// What:
///  A press outside the viewport box:
/// move the caret to the visual row under the pointer and the character under it
///  (the text's width is the minimap's),
/// extend the selection from the old caret when Shift is held,
///  and scroll that row to the middle of the view.
/// The answer is the scroll offset it took.
/// Why:
///  CodeGlance Pro's defaults (`jumpOnMouseDown = MOUSE_DOWN`,
/// `clickType = CODE_POSITION`,
/// `moveOnly = false`):
///  the caret goes to the clicked position,
/// which is then centered,
/// and a Shift press selects up to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jump(window: AppWindow, state: Shared<State>, y: number, x: number, shift: boolean): number;
/// ```
fn jump(window: &AppWindow, state: &Rc<RefCell<State>>, y: f32, x: f32, shift: bool) -> f32 {
    let height = window.get_viewport_height();
    let width = window.get_viewport_width();
    let scale = window.window().scale_factor();
    let mut current = state.borrow_mut();
    let before = -window.get_scroll_y();
    let press = Press { y, x, shift };
    let share = aim::share(&Geometry::new(current.row_map.height(), height), before);
    let (target, offset) = aim::map(&mut current, press, before, scale, height);
    tracing::debug!(
        line = target.line,
        row = target.row,
        y,
        x,
        shift,
        offset,
        "minimap press jumped"
    );
    viewport::place(&mut current, offset, width, height);
    // The layout may change before the reader does anything else; the press is then mapped through it again.
    aim::remember(&mut current, press, share, (offset, (width, height)));
    drop(current);
    window.set_scroll_y(-offset);
    render(window, state);
    return offset;
}

/// Bind the minimap's pointer,
/// drag,
/// step,
/// and size callbacks.
pub(in crate::native) fn bind(window: &AppWindow, shared: &Rc<RefCell<State>>) {
    let press_state = Rc::clone(shared);
    let press_window = window.as_weak();
    window.on_minimap_press(move |y, x, shift, inside| {
        let Some(active) = press_window.upgrade() else {
            return;
        };
        let mut offset = -active.get_scroll_y();
        if !inside {
            offset = jump(&active, &press_state, y, x, shift);
        }
        // The box follows the pointer from here, from the place the press left it.
        press_state.borrow_mut().minimap.drag = Some(Drag {
            start_y: y,
            start_offset: offset,
        });
        tracing::debug!(y, inside, offset, "minimap press");
    });
    let drag_state = Rc::clone(shared);
    let drag_window = window.as_weak();
    window.on_minimap_drag(move |y| {
        let Some(active) = drag_window.upgrade() else {
            return;
        };
        let current = drag_state.borrow();
        let Some(drag) = current.minimap.drag else {
            return;
        };
        let geometry = Geometry::new(current.row_map.height(), active.get_viewport_height());
        let offset = geometry.drag_offset(drag.start_offset, drag.start_y, y);
        drop(current);
        scroll_to(&active, &drag_state, offset);
    });
    let release_state = Rc::clone(shared);
    window.on_minimap_release(move || {
        release_state.borrow_mut().minimap.drag = None;
    });
    let step_state = Rc::clone(shared);
    let step_window = window.as_weak();
    window.on_minimap_step(move |direction| {
        let Some(active) = step_window.upgrade() else {
            return;
        };
        let height = active.get_viewport_height();
        scroll_to(
            &active,
            &step_state,
            -active.get_scroll_y() + direction as f32 * height,
        );
    });
    let size_state = Rc::clone(shared);
    let size_window = window.as_weak();
    window.on_minimap_resized(move |width, height| {
        let Some(active) = size_window.upgrade() else {
            return;
        };
        size_state.borrow_mut().minimap.size = (width, height);
        note(&active, &size_state);
    });
}
