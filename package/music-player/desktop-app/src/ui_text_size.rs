//! Window binding for the Text size setting: the stepper and reset callbacks of Settings, and the `TextSize` global.
//!
//! What: keeps the four sizes in the screen state, applies a stepper press or a reset to them, saves the result to the
//! session file and writes the sizes into the `TextSize` global that the pane titles, the group heading letter, the folder
//! names and the track titles read.
//! Why: the rules (limits, step, defaults) live in `music_player::text_size` and the file format in `session`; this module
//! only maps a press to them and the result to the live window, so the window changes at once and a restart restores it.

use music_player::session::Session;
use music_player::text_size::{MAX_TEXT_SIZE_PX, MIN_TEXT_SIZE_PX, TextElement, TextSizes};
use slint::ComponentHandle;

use crate::ui_screen::{project, read_state, update_state};
use crate::{AppWindow, TextSize};

/// What: `SaveTextSizes` is the function that persists the four text sizes.
/// Why: the production function writes the user's session file, so the window tests swap in one that records the sizes.
type SaveTextSizes = fn(TextSizes) -> std::io::Result<()>;

thread_local! {
    /// What: the persistence function of the Text size setting on this UI thread.
    /// Why: only the event-loop thread saves it, and tests replace it per test thread so a stepper press never writes the
    /// user's file.
    static SAVER: std::cell::Cell<SaveTextSizes> = const { std::cell::Cell::new(Session::save_text_sizes) };
}

/// What: `replace_saver` makes this thread persist the Text size setting through `saver`.
/// Why: a test that presses a stepper must never write the user's session file.
#[cfg(test)]
pub(crate) fn replace_saver(saver: SaveTextSizes) {
    SAVER.set(saver);
}

/// What: `load_into_state` reads the saved sizes from the session file into the screen state.
/// Why: the sizes must be in the state before the first projection, so the first frame already draws the saved sizes.
pub(crate) fn load_into_state() {
    let text_sizes = Session::load_text_sizes();
    tracing::info!(?text_sizes, "loaded the Text size setting");
    update_state(|state| {
        state.text_sizes = text_sizes;
    });
}

/// What: `wire_callbacks` registers the stepper and the reset callbacks of the Settings page.
/// Why: split from `load_into_state` so the window tests register the same callbacks without reading the user's session file.
pub(crate) fn wire_callbacks(app: &AppWindow) {
    let step_weak = app.as_weak();
    app.on_settings_step_text_size(move |index, step| {
        if let Some(window) = step_weak.upgrade() {
            step_text_size(&window, index, step);
        }
    });
    let reset_weak = app.as_weak();
    app.on_settings_reset_text_sizes(move || {
        if let Some(window) = reset_weak.upgrade() {
            reset_text_sizes(&window);
        }
    });
}

/// What: `step_text_size` moves the size of one text by `step` pixels, saves it and reprojects the window.
/// Why: a stepper changes a size by one pixel (`index` is the stepper of `TextElement::from_index`); the live window
/// follows at once, a press past an end changes nothing and saves nothing, and an index outside the four is logged.
fn step_text_size(window: &AppWindow, index: i32, step: i32) {
    let Some(element) = TextElement::from_index(index) else {
        tracing::warn!(index, step, "ignored a step of a text size stepper that does not exist");
        return;
    };
    let current = read_state(|state| return state.text_sizes);
    let next = current.stepped(element, step);
    if next == current {
        tracing::debug!(?element, step, px = current.get(element), "the text size is at its limit; nothing changes");
        return;
    }
    tracing::info!(?element, from = current.get(element), to = next.get(element), "stepped a text size");
    apply(window, next);
}

/// What: `reset_text_sizes` puts all four sizes back to their defaults, saves them and reprojects the window.
/// Why: one action restores the sizes the app always drew; at the defaults there is nothing to do and nothing is saved.
fn reset_text_sizes(window: &AppWindow) {
    let current = read_state(|state| return state.text_sizes);
    if current.is_default() {
        tracing::debug!("the text sizes are at their defaults; nothing to reset");
        return;
    }
    tracing::info!(?current, "reset the text sizes");
    apply(window, TextSizes::default());
}

/// What: `apply` keeps `sizes` in the state, shows them in the window and saves them to the session.
/// Why: both paths that change a size end the same way: the window shows the new sizes before the save is done, a failed
/// save is logged and leaves the sizes in force for this run, and the end-to-end run reads the sizes from the log line.
fn apply(window: &AppWindow, sizes: TextSizes) {
    update_state(|state| {
        state.text_sizes = sizes;
    });
    project(window);
    if let Err(error) = SAVER.get()(sizes) {
        tracing::warn!(?error, "failed to save the Text size setting");
    }
    crate::ui_state_log::emit(window, "text-size");
}

/// What: `project` writes the four sizes and the limits of the stepper into the `TextSize` global and tells the Settings
/// page whether all four are at their defaults.
/// Why: the five texts and the stepper rows read the global, so one write changes them all in the live window.
pub(crate) fn project_sizes(window: &AppWindow) {
    let sizes = read_state(|state| return state.text_sizes);
    let global = window.global::<TextSize>();
    global.set_pane_title(f32::from(sizes.get(TextElement::PaneTitle)));
    global.set_group_heading(f32::from(sizes.get(TextElement::GroupHeading)));
    global.set_folder_name(f32::from(sizes.get(TextElement::FolderName)));
    global.set_track_title(f32::from(sizes.get(TextElement::TrackTitle)));
    global.set_min_size(f32::from(MIN_TEXT_SIZE_PX));
    global.set_max_size(f32::from(MAX_TEXT_SIZE_PX));
    window.set_text_sizes_default(sizes.is_default());
}
