// Unit tests for the mistake group of `template_eval.rs`, pulled in by
// `#[cfg(test)] #[path = "template_eval_mistake_tests.rs"] mod mistake_tests;`.
//
// Mirrors the Kotlin `mistakesAreNamedOrderedAndNeverShownAsText` case: each mistake names its
// function, sits in template order, and never leaks into the shown line.

use super::evaluate_template;
use crate::template_fixtures::analysed;
use crate::template_model::{TemplateResult, TemplateTrack, error_lines, track_template_fields};

// What: `fn assert_refuses(text: &str, lines: &[&str])` asserts that the template on the analysed track is refused with exactly these lines.
// Why: Every mistake case compares its wording, order, and count in one call.
//
// In TS you'd write (pseudocode):
// ```ts
// function refuses(text: string, lines: string[]): void { ... }
// ```
fn assert_refuses(text: &str, lines: &[&str]) {
    let track = analysed();
    match evaluate_template(text, &track as &dyn TemplateTrack, &track_template_fields()) {
        TemplateResult::Refused(errors) => assert_eq!(error_lines(&errors), lines, "{text}"),
        TemplateResult::Shown(shown) => panic!("{text} should be refused but shows {shown:?}"),
    }
}

// What: `fn assert_shows_analysed(text: &str, expected: &str)` asserts that a template shows the expected text on the analysed track.
// Why: The text that looks like other syntax must come out unchanged, so each case is shown rather than refused.
//
// In TS you'd write (pseudocode):
// ```ts
// function shows(text: string, expected: string): void { ... }
// ```
fn assert_shows_analysed(text: &str, expected: &str) {
    let track = analysed();
    assert_eq!(
        evaluate_template(text, &track as &dyn TemplateTrack, &track_template_fields()),
        TemplateResult::Shown(expected.to_string()),
        "{text}"
    );
}

// What: `fn unknown_functions_and_fields_are_named_in_order(...)` checks the two unknown names in one template.
// Why: Each mistake names its own function and they appear in the order they sit in the template.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$xx(1)$ and $mi(nope)$")).toEqual(["xx: unknown function", "mi: unknown field nope"]);
// ```
#[test]
fn unknown_functions_and_fields_are_named_in_order() {
    assert_refuses("$xx(1)$ and $mi(nope)$", &["xx: unknown function", "mi: unknown field nope"]);
    assert_refuses("$constructor(1)$", &["constructor: unknown function"]);
    assert_refuses("$mi(__proto__)$", &["mi: unknown field __proto__"]);
}

// What: `fn unknown_function_stops_before_its_arguments(...)` checks that an unknown function's arguments are not read.
// Why: The name is refused first, so a mistake inside its arguments is not reported.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$xx(mi(nope))$")).toEqual(["xx: unknown function"]);
// ```
#[test]
fn unknown_function_stops_before_its_arguments() {
    assert_refuses("$xx(mi(nope))$", &["xx: unknown function"]);
}

// What: `fn every_argument_is_read_before_the_call(...)` checks that a call with an empty first argument still reads the others.
// Why: All arguments are evaluated before the call's own checks, so a mistake in any argument is reported.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$tf(mi(peak), mi(nope))$")).toEqual(["mi: unknown field nope"]);
// ```
#[test]
fn every_argument_is_read_before_the_call() {
    let track = crate::template_fixtures::waiting();
    match evaluate_template("$tf(mi(peak), mi(nope))$", &track as &dyn TemplateTrack, &track_template_fields()) {
        TemplateResult::Refused(errors) => assert_eq!(error_lines(&errors), vec!["mi: unknown field nope"]),
        TemplateResult::Shown(shown) => panic!("expected a refusal, got {shown:?}"),
    }
}

// What: `fn unclosed_formulas_and_quotes_are_reported(...)` checks the missing dollar sign, bracket, and quotation mark.
// Why: A half-written formula is reported at its start and is never shown as literal text.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$mi(title)")).toEqual(["formula: the $ at character 1 has no closing $"]);
// ```
#[test]
fn unclosed_formulas_and_quotes_are_reported() {
    assert_refuses("$mi(title)", &["formula: the $ at character 1 has no closing $"]);
    assert_refuses(
        "a $mi(title) - text",
        &["formula: the $ at character 3 has no closing $", "formula: unexpected -"],
    );
    assert_refuses("$mi(title$", &["mi: a closing bracket is missing"]);
    assert_refuses(
        "$mi(\"title)$",
        &["mi: a closing bracket is missing", "text: a quotation mark is not closed"],
    );
}

// What: `fn empty_and_misplaced_values_are_reported(...)` checks the empty formula, the leading operator, and the missing argument.
// Why: Each mistake says what is missing and where, so the user can find it in the template.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$+$")).toEqual(["formula: a value is missing before +", "formula: a value is missing"]);
// ```
#[test]
fn empty_and_misplaced_values_are_reported() {
    assert_refuses("$$", &["formula: a value is missing"]);
    assert_refuses("$+$", &["formula: a value is missing before +", "formula: a value is missing"]);
    assert_refuses("$mi(title) mi(ext)$", &["formula: unexpected mi"]);
    assert_refuses("$mi(,)$", &["formula: a value is missing before ,", "formula: a value is missing before )"]);
    assert_refuses("$mi()$", &["mi: takes 1 value, as in mi(field)"]);
}

// What: `fn nesting_is_refused_past_twelve_calls_and_shown_at_twelve(...)` checks the nesting bound on both sides.
// Why: Thirteen nested calls are refused and reported once, while twelve still evaluate.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(evaluateTemplate("$" + "tc(up, ".repeat(12) + "x" + ")".repeat(12) + "$").text).toBe("X");
// ```
#[test]
fn nesting_is_refused_past_twelve_calls_and_shown_at_twelve() {
    let thirteen = format!("${}x{}$", "tc(up, ".repeat(13), ")".repeat(13));
    assert_refuses(&thirteen, &["tc: calls are nested too deeply"]);
    let twelve = format!("${}x{}$", "tc(up, ".repeat(12), ")".repeat(12));
    assert_shows_analysed(&twelve, "X");
}

// What: `fn text_that_looks_like_other_syntax_stays_text(...)` checks markup, escapes, and shell-like text.
// Why: Only dollar signs and the formula syntax are special, so everything else is shown exactly as written.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(evaluateTemplate('<b>&amp; "quoted" \\n {x} `y` ; rm -rf ../').text).toBe('<b>&amp; "quoted" \\n {x} `y` ; rm -rf ../');
// ```
#[test]
fn text_that_looks_like_other_syntax_stays_text() {
    assert_shows_analysed(
        "<b>&amp; \"quoted\" \\n {x} `y` ; rm -rf ../",
        "<b>&amp; \"quoted\" \\n {x} `y` ; rm -rf ../",
    );
    assert_shows_analysed("line one\nline two $mi(ext)$", "line one\nline two flac");
    assert_shows_analysed("$\"<script>\" + mi(ext)$", "<script>flac");
}

// What: `fn carriage_return_inside_a_formula_is_a_mistake(...)` checks that a carriage return is part of a word.
// Why: Only space, tab, and line feed separate tokens, so a stray carriage return reads as an unexpected word.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$mi(ext)\r$")).toEqual(["formula: unexpected \r"]);
// ```
#[test]
fn carriage_return_inside_a_formula_is_a_mistake() {
    assert_refuses("$mi(ext)\r$", &["formula: unexpected \r"]);
}

// What: `fn small_probes_match_the_reference(...)` checks negative seconds, astral caps, and the `h` inside apostrophes.
// Why: These are the reference's quirks, and the port keeps them rather than fixing them.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(evaluateTemplate("$tf(4000, \"'h'm:ss\")$").text).toBe("h6:40");
// ```
#[test]
fn small_probes_match_the_reference() {
    assert_shows_analysed("$tf(-5)$", "0:00");
    assert_shows_analysed("$tc(cap, \"\u{10428}abc\")$", "\u{10428}abc");
    assert_shows_analysed("$tc(cap, \"\u{e9}t\u{e9}\")$", "\u{c9}t\u{e9}");
    assert_shows_analysed("$tf(4000, \"'h'm:ss\")$", "h6:40");
    assert_shows_analysed("$tf(1e300, h)$", "2.777777777777778e+296");
    assert_shows_analysed("$tf(1e21, m:ss)$", "16666666666666666000:40");
    assert_shows_analysed("$tc(cut, mi(title), 2.9)$", "An");
    assert_shows_analysed("$tc(up, \"stra\u{df}e\")$", "STRASSE");
    assert_shows_analysed("$tc(low, \"\u{391}\u{3a3}\")$", "\u{3b1}\u{3c2}");
}
