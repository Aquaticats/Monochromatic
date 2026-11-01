//! Pure models for the Settings template rows and the template editor page.
//!
//! What: `settings_rows` and `editor_view` turn the template editor state into plain rows and strings.
//! Why: the editor's rules (applied lines, previews, mistakes, help, fields) come from `template_editor_state`;
//! this module only reshapes them for the window.

use music_player::template_editor_state::TemplateEditorState;
use music_player::template_model::TemplateKind;

/// What: `SettingsRow` is one Templates row: its heading and the line it applies to the first preview track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettingsRow = { label: string; line: string; icon: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsRow {
    /// Heading of the template.
    pub label: String,
    /// Applied line for the first preview track, or empty when none applies.
    pub line: String,
    /// Name of the Material Symbols icon the row leads with, as `icon_name` gives it.
    pub icon: &'static str,
}

/// What: `EditorField` is one insertable field row of the editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorField {
    /// Field name shown as the headline.
    pub label: String,
    /// Call a tap inserts.
    pub insert: String,
    /// Value for the first preview track.
    pub value: String,
}

/// What: `EditorView` is every value the template editor page shows.
/// Why: the page takes plain values, and the editor state changes only through its own methods.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EditorView = { title: string; text: string; preview: [string, string][]; mistakes: string[]; ... };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorView {
    /// Heading naming the open template.
    pub title: String,
    /// Template text as typed.
    pub text: String,
    /// Preview rows as (track title, applied line).
    pub preview: Vec<(String, String)>,
    /// One line per mistake of the open template.
    pub mistakes: Vec<String>,
    /// Signature of the call around the caret, or empty.
    pub help_signature: String,
    /// Description of the argument around the caret, or empty.
    pub help_description: String,
    /// Whether the help applies.
    pub has_help: bool,
    /// Insertable fields in list order.
    pub fields: Vec<EditorField>,
    /// Notes drawn under the preview: where its rows come from.
    pub notes: Vec<String>,
    /// The note that says the rows keep the last valid template, or empty while the template applies.
    pub kept_note: String,
    /// Whether Reset to default is offered.
    pub can_reset: bool,
}

/// What: `kind_of_index` gives the template a Settings row index opens.
/// Why: Settings lists the templates in `TemplateKind::all()` order, so the index maps back to one template.
pub fn kind_of_index(index: usize) -> Option<TemplateKind> {
    return TemplateKind::all().get(index).copied();
}

/// What: `index_of_kind` gives the Settings row index of a template, the inverse of `kind_of_index`.
/// Why: the window marks the row of the template the editor is open on as the selected one.
pub fn index_of_kind(kind: TemplateKind) -> usize {
    return TemplateKind::all().iter().position(|candidate| return *candidate == kind).unwrap_or(0);
}

/// What: `icon_name` gives the Material Symbols icon a template's row leads with (D109).
/// Why: the Track rows template is a list of rows (`queue_music`) and the Playing track template is the deck line of
/// what plays (`play_circle`); the window draws the embedded path of the named icon.
pub fn icon_name(kind: TemplateKind) -> &'static str {
    return match kind {
        TemplateKind::TrackRows => "queue_music",
        TemplateKind::PlayingTrack => "play_circle",
    };
}

/// What: `settings_rows` gives the Templates rows for Settings.
/// Why: each row shows the applied line for the first preview track; a template that no longer applies shows an empty line.
pub fn settings_rows(editor: &TemplateEditorState) -> Vec<SettingsRow> {
    return TemplateKind::all()
        .into_iter()
        .map(|kind| return SettingsRow {
            label: kind.label().to_string(),
            line: editor.settings_line(kind).unwrap_or_default(),
            icon: icon_name(kind),
        })
        .collect();
}

/// What: `editor_view` gives every value of the template editor for its open template.
/// Why: the page draws the preview, mistakes, help and fields from one state read.
pub fn editor_view(editor: &TemplateEditorState) -> EditorView {
    let help = editor.help();
    return EditorView {
        title: editor.kind().label().to_string(),
        text: editor.text().to_string(),
        preview: editor
            .preview_rows()
            .map(|rows| return rows.into_iter().map(|row| return (row.title, row.supporting)).collect())
            .unwrap_or_default(),
        mistakes: editor.mistake_lines(),
        help_signature: help.as_ref().map_or_else(String::new, |value| return value.signature.to_string()),
        help_description: help.as_ref().map_or_else(String::new, |value| return value.description.clone()),
        has_help: help.is_some(),
        fields: editor
            .fields()
            .into_iter()
            .map(|field| return EditorField {
                label: field.label.to_string(),
                insert: field.insert,
                value: field.value,
            })
            .collect(),
        notes: vec![editor.source_note().to_string()],
        kept_note: editor.kept_note().map_or_else(String::new, str::to_string),
        can_reset: editor.can_reset(),
    };
}

#[cfg(test)]
#[path = "ui_settings_model_tests.rs"]
mod tests;
