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
/// `mod shown_values;` the read-only methods that turn the state into what the Settings page
/// and the editor pane draw (`src/template_editor_state/shown_values.rs`).
/// Why:
/// the state and its immutable transitions are one role and the derived values another,
/// so each
/// half can be read on its own;
/// nothing in the second half changes the state.
mod shown_values;

/// What:
///  imports the editor values,
///  constants,
///  and helpers the state uses.
/// Why:
///  The state builds its previews and edits from them.
use crate::template_editor::{
    TEMPLATE_PREVIEW_ROW_COUNT,
    TemplateEditorEdit,
    TemplateEditorStateError,
    TemplateTexts,
    clamp_offset,
    evaluate_for,
    inside_formula,
    splice_text,
    template_sample_tracks,
};
/// What:
///  imports the UTF-16 length helper.
/// Why:
///  Caret offsets are UTF-16 positions.
use crate::template_format::utf16_length;
/// What:
///  imports the track,
///  template kind and result types.
/// Why:
///  The state stores and evaluates them.
use crate::template_model::{
    PlayingTrackFields,
    TemplateKind,
    TemplateResult,
    error_lines,
};

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
            library
                .iter()
                .take(TEMPLATE_PREVIEW_ROW_COUNT)
                .cloned()
                .collect()
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
    ///  `pub fn with_edit(&self, edit: TemplateEditorEdit) -> Self` applies one field edit.
    /// Why:
    ///  The typed text always becomes the draft,
    ///  and it becomes the applied template only when it applies to every preview track.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withEdit(edit: Edit): State { /* draft = text; applied = valid ? text : applied; clamp caret */ }
    /// ```
    pub fn with_edit(
        &self,
        edit: TemplateEditorEdit,
    ) -> Self {
        let mut applies = true;
        for result in &self.results_for(self.kind, &edit.text) {
            if !matches!(result, TemplateResult::Shown(_)) {
                applies = false;
                break;
            }
        }
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
    pub fn insert_field(
        &self,
        mode: &str,
    ) -> Self {
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
    pub fn insert_function(
        &self,
        name: &str,
    ) -> Self {
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
    pub fn open(
        &self,
        template: TemplateKind,
    ) -> Self {
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
    pub fn with_focus(
        &self,
        focused: bool,
    ) -> Self {
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
    fn insert_call(
        &self,
        call: &str,
        caret_in_call: usize,
    ) -> Self {
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
    fn results_for(
        &self,
        kind: TemplateKind,
        text: &str,
    ) -> Vec<TemplateResult> {
        let mut results: Vec<TemplateResult> = Vec::with_capacity(self.preview_tracks.len());
        for track in &self.preview_tracks {
            results.push(evaluate_for(kind, text, track));
        }
        return results;
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
    fn applied_line(
        &self,
        kind: TemplateKind,
        track: &PlayingTrackFields,
    ) -> Result<String, TemplateEditorStateError> {
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
