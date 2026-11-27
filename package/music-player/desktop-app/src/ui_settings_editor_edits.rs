//! The editor's text,
//! insert,
//! reset and leaving callbacks.
//!
//! What:
//! `edit_text` applies one reported edit,
//! `field_mode_at` and `insert_field` insert a field row's call at the caret,
//! `reset_open_template` puts the default back,
//! and `apply_templates` and `editor_back` commit the drafts and move on.
//! Why:
//! each of these is one editor report turned into one state change,
//! so they are read together,
//! split from `src/ui_settings.rs` to stay within the line budget.

/// What:
///      `use music_player::template_editor::TemplateEditorEdit` brings the edit value
///           the editor state takes.
/// Why:
///      Every report becomes one edit,
///  and the library owns what an edit is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TemplateEditorEdit } from "music_player/template_editor";
/// ```
use music_player::template_editor::TemplateEditorEdit;

/// What:
///      `use crate::AppWindow` brings the window whose editor is edited.
/// Why:
///      Every callback here reports through the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { AppWindow } from "./app.slint";
/// ```
use crate::AppWindow;
/// What:
///      `use crate::ui_screen::{...}` brings the projection and the state
///           accessors.
/// Why:
///      Each callback writes the state and re-projects the window.
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
///      Leaving the editor sets the route,
///  and the field lookup reads the state.
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
///      `use crate::ui_templates::utf16_offset` brings the byte to UTF-16 offset
///           conversion the editor reports need.
/// Why:
///      The editor reports byte offsets,
///  and the template state counts UTF-16 code units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { utf16_offset } from "./ui_templates";
/// ```
use crate::ui_templates::utf16_offset;

/// What:
///      `use super::commit_and_projection::{...}` brings the commit and the two editor
///           accessors.
/// Why:
///      Saving what applies and remembering what does not is that child module's
///           job,
///  and every callback here ends with one of its calls.
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
/// `edit_text` applies one text edit with byte offsets from the editor.
/// Why:
/// the editor reports byte offsets,
/// and the template state expects UTF-16 code units;
/// the callback also fires for caret moves,
/// so a repeated report leaves the state unchanged.
pub(super) fn edit_text(
    window: &AppWindow,
    text: &str,
    start_bytes: usize,
    end_bytes: usize,
) {
    if let Some(editor) = current_editor() {
        store_editor(editor.with_edit(TemplateEditorEdit {
            text: text.to_string(),
            selection_start: utf16_offset(text, start_bytes),
            selection_end: utf16_offset(text, end_bytes),
        }));
    }
    project(window);
}

/// What:
/// `field_mode_at` gives the mode of the field row `index` of the open editor,
/// or nothing when the row does not exist or no editor is open.
/// Why:
/// a tap reports a row number from the window,
/// which may be stale by the time it arrives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldModeAt = (state: ScreenState, index: number): string | null => state.editor?.fields[index]?.mode ?? null;
/// ```
fn field_mode_at(
    state: &ScreenState,
    index: i32,
) -> Option<&'static str> {
    let position = usize::try_from(index).ok()?;
    let editor = state.editor.as_ref()?;
    return match editor.fields().get(position) {
        Some(field) => Some(field.mode),
        None => None,
    };
}

/// What:
/// `insert_field` inserts the call of the field row `index` at the caret.
/// Why:
/// a tap on a field row inserts its call (D96).
pub(super) fn insert_field(
    window: &AppWindow,
    index: i32,
) {
    let field_mode = field_mode_at(&read_screen(), index);
    if let Some(mode) = field_mode {
        if let Some(editor) = current_editor() {
            store_editor(editor.insert_field(mode));
        }
        project(window);
    }
}

/// What:
/// `reset_open_template` resets the open template to its default and shows it.
/// Why:
/// the Reset control and Ctrl+Shift+Backspace (D106) do the same thing.
pub(crate) fn reset_open_template(window: &AppWindow) {
    if let Some(editor) = current_editor() {
        store_editor(editor.reset_to_default());
    }
    project(window);
}

/// What:
/// `apply_templates` applies and saves the drafts without leaving Settings.
/// Why:
/// Ctrl+Enter in the two-pane Settings page applies the open template and returns to the list (D106).
pub(crate) fn apply_templates(window: &AppWindow) {
    commit_templates();
    project(window);
}

/// What:
/// `editor_back` saves the valid templates to the session file and returns to Settings.
/// Why:
/// a template that does not apply keeps its previously applied text,
/// so the rows never show a broken template.
pub(crate) fn editor_back(window: &AppWindow) {
    commit_templates();
    {
        let mut state = write_screen();
        state.route = Route::Settings;
    }
    project(window);
}
