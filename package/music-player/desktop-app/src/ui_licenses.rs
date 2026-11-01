//! Window binding for the Licenses page that the Licenses row of Settings opens.
//!
//! What: opens and closes the Licenses route and hands the page the embedded notice text one line at a time.
//! Why: the notice text is about 2 MB and 40,000 lines; copying it into a Slint string, or one string per line, at startup
//! or on every projection would cost time on a page most runs never open. The page gets a model that slices the embedded
//! text on demand, built when the page opens and released when it closes.

use std::time::Instant;

use music_player::licenses::NOTICES;
use slint::{ComponentHandle, Model, ModelRc, ModelTracker, SharedString};

use crate::ui_screen::{project, read_state, update_state};
use crate::ui_screen_view::Route;
use crate::AppWindow;

/// What: `NoticeLines` is a read-only Slint model whose rows are the lines of a text, sliced when a row is read.
/// Why: the ListView of the Licenses page reads only the rows in view, so only those lines are ever copied.
pub(crate) struct NoticeLines {
    /// The text the rows slice.
    text: &'static str,
    /// Byte offset where each line starts; a line ends at the next newline.
    starts: Vec<usize>,
}

/// Methods of `NoticeLines`.
impl NoticeLines {
    /// What: `new` records where each line of `text` starts.
    /// Why: one pass over the bytes is the whole cost of opening the page; a final newline ends the last line instead of
    /// starting an empty one.
    pub(crate) fn new(full_text: &'static str) -> Self {
        let text = full_text.strip_suffix('\n').unwrap_or(full_text);
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(index, _)| return index + 1));
        return Self { text, starts };
    }
}

/// Model view of `NoticeLines`.
impl Model for NoticeLines {
    type Data = SharedString;

    fn row_count(&self) -> usize {
        return self.starts.len();
    }

    fn row_data(&self, row: usize) -> Option<SharedString> {
        let start = *self.starts.get(row)?;
        let end = self.starts.get(row + 1).map_or(self.text.len(), |next| return next - 1);
        return Some(SharedString::from(self.text[start..end].trim_end_matches('\r')));
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        return &();
    }
}

/// What: `wire_callbacks` registers the Licenses row and the Licenses page's Back.
/// Why: split from the app's start so the window tests register the same callbacks.
pub(crate) fn wire_callbacks(app: &AppWindow) {
    let open_weak = app.as_weak();
    app.on_settings_open_licenses(move || {
        if let Some(window) = open_weak.upgrade() {
            open_licenses(&window);
        }
    });
    let back_weak = app.as_weak();
    app.on_licenses_back(move || {
        if let Some(window) = back_weak.upgrade() {
            close_licenses(&window);
        }
    });
}

/// What: `open_licenses` shows the Licenses page over Settings and gives it the notice lines.
/// Why: the Licenses row of Settings opens it; the template editor value and its selection stay in the state, so Back
/// returns to the same Settings page. The right pane's text field is destroyed with the Settings page, which may not report
/// that it lost focus, so the editor value and the focus zone are cleared of it.
pub(crate) fn open_licenses(window: &AppWindow) {
    if read_state(|state| return state.route) != Route::Settings {
        tracing::warn!("the Licenses row was activated while Settings was not open; nothing opens");
        return;
    }
    let started = Instant::now();
    let lines = NoticeLines::new(NOTICES);
    let line_count = lines.row_count();
    window.set_licenses_lines(ModelRc::new(lines));
    window.set_settings_start_on_licenses(false);
    update_state(|state| {
        if let Some(editor) = state.editor.as_ref() {
            state.editor = Some(editor.with_focus(false));
        }
        state.route = Route::Licenses;
    });
    let focus = window.global::<crate::Focus>();
    if focus.get_zone() == 7 {
        focus.set_zone(0);
        focus.set_label("".into());
    }
    project(window);
    tracing::info!(line_count, build_us = started.elapsed().as_micros(), "opened the Licenses page");
    // The Licenses row handles Enter and Space itself, so no key line follows; the end-to-end run reads the route from this.
    crate::ui_state_log::emit(window, "licenses");
}

/// What: `close_licenses` returns from the Licenses page to Settings and releases the notice lines.
/// Why: Back and Escape return to the page that opened it, with the keyboard on the Licenses row; the lines are dropped so
/// a closed page holds no text.
pub(crate) fn close_licenses(window: &AppWindow) {
    tracing::info!("closing the Licenses page");
    window.set_licenses_lines(ModelRc::default());
    window.set_settings_start_on_licenses(true);
    update_state(|state| {
        state.route = Route::Settings;
    });
    project(window);
    crate::ui_state_log::emit(window, "licenses-closed");
}

/// Unit tests in `ui_licenses_tests.rs`, pulled in at this point.
#[cfg(test)]
#[path = "ui_licenses_tests.rs"]
mod tests;
