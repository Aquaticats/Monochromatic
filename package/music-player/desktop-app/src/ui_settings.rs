//! Window binding for the Settings page and the template editor.
//!
//! What:
//! opens and closes Settings and the editor,
//! routes the editor's edits into `TemplateEditorState`,
//! and saves the applied templates to the session file when the editor closes.
//! Why:
//! the editor rules live in `template_editor_state`;
//! this module keeps the editor value in the screen
//! state while either page is open and applies the valid templates to the track rows and the deck.

/// Imports the supporting text of the auto-play setting.
use music_player::row_activation::auto_play_on_select_supporting_text;
/// Imports the session that stores the settings.
use music_player::session::Session;
/// Imports the template editor state that the page edits.
use music_player::template_editor_state::TemplateEditorState;
/// Imports the template kind that a row opens.
use music_player::template_model::TemplateKind;
/// Imports the component handle that reaches the window.
use slint::ComponentHandle;

/// Imports the model builder,
/// the projection and the state accessors of the screen.
use crate::ui_screen::{
    model_of,
    project,
    read_screen,
    write_screen,
};
/// Imports the screen state the editor is held in.
use crate::ui_screen_view::ScreenState;
/// Imports the pure rows and editor view and the index conversion.
use crate::ui_settings_model::{
    EditorView,
    SettingsRow,
    editor_view,
    index_of_kind,
    settings_rows,
};
/// Imports the window and the generated settings row types.
use crate::{
    AppWindow,
    EditorFieldRow,
    EditorPreviewRow,
    SettingsTemplateRow,
};

/// What:
/// `SaveTemplates` is the function that persists both template texts.
/// Why:
/// the production function writes the user's session file,
/// so the window tests swap in one that records the texts.
type SaveTemplates = fn(Option<&str>, Option<&str>) -> std::io::Result<()>;

/// What:
/// `SaveAutoPlay` is the function that persists the Auto-play on select setting.
/// Why:
/// the production function writes the user's session file,
/// so the window tests swap in one that records the value.
type SaveAutoPlay = fn(bool) -> std::io::Result<()>;

thread_local! {
    /// What:
    /// the text of each template whose draft did not apply when the editor was last left,
    /// as (track rows,
    /// playing track).
    /// Why:
    /// leaving the editor rebuilds it from the applied texts,
    /// which used to drop a draft that did not apply without a
    /// word;
    /// the draft is kept here for the rest of the run (it is never saved to the session,
    /// because it does not apply),
    /// and the next editor shows it again with its mistakes and the note that the rows keep the last valid template.
    static KEPT_DRAFTS: std::cell::RefCell<(Option<String>, Option<String>)> = const { std::cell::RefCell::new((None, None)) };
    /// What:
    /// the persistence function of this UI thread.
    /// Why:
    /// only the event-loop thread saves templates,
    /// and tests replace it per test thread.
    static SAVER: std::cell::Cell<SaveTemplates> = const { std::cell::Cell::new(Session::save_templates) };
    /// What:
    /// the persistence function of the Auto-play on select setting on this UI thread.
    /// Why:
    /// only the event-loop thread saves it,
    /// and tests replace it per test thread so a click never writes the user's file.
    static AUTO_PLAY_SAVER: std::cell::Cell<SaveAutoPlay> = const { std::cell::Cell::new(Session::save_auto_play_on_select) };
}

/// What:
/// `replace_saver` makes this thread persist templates through `saver`.
/// Why:
/// a test that edits a template must never write the user's session file.
#[cfg(test)]
pub(crate) fn replace_saver(saver: SaveTemplates) {
    SAVER.set(saver);
}

/// What:
/// `replace_auto_play_saver` makes this thread persist the Auto-play on select setting through `saver`.
/// Why:
/// a test that clicks the switch must never write the user's session file.
#[cfg(test)]
pub(crate) fn replace_auto_play_saver(saver: SaveAutoPlay) {
    AUTO_PLAY_SAVER.set(saver);
}

/// What:
/// `wire` registers the Settings and editor callbacks and loads the saved templates,
/// the Auto-play on select
/// setting and the text sizes into the state.
/// Why:
/// both must be in the state before the first projection,
/// so the rows show the templates,
/// the switch shows its
/// state,
/// and the first click on a track already follows the saved setting.
pub(crate) fn wire(app: &AppWindow) {
    let (track_rows, playing) = Session::load_templates();
    let auto_play_on_select = Session::load_auto_play_on_select();
    tracing::info!(
        auto_play_on_select,
        "loaded the Auto-play on select setting"
    );
    crate::ui_text_size::load_into_state();
    {
        let mut state = write_screen();
        state.track_rows_template = track_rows;
        state.playing_template = playing;
        state.auto_play_on_select = auto_play_on_select;
    }
    wire_callbacks(app);
}

/// What:
/// `wire_callbacks` registers the Settings page's callbacks and the editor's.
/// Why:
/// split from `wire` so the window tests register the same callbacks without reading the user's session file.
pub(crate) fn wire_callbacks(app: &AppWindow) {
    let open_weak = app.as_weak();
    app.on_open_settings(move || {
        if let Some(window) = open_weak.upgrade() {
            open_settings(&window);
        }
    });
    let template_weak = app.as_weak();
    app.on_settings_open_template(move |index| {
        if let Some(window) = template_weak.upgrade() {
            open_template(&window, index);
        }
    });
    let select_weak = app.as_weak();
    app.on_settings_select_template(move |index| {
        if let Some(window) = select_weak.upgrade() {
            select_template(&window, index);
        }
    });
    let back_weak = app.as_weak();
    app.on_settings_back(move || {
        if let Some(window) = back_weak.upgrade() {
            close_settings(&window);
        }
    });
    let toggle_weak = app.as_weak();
    app.on_settings_toggle_auto_play(move || {
        if let Some(window) = toggle_weak.upgrade() {
            toggle_auto_play_on_select(&window);
        }
    });
    let layout_weak = app.as_weak();
    app.on_layout_changed(move |wide| {
        if let Some(window) = layout_weak.upgrade() {
            layout_changed(&window, wide);
        }
    });
    crate::ui_licenses::wire_callbacks(app);
    crate::ui_text_size::wire_callbacks(app);
    wire_editor(app);
}

/// What:
/// `wire_editor` registers the editor's text,
/// focus,
/// insert,
/// reset and back callbacks.
/// Why:
/// the editor reports every change,
/// and each change is applied to the state and reprojected.
fn wire_editor(app: &AppWindow) {
    let text_weak = app.as_weak();
    app.on_editor_text_edited(move |text, start, end| {
        if let Some(window) = text_weak.upgrade() {
            edit_text(
                &window,
                text.as_str(),
                usize::try_from(start).unwrap_or(0),
                usize::try_from(end).unwrap_or(0),
            );
        }
    });
    let focus_weak = app.as_weak();
    app.on_editor_focus_changed(move |focused| {
        if let Some(window) = focus_weak.upgrade() {
            if let Some(editor) = current_editor() {
                store_editor(editor.with_focus(focused));
            }
            project(&window);
        }
    });
    let insert_weak = app.as_weak();
    app.on_editor_insert_field(move |index| {
        if let Some(window) = insert_weak.upgrade() {
            insert_field(&window, index);
        }
    });
    let reset_weak = app.as_weak();
    app.on_editor_reset(move || {
        if let Some(window) = reset_weak.upgrade() {
            reset_open_template(&window);
        }
    });
    let back_weak = app.as_weak();
    app.on_editor_back(move || {
        if let Some(window) = back_weak.upgrade() {
            editor_back(&window);
        }
    });
}
/// What:
///      `mod ui_settings_pages;` loads the sibling `src/ui_settings_pages.rs`,
///  the functions that open,
///  close and switch the Settings page and the editor.
/// Why:
///       Every route change goes through them,
///           and this module keeps the wiring and the saved values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiSettingsPages from "./ui_settings_pages";
/// ```
#[path = "ui_settings_pages.rs"]
mod ui_settings_pages;

/// What:
///      `pub(crate) use ui_settings_pages::{...}` re-exports the three page entry
///           points other modules call at this module's path.
/// Why:
///       The key actions and the window tests name `ui_settings::open_settings` and
///           its siblings,
///  and only where they are declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { build_editor, close_settings, open_settings } from "./ui_settings_pages";
/// ```
pub(crate) use ui_settings_pages::{
    build_editor,
    close_settings,
    open_settings,
};

/// What:
///      `mod ui_settings_editor_edits;` loads the sibling
///           `src/ui_settings_editor_edits.rs`,
///  the editor's text,
///  insert,
///  reset and leaving callbacks.
/// Why:
///       Each is one editor report turned into one state change,
///  and the projections in `ui_settings_pages` are the other half.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiSettingsEditorEdits from "./ui_settings_editor_edits";
/// ```
#[path = "ui_settings_editor_edits.rs"]
mod ui_settings_editor_edits;

/// What:
///      `pub(crate) use ui_settings_editor_edits::{...}` re-exports the three editing
///           entry points the key actions call.
/// Why:
///       Ctrl+Enter,
///  Ctrl+Shift+Backspace and Escape on the editor page reach them as
///  `ui_settings::apply_templates` and its siblings,
///  and only where they are declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { apply_templates, editor_back, reset_open_template } from "./ui_settings_editor_edits";
/// ```
pub(crate) use ui_settings_editor_edits::{
    apply_templates,
    editor_back,
    reset_open_template,
};

/// What:
///      `use ui_settings_pages::{...}` brings the four callbacks that `wire_callbacks`
///           registers.
/// Why:
///       Opening a template,
///  selecting one,
///  toggling the setting and the layout switch are handled in the sibling file,
///  and this module only connects them to the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { layout_changed, open_template, select_template, toggle_auto_play_on_select } from "./ui_settings_pages";
/// ```
use ui_settings_pages::{
    layout_changed,
    open_template,
    select_template,
    toggle_auto_play_on_select,
};

/// What:
///      `use ui_settings_editor_edits::{...}` brings the two callbacks that
///           `wire_editor` registers.
/// Why:
///       The editor's text and field reports are handled in the sibling file,
///  and this module only connects them to the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { edit_text, insert_field } from "./ui_settings_editor_edits";
/// ```
use ui_settings_editor_edits::{
    edit_text,
    insert_field,
};

/// What:
///      `#[path = "ui_settings_commit_and_projection.rs"] mod commit_and_projection;` declares a child module
///           whose code lives in the sibling file `ui_settings_commit_and_projection.rs`.
/// Why:
///       The commit rules and the page projection fill a file of their own,
///  and a child module reaches the private saver and kept drafts they use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // uiSettingsCommitAndProjection.ts: more functions of the same module
/// ```
#[path = "ui_settings_commit_and_projection.rs"]
mod commit_and_projection;

/// Re-exports the page projection,
/// which the screen projection calls.
pub(crate) use commit_and_projection::project_pages;

/// Imports the editor value and its setter that the focus callback uses.
use commit_and_projection::{
    current_editor,
    store_editor,
};

/// Unit tests of `ui_settings`,
/// in the sidecar file `ui_settings_tests.rs`.
/// Role of the module:
/// window binding for the Settings page and the template editor.
#[cfg(test)]
#[path = "ui_settings_tests.rs"]
mod tests;
