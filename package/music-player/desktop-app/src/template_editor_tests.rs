// Unit tests for `template_editor.rs`, pulled in by `#[cfg(test)] #[path = "template_editor_tests.rs"] mod tests;`.
//
// Covers the stand-in tracks, the caret clamp, the formula test, and the source each template reads.

use super::*;
use crate::template_model::TemplateResult;

// What: `fn sample_tracks_show_an_analysed_and_an_unanalysed_stand_in(...)` checks the two stand-in tracks.
// Why: The empty-library preview shows one analyzed and one not yet analyzed track, as the study does.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(TEMPLATE_SAMPLE_TRACKS.map(t => t.track.title)).toEqual(["Track title", "Track not analyzed yet"]);
// ```
#[test]
fn sample_tracks_show_an_analysed_and_an_unanalysed_stand_in() {
    let samples = template_sample_tracks();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].track.title, "Track title");
    assert_eq!(samples[0].track.peak.as_deref(), Some("\u{2212}1.0 dBTP"));
    assert_eq!(samples[1].track.title, "Track not analyzed yet");
    assert_eq!(samples[1].track.peak, None);
    assert_eq!(samples[1].place_in_folder, Some(2));
    assert_eq!(samples[0].tracks_in_folder, Some(16));
}

// What: `fn notes_and_row_count_keep_the_study_copy(...)` checks the constants the preview shows.
// Why: The notes and the row count are the study's copy, so their wording and value are part of the contract.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(TEMPLATE_LIBRARY_NOTE).toBe("From your library.");
// ```
#[test]
fn notes_and_row_count_keep_the_study_copy() {
    assert_eq!(TEMPLATE_PREVIEW_ROW_COUNT, 2);
    assert_eq!(TEMPLATE_LIBRARY_NOTE, "From your library.");
    assert_eq!(TEMPLATE_KEPT_NOTE, "Rows keep the last valid template.");
    assert_eq!(TEMPLATE_SAMPLE_NOTE, "Your library has no tracks yet. These are sample values.");
}

// What: `fn clamp_offset_keeps_the_caret_inside_the_text(...)` checks the clamp at both ends and across a surrogate pair.
// Why: The state never holds a caret outside its text, and a surrogate pair counts as two offsets.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(clampOffset(10, "abc")).toBe(3);
// ```
#[test]
fn clamp_offset_keeps_the_caret_inside_the_text() {
    assert_eq!(clamp_offset(10, "abc"), 3);
    assert_eq!(clamp_offset(2, "abc"), 2);
    assert_eq!(clamp_offset(0, ""), 0);
    assert_eq!(clamp_offset(9, "\u{10428}"), 2);
}

// What: `fn inside_formula_follows_the_formula_spans(...)` checks the caret in, at the edge of, and outside a formula.
// Why: A field call is written bare only when the caret is inside a formula's span.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(insideFormula("a$mi()$", 3)).toBe(true);
// ```
#[test]
fn inside_formula_follows_the_formula_spans() {
    assert!(inside_formula("a$mi()$", 3));
    assert!(!inside_formula("a$mi()$", 1));
    assert!(!inside_formula("a$mi()$", 0));
    assert!(!inside_formula("a$mi()$b", 8));
    assert!(inside_formula("$mi(", 3));
    assert!(!inside_formula("plain", 2));
}

// What: `fn track_source_picks_the_track_rows_or_the_playing_track(...)` checks which values each template reads.
// Why: The track rows read the track alone, and the playing track also reads its place in the folder.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(templateTrackFor("track", playing).fieldText("track")).toBeUndefined();
// ```
#[test]
fn track_source_picks_the_track_rows_or_the_playing_track() {
    let track = crate::template_fixtures::another_xronixle();
    assert_eq!(track_source(TemplateKind::TrackRows, &track).field_text("track"), None);
    assert_eq!(track_source(TemplateKind::PlayingTrack, &track).field_text("track"), Some(String::from("1")));
    assert_eq!(track_source(TemplateKind::TrackRows, &track).field_text("title"), Some(String::from("Another Xronixle")));
}

// What: `fn evaluate_for_applies_the_kind_fields(...)` checks that the playing track's field refuses on the track rows.
// Why: Each template evaluates with its own field list, so the same text can apply to one and refuse on the other.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(evaluateFor("track", "$mi(track)$", playing).valid).toBe(false);
// ```
#[test]
fn evaluate_for_applies_the_kind_fields() {
    let track = crate::template_fixtures::another_xronixle();
    assert_eq!(
        evaluate_for(TemplateKind::PlayingTrack, "$mi(track)$", &track),
        TemplateResult::Shown(String::from("1"))
    );
    assert!(matches!(
        evaluate_for(TemplateKind::TrackRows, "$mi(track)$", &track),
        TemplateResult::Refused(_)
    ));
}
