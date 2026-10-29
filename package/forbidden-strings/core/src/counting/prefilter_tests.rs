// What:  unit tests for required-literal seed extraction and the SIMD prefilter.
// Why:     This file groups the prefilter test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("prefilter", () => {
//   // test cases below
// });
// ```

use super::{
    Prefilter, leading_literals, leading_seeds, leading_seeds_min, seeds_from_node,
    seeds_from_node_min,
};
use crate::parse::parse;

// What:    Parses a pattern (panicking on a bad test input) for seed extraction.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function node(pattern: string): crate.ast.node.Node {
//   // Rust body below is the implementation.
// }
// ```
fn node(pattern: &str) -> crate::ast::node::Node {
    return parse(pattern).expect("test pattern parses")
}

#[test]
fn leading_literal_is_the_seed_for_a_prefixed_rule() {
    assert_eq!(seeds_from_node(&node("AKIA[A-Z2-7]{16}")), vec![b"AKIA".to_vec()]);
    assert_eq!(leading_seeds(&node("AKIA[A-Z2-7]{16}")), vec![b"AKIA".to_vec()]);
}

#[test]
fn inner_keyword_is_a_required_seed() {
    // What:    The mandatory inner keyword is a required substring even without a leading
    //          literal.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let seeds = seeds_from_node(&node("[a-z]{3}adafruit[a-z]{3}"));
    assert_eq!(seeds, vec![b"adafruit".to_vec()]);
    // What:    It is not a LEADING literal (the class run precedes it), so leading_seeds is
    //          empty.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(leading_seeds(&node("[a-z]{3}adafruit[a-z]{3}")).is_empty());
}

#[test]
fn class_only_rule_has_no_seed() {
    assert!(seeds_from_node(&node("[a-z]{5}")).is_empty());
    assert!(leading_seeds(&node("[A-Z2-7]{16}")).is_empty());
}

#[test]
fn alternation_seeds_require_every_branch_to_contribute() {
    // What:    Every branch begins with a >=3 literal, so the union is a sound leading seed.
    //          Multi-byte alternation branches must each be wrapped as a single atom.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let seeds = leading_seeds(&node("(?:(?:AKIA)|(?:ASIA))[A-Z2-7]{16}"));
    assert!(seeds.contains(&b"AKIA".to_vec()));
    assert!(seeds.contains(&b"ASIA".to_vec()));
    // What:    One branch with no literal sinks the whole alternation's seed.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(leading_seeds(&node("(?:(?:AKIA)|(?:[A-Z]{4}))[A-Z2-7]{16}")).is_empty());
}

#[test]
fn short_literals_are_below_the_default_floor_but_pass_the_weak_floor() {
    // What:    "SK" (2 bytes) is too short for the default floor, fine for the weak leading
    //          floor.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(leading_seeds(&node("SK[0-9a-f]{32}")).is_empty());
    assert_eq!(leading_seeds_min(&node("SK[0-9a-f]{32}"), 2), vec![b"SK".to_vec()]);
}

#[test]
fn one_byte_required_literal_only_at_the_weakest_floor() {
    // What:    facebook-shape: the only literal is the one-byte alternation, seen at floor 1.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let seeds = seeds_from_node_min(&node("\\d{15,16}(?:(?:\\|)|(?:%))[0-9a-z]{27,40}"), 1);
    assert!(seeds.contains(&b"|".to_vec()));
    assert!(seeds.contains(&b"%".to_vec()));
    // What:    Nothing at the default floor.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(seeds_from_node(&node("\\d{15,16}(?:(?:\\|)|(?:%))[0-9a-z]{27,40}")).is_empty());
}

#[test]
fn intersection_takes_a_positive_operands_required_literal() {
    // What:    A match must satisfy every positive operand, so a positive operand's required
    //          literal is a sound seed; the complement operand contributes none.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let pat = "(?:AKIA[A-Z2-7]{16})&~(AKIA2{16})";
    assert!(seeds_from_node(&node(pat)).contains(&b"AKIA".to_vec()));
    assert!(leading_literals(&node(pat)).contains(&b"AKIA".to_vec()));
}

#[test]
fn complement_only_intersection_has_no_positive_seed() {
    // What:    Both positive operands are class-only, so the only literal lives in a
    //          complement, which is not required of a match; no seed.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let pat = "(?:[a-z]{4})&~(abcd)";
    assert!(seeds_from_node_min(&node(pat), 1).is_empty());
}

#[test]
fn best_inner_literal_keeps_the_first_of_equal_length() {
    // What:    Two equal-length mandatory runs ("ab" then "cd"); the first is kept (a later
    //          equal-length candidate does not displace it).
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(seeds_from_node_min(&node("ab[0-9]cd"), 1), vec![b"ab".to_vec()]);
    // What:    A strictly longer later run does displace the shorter earlier one.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(seeds_from_node_min(&node("ab[0-9]cdef"), 1), vec![b"cdef".to_vec()]);
}

#[test]
fn prefilter_allows_only_lines_containing_a_seed() {
    let prefilter = Prefilter::from_seeds(&[b"AKIA".to_vec()]);
    assert!(prefilter.allows(b"xx AKIA yy"));
    assert!(!prefilter.allows(b"no key here"));
}

#[test]
fn empty_prefilter_allows_everything() {
    let prefilter = Prefilter::from_seeds(&[]);
    assert!(prefilter.allows(b"anything"));
    assert!(prefilter.allows(b""));
}

#[test]
fn prefilter_allows_when_any_seed_present() {
    let prefilter = Prefilter::from_seeds(&[b"AKIA".to_vec(), b"ASIA".to_vec()]);
    assert!(prefilter.allows(b"has ASIA only"));
    assert!(!prefilter.allows(b"has neither"));
}
