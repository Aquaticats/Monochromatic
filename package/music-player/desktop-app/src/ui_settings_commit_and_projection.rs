//! Saving the applied templates and projecting the Settings pages into the window.
//!
//! What:
//!  `commit_templates` saves the templates that the editor applies and remembers the drafts that do not apply,
//!  `update_editor` builds the next editor value from the current one,
//!  and `project_pages` pushes the Settings rows and every editor page value to the window.
//! Why:
//!  The callbacks in `ui_settings.rs` decide when to commit and project;
//!  the commit rules and the projection fill a file of their own.
//!  The module is a child of `ui_settings`,
//! so it reaches the saver and the kept drafts that stay private there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export { commitTemplates, updateEditor, projectPages };
//! ```

/// Imports everything the Settings binding declares,
/// including its private saver and kept drafts.
use super::*;

/// What:
/// `commit_templates` applies the editor's valid drafts to the track rows and the deck,
/// saves them when they
/// changed,
/// and rebuilds the editor on the applied texts.
/// Why:
/// both forms leave a template the same way:
/// the editor page's Back,
/// a new selection in the right pane,
/// and closing
/// Settings.
/// Rebuilding keeps the open template unchanged except that its draft equals what is applied.
pub(super) fn commit_templates() {
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

/// What:
/// `remember_drafts_that_do_not_apply` keeps the text of every template whose draft does not apply,
/// and forgets the rest.
/// Why:
/// the editor is rebuilt from the applied texts when it is left,
/// so a draft that does not apply would otherwise vanish
/// without a word;
/// a draft that does apply is the applied text and needs no keeping.
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

/// What:
/// `effective_text` gives the text a template actually uses:
/// the saved one,
/// or the default when none is saved.
/// Why:
/// saved None and saved default text are the same template,
/// so comparing them must not report a change.
fn effective_text(kind: TemplateKind, saved: Option<&str>) -> String {
    return saved.map_or_else(|| return kind.default_template().to_string(), str::to_string);
}

/// What:
/// `applied_text` gives the draft of `kind` when it applies,
/// or the previously applied text otherwise;
/// a text equal
/// to the template's default is given as None.
/// Why:
/// only texts that apply reach the track rows and the saved session,
/// and a template that reads as its default is
/// stored as no text,
/// so it keeps following the default.
fn applied_text(editor: &TemplateEditorState, kind: TemplateKind, previous: Option<&str>) -> Option<String> {
    let opened = editor.open(kind);
    if opened.is_valid() {
        let text = opened.text();
        return if text == kind.default_template() { None } else { Some(text.to_string()) };
    }
    return previous.map(str::to_string);
}

/// What:
/// `update_editor` replaces the editor value in the state with the result of `change`.
/// Why:
/// the editor is an immutable value,
/// so every change builds the next value from the current one.
pub(super) fn update_editor(change: impl FnOnce(&TemplateEditorState) -> TemplateEditorState) {
    update_state(|state| {
        if let Some(editor) = state.editor.as_ref() {
            state.editor = Some(change(editor));
        }
    });
}

/// What:
/// `project_pages` pushes the Settings rows and every editor page value to the window.
/// Why:
/// the pages take Slint structs,
/// and the pure models supply the values in one state read.
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
