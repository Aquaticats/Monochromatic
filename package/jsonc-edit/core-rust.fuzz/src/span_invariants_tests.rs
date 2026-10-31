//! What:
//!  Unit tests for the span checks, each rule shown to pass on an honest result and fail on a tampered one.
//! Why:
//!  A check that never fails turns a clean fuzz campaign into silence rather than evidence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('span invariants', () => { it('rejects a shifted span', () => { /* tamper, expect throw */ }); });
//! ```

/// What:
///  Import the panic-capture helpers.
/// Why:
///  The negative controls intentionally trip assertions, and the default hook would print each backtrace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { catchUnwind, setHook, takeHook } from 'std:panic';
/// ```
use std::panic::{catch_unwind, set_hook, take_hook};

/// What:
///  Import the checks under test.
/// Why:
///  Every exported check needs a passing and a failing case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { assertSpansConsistent, assertSpansMatch } from './spanInvariants';
/// ```
use crate::span_invariants::{assert_spans_consistent, assert_spans_match};

/// What:
///  Import the span-reporting parse the tests tamper with.
/// Why:
///  A negative control needs a result the parser would never produce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsoncWithSpans } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{parse_jsonc_with_spans, JsoncSpanned};

/// What:
///  Report whether a closure panics, without printing the panic.
/// Why:
///  Negative controls intentionally trip assertions.
fn panics<F>(body: F) -> bool
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let previous = take_hook();
    set_hook(Box::new(|_| return));
    let outcome = catch_unwind(body);
    set_hook(previous);
    return outcome.is_err();
}

/// The document every case tampers with: nested containers, comments, CRLF and a multi-byte key.
const DOCUMENT: &str = "// lead\n{\r\n  \"a\": [1, {\"b\": null}], /* c */\n  \"\u{00e9}\": true\n}";

/// What:
///  Parse the shared document with spans.
/// Why:
///  Each case starts from an honest result and changes one thing.
fn honest() -> JsoncSpanned {
    return parse_jsonc_with_spans(DOCUMENT).unwrap_or_else(|error| panic!("test document must parse: {error}"));
}

/// Honest results pass, for the shared document and a few shapes around it.
#[test]
fn honest_results_satisfy_every_rule() {
    assert_spans_consistent(DOCUMENT);
    for source in ["{}", "[]", "[[[]]]", "{\"a\":{\"b\":{}}}", "[1,2,3,]"] {
        assert_spans_consistent(source);
    }
}

/// A span moved off its token must trip the slice rule.
#[test]
fn a_shifted_value_span_fails() {
    let mut tampered = honest();
    tampered.spans.values[2].start.offset += 1;
    assert!(panics(|| return assert_spans_match(DOCUMENT, &tampered)));
}

/// A missing key span must trip the use-every-span rule or the lookup.
#[test]
fn a_dropped_key_span_fails() {
    let mut tampered = honest();
    tampered.spans.keys.pop();
    assert!(panics(|| return assert_spans_match(DOCUMENT, &tampered)));
}

/// An extra span nobody owns must trip the exhaustion rule.
#[test]
fn an_extra_value_span_fails() {
    let mut tampered = honest();
    let extra = tampered.spans.values[1];
    tampered.spans.values.push(extra);
    assert!(panics(|| return assert_spans_match(DOCUMENT, &tampered)));
}

/// A wrong line number with the right offset must trip the position-agreement rule.
#[test]
fn a_wrong_line_fails() {
    let mut tampered = honest();
    tampered.spans.values[1].start.line += 1;
    assert!(panics(|| return assert_spans_match(DOCUMENT, &tampered)));
}

/// A wrong column with the right offset must trip the position-agreement rule.
#[test]
fn a_wrong_column_fails() {
    let mut tampered = honest();
    tampered.spans.keys[0].start.column += 1;
    assert!(panics(|| return assert_spans_match(DOCUMENT, &tampered)));
}

/// A tree that no longer matches the text its spans describe must trip the token rule.
#[test]
fn a_tree_that_does_not_match_its_spans_fails() {
    let tampered_source = "{\"a\":1}";
    let mut tampered = parse_jsonc_with_spans(tampered_source).unwrap_or_else(|error| panic!("{error}"));
    tampered.value = parse_jsonc_with_spans("{\"a\":2}").unwrap_or_else(|error| panic!("{error}")).value;
    assert!(panics(|| return assert_spans_match(tampered_source, &tampered)));
}
