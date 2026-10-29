//! What:
//!     Tests for the counting-set simulation:
//!  differential vs the eager DFA,
//!  and a
//!          serialized-size proof that bounded repetition no longer blows up.
//!  Exempt from the
//!          max-lines and rustdoc budgets as a `*_tests.rs` file.
//! Why:
//!      This file is the Rust module that groups the run_tests implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module run_tests: see exported functions and types below.
//! ```

use crate::ast::node::Node;
use crate::ast::smart::concat;
use crate::counting::build::build_nfa;
use crate::counting::nfa::CountingNfa;
use crate::dfa::build_dfa_within;
use crate::parse::parse;

// What:    Builds the eager-DFA oracle for a pattern: the trusted (if large) reference.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function oracle(pattern: string): impl Fn(&[u8]) -> bool {
//   // Rust body below is the implementation.
// }
// ```
fn oracle(pattern: &str) -> impl Fn(&[u8]) -> bool {
    let node = parse(pattern).expect("oracle pattern parses");
    let dfa = build_dfa_within(concat(vec![Node::Top, node]), 200_000).expect("oracle dfa builds");
    return move |line: &[u8]| return dfa.is_match(line)
}

// What:    Builds the counting NFA for a pattern, asserting it takes the counting back-end.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function linear(pattern: string): CountingNfa {
//   // Rust body below is the implementation.
// }
// ```
fn linear(pattern: &str) -> CountingNfa {
    let node = parse(pattern).expect("pattern parses");
    return build_nfa(&node).expect("pattern builds an nfa")
}

// What:    Every counting-NFA pattern must agree with the eager DFA on every probe input.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function linear_agrees_with_oracle(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn linear_agrees_with_oracle() {
    let patterns = [
        "abc",
        "[A-Z]",
        "[A-Z]{2}",
        "[A-Z]{2,4}",
        "a[0-9]{1,3}z",
        "\\bAKIA[A-Z2-7]{3}\\b",
        "^abc",
        "abc$",
        "[A-Z]{2}[0-9]{2}",
        "x?y",
        "AKIA[A-Z2-7]{4}",
        "(?:(?:AKIA)|(?:ASIA))[A-Z2-7]{4}",
        "(?:(?:abc)|(?:de)|f)",
        "\\b(?:(?:A3T[A-Z0-9])|(?:AKIA)|(?:ASIA))[A-Z2-7]{3}\\b",
        "x(?:(?:ab)|(?:cd))y",
    ];
    let inputs: [&[u8]; 24] = [
        b"",
        b"a",
        b"abc",
        b"xabcx",
        b"AB",
        b"ABCD",
        b"ABCDE",
        b"a12z",
        b"a1234z",
        b" AKIAABC ",
        b"xAKIAABCy",
        b"abc\n",
        b"\nabc",
        b"AB12",
        b"y",
        b"xy",
        b"ASIAB2C7",
        b"AKIAB2C7",
        b"A3TXB2C7",
        b"de",
        b"f",
        b"abz",
        b"cdz",
        b"z",
    ];
    for pattern in patterns {
        let prog = linear(pattern);
        let oracle_fn = oracle(pattern);
        for input in inputs {
            assert_eq!(
                prog.is_match(input),
                oracle_fn(input),
                "pattern {pattern:?} disagreed on input {:?}",
                String::from_utf8_lossy(input)
            );
        }
    }
}

// What:    The AWS-key shape that blew the eager DFA to 72 KB must stay tiny here.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function counted_key_stays_small(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn counted_key_stays_small() {
    let prog = linear("AKIA[A-Z2-7]{16}");
    // What:    Four literal classes plus one counted element.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(prog.elements.len(), 5);
    let bytes = bincode::serialize(&prog).expect("serializes");
    assert!(
        bytes.len() < 2_000,
        "linear program serialized to {} bytes, expected under 2 KB",
        bytes.len()
    );
}

// What:    The counted bound must match exactly: 16 trailing chars accept, 15 reject.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function counted_bound_is_exact(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn counted_bound_is_exact() {
    let prog = linear("AKIA[A-Z2-7]{16}");
    assert!(prog.is_match(b"AKIAABCDEFGHIJKLMNO\x50"));
    assert!(prog.is_match(b"prefix AKIAABCDEFGHIJKLMNO\x50 suffix"));
    assert!(!prog.is_match(b"AKIAABCDEFGHIJKLMNO"));
    // What:    A digit outside [A-Z2-7] breaks the run (0 and 1 are excluded).
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!prog.is_match(b"AKIAABCDEFGHIJKLMN0P"));
}

// What:    A count of 64 needs bit 64, which spills past the first 64-bit word: count_set_for
//          must size each counted position's bitset to the element's own max, not a one-word
//          placeholder.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function a_count_past_one_word_is_sized_correctly(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn a_count_past_one_word_is_sized_correctly() {
    let prog = linear("[a-z]{64}");
    assert!(prog.is_match(&[b'a'; 64]), "exactly 64 letters must satisfy {{64}}");
    assert!(!prog.is_match(&[b'a'; 63]), "63 letters fall one short of {{64}}");
}
