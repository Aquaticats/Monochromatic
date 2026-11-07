//! Supporting-line function for track rows,
//!  built from a track rows template.
//!
//! Ported from `trackListEntries` in `core/TrackListEntries.kt`,
//!  where `supportingLineFor` evaluates
//! the template and drops a refused or blank result.
//!  `track_list_entries` takes the returned function
//! as its supporting-line parameter,
//!  so the template engine is reached only through this module.

/// What:
///      `use crate::template_eval::evaluate_parsed_template;` imports the one function that shows a parsed template's text.
/// Why:
///       Evaluation is the only place that reads field values and runs functions,
///  so this module reuses it.
use crate::template_eval::evaluate_parsed_template;
/// What:
///      `use crate::template_parse::{ParsedTemplate, parse_template};` imports the parser.
/// Why:
///       The template is parsed once when the function is built,
/// not once per row.
use crate::template_parse::{ParsedTemplate, parse_template};
/// What:
///      `use crate::template_model::{...};` imports the field list,
///  the result type,
///  and the track fields.
/// Why:
///       The track rows template reads these fields,
///  and the result says whether the template applied.
use crate::template_model::{TemplateField, TemplateResult, TrackFields, track_template_fields};

/// What:
///      `pub trait SupportingLine { fn evaluate(&self, track: &TrackFields) -> Option<String>; }`.
///  Something that shows a row's supporting line for one track.
/// Why:
///       `track_list_entries`,
/// `build_player_content` and `view` take the line maker as a parameter,
/// so the window passes the template's own `TemplateSupportingLine` and tests pass a plain function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface SupportingLine { evaluate(track: TrackFields): string | null }
/// ```
pub trait SupportingLine {
    /// What:
    /// `evaluate` shows the line for one track,
    /// or nothing when the template refuses it or shows only blanks.
    /// Why:
    /// a row with no line shows only its title.
    fn evaluate(&self, track: &TrackFields) -> Option<String>;
}

/// A plain function of the same shape is a supporting-line maker too,
/// so a test can pass one.
impl<F: Fn(&TrackFields) -> Option<String>> SupportingLine for F {
    /// What:
    /// `evaluate` calls the function.
    /// Why:
    /// it lets a plain function stand in for the template's maker.
    fn evaluate(&self, track: &TrackFields) -> Option<String> {
        return self(track);
    }
}

/// What:
///      `pub struct TemplateSupportingLine { parsed, fields }`.
///  The supporting-line maker of one track rows template.
/// Why:
///       The template is parsed once when the maker is built,
/// not once per row,
/// and the maker owns the parsed template and the track field list,
/// so the caller can pass it to `track_list_entries` or `build_player_content` without holding either one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TemplateSupportingLine implements SupportingLine { constructor(private parsed: ParsedTemplate, private fields: TemplateField[]) {} }
/// ```
pub struct TemplateSupportingLine {
    /// The template,
    /// parsed once.
    parsed: ParsedTemplate,
    /// The track fields the template may read.
    fields: Vec<TemplateField>,
}

/// The template's maker shows the template's text for a track.
impl SupportingLine for TemplateSupportingLine {
    /// What:
    /// `evaluate` shows the template's text for the track,
    /// trimmed,
    /// or nothing when the template refuses it or the text is blank.
    /// Why:
    /// a refused or blank result is dropped,
    /// as `supportingLineFor` does in the Kotlin app.
    fn evaluate(&self, track: &TrackFields) -> Option<String> {
        return match evaluate_parsed_template(&self.parsed, track, &self.fields) {
            TemplateResult::Shown(text) => {
                let trimmed: String = text.trim().to_string();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            }
            TemplateResult::Refused(_) => None,
        };
    }
}

/// What:
///      `pub fn supporting_line_from_template(template) -> TemplateSupportingLine`.
///  Builds the supporting-line maker of a track rows template.
/// Why:
///       The window builds one maker per projection and hands it to the row builders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const supportingLineFromTemplate = (template: string): SupportingLine => new TemplateSupportingLine(parseTemplate(template), trackTemplateFields());
/// ```
pub fn supporting_line_from_template(template: &str) -> TemplateSupportingLine {
    return TemplateSupportingLine { parsed: parse_template(template), fields: track_template_fields() };
}

/// What:
///      `#[cfg(test)] #[path = "track_supporting_line_tests.rs"] mod tests;` declares the test-only submodule.
/// Why:
///       Keep `track_supporting_line.rs` to production code,
///  with the tests beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // trackSupportingLine.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "track_supporting_line_tests.rs"]
mod tests;
