//! Window binding for the Settings page and the template editor.
//!
//! What: opens and closes Settings and the editor, routes the editor's edits into `TemplateEditorState`,
//! and saves the applied templates to the session file when the editor closes.
//! Why: the editor rules live in `template_editor_state`; this module keeps the editor value in the screen
//! state while either page is open and applies the valid templates to the track rows and the deck.

use music_player::row_activation::auto_play_on_select_supporting_text;
use music_player::session::Session;
use music_player::template_editor_state::TemplateEditorState;
use music_player::template_editor::TemplateEditorEdit;
use music_player::template_model::TemplateKind;
use music_player::track_measure::lookup_by_name;
use slint::ComponentHandle;

use crate::ui_screen::{model_of, project, read_state, update_state};
use crate::ui_screen_view::{Route, ScreenState};
use crate::ui_settings_model::{editor_view, index_of_kind, kind_of_index, settings_rows, EditorView, SettingsRow};
use crate::ui_templates::{editor_library, utf16_offset};
use crate::{AppWindow, EditorFieldRow, EditorPreviewRow, SettingsTemplateRow};

/// What: `SaveTemplates` is the function that persists both template texts.
/// Why: the production function writes the user's session file, so the window tests swap in one that records the texts.
type SaveTemplates = fn(Option<&str>, Option<&str>) -> std::io::Result<()>;

/// What: `SaveAutoPlay` is the function that persists the Auto-play on select setting.
/// Why: the production function writes the user's session file, so the window tests swap in one that records the value.
type SaveAutoPlay = fn(bool) -> std::io::Result<()>;

thread_local! {
    /// What: the text of each template whose draft did not apply when the editor was last left, as (track rows, playing track).
    /// Why: leaving the editor rebuilds it from the applied texts, which used to drop a draft that did not apply without a
    /// word; the draft is kept here for the rest of the run (it is never saved to the session, because it does not apply),
    /// and the next editor shows it again with its mistakes and the note that the rows keep the last valid template.
    static KEPT_DRAFTS: std::cell::RefCell<(Option<String>, Option<String>)> = const { std::cell::RefCell::new((None, None)) };
    /// What: the persistence function of this UI thread.
    /// Why: only the event-loop thread saves templates, and tests replace it per test thread.
    static SAVER: std::cell::Cell<SaveTemplates> = const { std::cell::Cell::new(Session::save_templates) };
    /// What: the persistence function of the Auto-play on select setting on this UI thread.
    /// Why: only the event-loop thread saves it, and tests replace it per test thread so a click never writes the user's file.
    static AUTO_PLAY_SAVER: std::cell::Cell<SaveAutoPlay> = const { std::cell::Cell::new(Session::save_auto_play_on_select) };
}

/// What: `replace_saver` makes this thread persist templates through `saver`.
/// Why: a test that edits a template must never write the user's session file.
#[cfg(test)]
pub(crate) fn replace_saver(saver: SaveTemplates) {
    SAVER.set(saver);
}

/// What: `replace_auto_play_saver` makes this thread persist the Auto-play on select setting through `saver`.
/// Why: a test that clicks the switch must never write the user's session file.
#[cfg(test)]
pub(crate) fn replace_auto_play_saver(saver: SaveAutoPlay) {
    AUTO_PLAY_SAVER.set(saver);
}

/// What: `wire` registers the Settings and editor callbacks and loads the saved templates, the Auto-play on select
/// setting and the text sizes into the state.
/// Why: both must be in the state before the first projection, so the rows show the templates, the switch shows its
/// state, and the first click on a track already follows the saved setting.
pub(crate) fn wire(app: &AppWindow) {
    let (track_rows, playing) = Session::load_templates();
    let auto_play_on_select = Session::load_auto_play_on_select();
    tracing::info!(auto_play_on_select, "loaded the Auto-play on select setting");
    crate::ui_text_size::load_into_state();
    update_state(|state| {
        state.track_rows_template = track_rows;
        state.playing_template = playing;
        state.auto_play_on_select = auto_play_on_select;
    });
    wire_callbacks(app);
}

/// What: `wire_callbacks` registers the Settings page's callbacks and the editor's.
/// Why: split from `wire` so the window tests register the same callbacks without reading the user's session file.
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

/// What: `wire_editor` registers the editor's text, focus, insert, reset and back callbacks.
/// Why: the editor reports every change, and each change is applied to the state and reprojected.
fn wire_editor(app: &AppWindow) {
    let text_weak = app.as_weak();
    app.on_editor_text_edited(move |text, start, end| {
        if let Some(window) = text_weak.upgrade() {
            edit_text(&window, text.as_str(), usize::try_from(start).unwrap_or(0), usize::try_from(end).unwrap_or(0));
        }
    });
    let focus_weak = app.as_weak();
    app.on_editor_focus_changed(move |focused| {
        if let Some(window) = focus_weak.upgrade() {
            update_editor(|editor| return editor.with_focus(focused));
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

/// What: `build_editor` starts the editor on the queue with the saved templates applied.
/// Why: a saved template that still applies becomes the editor's draft and applied text; others fall back to the default.
pub(crate) fn build_editor(state: &ScreenState) -> TemplateEditorState {
    let measures = lookup_by_name(&state.names, &state.paths, &state.measures);
    let mut editor = TemplateEditorState::initial(&editor_library(&state.names, &measures));
    for (kind, saved) in [
        (TemplateKind::TrackRows, state.track_rows_template.as_deref()),
        (TemplateKind::PlayingTrack, state.playing_template.as_deref()),
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
    let (kept_track_rows, kept_playing) = KEPT_DRAFTS.with(|cell| return cell.borrow().clone());
    for (kind, kept) in [(TemplateKind::TrackRows, kept_track_rows), (TemplateKind::PlayingTrack, kept_playing)] {
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

/// What: `open_settings` opens the Settings route with the template rows.
/// Why: Settings shows the applied line of each template, so the editor value is built from the saved texts.
pub(crate) fn open_settings(window: &AppWindow) {
    window.set_settings_start_on_licenses(false);
    update_state(|state| {
        state.editor = Some(build_editor(state));
        state.route = Route::Settings;
    });
    project(window);
}

/// What: `open_template` opens the editor on the template of Settings row `index`.
/// Why: a tap on a Templates row opens that template's editor page.
fn open_template(window: &AppWindow, index: i32) {
    let Some(kind) = usize::try_from(index).ok().and_then(kind_of_index) else {
        return;
    };
    update_editor(|editor| return editor.open(kind));
    update_state(|state| {
        state.route = Route::TemplateEditor;
    });
    project(window);
}

/// What: `select_template` shows the editor of Settings row `index` in the right pane without leaving Settings.
/// Why: on a wide window a row selects instead of navigating (D110); the template being left is applied and saved
/// first, exactly as leaving its editor page does, so no draft is lost by choosing the other row.
fn select_template(window: &AppWindow, index: i32) {
    let Some(kind) = usize::try_from(index).ok().and_then(kind_of_index) else {
        tracing::warn!(index, "ignored a selection of a Settings row that does not exist");
        return;
    };
    let already_selected = read_state(|state| return state.editor.as_ref().is_some_and(|editor| return editor.kind() == kind));
    if already_selected {
        tracing::debug!(?kind, "the selected Settings row was chosen again; nothing changes");
        return;
    }
    tracing::info!(?kind, "selected a Settings row");
    commit_templates();
    update_editor(|editor| return editor.open(kind));
    project(window);
}

/// What: `close_settings` applies and saves the templates, then returns from Settings to the player.
/// Why: Back and Escape close Settings in both forms (D110); on a wide window the drafts were never left through an
/// editor page, so closing is where they are saved. The editor value is dropped afterwards.
pub(crate) fn close_settings(window: &AppWindow) {
    tracing::info!("closing Settings");
    commit_templates();
    update_state(|state| {
        state.editor = None;
        state.route = Route::Player;
    });
    project(window);
}

/// What: `toggle_auto_play_on_select` flips Auto-play on select, saves it and reprojects the switch row.
/// Why: the row toggles in place (D129): the route, the selected template and the editor are left as they are, and the
/// new value governs the next click, Enter or Search result at once.
fn toggle_auto_play_on_select(window: &AppWindow) {
    let enabled = !read_state(|state| return state.auto_play_on_select);
    update_state(|state| {
        state.auto_play_on_select = enabled;
    });
    tracing::info!(enabled, "toggled Auto-play on select");
    if let Err(error) = AUTO_PLAY_SAVER.get()(enabled) {
        tracing::warn!(?error, "failed to save the Auto-play on select setting");
    }
    project(window);
    // The end-to-end run reads the setting from this line: the switch row handles Space itself, so no key line follows.
    crate::ui_state_log::emit(window, "auto-play");
}

/// What: `layout_changed` reacts to the window crossing the two-pane width.
/// Why: a template editor page only exists in the single form; when the window widens while it is open, the same editor
/// continues in the right pane of the two-pane Settings, so the route becomes Settings and the template stays selected.
fn layout_changed(window: &AppWindow, wide: bool) {
    let on_editor_page = read_state(|state| return state.route == Route::TemplateEditor);
    tracing::debug!(wide, on_editor_page, "window crossed the Settings width switch");
    if wide && on_editor_page {
        update_state(|state| {
            state.route = Route::Settings;
        });
        project(window);
    }
    if !wide {
        // Narrowing destroys the right pane's text field, which may never report that it lost focus, and a focus left
        // set would make Escape on the list page look like typing in a template line.
        update_editor(|editor| return editor.with_focus(false));
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

/// What: `edit_text` applies one text edit with byte offsets from the editor.
/// Why: the editor reports byte offsets, and the template state expects UTF-16 code units; the callback also fires for caret moves, so a repeated report leaves the state unchanged.
fn edit_text(window: &AppWindow, text: &str, start_bytes: usize, end_bytes: usize) {
    update_editor(|editor| {
        return editor.with_edit(TemplateEditorEdit {
            text: text.to_string(),
            selection_start: utf16_offset(text, start_bytes),
            selection_end: utf16_offset(text, end_bytes),
        });
    });
    project(window);
}

/// What: `insert_field` inserts the call of the field row `index` at the caret.
/// Why: a tap on a field row inserts its call (D96).
fn insert_field(window: &AppWindow, index: i32) {
    let field_mode = read_state(|state| {
        let position = usize::try_from(index).ok()?;
        return state.editor.as_ref()?.fields().get(position).map(|field| return field.mode);
    });
    if let Some(mode) = field_mode {
        update_editor(|editor| return editor.insert_field(mode));
        project(window);
    }
}

/// What: `reset_open_template` resets the open template to its default and shows it.
/// Why: the Reset control and Ctrl+Shift+Backspace (D106) do the same thing.
pub(crate) fn reset_open_template(window: &AppWindow) {
    update_editor(|editor| return editor.reset_to_default());
    project(window);
}

/// What: `apply_templates` applies and saves the drafts without leaving Settings.
/// Why: Ctrl+Enter in the two-pane Settings page applies the open template and returns to the list (D106).
pub(crate) fn apply_templates(window: &AppWindow) {
    commit_templates();
    project(window);
}

/// What: `editor_back` saves the valid templates to the session file and returns to Settings.
/// Why: a template that does not apply keeps its previously applied text, so the rows never show a broken template.
pub(crate) fn editor_back(window: &AppWindow) {
    commit_templates();
    update_state(|state| {
        state.route = Route::Settings;
    });
    project(window);
}

/// What: `commit_templates` applies the editor's valid drafts to the track rows and the deck, saves them when they
/// changed, and rebuilds the editor on the applied texts.
/// Why: both forms leave a template the same way: the editor page's Back, a new selection in the right pane, and closing
/// Settings. Rebuilding keeps the open template unchanged except that its draft equals what is applied.
fn commit_templates() {
    let saved = read_state(|state| {
        let editor = state.editor.as_ref()?;
        return Some((
            applied_text(editor, TemplateKind::TrackRows, state.track_rows_template.as_deref()),
            applied_text(editor, TemplateKind::PlayingTrack, state.playing_template.as_deref()),
            editor.kind(),
        ));
    });
    let Some((track_rows, playing, open_kind)) = saved else {
        tracing::debug!("no editor value to commit");
        return;
    };
    remember_drafts_that_do_not_apply();
    // A template the user never changed is stored as None and applied as its default text, which is the same template.
    let changed = read_state(|state| {
        return effective_text(TemplateKind::TrackRows, state.track_rows_template.as_deref())
            != effective_text(TemplateKind::TrackRows, track_rows.as_deref())
            || effective_text(TemplateKind::PlayingTrack, state.playing_template.as_deref())
                != effective_text(TemplateKind::PlayingTrack, playing.as_deref());
    });
    if changed {
        tracing::info!("saving the applied template texts");
        if let Err(error) = SAVER.get()(track_rows.as_deref(), playing.as_deref()) {
            tracing::warn!(?error, "failed to save the template texts");
        }
    } else {
        tracing::debug!("the template texts are unchanged; nothing to save");
    }
    update_state(|state| {
        state.track_rows_template = track_rows;
        state.playing_template = playing;
        state.editor = Some(build_editor(state).open(open_kind));
    });
}

/// What: `remember_drafts_that_do_not_apply` keeps the text of every template whose draft does not apply, and forgets the rest.
/// Why: the editor is rebuilt from the applied texts when it is left, so a draft that does not apply would otherwise vanish
/// without a word; a draft that does apply is the applied text and needs no keeping.
fn remember_drafts_that_do_not_apply() {
    let kept = read_state(|state| {
        let editor = state.editor.as_ref()?;
        let draft_of = |kind: TemplateKind| {
            let opened = editor.open(kind);
            return if opened.is_valid() { None } else { Some(opened.text().to_string()) };
        };
        return Some((draft_of(TemplateKind::TrackRows), draft_of(TemplateKind::PlayingTrack)));
    });
    if let Some(drafts) = kept {
        tracing::debug!(track_rows = drafts.0.is_some(), playing = drafts.1.is_some(), "drafts that do not apply are kept");
        KEPT_DRAFTS.with(|cell| *cell.borrow_mut() = drafts);
    }
}

/// What: `effective_text` gives the text a template actually uses: the saved one, or the default when none is saved.
/// Why: saved None and saved default text are the same template, so comparing them must not report a change.
fn effective_text(kind: TemplateKind, saved: Option<&str>) -> String {
    return saved.map_or_else(|| return kind.default_template().to_string(), str::to_string);
}

/// What: `applied_text` gives the draft of `kind` when it applies, or the previously applied text otherwise; a text equal
/// to the template's default is given as None.
/// Why: only texts that apply reach the track rows and the saved session, and a template that reads as its default is
/// stored as no text, so it keeps following the default.
fn applied_text(editor: &TemplateEditorState, kind: TemplateKind, previous: Option<&str>) -> Option<String> {
    let opened = editor.open(kind);
    if opened.is_valid() {
        let text = opened.text();
        return if text == kind.default_template() { None } else { Some(text.to_string()) };
    }
    return previous.map(str::to_string);
}

/// What: `update_editor` replaces the editor value in the state with the result of `change`.
/// Why: the editor is an immutable value, so every change builds the next value from the current one.
fn update_editor(change: impl FnOnce(&TemplateEditorState) -> TemplateEditorState) {
    update_state(|state| {
        if let Some(editor) = state.editor.as_ref() {
            state.editor = Some(change(editor));
        }
    });
}

/// What: `project_pages` pushes the Settings rows and every editor page value to the window.
/// Why: the pages take Slint structs, and the pure models supply the values in one state read.
pub(crate) fn project_pages(window: &AppWindow) {
    let (rows, view) = read_state(|state| {
        let rows: Vec<SettingsRow> = state.editor.as_ref().map_or_else(Vec::new, settings_rows);
        let view: Option<EditorView> = state.editor.as_ref().map(editor_view);
        return (rows, view);
    });
    window.set_settings_rows(model_of(
        rows.into_iter()
            .map(|row| return SettingsTemplateRow {
                label: row.label.as_str().into(),
                line: row.line.as_str().into(),
                icon: row.icon.into(),
            })
            .collect(),
    ));
    crate::ui_text_size::project_sizes(window);
    let auto_play_on_select = read_state(|state| return state.auto_play_on_select);
    window.set_auto_play_on_select(auto_play_on_select);
    window.set_auto_play_on_select_text(auto_play_on_select_supporting_text(auto_play_on_select).into());
    let selected = read_state(|state| return state.editor.as_ref().map_or(0, |editor| return index_of_kind(editor.kind())));
    window.set_settings_selected(i32::try_from(selected).unwrap_or(0));
    let Some(page) = view else {
        return;
    };
    window.set_editor_title(page.title.as_str().into());
    window.set_editor_text(page.text.as_str().into());
    window.set_editor_preview_rows(model_of(
        page.preview
            .iter()
            .map(|(title, supporting)| return EditorPreviewRow {
                title: title.as_str().into(),
                supporting: supporting.as_str().into(),
            })
            .collect(),
    ));
    window.set_editor_mistakes(model_of(page.mistakes.iter().map(|line| return line.as_str().into()).collect()));
    window.set_editor_help_signature(page.help_signature.as_str().into());
    window.set_editor_help_description(page.help_description.as_str().into());
    window.set_editor_has_help(page.has_help);
    window.set_editor_fields(model_of(
        page.fields
            .iter()
            .map(|field| return EditorFieldRow {
                label: field.label.as_str().into(),
                insert: field.insert.as_str().into(),
                value: field.value.as_str().into(),
            })
            .collect(),
    ));
    window.set_editor_notes(model_of(page.notes.iter().map(|note| return note.as_str().into()).collect()));
    window.set_editor_kept_note(page.kept_note.as_str().into());
    window.set_editor_can_reset(page.can_reset);
}

#[cfg(test)]
#[path = "ui_settings_tests.rs"]
mod tests;
