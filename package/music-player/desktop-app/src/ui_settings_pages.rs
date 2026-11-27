//! Opening,
//! closing and switching the Settings page and the template editor.
//!
//! What:
//! `build_editor` starts the editor on the queue,
//! `open_settings`,
//! `open_template`,
//! `select_template`,
//! `close_settings`,
//! `toggle_auto_play_on_select` and `layout_changed` move between the two pages
//! and the two forms,
//! and each one ends by projecting the window.
//! Why:
//! every route change goes through these functions,
//! so they are read together,
//! split from `src/ui_settings.rs` to stay within the line budget.

/// What:
///      `use music_player::template_editor::TemplateEditorEdit` brings the edit value
///           the editor state takes.
/// Why:
///      Starting the editor opens a template with a draft,
///  which the library states as one edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TemplateEditorEdit } from "music_player/template_editor";
/// ```
use music_player::template_editor::TemplateEditorEdit;
/// What:
///      `use music_player::template_editor_state::TemplateEditorState` brings the editor
///           value the state holds while a page is open.
/// Why:
///      Both pages read the same editor value,
///  so the window binding keeps it in one type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TemplateEditorState } from "music_player/template_editor_state";
/// ```
use music_player::template_editor_state::TemplateEditorState;
/// What:
///      `use music_player::template_model::TemplateKind` brings the two templates a row
///           opens.
/// Why:
///      A Settings row names one template,
///  and its row index is converted to this kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TemplateKind } from "music_player/template_model";
/// ```
use music_player::template_model::TemplateKind;
/// What:
///      `use music_player::track_measure::lookup_by_name` brings the measures the editor
///           previews with.
/// Why:
///      The editor shows one real track's values,
///  and those are looked up by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { lookup_by_name } from "music_player/track_measure";
/// ```
use music_player::track_measure::lookup_by_name;

/// What:
///      `use crate::AppWindow` brings the window whose pages are opened and closed.
/// Why:
///      Every function here takes the window and ends by projecting it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { AppWindow } from "./app.slint";
/// ```
use crate::AppWindow;
/// What:
///      `use crate::ui_screen::{...}` brings the projection and the two state
///           accessors.
/// Why:
///      Each route change writes the state and re-projects the window,
///  and those are the only two steps it takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { project, read_screen, write_screen } from "./ui_screen";
/// ```
use crate::ui_screen::{
    project,
    read_screen,
    write_screen,
};
/// What:
///      `use crate::ui_screen_view::{...}` brings the route and the screen state.
/// Why:
///      A route change is one assignment on the state,
///  and the editor is built from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Route, ScreenState } from "./ui_screen_view";
/// ```
use crate::ui_screen_view::{
    Route,
    ScreenState,
};
/// What:
///      `use crate::ui_settings_model::kind_of_index` brings the row-index to
///           template-kind conversion.
/// Why:
///      A tap on a Settings row reports a number,
///  and the editor opens by kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { kind_of_index } from "./ui_settings_model";
/// ```
use crate::ui_settings_model::kind_of_index;
/// What:
///      `use crate::ui_templates::editor_library` brings the field and function list
///           of the editor.
/// Why:
///      The editor is started on that library,
///  and nothing else in this file needs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { editor_library } from "./ui_templates";
/// ```
use crate::ui_templates::editor_library;
/// What:
///      `use slint::ComponentHandle` brings the trait whose methods reach the window's
///           callbacks and its global values.
/// Why:
///      `as_weak`,
///  the `on_` callbacks and `global` are trait methods,
///  so the trait must be in scope although no line names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { ComponentHandle } from "slint";
/// ```
use slint::ComponentHandle;

/// What:
///      `use super::commit_and_projection::{...}` brings the commit and the two editor
///           accessors.
/// Why:
///      Leaving a template applies and saves it,
///  and those rules live in the child module of the parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { commit_templates, current_editor, store_editor } from "./ui_settings_commit_and_projection";
/// ```
use super::commit_and_projection::{
    commit_templates,
    current_editor,
    store_editor,
};
/// What:
///      `use super::{...}` brings the drafts kept for the next editor,
///  the Auto-play on select saver and the function that reads the saved templates.
/// Why:
///      Those values stay private to the parent module,
///  and this child is where they are read and written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KEPT_DRAFTS, AUTO_PLAY_SAVER } from "./ui_settings";
/// ```
use super::{
    AUTO_PLAY_SAVER,
    KEPT_DRAFTS,
};

/// What:
/// `build_editor` starts the editor on the queue with the saved templates applied.
/// Why:
/// a saved template that still applies becomes the editor's draft and applied text;
/// others fall back to the default.
pub(crate) fn build_editor(state: &ScreenState) -> TemplateEditorState {
    let measures = lookup_by_name(&state.names, &state.paths, &state.measures);
    let mut editor = TemplateEditorState::initial(&editor_library(&state.names, &measures));
    for (kind, saved) in [
        (
            TemplateKind::TrackRows,
            state.track_rows_template.as_deref(),
        ),
        (
            TemplateKind::PlayingTrack,
            state.playing_template.as_deref(),
        ),
    ] {
        if let Some(text) = saved {
            let length = text.encode_utf16().count();
            editor = editor.open(kind).with_edit(TemplateEditorEdit {
                text: text.to_string(),
                selection_start: length,
                selection_end: length,
            });
        }
    }
    // A draft that did not apply when the editor was left comes back as the draft; it does not change what is applied.
    let (kept_track_rows, kept_playing) = KEPT_DRAFTS.with(std::cell::RefCell::clone).into_inner();
    for (kind, kept) in [
        (TemplateKind::TrackRows, kept_track_rows),
        (TemplateKind::PlayingTrack, kept_playing),
    ] {
        if let Some(text) = kept {
            let length = text.encode_utf16().count();
            tracing::debug!(?kind, "restoring a draft that did not apply");
            editor = editor.open(kind).with_edit(TemplateEditorEdit {
                text,
                selection_start: length,
                selection_end: length,
            });
        }
    }
    return editor.open(TemplateKind::TrackRows);
}

/// What:
/// `open_settings` opens the Settings route with the template rows.
/// Why:
/// Settings shows the applied line of each template,
/// so the editor value is built from the saved texts.
pub(crate) fn open_settings(window: &AppWindow) {
    window.set_settings_start_on_licenses(false);
    {
        let mut state = write_screen();
        state.editor = Some(build_editor(&state));
        state.route = Route::Settings;
    }
    project(window);
}

/// What:
/// `open_template` opens the editor on the template of Settings row `index`.
/// Why:
/// a tap on a Templates row opens that template's editor page.
pub(super) fn open_template(
    window: &AppWindow,
    index: i32,
) {
    let Some(kind) = usize::try_from(index).ok().and_then(kind_of_index) else {
        return;
    };
    if let Some(editor) = current_editor() {
        store_editor(editor.open(kind));
    }
    {
        let mut state = write_screen();
        state.route = Route::TemplateEditor;
    }
    project(window);
}

/// What:
/// `select_template` shows the editor of Settings row `index` in the right pane without leaving Settings.
/// Why:
/// on a wide window a row selects instead of navigating (D110);
/// the template being left is applied and saved
/// first,
/// exactly as leaving its editor page does,
/// so no draft is lost by choosing the other row.
pub(super) fn select_template(
    window: &AppWindow,
    index: i32,
) {
    let Some(kind) = usize::try_from(index).ok().and_then(kind_of_index) else {
        tracing::warn!(
            index,
            "ignored a selection of a Settings row that does not exist"
        );
        return;
    };
    let already_selected = match read_screen().editor.as_ref() {
        Some(editor) => editor.kind() == kind,
        None => false,
    };
    if already_selected {
        tracing::debug!(
            ?kind,
            "the selected Settings row was chosen again; nothing changes"
        );
        return;
    }
    tracing::info!(?kind, "selected a Settings row");
    commit_templates();
    if let Some(editor) = current_editor() {
        store_editor(editor.open(kind));
    }
    project(window);
}

/// What:
/// `close_settings` applies and saves the templates,
/// then returns from Settings to the player.
/// Why:
/// Back and Escape close Settings in both forms (D110);
/// on a wide window the drafts were never left through an
/// editor page,
/// so closing is where they are saved.
/// The editor value is dropped afterwards.
pub(crate) fn close_settings(window: &AppWindow) {
    tracing::info!("closing Settings");
    commit_templates();
    {
        let mut state = write_screen();
        state.editor = None;
        state.route = Route::Player;
    }
    project(window);
}

/// What:
/// `toggle_auto_play_on_select` flips Auto-play on select,
/// saves it and reprojects the switch row.
/// Why:
/// the row toggles in place (D129):
/// the route,
/// the selected template and the editor are left as they are,
/// and the
/// new value governs the next click,
/// Enter or Search result at once.
pub(super) fn toggle_auto_play_on_select(window: &AppWindow) {
    let enabled = !read_screen().auto_play_on_select;
    {
        let mut state = write_screen();
        state.auto_play_on_select = enabled;
    }
    tracing::info!(enabled, "toggled Auto-play on select");
    if let Err(error) = AUTO_PLAY_SAVER.get()(enabled) {
        tracing::warn!(?error, "failed to save the Auto-play on select setting");
    }
    project(window);
    // The end-to-end run reads the setting from this line: the switch row handles Space itself, so no key line follows.
    crate::ui_state_log::emit(window, "auto-play");
}

/// What:
/// `layout_changed` reacts to the window crossing the two-pane width.
/// Why:
/// a template editor page only exists in the single form;
/// when the window widens while it is open,
/// the same editor
/// continues in the right pane of the two-pane Settings,
/// so the route becomes Settings and the template stays selected.
pub(super) fn layout_changed(
    window: &AppWindow,
    wide: bool,
) {
    let on_editor_page = {
        let state = read_screen();
        state.route == Route::TemplateEditor
    };
    tracing::debug!(
        wide,
        on_editor_page,
        "window crossed the Settings width switch"
    );
    if wide && on_editor_page {
        {
            let mut state = write_screen();
            state.route = Route::Settings;
        }
        project(window);
    }
    if !wide {
        // Narrowing destroys the right pane's text field, which may never report that it lost focus, and a focus left
        // set would make Escape on the list page look like typing in a template line.
        if let Some(editor) = current_editor() {
            store_editor(editor.with_focus(false));
        }
        // The template line's focus zone went with it.
        let focus = window.global::<crate::Focus>();
        if focus.get_zone() == 7 {
            focus.set_zone(0);
            focus.set_label("".into());
        }
    }
    // The end-to-end run waits for the layout it asked for before it presses keys.
    crate::ui_state_log::emit(window, "layout");
}
