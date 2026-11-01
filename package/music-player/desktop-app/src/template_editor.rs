//! Values the template editor passes around,
//!  its constants,
//!  and the helpers the
//! editor state shares:
//!  stand-in tracks,
//!  the caret clamp,
//!  and the formula test.
//!
//! The state itself lives in `template_editor_state`;
//!  this module has no state of its own.

/// What:
///  imports the standard error trait.
/// Why:
///  The state error implements it.
use std::error::Error;
/// What:
///  imports the formatting traits and types.
/// Why:
///  The state error implements `Display`.
use std::fmt;

/// What:
///  imports the evaluator.
/// Why:
///  `evaluate_for` runs one template against one track.
use crate::template_eval::evaluate_template;
/// What:
///  imports the UTF-16 length helper.
/// Why:
///  Caret clamps and the formula test count UTF-16 units.
use crate::template_format::utf16_length;
/// What:
///  imports the track,
///  template kind,
///  and result types.
/// Why:
///  The editor values and helpers are built from them.
use crate::template_model::{PlayingTrackFields, TemplateKind, TemplateResult, TemplateTrack, TrackFields};
/// What:
///  imports the parts,
///  the formula span rule,
///  and the template splitter.
/// Why:
///  `inside_formula` uses the same rule as help.
use crate::template_parse::{TemplatePart, formula_holds_caret, parse_template};

/// What:
///  `pub const TEMPLATE_PREVIEW_ROW_COUNT: usize = 2;` is how many library tracks the preview draws.
/// Why:
///  Two rows,
///  one analysed and one not,
///  show what a template yields with and without a true peak.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPLATE_PREVIEW_ROW_COUNT = 2;
/// ```
pub const TEMPLATE_PREVIEW_ROW_COUNT: usize = 2;

/// What:
///  `pub const TEMPLATE_SAMPLE_NOTE: &str` says the preview shows stand-in values.
/// Why:
///  The editor must say so whenever the open library holds no track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPLATE_SAMPLE_NOTE = "Your library has no tracks yet. These are sample values.";
/// ```
pub const TEMPLATE_SAMPLE_NOTE: &str = "Your library has no tracks yet. These are sample values.";

/// What:
///  `pub const TEMPLATE_LIBRARY_NOTE: &str` says the preview rows come from the open library.
/// Why:
///  The user should know whose tracks the preview is showing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPLATE_LIBRARY_NOTE = "From your library.";
/// ```
pub const TEMPLATE_LIBRARY_NOTE: &str = "From your library.";

/// What:
///  `pub const TEMPLATE_KEPT_NOTE: &str` says the rows keep the last template that applied.
/// Why:
///  While the text is invalid the rows do not change,
///  and the note explains why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPLATE_KEPT_NOTE = "Rows keep the last valid template.";
/// ```
pub const TEMPLATE_KEPT_NOTE: &str = "Rows keep the last valid template.";

/// What:
///  `pub const TEMPLATE_KEPT_EMPTY_NOTE: &str` says the rows keep the last valid template and that it shows nothing.
/// Why:
///  A cleared template is valid and shows no second line, so a rejected draft typed after it leaves empty rows,
///  and the note must say that this is the kept template and not a broken preview.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPLATE_KEPT_EMPTY_NOTE = "Rows keep the last valid template, which shows no second line.";
/// ```
pub const TEMPLATE_KEPT_EMPTY_NOTE: &str = "Rows keep the last valid template, which shows no second line.";

/// What:
///  `pub fn template_sample_tracks() -> Vec<PlayingTrackFields>` holds the two stand-in tracks.
/// Why:
///  Sample values show only while the open library holds no track;
///  one analyzed and one not yet analyzed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPLATE_SAMPLE_TRACKS = [
///   { title: "Track title", len: 200, peak: "−1.0 dBTP", place: 1, total: 16 },
///   { title: "Track not analyzed yet", len: 200, peak: undefined, place: 2, total: 16 },
/// ];
/// ```
pub fn template_sample_tracks() -> Vec<PlayingTrackFields> {
    return vec![
        stand_in("Track title", Some("\u{2212}1.0 dBTP"), 1),
        stand_in("Track not analyzed yet", None, 2),
    ];
}

/// What:
///  `fn stand_in(title: &str, peak: Option<&str>, place: usize) -> PlayingTrackFields` builds one stand-in track in the sample folder.
/// Why:
///  Both stand-ins share the file,
///  folder,
///  length,
///  and total,
///  so only the title,
///  peak,
///  and place differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standIn = (title, peak, place) => ({ title, file: "File name", ext: "flac", len: 200, peak, place, total: 16 });
/// ```
fn stand_in(title: &str, peak: Option<&str>, place: usize) -> PlayingTrackFields {
    return PlayingTrackFields {
        track: TrackFields {
            title: title.to_string(),
            file: String::from("File name"),
            ext: String::from("flac"),
            folder: String::from("Folder"),
            path: String::from("Folder/File name.flac"),
            len: Some(200.0),
            peak: peak.map(String::from),
        },
        place_in_folder: Some(place),
        tracks_in_folder: Some(16),
    };
}

/// What:
///  `pub struct TemplateEditorEdit` is one edit the template field reports:
///  the text after it and where the selection sits.
/// Why:
///  The field reports text and selection together,
///  so one value carries both into the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateEditorEdit = { text: string; selectionStart: number; selectionEnd: number };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateEditorEdit {
    /// Whole template text after the edit.
    pub text: String,
    /// Selection start after the edit,
    ///  in UTF-16 units.
    pub selection_start: usize,
    /// Selection end after the edit;
    ///  equal to the start when there is only a caret.
    pub selection_end: usize,
}

/// What:
///  `pub struct TemplatePreviewRow` is one row the preview draws.
/// Why:
///  The editor shows each track's title with the line the applied template yields for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplatePreviewRow = { title: string; supporting: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePreviewRow {
    /// The track's title as the row shows it.
    pub title: String,
    /// The applied template's line for the track;
    ///  empty when the template yields nothing.
    pub supporting: String,
}

/// What:
///  `pub struct TemplateEditorFieldEntry` is one insertable field of the field list.
/// Why:
///  The list shows each field's name,
///  the call a tap inserts,
///  and a value to show what it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateEditorFieldEntry = { label: string; mode: string; insert: string; value: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateEditorFieldEntry {
    /// Label the field list shows for the field.
    pub label: &'static str,
    /// Mode word used inside the call,
    ///  such as `len`.
    pub mode: &'static str,
    /// Call text a tap inserts at the caret,
    ///  such as `mi(len)`.
    pub insert: String,
    /// Value of the field for the first preview track;
    ///  empty when that track has none.
    pub value: String,
}

/// What:
///  `pub struct TemplateEditorStateError` signals that a stored template no longer applies to a preview track.
/// Why:
///  A stored template that stopped applying must fail loudly rather than show a wrong preview.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TemplateEditorStateError extends Error {}
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateEditorStateError {
    /// What no longer holds,
    ///  naming the template and each mistake.
    pub message: String,
}

/// What:
///  `Display` writes the state error message.
/// Why:
///  Callers that print the error see the stored text.
impl fmt::Display for TemplateEditorStateError {
    /// What:
    ///  `fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result` writes the message.
    /// Why:
    ///  Callers that print the error see the same text the state stores.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.message; }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.message);
    }
}

/// What:
///  marks the state error as a standard error.
/// Why:
///  Callers can box or chain it like any other error.
impl Error for TemplateEditorStateError {}

/// What:
///  `pub fn clamp_offset(offset: usize, text: &str) -> usize` keeps a caret offset inside the text.
/// Why:
///  The field can report offsets past the end after an edit,
///  and the state must never hold them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clampOffset = (offset: number, text: string) => Math.min(Math.max(offset, 0), text.length);
/// ```
pub fn clamp_offset(offset: usize, text: &str) -> usize {
    return offset.min(utf16_length(text));
}

/// What:
///  `pub fn inside_formula(text: &str, caret: usize) -> bool` tells whether a caret offset lies inside one of the text's formulas.
/// Why:
///  A field call goes bare inside a formula and wrapped in dollar signs outside one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const insideFormula = (text, caret) => parse(text).parts.some(p => isFormula(p) && inSpan(caret, p));
/// ```
pub fn inside_formula(text: &str, caret: usize) -> bool {
    return parse_template(text).parts.iter().any(|part| {
        return match part {
            TemplatePart::Formula(formula) => formula_holds_caret(formula, caret),
            TemplatePart::Literal(_) => false,
        };
    });
}

/// What:
///  `pub fn track_source(kind: TemplateKind, track: &PlayingTrackFields) -> &dyn TemplateTrack` picks the field source one template reads for a preview track.
/// Why:
///  Track rows read the track alone,
///  and the playing track also reads its place in the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function templateTrackFor(kind, track) { return kind === "playing" ? track : track.track; }
/// ```
pub fn track_source(kind: TemplateKind, track: &PlayingTrackFields) -> &dyn TemplateTrack {
    return match kind {
        TemplateKind::TrackRows => &track.track,
        TemplateKind::PlayingTrack => track,
    };
}

/// What:
///  `pub fn evaluate_for(kind: TemplateKind, text: &str, track: &PlayingTrackFields) -> TemplateResult` evaluates one template text for one preview track with that template's fields.
/// Why:
///  Validation,
///  the preview,
///  and the Settings line all need the engine's result for the same pairing of text,
///  track,
///  and fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = evaluateTemplate(text, templateTrackFor(kind, track), kind.fields);
/// ```
pub fn evaluate_for(kind: TemplateKind, text: &str, track: &PlayingTrackFields) -> TemplateResult {
    return evaluate_template(text, track_source(kind, track), &kind.fields());
}

/// What:
///  `pub struct TemplateTexts` holds one text per template,
///  keyed by kind.
/// Why:
///  Each template keeps its own text while the other is open,
///  so both are stored side by side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateTexts = Record<TemplateKind, string>;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateTexts {
    /// Text of the track rows template.
    track_rows: String,
    /// Text of the playing track template.
    playing_track: String,
}

/// What:
///  the defaults,
///  reads,
///  and copies of the two template texts.
/// Why:
///  The state keeps each template text in its own slot.
impl TemplateTexts {
    /// What:
    ///  `fn defaults() -> TemplateTexts` gives each template its default text.
    /// Why:
    ///  The editor starts from the defaults,
    ///  and both are also the last applied texts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const defaults = { track: DEFAULT_TRACK_TEMPLATE, playing: playingDefault };
    /// ```
    pub fn defaults() -> Self {
        return Self {
            track_rows: TemplateKind::TrackRows.default_template().to_string(),
            playing_track: TemplateKind::PlayingTrack.default_template().to_string(),
        };
    }

    /// What:
    ///  `fn get(&self, kind: TemplateKind) -> &str` reads one template's text.
    /// Why:
    ///  Each read names the template it wants,
    ///  so no text is read from the wrong slot.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = texts[kind];
    /// ```
    pub fn get(&self, kind: TemplateKind) -> &str {
        return match kind {
            TemplateKind::TrackRows => &self.track_rows,
            TemplateKind::PlayingTrack => &self.playing_track,
        };
    }

    /// What:
    ///  `fn with(&self, kind: TemplateKind, text: String) -> TemplateTexts` returns a copy with one template's text replaced.
    /// Why:
    ///  The state is immutable,
    ///  so each change builds the next set of texts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const next = { ...texts, [kind]: text };
    /// ```
    pub fn with(&self, kind: TemplateKind, text: String) -> Self {
        return match kind {
            TemplateKind::TrackRows => Self {
                track_rows: text,
                playing_track: self.playing_track.clone(),
            },
            TemplateKind::PlayingTrack => Self {
                track_rows: self.track_rows.clone(),
                playing_track: text,
            },
        };
    }
}

/// What:
///  `pub fn splice_text(text: &str, start: usize, end: usize, inserted: &str) -> String` replaces the UTF-16 span `start..end` with `inserted`.
/// Why:
///  Offsets are UTF-16 positions,
///  so the splice works on code units and converts back to text once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const next = text.slice(0, start) + inserted + text.slice(end);
/// ```
pub fn splice_text(text: &str, start: usize, end: usize, inserted: &str) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut joined: Vec<u16> = units[..start].to_vec();
    joined.extend(inserted.encode_utf16());
    joined.extend_from_slice(&units[end..]);
    return String::from_utf16_lossy(&joined);
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_editor_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_editor_tests.rs"]
mod tests;
