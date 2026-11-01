// Unit tests for `template_eval.rs`, pulled in by `#[cfg(test)] #[path = "template_eval_tests.rs"] mod tests;`.
//
// Each test mirrors one group of the Kotlin `TrackTemplateTest`: text and fields, durations,
// conversion, joins, and the template kinds. The mistake group lives in `template_eval_mistake_tests.rs`.

use super::evaluate_template;
use crate::template_fixtures::{analysed, bare, playing, waiting};
use crate::template_model::{
    DEFAULT_TRACK_TEMPLATE, TemplateField, TemplateKind, TemplateResult, TemplateTrack, error_lines,
    playing_template_fields, track_template_fields,
};

// What: `fn assert_shows(text: &str, track: &dyn TemplateTrack, expected: &str)` asserts that a template shows the expected text with the track rows fields.
// Why: One helper keeps each shown case a single readable line.
//
// In TS you'd write (pseudocode):
// ```ts
// function shows({ text, track, expected }: ShowCase): void { ... }
// ```
fn assert_shows(text: &str, track: &dyn TemplateTrack, expected: &str) {
    assert_eq!(
        evaluate_template(text, track, &track_template_fields()),
        TemplateResult::Shown(expected.to_string()),
        "{text}"
    );
}

// What: `fn assert_shows_with(...)` asserts that a template shows the expected text with the given field list.
// Why: The playing track's template reads its own field list, so the field list is passed explicitly.
//
// In TS you'd write (pseudocode):
// ```ts
// function showsWith({ text, track, fields, expected }: ShowCase): void { ... }
// ```
fn assert_shows_with(text: &str, track: &dyn TemplateTrack, fields: &[TemplateField], expected: &str) {
    assert_eq!(evaluate_template(text, track, fields), TemplateResult::Shown(expected.to_string()), "{text}");
}

// What: `fn assert_refuses(text: &str, track: &dyn TemplateTrack, lines: &[&str])` asserts that a template is refused with exactly these mistake lines.
// Why: The lines name each mistake and its order, so both the wording and the position are checked.
//
// In TS you'd write (pseudocode):
// ```ts
// function refuses({ text, track, lines }: RefuseCase): void { ... }
// ```
fn assert_refuses(text: &str, track: &dyn TemplateTrack, lines: &[&str]) {
    match evaluate_template(text, track, &track_template_fields()) {
        TemplateResult::Refused(errors) => assert_eq!(error_lines(&errors), lines, "{text}"),
        TemplateResult::Shown(shown) => panic!("{text} should be refused but shows {shown:?}"),
    }
}

// What: `fn text_fields_and_default_line_show_substitutions(...)` checks plain text, the default line, and each field.
// Why: Text outside formulas is always shown, and a missing value leaves its surrounding text in place.
//
// In TS you'd write (pseudocode):
// ```ts
// test("text, fields and the default line", () => { ... });
// ```
#[test]
fn text_fields_and_default_line_show_substitutions() {
    let analysed_track = analysed();
    assert_shows("", &analysed_track, "");
    assert_shows("plain text, no formula", &analysed_track, "plain text, no formula");
    assert_shows(DEFAULT_TRACK_TEMPLATE, &analysed_track, "4:35 \u{2212}0.3 dBTP");
    assert_shows(DEFAULT_TRACK_TEMPLATE, &waiting(), "4:35 ");
    assert_shows(DEFAULT_TRACK_TEMPLATE, &bare(), " ");
    let field_values = [
        ("title", "Another Xronixle"),
        ("file", "かめりあ(Camellia) - Another Xronixle"),
        ("ext", "flac"),
        ("folder", "Camellia"),
        ("path", "Camellia/かめりあ(Camellia) - Another Xronixle.flac"),
        ("len", "275"),
        ("peak", "\u{2212}0.3 dBTP"),
    ];
    for (mode, value) in field_values {
        assert_shows(&format!("[$mi({mode})$]"), &analysed_track, &format!("[{value}]"));
    }
    assert_shows(
        "$mi(folder)$/$mi(file)$.$mi(ext)$",
        &analysed_track,
        "Camellia/かめりあ(Camellia) - Another Xronixle.flac",
    );
    assert_shows("$mi(peak)$", &waiting(), "");
    assert_shows("$mi(title)$$mi(ext)$", &analysed_track, "Another Xronixleflac");
}

// What: `fn duration_formats_follow_the_format_letters(...)` checks each duration format and its refusals.
// Why: Letters, doubling, and apostrophes change the output, so each rule has its own case.
//
// In TS you'd write (pseudocode):
// ```ts
// test("duration formats", () => { ... });
// ```
#[test]
fn duration_formats_follow_the_format_letters() {
    let analysed_track = analysed();
    assert_shows("$tf(mi(len))$", &analysed_track, "4:35");
    assert_shows("$tf(mi(len), mm:ss)$", &analysed_track, "04:35");
    assert_shows("$tf(mi(len), h:mm:ss)$", &analysed_track, "0:04:35");
    assert_shows("$tf(4000, m:ss)$", &analysed_track, "66:40");
    assert_shows("$tf(4000, h:mm:ss)$", &analysed_track, "1:06:40");
    assert_shows("$tf(4000, hh:mm:ss)$", &analysed_track, "01:06:40");
    assert_shows("$tf(59.9, m:ss)$", &analysed_track, "0:59");
    // TE5: a letter that is not h, m or s outside apostrophes is a mistake, not text; it used to print as itself.
    assert_refuses("$tf(mi(len), \"m min\")$", &analysed_track, &["tf: unknown format letter i"]);
    assert_refuses("$tf(mi(len), \"MM:SS\")$", &analysed_track, &["tf: unknown format letter M"]);
    assert_refuses("$tf(mi(len), q:q)$", &analysed_track, &["tf: unknown format letter q"]);
    assert_refuses("$tf(mi(len), q:q)$", &bare(), &["tf: unknown format letter q"]);
    assert_shows("$tf(mi(len), \"m'.'ss\")$", &analysed_track, "4.35");
    assert_shows("$tf(mi(len), \"m / ss\")$", &analysed_track, "4 / 35");
    assert_shows("$tf(mi(len), \"m' min'\")$", &analysed_track, "4 min");
    assert_shows(
        "$tf(4000, \"h' hours and 'm' minutes'\")$",
        &analysed_track,
        "1 hours and 6 minutes",
    );
    assert_shows("$tf(mi(len), m:ss)$", &bare(), "");
    assert_refuses("$tf(mi(title))$", &analysed_track, &["tf: needs a number of seconds, not Another Xronixle"]);
    assert_refuses("$tf()$", &analysed_track, &["tf: takes 1 or 2 values, as in tf(seconds, [format])"]);
    assert_refuses("$tf(1, m, s)$", &analysed_track, &["tf: takes 1 or 2 values, as in tf(seconds, [format])"]);
}

// What: `fn text_conversion_covers_each_mode(...)` checks low, up, cap, and cut, and their refusals.
// Why: Each conversion mode has its own rule, and the cut length has its own refusal.
//
// In TS you'd write (pseudocode):
// ```ts
// test("text conversion", () => { ... });
// ```
#[test]
fn text_conversion_covers_each_mode() {
    let analysed_track = analysed();
    assert_shows("$tc(low, mi(title))$", &analysed_track, "another xronixle");
    assert_shows("$tc(up, mi(ext))$", &analysed_track, "FLAC");
    assert_shows("$tc(cap, \"hello there\")$", &analysed_track, "Hello there");
    assert_shows("$tc(cap, \"\")$", &analysed_track, "");
    assert_shows("$tc(cut, mi(title), 7)$", &analysed_track, "Another");
    assert_shows("$tc(cut, mi(file), 4)$", &analysed_track, "かめりあ");
    assert_shows("$tc(cut, mi(title), 0)$", &analysed_track, "");
    assert_shows("$tc(cut, mi(title), 99)$", &analysed_track, "Another Xronixle");
    assert_refuses(
        "$tc(cut, mi(title))$",
        &analysed_track,
        &["tc: cut needs a length, as in tc(cut, text, 10)"],
    );
    assert_refuses(
        "$tc(cut, mi(title), -1)$",
        &analysed_track,
        &["tc: cut needs a length, as in tc(cut, text, 10)"],
    );
    assert_refuses("$tc(ell, mi(title), 5)$", &analysed_track, &["tc: unknown mode ell"]);
    assert_refuses("$tc(up)$", &analysed_track, &["tc: takes 2 or 3 values, as in tc(mode, text, [length])"]);
}

// What: `fn joining_and_no_conditional_follow_the_reference(...)` checks `+`, separators next to empty fields, comparison signs as text, and the refused conditional.
// Why: The template has no conditional, so a separator never drops out with an empty field.
//
// In TS you'd write (pseudocode):
// ```ts
// test("joining, and no conditional", () => { ... });
// ```
#[test]
fn joining_and_no_conditional_follow_the_reference() {
    let analysed_track = analysed();
    assert_shows("$\"a\" + \"b\" + mi(ext)$", &analysed_track, "abflac");
    assert_shows("$1 + 2$", &analysed_track, "12");
    assert_shows("$mi(folder)$ \u{00B7} $mi(title)$", &bare(), " \u{00B7} x");
    assert_shows("$mi(peak)$ dBTP", &waiting(), " dBTP");
    assert_shows("$tc(up, a<b>=c!=d)$", &analysed_track, "A<B>=C!=D");
    assert_refuses("$if(mi(peak), yes, no)$", &analysed_track, &["if: unknown function"]);
    assert_refuses("$if(1, fine, mi(nope))$", &analysed_track, &["if: unknown function"]);
}

// What: `fn long_join_evaluates_without_recursing_per_plus(...)` checks a join of twenty thousand operands.
// Why: The join is a flat list, so a long template cannot exhaust the stack by nesting once per `+`.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(evaluateTemplate('$"a" + '.repeat(20000) + '"b"$', track).text).toBe("a".repeat(20000) + "b");
// ```
#[test]
fn long_join_evaluates_without_recursing_per_plus() {
    let text = format!("${}\"b\"$", "\"a\" + ".repeat(20_000));
    let expected = format!("{}b", "a".repeat(20_000));
    assert_eq!(evaluate_template(&text, &analysed(), &track_template_fields()), TemplateResult::Shown(expected));
}

// What: `fn each_template_has_its_own_defaults_and_fields(...)` checks the playing track's default, fields, and refusals.
// Why: The track rows and the playing track must not share field lists or default text.
//
// In TS you'd write (pseudocode):
// ```ts
// test("each template has its own default and fields", () => { ... });
// ```
#[test]
fn each_template_has_its_own_defaults_and_fields() {
    assert_eq!(TemplateKind::TrackRows.default_template(), DEFAULT_TRACK_TEMPLATE);
    let playing_track = playing();
    assert_shows_with(
        TemplateKind::PlayingTrack.default_template(),
        &playing_track,
        &playing_template_fields(),
        "1 of 16 \u{2212}0.3 dBTP",
    );
    let mut without_peak = playing();
    without_peak.track.peak = None;
    assert_shows_with(
        TemplateKind::PlayingTrack.default_template(),
        &without_peak,
        &playing_template_fields(),
        "1 of 16 ",
    );
    assert_shows_with(
        "$mi(title)$ ($mi(track)$/$mi(total)$)",
        &playing_track,
        &playing_template_fields(),
        "Another Xronixle (1/16)",
    );
    assert_refuses(
        "$mi(track)$ of $mi(total)$",
        &analysed(),
        &["mi: unknown field track", "mi: unknown field total"],
    );
}
