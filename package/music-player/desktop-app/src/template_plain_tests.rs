// Unit tests for `template_plain.rs`, pulled in by `#[cfg(test)] #[path = "template_plain_tests.rs"] mod tests;`.
//
// Every case runs the real parser and evaluator on a mistaken template and passes the mistakes it records to
// `plain_lines`, so a note the engine words differently later cannot slip past as a developer fragment.

use super::*;
use crate::template_eval::evaluate_template;
use crate::template_fixtures::analysed;
use crate::template_model::{TemplateResult, track_template_fields};

// What: `fn lines(text: &str) -> Vec<String>` evaluates a template on the analysed track and gives the plain lines of its mistakes.
// Why: the editor passes the mistakes of the typed text, and a template that applies has none.
fn lines(text: &str) -> Vec<String> {
    return match evaluate_template(text, &analysed(), &track_template_fields()) {
        TemplateResult::Refused(errors) => plain_lines(text, &errors),
        TemplateResult::Shown(_) => Vec::new(),
    };
}

// What: `fn a_half_written_call_is_one_problem(...)` types `$mi(`, which the parser records as three mistakes.
// Why: TE1: the editor listed "the $ has no closing $", "a closing bracket is missing" and "a value is missing" for one
// unfinished call; one sentence is what a person can act on.
#[test]
fn a_half_written_call_is_one_problem() {
    assert_eq!(lines("$mi(").len(), 1, "one sentence for $mi(");
    assert_eq!(
        lines("$mi("),
        vec!["The $ at character 1 has no closing $. Add a $ where the formula ends."]
    );
    assert_eq!(
        lines("$mi($"),
        vec!["mi is missing its closing bracket. Add a ) after its last value."],
        "closing the formula leaves the bracket as the one problem"
    );
}

// What: `fn an_empty_formula_asks_for_a_value(...)` types `$$` and a lone operator.
// Why: TE1: `formula: a value is missing` named no fix; the sentence says what to put there.
#[test]
fn an_empty_formula_asks_for_a_value() {
    assert_eq!(lines("$$"), vec!["A formula is missing a value. Put a field such as mi(title), or some text, in it."]);
    assert_eq!(
        lines("$+$"),
        vec!["A value is missing before + in a formula. Put a field such as mi(title), or some text, there."]
    );
}

// What: `fn each_formula_with_a_parse_problem_gets_its_own_sentence(...)` breaks two formulas.
// Why: the first problem of each formula is shown, so two formulas make two sentences and not six.
#[test]
fn each_formula_with_a_parse_problem_gets_its_own_sentence() {
    let found = lines("$mi($ and $tf(");
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].starts_with("mi is missing its closing bracket"), "{found:?}");
    assert!(found[1].starts_with("The $ at character"), "{found:?}");
}

// What: `fn mistakes_of_a_call_that_parses_are_all_kept(...)` misuses two calls of one formula.
// Why: a formula that parses has one mistake per call, and each needs its own fix, so none is hidden.
#[test]
fn mistakes_of_a_call_that_parses_are_all_kept() {
    let found = lines("$tf(mi(title), mi(nope))$");
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(found[0], "tf needs a number of seconds, not Another Xronixle. Use mi(len) for the duration.");
    assert_eq!(found[1], "mi has no field called nope. Pick a name from the Fields list.");
}

// What: `fn engine_mistakes_read_as_instructions(...)` checks the sentence of each mistake the evaluator can record.
// Why: TE1 and TE5: every sentence names the call and says what to do, including the new format-letter mistake.
#[test]
fn engine_mistakes_read_as_instructions() {
    assert_eq!(lines("$zzz(1)$"), vec!["zzz is not a function. Use mi, tf or tc."]);
    assert_eq!(lines("$mi(nope)$"), vec!["mi has no field called nope. Pick a name from the Fields list."]);
    assert_eq!(lines("$tc(ell, mi(title), 5)$"), vec!["tc has no mode called ell. Use low, up, cap or cut."]);
    assert_eq!(lines("$tc(cut, mi(title))$"), vec!["tc needs a length to cut to, as in tc(cut, text, 10)."]);
    assert_eq!(lines("$mi()$"), vec!["mi takes 1 value, as in mi(field)."]);
    assert_eq!(lines("$tf(1, m, s)$"), vec!["tf takes 1 or 2 values, as in tf(seconds, [format])."]);
    assert_eq!(
        lines("$tf(mi(len), q:q)$"),
        vec!["tf does not know the format letter q. Use h, m or s, and put any other words between apostrophes."]
    );
    assert_eq!(
        lines("$mi(title) mi(ext)$"),
        vec!["Unexpected mi in a formula. Join values with + or put each one in its own pair of $ signs."]
    );
    assert_eq!(lines("$mi(\"title)$"), vec!["mi is missing its closing bracket. Add a ) after its last value."]);
}

// What: `fn no_line_keeps_a_developer_prefix(...)` runs a list of mistaken templates and checks every line.
// Why: TE1: a line must read as a sentence, so none starts with a `name:` note and each ends with a full stop.
#[test]
fn no_line_keeps_a_developer_prefix() {
    let mistaken = [
        "$mi(", "$$", "$+$", "$mi(,)$", "$mi($", "$xx(1)$", "$mi(nope)$", "$tf(mi(title))$", "$tf()$", "$tc(up)$", "$tc(ell, 1, 2)$",
        "$tc(cut, mi(title), x)$", "$mi(title) mi(ext)$", "$mi(\"title)$", "$tf(mi(len), q:q)$", "$tc(up, tc(up, tc(up, tc(up, tc(up, tc(up, tc(up, tc(up, tc(up, tc(up, tc(up, tc(up, tc(up, x)))))))))))))$",
    ];
    for text in mistaken {
        let found = lines(text);
        assert!(!found.is_empty(), "{text:?} has a problem to report");
        for line in found {
            assert!(line.ends_with('.'), "{text:?}: {line:?} is not a sentence");
            for prefix in ["formula:", "text:", "mi:", "tf:", "tc:", "xx:"] {
                assert!(!line.starts_with(prefix), "{text:?}: {line:?} keeps the note prefix {prefix}");
            }
        }
    }
}

// What: `fn lines_follow_template_order(...)` checks two formulas that each misuse a call.
// Why: the editor lists problems where they sit in the template, so the user reads them top down.
#[test]
fn lines_follow_template_order() {
    assert_eq!(
        lines("$mi(peek)$ $mi(seek)$"),
        vec![
            "mi has no field called peek. Pick a name from the Fields list.",
            "mi has no field called seek. Pick a name from the Fields list.",
        ]
    );
}
