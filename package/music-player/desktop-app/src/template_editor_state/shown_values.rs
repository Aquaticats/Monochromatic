//! What the template editor's screen reads off its state:
//!  the mistakes,
//!  the preview rows and their notes,
//!  the insertable field list,
//!  the typing help and the
//! Settings lines.
//!
//! What:
//! one `impl TemplateEditorState` block of read-only methods that turn the stored state into
//!  the values the Settings page and the editor pane draw.
//! Why:
//! the state and its immutable transitions are one role,
//!  and every derived value the screen asks for is another,
//!  so the derived half lives beside the state in `template_editor_state.rs` instead of
//!  inside it.
//! Nothing here changes the state:
//!  each method answers from the stored texts,
//!  the preview tracks and the caret.

/// What:
///  imports the editor type this module extends.
/// Why:
///  Every method here is an inherent method on it.
use super::TemplateEditorState;
/// What:
///  imports the notes drawn under the preview and the editor's own value types.
/// Why:
///  The preview rows,
///  the field list and the two notes are built from them.
use crate::template_editor::{
    TEMPLATE_KEPT_EMPTY_NOTE,
    TEMPLATE_KEPT_NOTE,
    TEMPLATE_LIBRARY_NOTE,
    TEMPLATE_SAMPLE_NOTE,
    TemplateEditorFieldEntry,
    TemplateEditorStateError,
    TemplatePreviewRow,
    track_source,
};
/// What:
///  imports the typing help and its lookup.
/// Why:
///  The help is shown only while the field has focus.
use crate::template_help::{
    TemplateHelp,
    help_at,
};
/// What:
///  imports the template kind and the result types.
/// Why:
///  The methods below evaluate and name them.
use crate::template_model::{
    TemplateError,
    TemplateKind,
    TemplateResult,
};
/// What:
///  imports the plain mistake sentences.
/// Why:
///  The editor lists one plain sentence per problem,
/// not the engine's notes.
use crate::template_plain::plain_lines;

/// What:
///  the derived values the Settings page and the editor pane read.
/// Why:
///  They are pure reads of one state,
///  so they can sit in their own file without changing how a transition works.
impl TemplateEditorState {
    /// What:
    ///  `pub fn errors(&self) -> Vec<TemplateError>` lists every distinct mistake of the open template's text over all preview tracks.
    /// Why:
    ///  A template is valid only if it applies to every preview track,
    ///  so each track is checked,
    ///  and a repeated mistake is listed once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get errors(): TemplateError[] { return unique(previewTracks.flatMap(t => refusals(evaluate(text, t)))); }
    /// ```
    pub fn errors(&self) -> Vec<TemplateError> {
        let mut found: Vec<TemplateError> = Vec::new();
        for result in self.results_for(self.kind, self.text()) {
            if let TemplateResult::Refused(errors) = result {
                for error in errors {
                    if !found.contains(&error) {
                        found.push(error);
                    }
                }
            }
        }
        return found;
    }

    /// What:
    ///  `pub fn is_valid(&self) -> bool` tells whether the open template's text applies to every preview track.
    /// Why:
    ///  Validity decides whether the text becomes the applied template and whether mistakes show.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get isValid(): boolean { return this.errors.length === 0; }
    /// ```
    pub fn is_valid(&self) -> bool {
        return self.errors().is_empty();
    }

    /// What:
    ///  `pub fn mistake_lines(&self) -> Vec<String>` gives the current problems as one plain sentence each,
    ///  in template order.
    /// Why:
    ///  The editor names the call at fault and says what to change (D89),
    /// and a half-written formula is one problem,
    ///  not the three follow-on mistakes the parser records for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get mistakeLines(): string[] { return plainLines(this.text, this.errors); }
    /// ```
    pub fn mistake_lines(&self) -> Vec<String> {
        return plain_lines(self.text(), &self.errors());
    }

    /// What:
    ///  `pub fn preview_rows(&self) -> Result<Vec<TemplatePreviewRow>, TemplateEditorStateError>` gives each preview track its applied line.
    /// Why:
    ///  The rows keep the last template that applied,
    ///  so a mistake never changes what they show.
    ///  A stored template that no longer applies is an error,
    ///  because the state then holds a broken value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get previewRows() { return this.previewTracks.map(t => ({ title: t.track.title, supporting: applied(t) })); }
    /// ```
    pub fn preview_rows(&self) -> Result<Vec<TemplatePreviewRow>, TemplateEditorStateError> {
        let mut rows = Vec::new();
        for track in &self.preview_tracks {
            let supporting = self.applied_line(self.kind, track)?;
            rows.push(TemplatePreviewRow {
                title: track.track.title.clone(),
                supporting,
            });
        }
        return Ok(rows);
    }

    /// What:
    ///  `pub fn preview_notes(&self) -> Vec<&'static str>` gives the notes drawn under the preview.
    /// Why:
    ///  The source note comes first,
    ///  and the kept-template note follows while a mistake stands.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get previewNotes(): string[] { return [usesSample ? SAMPLE_NOTE : LIBRARY_NOTE, ...(valid ? [] : [KEPT_NOTE])]; }
    /// ```
    pub fn preview_notes(&self) -> Vec<&'static str> {
        let mut notes = vec![self.source_note()];
        if let Some(kept) = self.kept_note() {
            notes.push(kept);
        }
        return notes;
    }

    /// What:
    ///  `pub fn source_note(&self) -> &'static str` says where the preview rows come from.
    /// Why:
    ///  The note under the preview always names the source,
    /// library or stand-ins.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get sourceNote(): string { return this.usesSampleValues ? SAMPLE_NOTE : LIBRARY_NOTE; }
    /// ```
    pub fn source_note(&self) -> &'static str {
        return if self.uses_sample_values {
            TEMPLATE_SAMPLE_NOTE
        } else {
            TEMPLATE_LIBRARY_NOTE
        };
    }

    /// What:
    ///  `pub fn kept_note(&self) -> Option<&'static str>` is the note that says the rows keep the last valid template,
    ///  present only while the typed text does not apply.
    /// Why:
    ///  The page reserves a line for it so it can appear without moving the field,
    /// and it says when the kept template
    ///  shows nothing,
    /// because then the preview looks the same as a broken one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get keptNote(): string | undefined { return valid ? undefined : keptShowsNothing ? KEPT_EMPTY_NOTE : KEPT_NOTE; }
    /// ```
    pub fn kept_note(&self) -> Option<&'static str> {
        if self.is_valid() {
            return None;
        }
        let shows_nothing = match self.applied_line(self.kind, self.first_track()) {
            Ok(line) => line.trim().is_empty(),
            Err(_error) => false,
        };
        return Some(if shows_nothing {
            TEMPLATE_KEPT_EMPTY_NOTE
        } else {
            TEMPLATE_KEPT_NOTE
        });
    }

    /// What:
    ///  `pub fn fields(&self) -> Vec<TemplateEditorFieldEntry>` lists the open template's insertable fields with their values for the first preview track.
    /// Why:
    ///  The list shows what each field holds,
    ///  so the user sees the value the template will read.
    /// Panics:
    ///  never;
    ///  the first preview track exists by construction (see the module docs).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get fields() { return kind.fields.map(f => ({ label, mode, insert: `mi(${mode})`, value })); }
    /// ```
    pub fn fields(&self) -> Vec<TemplateEditorFieldEntry> {
        let source = track_source(self.kind, self.first_track());
        let mut entries: Vec<TemplateEditorFieldEntry> = Vec::new();
        for field in self.kind.fields() {
            entries.push(TemplateEditorFieldEntry {
                label: field.label,
                mode: field.mode,
                insert: format!("mi({})", field.mode),
                value: source.field_text(field.mode).unwrap_or_default(),
            });
        }
        return entries;
    }

    /// What:
    ///  `pub fn help(&self) -> Option<TemplateHelp>` is the typing help for the call around the caret.
    /// Why:
    ///  Help is shown only while the field has focus,
    ///  because only then is the caret typing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get help() { return this.fieldFocused ? helpAt(this.text, this.selectionEnd, fields) : undefined; }
    /// ```
    pub fn help(&self) -> Option<TemplateHelp> {
        if !self.field_focused {
            return None;
        }
        return help_at(self.text(), self.selection_end, &self.kind.fields());
    }

    /// What:
    ///  `pub fn can_reset(&self) -> bool` tells whether the open template differs from its default.
    /// Why:
    ///  The way back to the default is offered only when there is something to go back from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get canReset(): boolean { return this.text !== this.kind.defaultTemplate; }
    /// ```
    pub fn can_reset(&self) -> bool {
        return self.text() != self.kind.default_template();
    }

    /// What:
    ///  `pub fn settings_lines(&self) -> Result<Vec<(TemplateKind, String)>, TemplateEditorStateError>` gives each template's line for Settings.
    /// Why:
    ///  The Settings page lists both templates with the line each yields for the first preview track.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get settingsLines() { return Object.fromEntries(kinds.map(k => [k, settingsLine(k)])); }
    /// ```
    pub fn settings_lines(&self) -> Result<Vec<(TemplateKind, String)>, TemplateEditorStateError> {
        let mut lines = Vec::new();
        for kind in TemplateKind::all() {
            lines.push((kind, self.settings_line(kind)?));
        }
        return Ok(lines);
    }

    /// What:
    ///  `pub fn settings_line(&self, template: TemplateKind) -> Result<String, TemplateEditorStateError>` gives one template's applied line for the first preview track.
    /// Why:
    ///  Settings shows the applied template on the first preview track,
    ///  so its effect is visible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// settingsLine(template: TemplateKind): string { return applied(template, this.previewTracks[0]); }
    /// ```
    pub fn settings_line(
        &self,
        template: TemplateKind,
    ) -> Result<String, TemplateEditorStateError> {
        return self.applied_line(template, self.first_track());
    }
}
