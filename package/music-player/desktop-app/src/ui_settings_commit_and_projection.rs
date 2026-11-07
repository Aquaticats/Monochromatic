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
/// `pending_commit` gives the two applied texts and the kind that is open,
/// or nothing when no editor is open.
/// Why:
/// the commit saves what applies,
/// and the open kind decides which template the editor shows again afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pendingCommit(state: ScreenState): [string | null, string | null, TemplateKind] | null { ... }
/// ```
fn pending_commit(state: &ScreenState) -> Option<(Option<String>, Option<String>, TemplateKind)> {
    let editor = state.editor.as_ref()?;
    return Some((
        applied_text(editor, TemplateKind::TrackRows, state.track_rows_template.as_deref()),
        applied_text(editor, TemplateKind::PlayingTrack, state.playing_template.as_deref()),
        editor.kind(),
    ));
}

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
    let saved = pending_commit(&read_screen());
    let Some((track_rows, playing, open_kind)) = saved else {
        tracing::debug!("no editor value to commit");
        return;
    };
    remember_drafts_that_do_not_apply();
    // A template the user never changed is stored as None and applied as its default text, which is the same template.
    let changed = {
        let state = read_screen();
        effective_text(TemplateKind::TrackRows, state.track_rows_template.as_deref())
            != effective_text(TemplateKind::TrackRows, track_rows.as_deref())
            || effective_text(TemplateKind::PlayingTrack, state.playing_template.as_deref())
                != effective_text(TemplateKind::PlayingTrack, playing.as_deref())
    };
    if changed {
        tracing::info!("saving the applied template texts");
        if let Err(error) = SAVER.get()(track_rows.as_deref(), playing.as_deref()) {
            tracing::warn!(?error, "failed to save the template texts");
        }
    } else {
        tracing::debug!("the template texts are unchanged; nothing to save");
    }
    {
        let mut state = write_screen();
        state.track_rows_template = track_rows;
        state.playing_template = playing;
        state.editor = Some(build_editor(&state).open(open_kind));
    }
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
    let kept = {
        let state = read_screen();
        state.editor.as_ref().map(drafts_not_applying)
    };
    if let Some(drafts) = kept {
        tracing::debug!(track_rows = drafts.0.is_some(), playing = drafts.1.is_some(), "drafts that do not apply are kept");
        KEPT_DRAFTS.set(drafts);
    }
}

/// What:
/// `draft_not_applying` gives the draft text of `kind` when it does not apply,
/// or nothing when it does.
/// Why:
/// only a draft that does not apply needs keeping,
/// because one that applies is the applied text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const draftNotApplying = (editor: Editor, kind: TemplateKind): string | null => { ... };
/// ```
fn draft_not_applying(editor: &TemplateEditorState, kind: TemplateKind) -> Option<String> {
    let opened = editor.open(kind);
    return if opened.is_valid() { None } else { Some(opened.text().to_string()) };
}

/// What:
/// `drafts_not_applying` gives the draft texts of both templates that do not apply.
/// Why:
/// the editor is rebuilt from the applied texts when it is left,
/// so both drafts are kept together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const draftsNotApplying = (editor: Editor): [string | null, string | null] => [draftNotApplying(editor, "trackRows"), draftNotApplying(editor, "playing")];
/// ```
fn drafts_not_applying(editor: &TemplateEditorState) -> (Option<String>, Option<String>) {
    return (draft_not_applying(editor, TemplateKind::TrackRows), draft_not_applying(editor, TemplateKind::PlayingTrack));
}

/// What:
/// `effective_text` gives the text a template actually uses:
/// the saved one,
/// or the default when none is saved.
/// Why:
/// saved None and saved default text are the same template,
/// so comparing them must not report a change.
fn effective_text(kind: TemplateKind, saved: Option<&str>) -> String {
    return match saved {
        Some(text) => text.to_string(),
        None => kind.default_template().to_string(),
    };
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
/// `current_editor` gives a copy of the open editor value,
/// or nothing when no editor is open.
/// Why:
/// the editor is an immutable value,
/// so every change builds the next value from a copy of the current one and stores it with `store_editor`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const currentEditor = (): Editor | null => state.editor;
/// ```
pub(super) fn current_editor() -> Option<TemplateEditorState> {
    return read_screen().editor.clone();
}

/// What:
/// `store_editor` replaces the editor value in the state when an editor is open.
/// Why:
/// a change that arrives after the editor was left must not reopen it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function storeEditor(next: Editor): void { if (state.editor) state.editor = next; }
/// ```
pub(super) fn store_editor(next: TemplateEditorState) {
    let mut state = write_screen();
    if state.editor.is_some() {
        state.editor = Some(next);
    }
}

/// What:
/// `project_pages` pushes the Settings rows and every editor page value to the window.
/// Why:
/// the pages take Slint structs,
/// and the pure models supply the values in one state read.
pub(crate) fn project_pages(window: &AppWindow) {
    let (rows, view) = {
        let state = read_screen();
        let rows: Vec<SettingsRow> = match state.editor.as_ref() {
            Some(editor) => settings_rows(editor),
            None => Vec::new(),
        };
        let view: Option<EditorView> = state.editor.as_ref().map(editor_view);
        (rows, view)
    };
    let mut settings_rows: Vec<SettingsTemplateRow> = Vec::with_capacity(rows.len());
    for row in rows {
        settings_rows.push(SettingsTemplateRow {
            label: row.label.as_str().into(),
            line: row.line.as_str().into(),
            icon: row.icon.into(),
        });
    }
    window.set_settings_rows(model_of(settings_rows));
    crate::ui_text_size::project_sizes(window);
    let auto_play_on_select = read_screen().auto_play_on_select;
    window.set_auto_play_on_select(auto_play_on_select);
    window.set_auto_play_on_select_text(auto_play_on_select_supporting_text(auto_play_on_select).into());
    let selected = match read_screen().editor.as_ref() {
        Some(editor) => index_of_kind(editor.kind()),
        None => 0,
    };
    window.set_settings_selected(i32::try_from(selected).unwrap_or(0));
    let Some(page) = view else {
        return;
    };
    window.set_editor_title(page.title.as_str().into());
    window.set_editor_text(page.text.as_str().into());
    let mut preview_rows: Vec<EditorPreviewRow> = Vec::with_capacity(page.preview.len());
    for (title, supporting) in &page.preview {
        preview_rows.push(EditorPreviewRow {
            title: title.as_str().into(),
            supporting: supporting.as_str().into(),
        });
    }
    window.set_editor_preview_rows(model_of(preview_rows));
    let mut mistakes: Vec<slint::SharedString> = Vec::with_capacity(page.mistakes.len());
    for line in &page.mistakes {
        mistakes.push(line.as_str().into());
    }
    window.set_editor_mistakes(model_of(mistakes));
    window.set_editor_help_signature(page.help_signature.as_str().into());
    window.set_editor_help_description(page.help_description.as_str().into());
    window.set_editor_has_help(page.has_help);
    let mut field_rows: Vec<EditorFieldRow> = Vec::with_capacity(page.fields.len());
    for field in &page.fields {
        field_rows.push(EditorFieldRow {
            label: field.label.as_str().into(),
            insert: field.insert.as_str().into(),
            value: field.value.as_str().into(),
        });
    }
    window.set_editor_fields(model_of(field_rows));
    let mut notes: Vec<slint::SharedString> = Vec::with_capacity(page.notes.len());
    for note in &page.notes {
        notes.push(note.as_str().into());
    }
    window.set_editor_notes(model_of(notes));
    window.set_editor_kept_note(page.kept_note.as_str().into());
    window.set_editor_can_reset(page.can_reset);
}
