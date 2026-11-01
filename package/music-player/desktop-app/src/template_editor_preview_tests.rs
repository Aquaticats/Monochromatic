// Unit tests for the preview, notes, fields, Settings, and focus-help views of `template_editor_state.rs`,
// pulled in by `#[cfg(test)] #[path = "template_editor_preview_tests.rs"] mod preview_tests;`.
//
// Mirrors the Kotlin `TemplateEditorStateTest` cases about the rows, notes, field list, and Settings lines,
// plus one case where the Kotlin `appliedLine` throws `TemplateEditorStateError` and this port returns `Err`:
// a track whose length is NaN makes the stored default template refuse.

use super::*;
use crate::template_fixtures::{another_xronixle, burning_aquamarine, bare_playing, edit_at, library, typed};
use crate::template_model::{DEFAULT_TRACK_TEMPLATE, PlayingTrackFields};

// What: `fn row(title: &str, supporting: &str) -> TemplatePreviewRow` builds one expected preview row.
// Why: Each expected row reads like the study's copy, so each case compares whole rows.
//
// In TS you'd write (pseudocode):
// ```ts
// const row = (title, supporting) => ({ title, supporting });
// ```
fn row(title: &str, supporting: &str) -> TemplatePreviewRow {
    return TemplatePreviewRow {
        title: title.to_string(),
        supporting: supporting.to_string(),
    };
}

// What: `fn entry(label: &'static str, mode: &'static str, value: &str) -> TemplateEditorFieldEntry` builds one expected field entry.
// Why: The field-list cases compare whole entries, including the call a tap inserts.
//
// In TS you'd write (pseudocode):
// ```ts
// const entry = (label, mode, value) => ({ label, mode, insert: `mi(${mode})`, value });
// ```
fn entry(label: &'static str, mode: &'static str, value: &str) -> TemplateEditorFieldEntry {
    return TemplateEditorFieldEntry {
        label,
        mode,
        insert: format!("mi({mode})"),
        value: value.to_string(),
    };
}

// What: `fn settings_lines_show_each_template_line(...)` checks what Settings lists at opening.
// Why: Settings lists both templates with the line each yields for the first library track.
//
// In TS you'd write (pseudocode):
// ```ts
// test("settings lines", () => expect(state.settingsLines).toEqual({ ... }));
// ```
#[test]
fn settings_lines_show_each_template_line() {
    let state = TemplateEditorState::initial(&library());
    assert_eq!(
        state.settings_lines(),
        Ok(vec![
            (TemplateKind::TrackRows, String::from("4:35 \u{2212}1.2 dBTP")),
            (TemplateKind::PlayingTrack, String::from("1 of 16 \u{2212}1.2 dBTP")),
        ])
    );
}

// What: `fn track_row_preview_matches_study_copy(...)` checks the track rows preview against the study.
// Why: The default line of the analysed track has no trailing space, and the unanalysed one keeps its space.
//
// In TS you'd write (pseudocode):
// ```ts
// test("track row preview", () => expect(state.previewRows).toEqual([...]));
// ```
#[test]
fn track_row_preview_matches_study_copy() {
    let state = TemplateEditorState::initial(&library());
    assert_eq!(
        state.preview_rows(),
        Ok(vec![row("Another Xronixle", "4:35 \u{2212}1.2 dBTP"), row("Burning Aquamarine", "5:12 ")])
    );
}

// What: `fn playing_track_preview_matches_study_copy(...)` checks the playing track's two preview lines.
// Why: The playing track reads its place and folder total, so its lines differ from the track rows.
//
// In TS you'd write (pseudocode):
// ```ts
// test("playing preview", () => expect(...).toEqual([...]));
// ```
#[test]
fn playing_track_preview_matches_study_copy() {
    let state = TemplateEditorState::initial(&library()).open(TemplateKind::PlayingTrack);
    assert_eq!(
        state.preview_rows(),
        Ok(vec![row("Another Xronixle", "1 of 16 \u{2212}1.2 dBTP"), row("Burning Aquamarine", "2 of 16 ")])
    );
}

// What: `fn library_preview_note_names_the_library(...)` checks the note while the library has tracks.
// Why: The user should see that the rows come from the library, not from sample values.
//
// In TS you'd write (pseudocode):
// ```ts
// test("library note", () => expect(state.previewNotes).toEqual(["From your library."]));
// ```
#[test]
fn library_preview_note_names_the_library() {
    let state = TemplateEditorState::initial(&library());
    assert_eq!(state.preview_notes(), vec![TEMPLATE_LIBRARY_NOTE]);
    assert!(!state.uses_sample_values());
}

// What: `fn empty_library_uses_sample_values(...)` checks the stand-in preview while the library holds no track.
// Why: The sample values and their note show while the open library is empty.
//
// In TS you'd write (pseudocode):
// ```ts
// test("empty library", () => expect(initial([]).previewNotes).toEqual([SAMPLE_NOTE]));
// ```
#[test]
fn empty_library_uses_sample_values() {
    let state = TemplateEditorState::initial(&[]);
    assert!(state.uses_sample_values());
    assert_eq!(state.preview_notes(), vec![TEMPLATE_SAMPLE_NOTE]);
    assert_eq!(
        state.preview_rows(),
        Ok(vec![row("Track title", "3:20 \u{2212}1.0 dBTP"), row("Track not analyzed yet", "3:20 ")])
    );
}

// What: `fn empty_library_fields_show_stand_ins(...)` checks the field list while the library is empty.
// Why: The field values come from the stand-in first track.
//
// In TS you'd write (pseudocode):
// ```ts
// test("stand-in fields", () => expect(initial([]).fields[0].value).toBe("Track title"));
// ```
#[test]
fn empty_library_fields_show_stand_ins() {
    let fields = TemplateEditorState::initial(&[]).fields();
    assert_eq!(fields[0].value, "Track title");
    assert_eq!(fields[5].value, "200");
}

// What: `fn library_preview_takes_first_two_tracks(...)` checks that the preview draws two rows from a longer library.
// Why: The preview shows two files, so a longer library must not add rows.
//
// In TS you'd write (pseudocode):
// ```ts
// test("two rows", () => expect(initial([a, b, c]).previewTracks.length).toBe(2));
// ```
#[test]
fn library_preview_takes_first_two_tracks() {
    let mut three = library();
    three.push(PlayingTrackFields {
        place_in_folder: Some(3),
        ..another_xronixle()
    });
    let state = TemplateEditorState::initial(&three);
    assert_eq!(state.preview_rows().map(|rows| return rows.len()), Ok(2));
}

// What: `fn track_row_fields_list_seven_with_first_track_values(...)` checks the track rows field list.
// Why: The list shows the seven track fields, each with the value of the first preview track.
//
// In TS you'd write (pseudocode):
// ```ts
// test("track row fields", () => expect(state.fields.length).toBe(7));
// ```
#[test]
fn track_row_fields_list_seven_with_first_track_values() {
    let fields = TemplateEditorState::initial(&library()).fields();
    assert_eq!(fields.len(), 7);
    assert_eq!(
        fields[0],
        TemplateEditorFieldEntry {
            label: "Title",
            mode: "title",
            insert: String::from("mi(title)"),
            value: String::from("Another Xronixle"),
        }
    );
    assert_eq!(fields[5].value, "275");
    assert_eq!(fields[6].value, "\u{2212}1.2 dBTP");
}

// What: `fn playing_fields_add_place_and_total(...)` checks the playing track's two extra fields.
// Why: The playing track reads its place in the folder and the folder's track count.
//
// In TS you'd write (pseudocode):
// ```ts
// test("playing fields", () => expect(fields.slice(-2)).toEqual([...]));
// ```
#[test]
fn playing_fields_add_place_and_total() {
    let fields = TemplateEditorState::initial(&library()).open(TemplateKind::PlayingTrack).fields();
    assert_eq!(fields.len(), 9);
    assert_eq!(fields[7], entry("Place in folder", "track", "1"));
    assert_eq!(fields[8], entry("Tracks in folder", "total", "16"));
}

// What: `fn missing_peak_field_is_empty(...)` checks a field whose track has no value.
// Why: A field with no value must show as empty so the list can say so.
//
// In TS you'd write (pseudocode):
// ```ts
// test("missing peak", () => expect(fields.at(6).value).toBe(""));
// ```
#[test]
fn missing_peak_field_is_empty() {
    let fields = TemplateEditorState::initial(&[burning_aquamarine(), another_xronixle()]).fields();
    assert_eq!(fields[6].value, "");
}

// What: `fn custom_template_changes_both_rows(...)` types a valid custom template and reads both rows.
// Why: A template that applies changes every row at once, and the offer of the way back appears.
//
// In TS you'd write (pseudocode):
// ```ts
// test("custom", () => expect(state.typed(custom).previewRows).toEqual([...]));
// ```
#[test]
fn custom_template_changes_both_rows() {
    let state = typed(
        &TemplateEditorState::initial(&library()),
        "$tc(up, mi(ext))$ \u{b7} $tf(mi(len), m:ss)$ \u{b7} $mi(peak)$",
    );
    assert!(state.is_valid());
    assert!(state.can_reset());
    assert_eq!(
        state.preview_rows(),
        Ok(vec![
            row("Another Xronixle", "FLAC \u{b7} 4:35 \u{b7} \u{2212}1.2 dBTP"),
            row("Burning Aquamarine", "FLAC \u{b7} 5:12 \u{b7} "),
        ])
    );
}

// What: `fn unknown_field_is_refused_and_rows_keep_default(...)` types a misspelt field.
// Why: A mistake is reported on its own line while the rows keep the last template that applied.
//
// In TS you'd write (pseudocode):
// ```ts
// test("unknown field", () => expect(state.typed(bad).mistakeLines).toEqual(["mi: unknown field peek"]));
// ```
#[test]
fn unknown_field_is_refused_and_rows_keep_default() {
    let initial = TemplateEditorState::initial(&library());
    let state = typed(&initial, "$tf(mi(len), m:ss)$ $mi(peek)$");
    assert!(!state.is_valid());
    assert_eq!(state.mistake_lines(), vec!["mi has no field called peek. Pick a name from the Fields list."]);
    assert_eq!(state.preview_rows(), initial.preview_rows());
    assert_eq!(state.preview_notes(), vec![TEMPLATE_LIBRARY_NOTE, TEMPLATE_KEPT_NOTE]);
}

// What: `fn settings_line_for_empty_library_uses_stand_ins(...)` checks the Settings line with no library tracks.
// Why: The Settings page shows the same stand-in line the editor preview shows.
//
// In TS you'd write (pseudocode):
// ```ts
// test("empty settings", () => expect(initial([]).settingsLine("track")).toBe("3:20 −1.0 dBTP"));
// ```
#[test]
fn settings_line_for_empty_library_uses_stand_ins() {
    assert_eq!(
        TemplateEditorState::initial(&[]).settings_line(TemplateKind::TrackRows),
        Ok(String::from("3:20 \u{2212}1.0 dBTP"))
    );
}

// What: `fn track_without_duration_or_peak_keeps_default_rows(...)` opens the editor on a track with no metadata.
// Why: Production tracks often carry no duration, so the defaults must still apply without an error.
//
// In TS you'd write (pseudocode):
// ```ts
// test("bare track", () => expect(initial([bare]).settingsLines).toBeDefined());
// ```
#[test]
fn track_without_duration_or_peak_keeps_default_rows() {
    let state = TemplateEditorState::initial(&[bare_playing()]);
    assert!(state.is_valid());
    assert_eq!(state.preview_rows(), Ok(vec![row("Bare", " ")]));
    assert_eq!(
        state.open(TemplateKind::PlayingTrack).settings_line(TemplateKind::PlayingTrack),
        Ok(String::from(" of  "))
    );
}

// What: `fn track_whose_length_is_not_a_number_reports_the_applied_template_error(...)` opens the editor on a track with a NaN duration.
// Why: The default line refuses that track, so the stored applied template no longer applies, and the rows must report it as an error instead of showing a wrong line.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(evaluateTemplate(defaultTemplate, { ...analysed, len: NaN }).valid).toBe(false);
// ```
#[test]
fn track_whose_length_is_not_a_number_reports_the_applied_template_error() {
    let mut nan_track = another_xronixle();
    nan_track.track.len = Some(f64::NAN);
    let state = TemplateEditorState::initial(&[nan_track]);
    let error = match state.preview_rows() {
        Err(found) => found,
        Ok(rows) => panic!("expected the stored template to refuse, got {rows:?}"),
    };
    assert_eq!(
        error.message,
        "The applied Track rows template no longer applies: tf: needs a number of seconds, not NaN"
    );
    assert!(state.settings_lines().is_err());
    assert_eq!(state.settings_line(TemplateKind::PlayingTrack), Ok(String::from("1 of 16 \u{2212}1.2 dBTP")));
    assert_eq!(state.fields()[5].value, "NaN");
    assert!(!state.is_valid());
}

// What: `fn help_follows_focus_and_the_caret(...)` checks that help is hidden without focus and shown with it.
// Why: The help follows the selection end while the field has focus.
//
// In TS you'd write (pseudocode):
// ```ts
// test("help focus", () => expect(state.help?.parameter).toBe("format"));
// ```
#[test]
fn help_follows_focus_and_the_caret() {
    let state = edit_at(&TemplateEditorState::initial(&library()), DEFAULT_TRACK_TEMPLATE, 15, 15);
    assert!(state.help().is_none());
    assert_eq!(state.with_focus(true).help().map(|found| return found.parameter), Some("format"));
    assert_eq!(state.with_focus(true).text(), DEFAULT_TRACK_TEMPLATE);
}
