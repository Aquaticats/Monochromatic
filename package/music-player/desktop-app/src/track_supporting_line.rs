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
///      `use crate::template_parse::parse_template;` imports the parser.
/// Why:
///       The template is parsed once when the function is built, not once per row.
use crate::template_parse::parse_template;
/// What:
///      `use crate::template_model::{...};` imports the field list,
///  the result type,
///  and the track fields.
/// Why:
///       The track rows template reads these fields,
///  and the result says whether the template applied.
use crate::template_model::{TemplateField, TemplateResult, TrackFields, track_template_fields};

/// What:
///      `pub fn supporting_line_from_template(template) -> impl Fn(&TrackFields) -> Option<String>`.
///  Returns a function that shows the template's text for one track.
/// Why:
///       The returned closure owns the parsed template and the track field list,
///  so the caller can pass it
///  to `track_list_entries` or `build_player_content` without holding either one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const supportingLineFromTemplate = (template: string) => (track: TrackFields): string | null => {
///   const result = evaluateTemplate(template, track);
///   if (!result.valid) return null;
///   return result.text.trim() || null;
/// };
/// ```
pub fn supporting_line_from_template(template: &str) -> impl Fn(&TrackFields) -> Option<String> {
    let parsed = parse_template(template);
    let fields: Vec<TemplateField> = track_template_fields();
    return move |track: &TrackFields| -> Option<String> {
        return match evaluate_parsed_template(&parsed, track, &fields) {
            TemplateResult::Shown(text) => Some(text.trim().to_string()).filter(|trimmed| return !trimmed.is_empty()),
            TemplateResult::Refused(_) => None,
        };
    };
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
