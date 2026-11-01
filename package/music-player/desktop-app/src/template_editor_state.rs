//! The template editor's whole state:
//!  both templates,
//!  the open one,
//!  its caret,
//! focus,
//!  and the tracks it previews,
//!  plus every transition as an immutable step.
//!
//! The preview tracks are private and set only by `initial`,
//!  which always stores at
//! least one track,
//!  so the first track is present on every path.

/// What:
///  imports the editor values,
///  constants,
///  and helpers the state uses.
/// Why:
///  The state builds its previews and edits from them.
use crate::template_editor::{
    TEMPLATE_KEPT_EMPTY_NOTE, TEMPLATE_KEPT_NOTE, TEMPLATE_LIBRARY_NOTE, TEMPLATE_PREVIEW_ROW_COUNT, TEMPLATE_SAMPLE_NOTE, TemplateEditorEdit,
    TemplateEditorFieldEntry, TemplateEditorStateError, TemplatePreviewRow, TemplateTexts, clamp_offset, evaluate_for,
    inside_formula, splice_text, template_sample_tracks, track_source,
};
/// What:
///  imports the UTF-16 length helper.
/// Why:
///  Caret offsets are UTF-16 positions.
use crate::template_format::utf16_length;
/// What:
///  imports the typing help.
/// Why:
///  The state exposes the help for the focused field.
use crate::template_help::{TemplateHelp, help_at};
/// What:
///  imports the track,
///  template kind,
///  result,
///  and error types.
/// Why:
///  The state stores and evaluates them.
use crate::template_model::{
    PlayingTrackFields, TemplateError, TemplateKind, TemplateResult, error_lines,
};
/// What:
///  imports the plain mistake sentences.
/// Why:
///  The editor lists one plain sentence per problem, not the engine's notes.
use crate::template_plain::plain_lines;

/// What:
///  `pub struct TemplateEditorState` holds both templates,
///  the open one,
///  its caret,
///  focus,
///  and the tracks it previews.
/// Why:
///  One immutable value lets the screen redraw from it and lets tests check each transition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateEditorState = Readonly<{ kind: TemplateKind; drafts: Record<TemplateKind, string>;
///   lastValid: Record<TemplateKind, string>; selectionStart: number; selectionEnd: number;
///   fieldFocused: boolean; previewTracks: PlayingTrackFields[]; usesSampleValues: boolean }>;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateEditorState {
    /// The template the editor is open on.
    kind: TemplateKind,
    /// Current text of each template as typed,
    ///  including text that does not apply.
    drafts: TemplateTexts,
    /// Last text of each template that applied;
    ///  the rows show these.
    last_valid: TemplateTexts,
    /// Selection start in the open template,
    ///  in UTF-16 units.
    selection_start: usize,
    /// Selection end in the open template,
    ///  in UTF-16 units.
    selection_end: usize,
    /// Whether the template field has focus,
    ///  which decides whether typing help is shown.
    field_focused: bool,
    /// The tracks the preview reads:
    ///  library tracks,
    ///  or stand-ins while the library is empty.
    ///  Never empty.
    preview_tracks: Vec<PlayingTrackFields>,
    /// Whether the preview shows stand-in values because the open library holds no track.
    uses_sample_values: bool,
}

/// What:
///  the state reads,
///  previews,
///  and immutable transitions.
/// Why:
///  Each transition returns a new state and leaves the old one unchanged.
impl TemplateEditorState {
    /// What:
    ///  `pub fn initial(library: &[PlayingTrackFields]) -> TemplateEditorState` starts the editor.
    /// Why:
    ///  The editor opens on the track rows template at its default,
    ///  previewing the library or the stand-ins.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function initial(libraryTracks: PlayingTrackFields[]): State { ... }
    /// ```
    pub fn initial(library: &[PlayingTrackFields]) -> Self {
        let uses_sample_values = library.is_empty();
        let preview_tracks = if uses_sample_values {
            template_sample_tracks()
        } else {
            library.iter().take(TEMPLATE_PREVIEW_ROW_COUNT).cloned().collect()
        };
        let defaults = TemplateTexts::defaults();
        let end = utf16_length(TemplateKind::TrackRows.default_template());
        return Self {
            kind: TemplateKind::TrackRows,
            drafts: defaults.clone(),
            last_valid: defaults,
            selection_start: end,
            selection_end: end,
            field_focused: false,
            preview_tracks,
            uses_sample_values,
        };
    }

    /// What:
    ///  `pub fn kind(&self) -> TemplateKind` is the template the editor is open on.
    /// Why:
    ///  The screen shows the open template's heading and field list.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get kind(): TemplateKind { return this.kind; }
    /// ```
    pub fn kind(&self) -> TemplateKind {
        return self.kind;
    }

    /// What:
    ///  `pub fn text(&self) -> &str` is the open template's current text,
    ///  as typed.
    /// Why:
    ///  The field,
    ///  the validation,
    ///  and the help all read the same text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get text(): string { return this.drafts[this.kind]; }
    /// ```
    pub fn text(&self) -> &str {
        return self.drafts.get(self.kind);
    }

    /// What:
    ///  `pub fn selection_start(&self) -> usize` is the selection start in the open template,
    ///  in UTF-16 units.
    /// Why:
    ///  The screen places the caret from this offset.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get selectionStart(): number { return this.selectionStart; }
    /// ```
    pub fn selection_start(&self) -> usize {
        return self.selection_start;
    }

    /// What:
    ///  `pub fn selection_end(&self) -> usize` is the selection end in the open template,
    ///  in UTF-16 units.
    /// Why:
    ///  The help follows the selection end,
    ///  which is where the caret types.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get selectionEnd(): number { return this.selectionEnd; }
    /// ```
    pub fn selection_end(&self) -> usize {
        return self.selection_end;
    }

    /// What:
    ///  `pub fn field_focused(&self) -> bool` tells whether the template field has focus.
    /// Why:
    ///  Typing help is shown only while the field has focus.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get fieldFocused(): boolean { return this.fieldFocused; }
    /// ```
    pub fn field_focused(&self) -> bool {
        return self.field_focused;
    }

    /// What:
    ///  `pub fn uses_sample_values(&self) -> bool` tells whether the preview shows stand-in values.
    /// Why:
    ///  The preview note says where its rows come from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get usesSampleValues(): boolean { return this.usesSampleValues; }
    /// ```
    pub fn uses_sample_values(&self) -> bool {
        return self.uses_sample_values;
    }

    /// What:
    ///  `pub fn preview_tracks(&self) -> &[PlayingTrackFields]` is the tracks the preview and validation read.
    /// Why:
    ///  The screen and the tests need the tracks behind the rows.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get previewTracks(): PlayingTrackFields[] { return this.previewTracks; }
    /// ```
    pub fn preview_tracks(&self) -> &[PlayingTrackFields] {
        return &self.preview_tracks;
    }

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
    ///  The editor names the call at fault and says what to change (D89), and a half-written formula is one problem,
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
    ///  The note under the preview always names the source, library or stand-ins.
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
    ///  The page reserves a line for it so it can appear without moving the field, and it says when the kept template
    ///  shows nothing, because then the preview looks the same as a broken one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get keptNote(): string | undefined { return valid ? undefined : keptShowsNothing ? KEPT_EMPTY_NOTE : KEPT_NOTE; }
    /// ```
    pub fn kept_note(&self) -> Option<&'static str> {
        if self.is_valid() {
            return None;
        }
        let shows_nothing = self
            .applied_line(self.kind, self.first_track())
            .is_ok_and(|line| return line.trim().is_empty());
        return Some(if shows_nothing { TEMPLATE_KEPT_EMPTY_NOTE } else { TEMPLATE_KEPT_NOTE });
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
        return self
            .kind
            .fields()
            .into_iter()
            .map(|field| {
                return TemplateEditorFieldEntry {
                    label: field.label,
                    mode: field.mode,
                    insert: format!("mi({})", field.mode),
                    value: source.field_text(field.mode).unwrap_or_default(),
                };
            })
            .collect();
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
    pub fn settings_line(&self, template: TemplateKind) -> Result<String, TemplateEditorStateError> {
        return self.applied_line(template, self.first_track());
    }

    /// What:
    ///  `pub fn with_edit(&self, edit: TemplateEditorEdit) -> Self` applies one field edit.
    /// Why:
    ///  The typed text always becomes the draft,
    ///  and it becomes the applied template only when it applies to every preview track.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withEdit(edit: Edit): State { /* draft = text; applied = valid ? text : applied; clamp caret */ }
    /// ```
    pub fn with_edit(&self, edit: TemplateEditorEdit) -> Self {
        let applies = self
            .results_for(self.kind, &edit.text)
            .iter()
            .all(|result| return matches!(result, TemplateResult::Shown(_)));
        let last_valid = if applies {
            self.last_valid.with(self.kind, edit.text.clone())
        } else {
            self.last_valid.clone()
        };
        let selection_start = clamp_offset(edit.selection_start, &edit.text);
        let selection_end = clamp_offset(edit.selection_end, &edit.text);
        return Self {
            drafts: self.drafts.with(self.kind, edit.text),
            last_valid,
            selection_start,
            selection_end,
            ..self.clone()
        };
    }

    /// What:
    ///  `pub fn insert_field(&self, mode: &str) -> Self` puts one field's call at the caret,
    ///  replacing any selection.
    /// Why:
    ///  The field list inserts at the caret,
    ///  so the user builds a template without typing a name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// insertField(mode: string): State { return this.insertCall(`mi(${mode})`, caretInCall); }
    /// ```
    pub fn insert_field(&self, mode: &str) -> Self {
        let call = format!("mi({mode})");
        let caret_in_call = utf16_length(&call);
        return self.insert_call(&call, caret_in_call);
    }

    /// What:
    ///  `pub fn insert_function(&self, name: &str) -> Self` puts an empty call of one function at the caret,
    ///  with the caret inside its brackets.
    /// Why:
    ///  The user types the arguments next,
    ///  so the caret lands inside the brackets.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// insertFunction(name: string): State { return this.insertCall(`${name}()`, name.length + 1); }
    /// ```
    pub fn insert_function(&self, name: &str) -> Self {
        let call = format!("{name}()");
        return self.insert_call(&call, utf16_length(name) + 1);
    }

    /// What:
    ///  `pub fn reset_to_default(&self) -> Self` restores the open template's default text,
    ///  with the caret at its end.
    /// Why:
    ///  The way back to the default must always work,
    ///  and the default always applies.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// resetToDefault(): State { return this.withEdit({ text: this.kind.defaultTemplate, ... }); }
    /// ```
    pub fn reset_to_default(&self) -> Self {
        let default_text = self.kind.default_template();
        let length = utf16_length(default_text);
        return self.with_edit(TemplateEditorEdit {
            text: default_text.to_string(),
            selection_start: length,
            selection_end: length,
        });
    }

    /// What:
    ///  `pub fn open(&self, template: TemplateKind) -> Self` opens one template with the caret at the end of its text and no focus in the field.
    /// Why:
    ///  Settings opens an editor per template,
    ///  and each keeps its own text while the other is open.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// open(template: TemplateKind): State { return { ...this, kind: template, fieldFocused: false }; }
    /// ```
    pub fn open(&self, template: TemplateKind) -> Self {
        let length = utf16_length(self.drafts.get(template));
        return Self {
            kind: template,
            selection_start: length,
            selection_end: length,
            field_focused: false,
            ..self.clone()
        };
    }

    /// What:
    ///  `pub fn with_focus(&self, focused: bool) -> Self` records whether the template field has focus.
    /// Why:
    ///  Typing help appears only with focus,
    ///  so focus changes must reach the state.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withFocus(focused: boolean): State { return { ...this, fieldFocused: focused }; }
    /// ```
    pub fn with_focus(&self, focused: bool) -> Self {
        return Self {
            field_focused: focused,
            ..self.clone()
        };
    }

    /// What:
    ///  `fn insert_call(&self, call: &str, caret_in_call: usize) -> Self` replaces the selection with a call,
    ///  wrapping it in dollar signs when the caret is outside a formula.
    /// Why:
    ///  A call written outside a formula would show as literal text,
    ///  so the editor adds the delimiters itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function insertCall(text, start, end, call, caretInCall) { /* wrap with $ outside a formula */ }
    /// ```
    fn insert_call(&self, call: &str, caret_in_call: usize) -> Self {
        let text = self.text();
        let start = clamp_offset(self.selection_start.min(self.selection_end), text);
        let end = clamp_offset(self.selection_start.max(self.selection_end), text);
        let inside = inside_formula(text, start);
        let (inserted, prefix) = if inside {
            (call.to_string(), 0)
        } else {
            (format!("${call}$"), 1)
        };
        let next = splice_text(text, start, end, &inserted);
        let caret = start + prefix + caret_in_call;
        return self.with_edit(TemplateEditorEdit {
            text: next,
            selection_start: caret,
            selection_end: caret,
        });
    }

    /// What:
    ///  `fn results_for(&self, kind: TemplateKind, text: &str) -> Vec<TemplateResult>` evaluates one template text against every preview track.
    /// Why:
    ///  Validation and the preview both need the engine's result for each track.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function resultsFor(template, text) { return previewTracks.map(t => evaluate(text, t, template.fields)); }
    /// ```
    fn results_for(&self, kind: TemplateKind, text: &str) -> Vec<TemplateResult> {
        return self
            .preview_tracks
            .iter()
            .map(|track| return evaluate_for(kind, text, track))
            .collect();
    }

    /// What:
    ///  `fn applied_line(&self, kind: TemplateKind, track: &PlayingTrackFields) -> Result<String, TemplateEditorStateError>` gives the last applied template's line for one track.
    /// Why:
    ///  The applied template should always apply,
    ///  so a refusal here means the stored state is broken,
    ///  and that is reported as an error rather than shown.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function appliedLine(template, track): string { /* the shown text, or throw */ }
    /// ```
    fn applied_line(&self, kind: TemplateKind, track: &PlayingTrackFields) -> Result<String, TemplateEditorStateError> {
        let applied = self.last_valid.get(kind);
        return match evaluate_for(kind, applied, track) {
            TemplateResult::Shown(text) => Ok(text),
            TemplateResult::Refused(errors) => Err(TemplateEditorStateError {
                message: format!(
                    "The applied {} template no longer applies: {}",
                    kind.label(),
                    error_lines(&errors).join("; ")
                ),
            }),
        };
    }

    /// What:
    ///  `fn first_track(&self) -> &PlayingTrackFields` is the first preview track,
    ///  whose field values the field list and Settings read.
    /// Why:
    ///  `initial` always stores at least one preview track,
    ///  and the field is private,
    ///  so the first track is always present.
    /// Panics:
    ///  only if the preview list is empty,
    ///  which no public constructor or transition can produce.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const first = this.previewTracks[0];
    /// ```
    fn first_track(&self) -> &PlayingTrackFields {
        return self
            .preview_tracks
            .first()
            .expect("the preview always holds at least one track, set by initial");
    }
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_editor_state_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_editor_state_tests.rs"]
mod tests;

/// What:
///  `mod preview_tests;` pulls in the preview group from `template_editor_preview_tests.rs`.
/// Why:
///  A second file keeps each test file under the code-line budget.
#[cfg(test)]
#[path = "template_editor_preview_tests.rs"]
mod preview_tests;
