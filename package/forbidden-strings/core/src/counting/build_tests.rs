// What:  unit tests for the Glushkov counting-NFA builder: which node shapes it can
//        express, and that the NFA it builds matches as a substring search.
// Why:     This file groups the build test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("build", () => {
//   // test cases below
// });
// ```

use super::build_nfa;
use crate::parse::parse;

fn node(pattern: &str) -> crate::ast::node::Node {
    return parse(pattern).expect("test pattern parses")
}

#[test]
fn builds_a_counted_class_chain() {
    let nfa = build_nfa(&node("AKIA[A-Z2-7]{4}")).expect("expressible as an NFA");
    assert!(nfa.is_match(b"xx AKIAB2C7 yy"));
    assert!(!nfa.is_match(b"AKIA01"));
    assert!(!nfa.is_match(b"no key"));
}

#[test]
fn builds_a_bare_class_repetition_as_substring_search() {
    let nfa = build_nfa(&node("[a-z]{4}")).expect("expressible");
    assert!(nfa.is_match(b"--abcd--"));
    assert!(!nfa.is_match(b"abc"));
}

#[test]
fn builds_an_alternation() {
    let nfa = build_nfa(&node("(?:(?:AKIA)|(?:ASIA))[A-Z2-7]{4}")).expect("expressible");
    assert!(nfa.is_match(b"AKIAB2C7"));
    assert!(nfa.is_match(b"ASIAB2C7"));
    assert!(!nfa.is_match(b"ABIAB2C7"));
}

#[test]
fn unrolls_a_small_non_class_group_repetition() {
    let nfa = build_nfa(&node("(?:ab){3}")).expect("small group repetition unrolls");
    assert!(nfa.is_match(b"zababab"));
    assert!(!nfa.is_match(b"abab"));
}

#[test]
fn declines_intersection_and_complement() {
    // What:    `&`/`~` are the product back-end's job, not the plain NFA's.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(build_nfa(&node("[a-z]&~(m)")).is_none());
    assert!(build_nfa(&node("(?:[a-z]{4})&~(abcd)")).is_none());
}

#[test]
fn group_repetition_unrolls_up_to_the_limit_and_declines_beyond() {
    // What:    A non-class body is unrolled only when its max is at most the unroll limit
    //          (64): a max at the limit builds, one past it the builder declines so the caller
    //          routes elsewhere. (min is 1 so the repetition is not empty-matchable, which
    //          parse rejects outright.).
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(build_nfa(&node("(?:ab){1,64}")).is_some(), "a 64-copy max unroll is allowed");
    assert!(build_nfa(&node("(?:ab){1,65}")).is_none(), "a 65-copy max unroll is declined");
}

#[test]
fn a_nullable_alternation_branch_makes_the_alternation_skippable() {
    // What:    build_alt ORs nullability across branches, so a single nullable branch
    //          (`(?:xy)?`) makes the whole alternation nullable, letting the following `w`
    //          match with the alternation skipped. A wrong combiner (AND) would lose that and
    //          reject "w".
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let prog = build_nfa(&node("(?:(?:xy)?|z)w")).expect("expressible as an NFA");
    assert!(prog.is_match(b"w"), "the nullable branch lets the alternation be skipped");
    assert!(prog.is_match(b"xyw"));
    assert!(prog.is_match(b"zw"));
}
