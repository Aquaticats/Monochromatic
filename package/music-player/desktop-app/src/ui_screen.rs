//! Window binding for the library screen:
//! applies engine updates to the UI-thread state and pushes
//! the projected view into `AppWindow`.
//!
//! What:
//! `apply_update` is the single entry for engine messages on the event-loop thread.
//! Why:
//! the page rules and the row and picker models stay in the pure modules;
//! this file only
//! converts them into Slint values and writes window properties.
//! The update path lives in `src/ui_screen_update.rs`,
//! the projection in `src/ui_screen_projection.rs`,
//! and this file holds the state they share and the guards that borrow it.

/// Imports the cell that holds the screen state on the window thread and the guards that borrow it.
use std::cell::{
    Ref,
    RefCell,
    RefMut,
};

/// Imports the screen state the cell holds.
use crate::ui_screen_view::ScreenState;

thread_local! {
    /// What:
    /// the UI thread's copy of the engine facts,
    /// read and written only on the event-loop thread.
    /// Why:
    /// `invoke_from_event_loop` runs every update on that thread,
    /// so no lock is needed.
    /// The cell is leaked once per thread so that `screen_cell` can hand out a plain reference,
    /// which lets callers hold a guard instead of passing a closure to `LocalKey::with`.
    static SCREEN: &'static RefCell<ScreenState> = Box::leak(Box::new(RefCell::new(ScreenState::default())));
}

/// What:
/// `screen_cell` returns this thread's screen state cell.
/// Why:
/// the cell is leaked once per thread,
/// so it outlives every borrow,
/// so the guards of `read_screen` and `write_screen` need no closure to stay inside the borrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const screenCell = (): Cell<ScreenState> => screen;
/// ```
fn screen_cell() -> &'static RefCell<ScreenState> {
    return SCREEN.with(Clone::clone);
}

/// What:
/// `read_screen` borrows the UI-thread state for reading until the guard is dropped.
/// Why:
/// callers read a field or two without wrapping the read in a closure;
/// a guard held across a write would panic,
/// so each guard lives only as long as the statement or block that needs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = readScreen(); // a read-only view until it goes out of scope
/// ```
pub(crate) fn read_screen() -> Ref<'static, ScreenState> {
    return screen_cell().borrow();
}

/// What:
/// `write_screen` borrows the UI-thread state for writing until the guard is dropped.
/// Why:
/// all writes go through this one function,
/// so the borrow stays short:
/// each guard lives only in the block that changes the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = writeScreen(); // the one writer until it goes out of scope
/// ```
pub(crate) fn write_screen() -> RefMut<'static, ScreenState> {
    return screen_cell().borrow_mut();
}

/// What:
/// `read_state` runs a read-only closure over the UI-thread state.
/// Why:
/// callers read the state without taking ownership of it.
pub(crate) fn read_state<T>(read: impl FnOnce(&ScreenState) -> T) -> T {
    return read(&read_screen());
}

/// What:
/// `update_state` runs a mutating closure over the UI-thread state.
/// Why:
/// the window tests fill the state through it;
/// production code borrows `write_screen` for the same short span.
#[cfg(test)]
pub(crate) fn update_state(update: impl FnOnce(&mut ScreenState)) {
    update(&mut write_screen());
}

/// What:
///      `mod ui_screen_update;` loads the sibling `src/ui_screen_update.rs`,
///  the one place an engine message changes the state.
/// Why:
///       Every queue,
///  playing and position message arrives there,
///  and this module holds the state they are written to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiScreenUpdate from "./ui_screen_update";
/// ```
#[path = "ui_screen_update.rs"]
mod ui_screen_update;

/// What:
///      `pub(crate) use ui_screen_update::apply_update;` re-exports the engine entry
///           point at this module's path.
/// Why:
///       The window glue and the window tests name `ui_screen::apply_update`,
///  and only where it is declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { apply_update } from "./ui_screen_update";
/// ```
pub(crate) use ui_screen_update::apply_update;

/// What:
///      `mod ui_screen_projection;` loads the sibling `src/ui_screen_projection.rs`,
///  which rebuilds the window's view models from the state.
/// Why:
///       One function writes every list,
///  picker and deck value,
///  and it is the only reader of the view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiScreenProjection from "./ui_screen_projection";
/// ```
#[path = "ui_screen_projection.rs"]
mod ui_screen_projection;

/// What:
///      `pub(crate) use ui_screen_projection::{...}` re-exports the projection and the
///           model builder at this module's path.
/// Why:
///       The debounce bridge and the Settings page name `ui_screen::project` and
///  `ui_screen::model_of`,
///  and only where they are declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { model_of, project } from "./ui_screen_projection";
/// ```
pub(crate) use ui_screen_projection::{
    model_of,
    project,
};
