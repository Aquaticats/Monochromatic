// What:     Unit tests for `track_supporting_line.rs`, pulled in by
//           `#[cfg(test)] #[path = "track_supporting_line_tests.rs"] mod tests;` at the bottom of
//           `track_supporting_line.rs`. Each case builds the function from a template and runs it on a
//           fixture track. Expected text comes from the fixtures and the template reference, not from a stub.
// Why:      Keep the tests beside the code without inflating `track_supporting_line.rs`
//           (sibling `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need `supporting_line_from_template` and the track fields type.
use super::*;
use crate::template_fixtures::{analysed, bare, waiting};
use crate::template_model::DEFAULT_TRACK_TEMPLATE;

// What:     `#[test]` marks the next function as a test case.
// Why:      The default template shows the duration and the true peak once both are known.
#[test]
fn default_template_shows_length_and_peak() {
    let line = supporting_line_from_template(DEFAULT_TRACK_TEMPLATE);
    assert_eq!(line(&analysed()), Some("4:35 \u{2212}0.3 dBTP".to_string()));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Before analysis the peak is empty, so the trailing space is trimmed and only the duration remains.
#[test]
fn default_template_with_only_length_shows_duration() {
    let line = supporting_line_from_template(DEFAULT_TRACK_TEMPLATE);
    assert_eq!(line(&waiting()), Some("4:35".to_string()));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A track with neither duration nor peak shows only blanks, so the line is dropped.
#[test]
fn default_template_with_neither_length_nor_peak_gives_none() {
    let line = supporting_line_from_template(DEFAULT_TRACK_TEMPLATE);
    assert_eq!(line(&bare()), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A custom template reads the folder and file fields, and the text between formulas is shown.
#[test]
fn custom_template_joins_folder_and_file_fields() {
    let line = supporting_line_from_template("$mi(folder)$ / $mi(file)$");
    assert_eq!(
        line(&analysed()),
        Some("Camellia / かめりあ(Camellia) - Another Xronixle".to_string())
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Text outside a formula is always shown, so a plain label stays in front of the field.
#[test]
fn custom_template_keeps_text_outside_formulas() {
    let line = supporting_line_from_template("Now: $mi(title)$");
    assert_eq!(line(&analysed()), Some("Now: Another Xronixle".to_string()));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Leading and trailing whitespace around the shown text is trimmed.
#[test]
fn shown_text_is_trimmed() {
    let line = supporting_line_from_template("  $mi(title)$  ");
    assert_eq!(line(&analysed()), Some("Another Xronixle".to_string()));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A template that shows only whitespace gives no line, as the Kotlin blank check does.
#[test]
fn whitespace_only_result_gives_none() {
    let line = supporting_line_from_template("  $mi(peak)$  ");
    assert_eq!(line(&waiting()), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An unknown function name is a mistake, so the engine refuses the template and no line is shown.
#[test]
fn invalid_template_gives_none() {
    let line = supporting_line_from_template("$nosuch(mi(title))$");
    assert_eq!(line(&analysed()), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An empty template shows nothing, so the line is dropped.
#[test]
fn empty_template_gives_none() {
    let line = supporting_line_from_template("");
    assert_eq!(line(&analysed()), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The returned function reads each track it is given, so one template serves a whole list.
#[test]
fn one_function_reads_each_track_it_is_given() {
    let line = supporting_line_from_template("$mi(title)$");
    assert_eq!(line(&analysed()), Some("Another Xronixle".to_string()));
    assert_eq!(line(&waiting()), Some("Exit This Earth's Atomosphere".to_string()));
}

// What:     Parsing the template once gives the lines that parsing it for every track gave.
// Why:      The function parses when it is built so a page of thirty thousand rows does not parse thirty thousand times;
//           the result for each template and track, including refused and blank ones, must be the one `evaluate_template` gives.
#[test]
fn parsing_once_gives_what_parsing_per_track_gave() {
    let fields = track_template_fields();
    for template in [DEFAULT_TRACK_TEMPLATE, "$mi(title)$ / $mi(folder)$", "$nosuch(mi(title))$", "", "plain text", "$tf(mi(len), m:ss)$"] {
        let line = supporting_line_from_template(template);
        for track in [analysed(), waiting(), bare()] {
            let expected: Option<String> = match crate::template_eval::evaluate_template(template, &track, &fields) {
                TemplateResult::Shown(text) => Some(text.trim().to_string()).filter(|trimmed| return !trimmed.is_empty()),
                TemplateResult::Refused(_) => None,
            };
            assert_eq!(line(&track), expected, "template {template:?}");
        }
    }
}
