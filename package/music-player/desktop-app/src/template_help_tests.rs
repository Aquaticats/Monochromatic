// Unit tests for `template_help.rs`, pulled in by `#[cfg(test)] #[path = "template_help_tests.rs"] mod tests;`.
//
// Mirrors the Kotlin `helpFollowsTheCaretIntoTheInnermostCall` and the template-kind help cases.
// The caret is written as a `|` marker, and the test strings are ASCII so byte and UTF-16 offsets agree.

use super::*;
use crate::template_model::{DEFAULT_TRACK_TEMPLATE, playing_template_fields, template_function, track_template_fields};

// What: `fn help_label(text: &str) -> Option<String>` runs help with the caret at the `|` marker and joins the signature and argument.
// Why: Writing the caret as a marker keeps each help case readable beside its expected label.
//
// In TS you'd write (pseudocode):
// ```ts
// function help({ text, marker = "|" }: HelpCase): string | undefined { ... }
// ```
fn help_label(text: &str) -> Option<String> {
    let caret = text.find('|').expect("a help case names its caret with a | marker");
    let stripped = text.replacen('|', "", 1);
    return help_at(&stripped, caret, &track_template_fields()).map(|found| return format!("{} / {}", found.signature, found.parameter));
}

// What: `fn help_follows_the_caret_into_the_innermost_call(...)` checks which call and argument the caret reports.
// Why: The editor shows help for the innermost call around the caret, and nothing outside a call.
//
// In TS you'd write (pseudocode):
// ```ts
// test("help follows the caret into the innermost call", () => { ... });
// ```
#[test]
fn help_follows_the_caret_into_the_innermost_call() {
    assert_eq!(help_label("plain| text"), None);
    assert_eq!(help_label("$|mi(title)$"), None);
    assert_eq!(help_label("$mi|(title)$"), None);
    assert_eq!(help_label("$mi(|title)$"), Some(String::from("mi(field) / field")));
    assert_eq!(help_label("$mi(title|)$"), Some(String::from("mi(field) / field")));
    assert_eq!(help_label("$mi(title)|$"), None);
    assert_eq!(help_label("$tf(|"), Some(String::from("tf(seconds, [format]) / seconds")));
    assert_eq!(help_label("$tf(mi(|len), m:ss)$"), Some(String::from("mi(field) / field")));
    assert_eq!(help_label("$tf(mi(len)|, m:ss)$"), Some(String::from("tf(seconds, [format]) / seconds")));
    assert_eq!(help_label("$tf(mi(len), m:|ss)$"), Some(String::from("tf(seconds, [format]) / format")));
    assert_eq!(help_label("$tf(mi(len), m:ss)$ $mi(peak|)$"), Some(String::from("mi(field) / field")));
    assert_eq!(help_label("$tc(cut, mi(title), |5)$"), Some(String::from("tc(mode, text, [length]) / length")));
    assert_eq!(help_label("$tc(a, b, c, |d)$"), Some(String::from("tc(mode, text, [length]) / length")));
    assert_eq!(help_label("$if(|1)$"), None);
    assert_eq!(help_label("$xx(|1)$"), None);
    assert_eq!(
        help_label("$mi(title)$ and $tc(up, |"),
        Some(String::from("tc(mode, text, [length]) / text"))
    );
}

// What: `fn help_description_of_a_duration_is_the_seconds_sentence(...)` checks the help sentence for an open `tf` call.
// Why: The sentence comes from the function table, so the editor shows the reference's wording.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(helpAt("$tf(", 4).description).toBe(functions.tf.parameters[0].description);
// ```
#[test]
fn help_description_of_a_duration_is_the_seconds_sentence() {
    let found = help_at("$tf(", 4, &track_template_fields());
    let expected = template_function("tf").map(|definition| return definition.parameters[0].description.clone());
    assert_eq!(found.map(|help| return help.description), expected);
}

// What: `fn mi_help_names_the_fields_of_its_own_template(...)` checks the `mi` sentence for the track rows and the playing track.
// Why: The track rows and the playing track expose different fields, so the help lists each template's own.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(helpAt("$mi()$", 4, playingFields).description).toBe("Field: one of title, ..., track, total.");
// ```
#[test]
fn mi_help_names_the_fields_of_its_own_template() {
    let playing_help = help_at("$mi()$", 4, &playing_template_fields()).map(|help| return help.description);
    assert_eq!(
        playing_help,
        Some(String::from("Field: one of title, file, ext, folder, path, len, peak, track, total."))
    );
    let track_help = help_at("$mi()$", 4, &track_template_fields()).map(|help| return help.description);
    assert_eq!(track_help, Some(String::from("Field: one of title, file, ext, folder, path, len, peak.")));
}

// What: `fn default_line_help_follows_the_caret_in_the_duration_call(...)` checks the help inside the default line.
// Why: The default line holds two formulas, and the caret must reach the call in whichever one it sits.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(helpAt(DEFAULT, 15, trackFields)?.parameter).toBe("format");
// ```
#[test]
fn default_line_help_follows_the_caret_in_the_duration_call() {
    let found = help_at(DEFAULT_TRACK_TEMPLATE, 15, &track_template_fields());
    assert_eq!(found.map(|help| return help.parameter), Some("format"));
}
