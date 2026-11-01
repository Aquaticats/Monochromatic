// Unit tests for the transitions of `template_editor_state.rs`, pulled in by
// `#[cfg(test)] #[path = "template_editor_state_tests.rs"] mod tests;`.
//
// Mirrors the Kotlin `TemplateEditorStateTest` cases about typing, opening, inserting, resetting,
// caret placement, focus, and help. Preview, notes, fields, and Settings cases are in `template_editor_preview_tests.rs`.

use super::*;
use crate::template_fixtures::{edit_at, library, typed};
use crate::template_model::DEFAULT_TRACK_TEMPLATE;

// What: `fn starts_on_track_rows_at_its_default(...)` checks the opening template and text.
// Why: The editor opens on the track rows template at its default, with nothing to reset.
//
// In TS you'd write (pseudocode):
// ```ts
// test("starts on track rows at its default", () => { ... });
// ```
#[test]
fn starts_on_track_rows_at_its_default() {
    let state = TemplateEditorState::initial(&library());
    assert_eq!(state.kind(), TemplateKind::TrackRows);
    assert_eq!(state.text(), DEFAULT_TRACK_TEMPLATE);
    assert!(!state.can_reset());
    assert!(state.is_valid());
}

// What: `fn track_only_field_is_refused_in_track_rows(...)` types the playing track's field into the track rows template.
// Why: Track rows never see the folder place, so the field is refused there.
//
// In TS you'd write (pseudocode):
// ```ts
// test("track field refused", () => expect(state.typed("$mi(track)$").isValid).toBe(false));
// ```
#[test]
fn track_only_field_is_refused_in_track_rows() {
    let state = typed(&TemplateEditorState::initial(&library()), "$mi(track)$");
    assert_eq!(state.mistake_lines(), vec!["mi has no field called track. Pick a name from the Fields list."]);
}

// What: `fn open_formula_is_reported(...)` types a formula with no closing dollar sign.
// Why: A formula left open is reported, never shown as literal text.
//
// In TS you'd write (pseudocode):
// ```ts
// test("open formula", () => expect(state.typed("$tf(mi(len), m:ss)").mistakeLines).toEqual([...]));
// ```
#[test]
fn open_formula_is_reported() {
    let state = typed(&TemplateEditorState::initial(&library()), "$tf(mi(len), m:ss)");
    assert_eq!(state.mistake_lines(), vec!["The $ at character 1 has no closing $. Add a $ where the formula ends."]);
}

// What: `fn valid_edit_becomes_applied_text(...)` types a valid template and reads the Settings line that follows it.
// Why: A valid text becomes the applied template, which the Settings line then shows.
//
// In TS you'd write (pseudocode):
// ```ts
// test("valid edit", () => expect(state.typed("$mi(len)$").settingsLine("track")).toBe("275"));
// ```
#[test]
fn valid_edit_becomes_applied_text() {
    let state = typed(&TemplateEditorState::initial(&library()), "$mi(len)$");
    assert_eq!(state.settings_line(TemplateKind::TrackRows), Ok(String::from("275")));
}

// What: `fn invalid_edit_keeps_last_valid_for_settings(...)` types a mistake after a valid custom template.
// Why: While the text is invalid, the Settings line keeps the last template that applied, and the draft keeps the typed text.
//
// In TS you'd write (pseudocode):
// ```ts
// test("kept", () => expect(state.typed(custom).typed(bad).settingsLine("track")).toBe(customLine));
// ```
#[test]
fn invalid_edit_keeps_last_valid_for_settings() {
    let custom = "$tc(up, mi(ext))$ \u{b7} $tf(mi(len), m:ss)$ \u{b7} $mi(peak)$";
    let state = typed(&typed(&TemplateEditorState::initial(&library()), custom), "$mi(peek)$");
    assert_eq!(
        state.settings_line(TemplateKind::TrackRows),
        Ok(String::from("FLAC \u{b7} 4:35 \u{b7} \u{2212}1.2 dBTP"))
    );
    assert_eq!(state.text(), "$mi(peek)$");
}

// What: `fn edits_stay_with_their_own_template(...)` edits the playing track and then opens the track rows.
// Why: Each template keeps its own text, so opening one must not change the other.
//
// In TS you'd write (pseudocode):
// ```ts
// test("separate", () => expect(state.open("track").text).toBe(DEFAULT_TRACK_TEMPLATE));
// ```
#[test]
fn edits_stay_with_their_own_template() {
    let playing_edit = typed(&TemplateEditorState::initial(&library()).open(TemplateKind::PlayingTrack), "$mi(total)$");
    let rows = playing_edit.open(TemplateKind::TrackRows);
    assert_eq!(rows.text(), DEFAULT_TRACK_TEMPLATE);
    assert_eq!(rows.open(TemplateKind::PlayingTrack).text(), "$mi(total)$");
}

// What: `fn open_puts_caret_at_end_without_focus(...)` checks what opening a template leaves focused.
// Why: A template opened from Settings starts unfocused, with the caret at its end.
//
// In TS you'd write (pseudocode):
// ```ts
// test("open", () => expect(state.open("playing")).toMatchObject({ fieldFocused: false }));
// ```
#[test]
fn open_puts_caret_at_end_without_focus() {
    let state = TemplateEditorState::initial(&library()).with_focus(true).open(TemplateKind::PlayingTrack);
    assert!(!state.field_focused());
    assert_eq!(state.selection_start(), TemplateKind::PlayingTrack.default_template().len());
}

// What: `fn insert_field_outside_formula_wraps_call(...)` inserts a field at a caret outside any formula.
// Why: A call outside a formula would show as literal text, so the editor adds the dollar signs.
//
// In TS you'd write (pseudocode):
// ```ts
// test("insert outside", () => expect(state.insertField("len").text).toBe("$mi(len)$"));
// ```
#[test]
fn insert_field_outside_formula_wraps_call() {
    let start = edit_at(&TemplateEditorState::initial(&library()), "", 0, 0);
    let state = start.insert_field("len");
    assert_eq!(state.text(), "$mi(len)$");
    assert_eq!(state.selection_start(), 8);
    assert_eq!(state.selection_end(), 8);
}

// What: `fn insert_field_inside_formula_writes_bare_call(...)` inserts a field at a caret inside a formula.
// Why: Inside a formula the call is written bare, so the formula does not gain nested delimiters.
//
// In TS you'd write (pseudocode):
// ```ts
// test("insert inside", () => expect(state.insertField("len").text).toBe("$tf(mi(len), m:ss)$"));
// ```
#[test]
fn insert_field_inside_formula_writes_bare_call() {
    let start = edit_at(&TemplateEditorState::initial(&library()), "$tf(, m:ss)$", 4, 4);
    let state = start.insert_field("len");
    assert_eq!(state.text(), "$tf(mi(len), m:ss)$");
    assert_eq!(state.selection_start(), 11);
}

// What: `fn insert_field_replaces_selection(...)` inserts a field over a selected word.
// Why: A selection is replaced by the inserted call, as a text editor does.
//
// In TS you'd write (pseudocode):
// ```ts
// test("replace selection", () => expect(state.insertField("peak").text).toBe("$tf(mi(len), mi(peak))$"));
// ```
#[test]
fn insert_field_replaces_selection() {
    let start = edit_at(&TemplateEditorState::initial(&library()), "$tf(mi(len), m:ss)$", 13, 17);
    let state = start.insert_field("peak");
    assert_eq!(state.text(), "$tf(mi(len), mi(peak))$");
    assert_eq!(state.selection_start(), 21);
}

// What: `fn insert_function_places_caret_inside_brackets(...)` inserts an empty call of a function.
// Why: The caret lands inside the brackets so the user types the arguments next.
//
// In TS you'd write (pseudocode):
// ```ts
// test("insert function", () => expect(state.insertFunction("tf").text).toBe("$tf()$"));
// ```
#[test]
fn insert_function_places_caret_inside_brackets() {
    let start = edit_at(&TemplateEditorState::initial(&library()), "", 0, 0);
    let state = start.insert_function("tf");
    assert_eq!(state.text(), "$tf()$");
    assert_eq!(state.selection_start(), 4);
}

// What: `fn reset_to_default_restores_the_template(...)` returns an edited template to its default.
// Why: The way back to the default must restore the text, the Settings line, and the caret position.
//
// In TS you'd write (pseudocode):
// ```ts
// test("reset", () => expect(state.typed(custom).resetToDefault().text).toBe(DEFAULT));
// ```
#[test]
fn reset_to_default_restores_the_template() {
    let state = typed(&TemplateEditorState::initial(&library()), "$mi(len)$").reset_to_default();
    assert_eq!(state.text(), DEFAULT_TRACK_TEMPLATE);
    assert!(!state.can_reset());
    assert_eq!(state.settings_line(TemplateKind::TrackRows), Ok(String::from("4:35 \u{2212}1.2 dBTP")));
    assert_eq!(state.selection_start(), DEFAULT_TRACK_TEMPLATE.len());
}

// What: `fn can_reset_only_when_text_differs(...)` checks when the way back is offered.
// Why: The way back is shown only when the template differs from its default.
//
// In TS you'd write (pseudocode):
// ```ts
// test("canReset", () => expect(initial.canReset).toBe(false));
// ```
#[test]
fn can_reset_only_when_text_differs() {
    let at_default = TemplateEditorState::initial(&library());
    assert!(!at_default.can_reset());
    assert!(typed(&at_default, "$mi(len)$").can_reset());
}

// What: `fn help_shows_only_while_field_is_focused(...)` checks the typing help against focus.
// Why: Help appears only while the field has focus, because only then is the caret typing.
//
// In TS you'd write (pseudocode):
// ```ts
// test("help focus", () => expect(state.help).toBeUndefined());
// ```
#[test]
fn help_shows_only_while_field_is_focused() {
    let unfocused = edit_at(&TemplateEditorState::initial(&library()), DEFAULT_TRACK_TEMPLATE, 15, 15);
    assert_eq!(unfocused.help(), None);
    let focused = unfocused.with_focus(true);
    let help = focused.help();
    assert_eq!(help.as_ref().map(|found| return found.name.as_str()), Some("tf"));
    assert_eq!(help.as_ref().map(|found| return found.signature), Some("tf(seconds, [format])"));
    assert_eq!(help.as_ref().map(|found| return found.parameter), Some("format"));
}

// What: `fn help_is_none_outside_calls(...)` checks the help with the caret outside every call.
// Why: With the caret outside a call there is no signature to show.
//
// In TS you'd write (pseudocode):
// ```ts
// test("outside", () => expect(helpAt(text, 0)).toBeUndefined());
// ```
#[test]
fn help_is_none_outside_calls() {
    let state = edit_at(&TemplateEditorState::initial(&library()), DEFAULT_TRACK_TEMPLATE, 0, 0).with_focus(true);
    assert_eq!(state.help(), None);
}

// What: `fn selection_is_clamped_to_the_text(...)` checks an edit whose caret runs past the end.
// Why: The state must never hold a caret position outside its text.
//
// In TS you'd write (pseudocode):
// ```ts
// test("clamp", () => expect(withEdit({ text: "abc", selectionStart: 10 }).selectionStart).toBe(3));
// ```
#[test]
fn selection_is_clamped_to_the_text() {
    let state = edit_at(&TemplateEditorState::initial(&library()), "abc", 10, 12);
    assert_eq!(state.selection_start(), 3);
    assert_eq!(state.selection_end(), 3);
}

// What: `fn error_lines_keep_template_order(...)` checks that two mistakes stay in template order.
// Why: The editor lists mistakes where they sit in the template, so the user reads them top down.
//
// In TS you'd write (pseudocode):
// ```ts
// test("order", () => expect(state.mistakeLines[0]).toContain("peek"));
// ```
#[test]
fn error_lines_keep_template_order() {
    let state = typed(&TemplateEditorState::initial(&library()), "$mi(peek)$ $mi(seek)$");
    assert_eq!(
        state.mistake_lines(),
        vec![
            "mi has no field called peek. Pick a name from the Fields list.",
            "mi has no field called seek. Pick a name from the Fields list.",
        ]
    );
}

// What: `fn valid_template_removes_the_kept_note(...)` checks the note once the template applies again.
// Why: The kept-template note disappears as soon as the text is valid.
//
// In TS you'd write (pseudocode):
// ```ts
// test("note", () => expect(state.previewNotes).not.toContain(KEPT_NOTE));
// ```
#[test]
fn valid_template_removes_the_kept_note() {
    let state = typed(&typed(&TemplateEditorState::initial(&library()), "$mi(peek)$"), "$mi(len)$");
    assert!(!state.preview_notes().contains(&TEMPLATE_KEPT_NOTE));
}

// What: `fn kept_note_says_what_the_rows_keep(...)` types a template that does not apply, after one that did and after a cleared one.
// Why: TE4: the rows keep the last valid template; when that template shows nothing, a bare preview looks broken, so the
// note says so.
#[test]
fn kept_note_says_what_the_rows_keep() {
    let initial = TemplateEditorState::initial(&library());
    assert_eq!(initial.kept_note(), None, "a template that applies has no kept note");
    let broken = typed(&initial, "$mi(peek)$");
    assert_eq!(broken.kept_note(), Some(TEMPLATE_KEPT_NOTE));
    let cleared_then_broken = typed(&typed(&initial, ""), "$mi(nope)$");
    assert_eq!(cleared_then_broken.kept_note(), Some(TEMPLATE_KEPT_EMPTY_NOTE), "the kept template is the empty one");
    assert_eq!(typed(&broken, "$mi(len)$").kept_note(), None, "the note goes when the text applies again");
}

// What: `fn source_note_names_the_library_or_the_stand_ins(...)` reads the note about where the preview rows come from.
// Why: the page draws it apart from the kept note, which has a reserved line of its own.
#[test]
fn source_note_names_the_library_or_the_stand_ins() {
    assert_eq!(TemplateEditorState::initial(&library()).source_note(), TEMPLATE_LIBRARY_NOTE);
    assert_eq!(TemplateEditorState::initial(&[]).source_note(), TEMPLATE_SAMPLE_NOTE);
}

// What: `fn focus_change_keeps_text(...)` checks that focus changes do not touch the text.
// Why: The field reports focus separately from edits, so focus must leave the template alone.
//
// In TS you'd write (pseudocode):
// ```ts
// test("focus", () => expect(state.withFocus(true).text).toBe(state.text));
// ```
#[test]
fn focus_change_keeps_text() {
    let state = TemplateEditorState::initial(&library());
    assert_eq!(state.with_focus(true).with_focus(false).text(), state.text());
}
