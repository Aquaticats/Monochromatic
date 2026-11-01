//! Template vocabulary:
//!  the fields a template may read,
//!  the two templates and
//! their defaults,
//!  the function table with its argument help,
//!  the track value
//! sources,
//!  and the mistake and result types.
//!
//! Nothing here evaluates text.
//!  `template_eval` reads these values and
//! `template_help` reads the function table.

/// What:
///  `use crate::template_format::number_text;` imports the JavaScript number printer.
/// Why:
///  A track length prints through the same rule as the reference.
use crate::template_format::number_text;

/// What:
///  `pub struct TemplateField` is one field a template may read,
///  named by the mode word used inside `mi(...)`.
/// Why:
///  Settings lists each field with its label,
///  and the editor help shows the same description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateField = { mode: string; label: string; description: string };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateField {
    /// Mode word used inside `mi(...)`.
    pub mode: &'static str,
    /// Label Settings shows for the field.
    pub label: &'static str,
    /// Sentence the editor help shows for the field.
    pub description: &'static str,
}

/// What:
///  `pub fn track_template_fields() -> Vec<TemplateField>` lists the fields a track row may read.
/// Why:
///  Track rows never see the playing track's place or folder total,
///  so they get the shorter list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trackTemplateFields: TemplateField[] = [{ mode: "title", label: "Title", description: "..." }];
/// ```
pub fn track_template_fields() -> Vec<TemplateField> {
    return vec![
        TemplateField { mode: "title", label: "Title", description: "The name the row shows as its title." },
        TemplateField { mode: "file", label: "File name", description: "The file name without its extension." },
        TemplateField { mode: "ext", label: "Extension", description: "The file extension." },
        TemplateField { mode: "folder", label: "Folder", description: "The folder that holds the file." },
        TemplateField { mode: "path", label: "Path", description: "The path from the library root." },
        TemplateField { mode: "len", label: "Duration", description: "The duration in seconds; tf formats it." },
        TemplateField {
            mode: "peak",
            label: "True peak",
            description: "The true peak with its unit, such as \u{2212}1.2 dBTP, or nothing before the file is analyzed.",
        },
    ];
}

/// What:
///  `pub fn playing_template_fields() -> Vec<TemplateField>` is the track rows' list plus `track` and `total`.
/// Why:
///  The playing track also shows where it sits in its folder and how many tracks the folder holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const playingTemplateFields = [...trackTemplateFields, { mode: "track" }, { mode: "total" }];
/// ```
pub fn playing_template_fields() -> Vec<TemplateField> {
    return [
        track_template_fields(),
        vec![
            TemplateField {
                mode: "track",
                label: "Place in folder",
                description: "Where the track is in its folder, counting from 1.",
            },
            TemplateField { mode: "total", label: "Tracks in folder", description: "How many tracks the folder holds." },
        ],
    ]
    .concat();
}

/// What:
///  `pub const DEFAULT_TRACK_TEMPLATE: &str` is the supporting line a track row starts with.
/// Why:
///  It shows the duration,
///  a space,
///  then the true peak,
///  which is empty before analysis.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_TRACK_TEMPLATE = "$tf(mi(len), m:ss)$ $mi(peak)$";
/// ```
pub const DEFAULT_TRACK_TEMPLATE: &str = "$tf(mi(len), m:ss)$ $mi(peak)$";

/// What:
///  `pub const DEFAULT_PLAYING_TEMPLATE: &str` is the supporting line the playing track starts with.
/// Why:
///  It shows the place in the folder,
///  the folder total,
///  and the true peak.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const playingDefault = "$mi(track)$ of $mi(total)$ $mi(peak)$";
/// ```
pub const DEFAULT_PLAYING_TEMPLATE: &str = "$mi(track)$ of $mi(total)$ $mi(peak)$";

/// What:
///  `pub enum TemplateKind` names the two templates Settings lists,
///  each with its label,
///  default text,
///  and allowed fields.
/// Why:
///  The track rows and the playing track differ in default text and in the fields they may read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateKind = "track" | "playing";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateKind {
    /// Supporting text for each track row in the library.
    TrackRows,
    /// Supporting text for the track that is playing.
    PlayingTrack,
}

/// What:
///  the template heading,
///  default text,
///  and field list,
///  per kind.
/// Why:
///  Each method switches on the two templates the same way.
impl TemplateKind {
    /// What:
    ///  `pub fn all() -> [TemplateKind; 2]` lists every template in Settings order.
    /// Why:
    ///  Settings and the editor's Settings lines walk both templates in the same order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kinds = ["track", "playing"] as const;
    /// ```
    pub fn all() -> [TemplateKind; 2] {
        return [TemplateKind::TrackRows, TemplateKind::PlayingTrack];
    }

    /// What:
    ///  `pub fn label(self) -> &'static str` is the heading Settings shows for the template.
    /// Why:
    ///  The applied-template mistake names the template by this heading.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const label = { track: "Track rows", playing: "Playing track" }[kind];
    /// ```
    pub fn label(self) -> &'static str {
        return match self {
            TemplateKind::TrackRows => "Track rows",
            TemplateKind::PlayingTrack => "Playing track",
        };
    }

    /// What:
    ///  `pub fn default_template(self) -> &'static str` is the text the template starts from.
    /// Why:
    ///  Each template has its own default,
    ///  and the way back to it must always apply.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const defaults = { track: DEFAULT_TRACK_TEMPLATE, playing: "$mi(track)$ of $mi(total)$ $mi(peak)$" };
    /// ```
    pub fn default_template(self) -> &'static str {
        return match self {
            TemplateKind::TrackRows => DEFAULT_TRACK_TEMPLATE,
            TemplateKind::PlayingTrack => DEFAULT_PLAYING_TEMPLATE,
        };
    }

    /// What:
    ///  `pub fn fields(self) -> Vec<TemplateField>` lists the fields the template may read.
    /// Why:
    ///  A field outside this list is refused,
    ///  so each template gets its own list.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fields = kind === "track" ? trackTemplateFields : playingTemplateFields;
    /// ```
    pub fn fields(self) -> Vec<TemplateField> {
        return match self {
            TemplateKind::TrackRows => track_template_fields(),
            TemplateKind::PlayingTrack => playing_template_fields(),
        };
    }
}

/// What:
///  `pub struct TemplateParameter` records one argument of a template function.
/// Why:
///  The editor help names each argument and says what it is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateParameter = { name: string; description: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateParameter {
    /// Argument name shown in the signature.
    pub name: &'static str,
    /// Sentence the editor help shows for the argument.
    pub description: String,
}

/// What:
///  `pub enum FunctionName` names the three template functions.
/// Why:
///  Evaluation and help match on this closed set,
///  so no unknown name reaches a dispatch branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FunctionName = "mi" | "tf" | "tc";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionName {
    /// Reads one field of the track.
    Mi,
    /// Formats seconds as a duration.
    Tf,
    /// Converts text by mode.
    Tc,
}

/// What:
///  `pub struct TemplateFunction` describes one template function's signature and arity.
/// Why:
///  Evaluation checks argument counts against it,
///  and the editor help displays it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateFunction = { signature: string; minimum: number; maximum: number; parameters: TemplateParameter[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateFunction {
    /// Which function this is.
    pub name: FunctionName,
    /// Call shape shown in the help,
    ///  such as `mi(field)`.
    pub signature: &'static str,
    /// Fewest arguments the call may take.
    pub minimum: usize,
    /// Most arguments the call may take.
    pub maximum: usize,
    /// Arguments in call order,
    ///  each with its help text.
    pub parameters: Vec<TemplateParameter>,
}

/// What:
///  `pub fn template_function(name: &str) -> Option<TemplateFunction>` looks up a function by its written name.
/// Why:
///  A table lookup,
///  not a property read,
///  makes names such as `constructor` unknown functions instead of inherited members.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const functions: Record<string, FunctionHelp> = { mi: { ... }, tf: { ... }, tc: { ... } };
/// ```
pub fn template_function(name: &str) -> Option<TemplateFunction> {
    return match name {
        "mi" => Some(TemplateFunction {
            name: FunctionName::Mi,
            signature: "mi(field)",
            minimum: 1,
            maximum: 1,
            parameters: vec![TemplateParameter {
                name: "field",
                description: format!("Field: one of {}.", field_modes(&track_template_fields())),
            }],
        }),
        "tf" => Some(TemplateFunction {
            name: FunctionName::Tf,
            signature: "tf(seconds, [format])",
            minimum: 1,
            maximum: 2,
            parameters: vec![
                TemplateParameter { name: "seconds", description: String::from("Seconds: a duration, such as mi(len).") },
                TemplateParameter {
                    name: "format",
                    description: String::from(
                        "Format: h, m and s for hours, minutes and seconds; a doubled letter pads with a zero; \
                         text between apostrophes is kept, and any other letter is a mistake. Without it, m:ss.",
                    ),
                },
            ],
        }),
        "tc" => Some(TemplateFunction {
            name: FunctionName::Tc,
            signature: "tc(mode, text, [length])",
            minimum: 2,
            maximum: 3,
            parameters: vec![
                TemplateParameter {
                    name: "mode",
                    description: String::from("Mode: low, up or cap to change case; cut to keep the first characters."),
                },
                TemplateParameter { name: "text", description: String::from("Text: the text to convert.") },
                TemplateParameter { name: "length", description: String::from("Length: how many characters cut keeps.") },
            ],
        }),
        _ => None,
    };
}

/// What:
///  `pub fn field_modes(fields: &[TemplateField]) -> String` joins the mode words with `, `.
/// Why:
///  The `mi` help and the function table both list a template's fields in the same form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modes = fields.map(field => field.mode).join(", ");
/// ```
pub fn field_modes(fields: &[TemplateField]) -> String {
    return fields.iter().map(|field| return field.mode).collect::<Vec<&str>>().join(", ");
}

/// What:
///  `pub trait TemplateTrack` is a source of field text for one track.
/// Why:
///  The track rows and the playing track expose different fields,
///  and one evaluator serves both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TemplateTrack { fieldText(mode: string): string | undefined }
/// ```
pub trait TemplateTrack {
    /// Text of one field,
    ///  or `None` when the track has no value for it or the mode names no field.
    fn field_text(&self, mode: &str) -> Option<String>;
}

/// What:
///  `pub struct TrackFields` holds one track's own values for the track rows template.
/// Why:
///  Each field has a fixed type,
///  and a missing duration or peak is `None` rather than an absent key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrackFields = { title: string; file: string; ext: string; folder: string; path: string;
///   len?: number; peak?: string };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TrackFields {
    /// The name the row shows as its title.
    pub title: String,
    /// The file name without its extension.
    pub file: String,
    /// The file extension.
    pub ext: String,
    /// The folder that holds the file.
    pub folder: String,
    /// The path from the library root.
    pub path: String,
    /// The duration in seconds,
    ///  or `None` when unknown.
    pub len: Option<f64>,
    /// The true peak text with its unit,
    ///  or `None` before analysis.
    pub peak: Option<String>,
}

/// What:
///  `TrackFields` answers each field mode from its own values.
/// Why:
///  The track rows template reads only these fields.
impl TemplateTrack for TrackFields {
    /// What:
    ///  `fn field_text(&self, mode: &str) -> Option<String>` maps a mode word to this track's text.
    /// Why:
    ///  Each mode reads one property,
    ///  and an unknown mode yields `None` so the caller can refuse it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fieldText(mode: string) { return this[mode as keyof TrackFields]; }
    /// ```
    fn field_text(&self, mode: &str) -> Option<String> {
        return match mode {
            "title" => Some(self.title.clone()),
            "file" => Some(self.file.clone()),
            "ext" => Some(self.ext.clone()),
            "folder" => Some(self.folder.clone()),
            "path" => Some(self.path.clone()),
            "len" => self.len.map(number_text),
            "peak" => self.peak.clone(),
            _ => None,
        };
    }
}

/// What:
///  `pub struct PlayingTrackFields` adds the place and folder total to one track.
/// Why:
///  The playing template reads the same seven fields plus `track` and `total`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlayingTrackFields = TrackFields & { track?: number; total?: number };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PlayingTrackFields {
    /// The track's own fields.
    pub track: TrackFields,
    /// Where the track is in its folder,
    ///  counting from 1,
    ///  or `None` when unknown.
    pub place_in_folder: Option<usize>,
    /// How many tracks the folder holds,
    ///  or `None` when unknown.
    pub tracks_in_folder: Option<usize>,
}

/// What:
///  `PlayingTrackFields` answers `track` and `total` itself and delegates the rest.
/// Why:
///  The place in the folder is not part of the track's own record.
impl TemplateTrack for PlayingTrackFields {
    /// What:
    ///  `fn field_text(&self, mode: &str) -> Option<String>` answers `track` and `total` itself and delegates every other mode to the track.
    /// Why:
    ///  The folder place is not part of the track's own record,
    ///  so it is read from here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fieldText(mode: string) { return mode === "track" ? this.placeInFolder : ...; }
    /// ```
    fn field_text(&self, mode: &str) -> Option<String> {
        return match mode {
            "track" => self.place_in_folder.map(|place| return place.to_string()),
            "total" => self.tracks_in_folder.map(|total| return total.to_string()),
            _ => self.track.field_text(mode),
        };
    }
}

/// What:
///  `pub struct TemplateError` records one mistake,
///  the name it concerns,
///  and its position.
/// Why:
///  Each mistake is named after the function or word at fault and placed by UTF-16 offset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateError = { subject: string; message: string; at: number };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateError {
    /// The function name,
    ///  or `formula` or `text` for a structural mistake.
    pub subject: String,
    /// What is wrong,
    ///  in plain words.
    pub message: String,
    /// UTF-16 offset in the whole template where the mistake starts.
    pub at: usize,
}

/// What:
///  `pub enum TemplateResult` is either the shown text or the list of mistakes.
/// Why:
///  A template that does not apply shows no text at all,
///  so the two outcomes must not overlap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateResult = { valid: true; text: string } | { valid: false; errors: Err[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateResult {
    /// The template applied;
    ///  the string is the line to show.
    Shown(
        /// The line to show.
        String,
    ),
    /// The template does not apply;
    ///  the errors list each mistake in the order found.
    Refused(
        /// Every mistake,
        /// in the order found.
        Vec<TemplateError>,
    ),
}

/// What:
///  `pub fn error_lines(errors: &[TemplateError]) -> Vec<String>` formats each mistake as one line.
/// Why:
///  The editor shows one line per mistake,
///  ordered by where each sits in the template.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = [...errors].sort((a, b) => a.at - b.at).map(e => `${e.subject}: ${e.message}`);
/// ```
pub fn error_lines(errors: &[TemplateError]) -> Vec<String> {
    let mut ordered: Vec<&TemplateError> = errors.iter().collect();
    ordered.sort_by_key(|error| return error.at);
    return ordered
        .iter()
        .map(|error| return format!("{}: {}", error.subject, error.message))
        .collect();
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_model_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_model_tests.rs"]
mod tests;
