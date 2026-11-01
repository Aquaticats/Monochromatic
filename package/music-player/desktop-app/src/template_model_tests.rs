// Unit tests for `template_model.rs`, pulled in by `#[cfg(test)] #[path = "template_model_tests.rs"] mod tests;`.
//
// Each case mirrors a Kotlin or reference case about field lists, defaults, the function table,
// and the track value sources.

use super::*;
use crate::template_fixtures::{analysed, bare, playing, waiting};

// What: `fn modes(fields: &[TemplateField]) -> Vec<&str>` lists the mode words in order.
// Why: The field-list tests compare the order of modes, which is the contract the editor relies on.
//
// In TS you'd write (pseudocode):
// ```ts
// const modes = fields.map(field => field.mode);
// ```
fn modes(fields: &[TemplateField]) -> Vec<&str> {
    return fields.iter().map(|field| return field.mode).collect();
}

// What: `fn track_rows_list_seven_fields_in_order(...)` checks the track rows field list.
// Why: The list is the track rows' contract, so its order and count are checked.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(trackTemplateFields.map(f => f.mode)).toEqual(["title", "file", "ext", "folder", "path", "len", "peak"]);
// ```
#[test]
fn track_rows_list_seven_fields_in_order() {
    assert_eq!(modes(&track_template_fields()), vec!["title", "file", "ext", "folder", "path", "len", "peak"]);
    assert_eq!(track_template_fields()[6].label, "True peak");
    assert_eq!(
        track_template_fields()[6].description,
        "The true peak with its unit, such as \u{2212}1.2 dBTP, or nothing before the file is analyzed."
    );
}

// What: `fn playing_list_adds_place_and_total(...)` checks the playing track's list.
// Why: The playing track reads its place in the folder and the folder's track count, and nothing else extra.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(playingTemplateFields.map(f => f.mode)).toEqual([...trackModes, "track", "total"]);
// ```
#[test]
fn playing_list_adds_place_and_total() {
    assert_eq!(
        modes(&playing_template_fields()),
        vec!["title", "file", "ext", "folder", "path", "len", "peak", "track", "total"]
    );
    assert_eq!(playing_template_fields()[7].label, "Place in folder");
    assert_eq!(playing_template_fields()[8].description, "How many tracks the folder holds.");
}

// What: `fn kinds_keep_their_own_defaults_labels_and_fields(...)` checks the two templates' order, defaults, and fields.
// Why: The track rows and the playing track must not share field lists or default text.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(Object.keys(templates)).toEqual(["track", "playing"]);
// ```
#[test]
fn kinds_keep_their_own_defaults_labels_and_fields() {
    assert_eq!(TemplateKind::all(), [TemplateKind::TrackRows, TemplateKind::PlayingTrack]);
    assert_eq!(TemplateKind::TrackRows.label(), "Track rows");
    assert_eq!(TemplateKind::PlayingTrack.label(), "Playing track");
    assert_eq!(TemplateKind::TrackRows.default_template(), DEFAULT_TRACK_TEMPLATE);
    assert_eq!(TemplateKind::TrackRows.fields(), track_template_fields());
    assert_eq!(TemplateKind::PlayingTrack.fields(), playing_template_fields());
    assert_eq!(TemplateKind::PlayingTrack.default_template(), "$mi(track)$ of $mi(total)$ $mi(peak)$");
}

// What: `fn default_track_template_is_the_reference_text(...)` checks the default line byte for byte.
// Why: The default is the supporting line every track row starts with, so its characters are part of the contract.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(defaultTemplate).toBe("$tf(mi(len), m:ss)$ $mi(peak)$");
// ```
#[test]
fn default_track_template_is_the_reference_text() {
    assert_eq!(DEFAULT_TRACK_TEMPLATE, "$tf(mi(len), m:ss)$ $mi(peak)$");
}

// What: `fn function_table_describes_the_three_functions(...)` checks signatures, arity, and parameter names.
// Why: Evaluation and help both read the table, so its arity bounds and call shapes are the contract.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(functions.tc.signature).toBe("tc(mode, text, [length])");
// ```
#[test]
fn function_table_describes_the_three_functions() {
    let mi = template_function("mi").map_or(0, |definition| return definition.maximum);
    assert_eq!(mi, 1);
    let tf = template_function("tf").map_or((0, 0), |definition| return (definition.minimum, definition.maximum));
    assert_eq!(tf, (1, 2));
    let tc = template_function("tc").map_or(0, |definition| return definition.parameters.len());
    assert_eq!(tc, 3);
    assert_eq!(template_function("tc").map(|definition| return definition.signature), Some("tc(mode, text, [length])"));
    assert_eq!(template_function("mi").map(|definition| return definition.name), Some(FunctionName::Mi));
    assert_eq!(template_function("tf").map(|definition| return definition.name), Some(FunctionName::Tf));
    assert_eq!(template_function("tc").map(|definition| return definition.name), Some(FunctionName::Tc));
}

// What: `fn function_table_refuses_names_it_does_not_hold(...)` checks names that a plain object would wrongly accept.
// Why: `constructor`, `__proto__`, and `if` are not template functions, so they must be unknown.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(Object.hasOwn(functions, "constructor")).toBe(false);
// ```
#[test]
fn function_table_refuses_names_it_does_not_hold() {
    assert_eq!(template_function("if"), None);
    assert_eq!(template_function("constructor"), None);
    assert_eq!(template_function("__proto__"), None);
    assert_eq!(template_function("MI"), None);
}

// What: `fn mi_help_lists_the_default_fields(...)` checks the `mi` argument help with the track rows field list.
// Why: The editor's `mi` help names the fields of the template, and the function table holds the default list.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(functions.mi.parameters[0].description).toBe("Field: one of title, file, ext, folder, path, len, peak.");
// ```
#[test]
fn mi_help_lists_the_default_fields() {
    let mi_description = template_function("mi").map(|definition| return definition.parameters[0].description.clone());
    assert_eq!(mi_description, Some(String::from("Field: one of title, file, ext, folder, path, len, peak.")));
    assert_eq!(field_modes(&playing_template_fields()), "title, file, ext, folder, path, len, peak, track, total");
}

// What: `fn track_fields_read_each_mode(...)` checks every track mode, its number text, and the missing values.
// Why: A mode yields the track's text, a missing value yields nothing, and an unknown mode yields nothing for the caller to refuse.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(analysed.len).toBe(275);
// ```
#[test]
fn track_fields_read_each_mode() {
    let track = analysed();
    assert_eq!(track.field_text("title"), Some(String::from("Another Xronixle")));
    assert_eq!(track.field_text("file"), Some(String::from("かめりあ(Camellia) - Another Xronixle")));
    assert_eq!(track.field_text("ext"), Some(String::from("flac")));
    assert_eq!(track.field_text("folder"), Some(String::from("Camellia")));
    assert_eq!(track.field_text("path"), Some(String::from("Camellia/かめりあ(Camellia) - Another Xronixle.flac")));
    assert_eq!(track.field_text("len"), Some(String::from("275")));
    assert_eq!(track.field_text("peak"), Some(String::from("\u{2212}0.3 dBTP")));
    assert_eq!(waiting().field_text("peak"), None);
    assert_eq!(bare().field_text("len"), None);
    assert_eq!(bare().field_text("folder"), Some(String::new()));
    assert_eq!(track.field_text("track"), None);
    assert_eq!(track.field_text("__proto__"), None);
}

// What: `fn number_fields_print_with_javascript_text(...)` checks that a fractional, negative, and non-finite length print as JavaScript does.
// Why: The `len` field is printed with `String()` rules, so the same value must give the same text as the reference.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(String(-1.5)).toBe("-1.5");
// ```
#[test]
fn number_fields_print_with_javascript_text() {
    let mut fractional = analysed();
    fractional.len = Some(59.9);
    assert_eq!(fractional.field_text("len"), Some(String::from("59.9")));
    let mut negative = analysed();
    negative.len = Some(-1.5);
    assert_eq!(negative.field_text("len"), Some(String::from("-1.5")));
    let mut not_a_number = analysed();
    not_a_number.len = Some(f64::NAN);
    assert_eq!(not_a_number.field_text("len"), Some(String::from("NaN")));
}

// What: `fn playing_fields_answer_place_and_total(...)` checks that `track` and `total` come from the playing track, and the rest from the track.
// Why: The place and folder total are not part of the track's own record, so the playing track answers them itself.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(playing.fieldText("track")).toBe("1");
// ```
#[test]
fn playing_fields_answer_place_and_total() {
    let track = playing();
    assert_eq!(track.field_text("track"), Some(String::from("1")));
    assert_eq!(track.field_text("total"), Some(String::from("16")));
    assert_eq!(track.field_text("title"), Some(String::from("Another Xronixle")));
    let mut unknown_place = playing();
    unknown_place.place_in_folder = None;
    unknown_place.tracks_in_folder = None;
    assert_eq!(unknown_place.field_text("track"), None);
    assert_eq!(unknown_place.field_text("total"), None);
}

// What: `fn error_lines_keep_template_order(...)` checks that lines follow the offset, not the order found.
// Why: The editor lists mistakes where they sit in the template, so the user reads them top down.
//
// In TS you'd write (pseudocode):
// ```ts
// expect([...errors].sort((a, b) => a.at - b.at)).toEqual(...);
// ```
#[test]
fn error_lines_keep_template_order() {
    let found_later = TemplateError { subject: "mi".to_string(), message: "unknown field seek".to_string(), at: 12 };
    let found_first = TemplateError { subject: "mi".to_string(), message: "unknown field peek".to_string(), at: 1 };
    assert_eq!(
        error_lines(&[found_later, found_first]),
        vec!["mi: unknown field peek", "mi: unknown field seek"]
    );
    assert_eq!(error_lines(&[]), Vec::<String>::new());
}
