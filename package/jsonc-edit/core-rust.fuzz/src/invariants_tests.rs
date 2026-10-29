//! What:
//!      Unit tests for the shared invariant checks.
//! Why:
//!       An invariant that cannot fail is not an invariant.
//!  Each check here is proven in both
//!           directions:
//!  a positive control where it must hold,
//!  and a deliberately broken input
//!           where it must panic,
//!  so a fuzz campaign that reports nothing is evidence rather than
//!           silence.

/// What:
///      Import the panic-capture helpers.
/// Why:
///       The negative controls assert that a check panics,
///  and the default hook would print a
///           backtrace for every one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { catchUnwind, setHook, takeHook } from 'std:panic';
/// ```
use std::panic::{catch_unwind, set_hook, take_hook};

/// What:
///      Import the checks under test.
/// Why:
///       Every exported invariant needs a passing and a failing case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { assertCanonicalStability, ... } from './invariants';
/// ```
use crate::invariants::{
    assert_canonical_stability, assert_comments_preserved, assert_depth_bound, assert_trees_equal,
    collect_comments, depth_of,
};

/// What:
///      Import the crate's model and parse entry point.
/// Why:
///       Negative controls need trees that the parser would never produce,
///  which means building
///           them directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc, JsoncKind, JsoncValue } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{emit_jsonc_value, parse_jsonc, JsoncKind, JsoncValue};

/// What:
///      Report whether a closure panics,
///  without printing the panic.
/// Why:
///       Negative controls intentionally trip assertions,
///  and their output would bury the real
///           test results.
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

/// What:
///      Parse one document,
///  failing the test with the parser's own message if it refuses.
/// Why:
///       Every case here starts from text,
///  and a rejection at that point is a test bug.
fn parse(source: &str) -> JsoncValue {
    return parse_jsonc(source).unwrap_or_else(|error| panic!("test source must parse: {error}\n{source}"));
}

/// Canonical emission must be a fixed point for a document that uses every feature.
#[test]
fn canonical_emission_is_a_fixed_point() {
    assert_canonical_stability("// lead\n{/* k */\"a\":1e0,// v\n\"b\":[true,null,],}/* block */");
}

/// Comments must be collected from the root,
///  from keys and from values.
#[test]
fn comments_are_collected_from_every_owner() {
    let document = parse("// root\n{/* key */\"a\":/* value */1}");
    let bodies = collect_comments(&document);
    assert!(bodies.iter().any(|body| return body == " root"), "root comment missing from {bodies:?}");
    assert!(bodies.iter().any(|body| return body == " key "), "key comment missing from {bodies:?}");
    assert!(bodies.iter().any(|body| return body == " value "), "value comment missing from {bodies:?}");
}

/// A dropped comment must trip the preservation check.
#[test]
fn dropped_comment_fails_the_preservation_check() {
    let document = parse("{/* keep me */\"a\":1}");
    let emitted = emit_jsonc_value(&document);
    assert!(!panics(|| return assert_comments_preserved(&document, &emitted)), "positive control must hold");
    let stripped = emitted.replace("keep me", "");
    assert!(
        panics(|| return assert_comments_preserved(&document, &stripped)),
        "preservation check accepted emission that dropped a comment"
    );
}

/// Depth must count containers,
///  and the bound must reject one past the limit.
#[test]
fn depth_bound_accepts_the_limit_and_rejects_one_past_it() {
    let accepted = parse(&format!("{}0{}", "[".repeat(512), "]".repeat(512)));
    assert_eq!(depth_of(&accepted), 512, "accepted document measured at the wrong depth");
    assert!(!panics(|| return assert_depth_bound(&accepted)), "depth 512 must satisfy the bound");

    // A tree this deep cannot come from the parser, which refuses it, so it is built by hand to
    // prove the check itself fails rather than trusting the parser to have caught it first.
    let mut too_deep = JsoncValue { kind: JsoncKind::Array { elements: Vec::new() }, comment: None };
    for _ in 0..512 {
        too_deep = JsoncValue { kind: JsoncKind::Array { elements: vec![too_deep] }, comment: None };
    }
    assert_eq!(depth_of(&too_deep), 513, "hand-built tree measured at the wrong depth");
    assert!(panics(|| return assert_depth_bound(&too_deep)), "depth 513 must violate the bound");
}

/// Two parses of the same text must compare equal,
///  and different documents must not.
#[test]
fn tree_equality_separates_identical_from_different() {
    let source = "{\"a\":[1,2],\"b\":null}";
    assert!(!panics(|| return assert_trees_equal(&parse(source), &parse(source))), "identical trees must compare equal");
    assert!(
        panics(|| return assert_trees_equal(&parse(source), &parse("{\"a\":[1,3],\"b\":null}"))),
        "different trees must not compare equal"
    );
    assert!(
        panics(|| return assert_trees_equal(&parse(source), &parse("{\"a\":[1,2]}"))),
        "a missing member must not compare equal"
    );
}

/// Comment style is presentation,
///  so a re-rendered comment must not break equality.
#[test]
fn comment_style_does_not_break_equality() {
    // A single-line block comment is re-rendered in `//` form by canonical emission, and a reparse
    // then reports the other style. The body is the contract, so equality must ignore the style.
    let document = parse("{\"a\":1,/* note */}");
    let emitted = emit_jsonc_value(&document);
    let reparsed = parse(&emitted);
    assert!(!panics(|| return assert_trees_equal(&document, &reparsed)), "style-only difference broke equality");
    let bodies_before = collect_comments(&document);
    let bodies_after = collect_comments(&reparsed);
    assert_eq!(bodies_before.len(), bodies_after.len(), "comment count changed across emission");
}
