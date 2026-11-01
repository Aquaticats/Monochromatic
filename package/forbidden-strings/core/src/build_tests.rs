// What:  unit tests for rule routing: which matching path (line-start, anchored gate,
//        plain gate, weak fold) each rule shape takes, and the seedless/reference flags.
// Why:     This file groups the build test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("build", () => {
//   // test cases below
// });
// ```

use super::{build_rule, build_seedless_union, route_rule, starts_with_line_anchor};
use crate::parse::parse;

fn node(pattern: &str) -> crate::ast::node::Node {
    return parse(pattern).expect("test pattern parses")
}

#[test]
fn line_anchor_is_detected_only_at_the_head() {
    assert!(starts_with_line_anchor(&node("^abc")));
    assert!(!starts_with_line_anchor(&node("abc")));
    assert!(!starts_with_line_anchor(&node("abc$")));
}

#[test]
fn marker_rule_routes_to_the_line_start_check() {
    let routing = route_rule(&node("^(?:(?:PR)|(?:TS))[0-9]:"));
    assert!(routing.line_start.is_some());
    assert!(routing.seeds.is_empty());
    assert!(routing.anchored.is_none());
}

#[test]
fn leading_literal_rule_routes_to_an_anchored_gate() {
    let routing = route_rule(&node("AKIA[A-Z2-7]{16}"));
    assert_eq!(routing.seeds, vec![b"AKIA".to_vec()]);
    assert!(routing.anchored.is_some());
    assert!(routing.line_start.is_none());
}

#[test]
fn inner_keyword_rule_routes_to_an_unanchored_gate() {
    let routing = route_rule(&node("[a-z]{3}adafruit[a-z]{3}"));
    assert_eq!(routing.seeds, vec![b"adafruit".to_vec()]);
    assert!(routing.anchored.is_none());
    assert!(routing.line_start.is_none());
}

#[test]
fn short_leading_literal_folds_to_a_weak_anchored_seed() {
    let routing = route_rule(&node("SK[0-9a-f]{32}"));
    assert_eq!(routing.seeds, vec![b"SK".to_vec()]);
    assert!(routing.anchored.is_some());
}

#[test]
fn weak_inner_literal_folds_to_an_unanchored_seed() {
    let routing = route_rule(&node("[a-z]{3}Q\\~[a-z]{3}"));
    assert_eq!(routing.seeds, vec![b"Q~".to_vec()]);
    assert!(routing.anchored.is_none());
    assert!(routing.line_start.is_none());
}

#[test]
fn build_rule_flags_seedless_and_reference_nodes() {
    let akia = build_rule(node("AKIA[A-Z2-7]{16}")).expect("build a rule for a leading-literal pattern");
    assert!(!akia.seedless);
    // What:    had a leading seed.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(akia.reference_node.is_none());

    let inner = build_rule(node("[a-z]{3}adafruit[a-z]{3}")).expect("build a rule for an inner-literal pattern");
    assert!(!inner.seedless);
    // What:    had an inner seed (not seedless).
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(inner.reference_node.is_none());

    let bare = build_rule(node("[a-z]{20}")).expect("build a rule for a seedless pattern");
    assert!(bare.seedless);
    // What:    truly literal-free.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(bare.reference_node.is_some());
}

#[test]
fn seedless_union_builds_only_for_nonempty_input() {
    assert!(build_seedless_union(&[]).is_none());
    assert!(build_seedless_union(&[node("[a-z]{20}")]).is_some());
}
