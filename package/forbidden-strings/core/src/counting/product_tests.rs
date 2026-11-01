//! What:
//!     Tests for the synchronized-product back-end:
//!  differential vs the eager DFA on
//!          small `&`/`~` patterns,
//!  and a serialized-size proof that the AWS-key complement
//!          shape stays tiny.
//!  Exempt from the max-lines and rustdoc budgets as a `*_tests.rs`
//!          file.
//! Why:
//!      This file is the Rust module that groups the product_tests implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module product_tests: see exported functions and types below.
//! ```

use crate::ast::node::Node;
use crate::ast::smart::concat;
use crate::counting::product::{ProductProgram, build_product};
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

// What:    Builds the product program, asserting the pattern takes the `&`/`~` back-end.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function product(pattern: string): ProductProgram {
//   // Rust body below is the implementation.
// }
// ```
fn product(pattern: &str) -> ProductProgram {
    let node = parse(pattern).expect("pattern parses");
    return build_product(&node).expect("pattern is a product")
}

// What:    Every linear `&`/`~` pattern must agree with the eager DFA on every probe input.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function product_agrees_with_oracle(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn product_agrees_with_oracle() {
    let patterns = [
        "(?:[A-Z]{3}) & ~(ABC)",
        "(?:[A-Z]{2,4}) & ~(AB)",
        "(?:AKIA[A-Z2-7]{4}) & ~(AKIA2222)",
        "(?:[A-Z]{3}) & (?:[A-Za-z]{3})",
        "(?:\\b[A-Z]{3}\\b) & ~(ABC)",
        "(?:[A-Za-z]{2,3}) & ~(ab) & ~(abc)",
        "(?:(?:(?:AKIA)|(?:ASIA))[A-Z2-7]{4}) & ~(AKIA2222)",
        "(?:\\b(?:(?:A3T[A-Z0-9])|(?:AKIA)|(?:ASIA))[A-Z2-7]{4}\\b) & ~(AKIA2222)",
    ];
    let inputs: [&[u8]; 24] = [
        b"",
        b"A",
        b"AB",
        b"ABC",
        b"ABCD",
        b"xABCx",
        b"xABx",
        b"ab",
        b"abc",
        b"abcd",
        b"AKIA2222",
        b"AKIA2223",
        b"AKIAABCD",
        b"prefix AKIAABCD suffix",
        b" ABC ",
        b"ABC\n",
        b"\nABC",
        b"aZ",
        b"ASIA2222",
        b"ASIAB2C7",
        b"A3TXB2C7",
        b" ASIAB2C7 ",
        b"AKIAB2C7",
        b"A3T2B2C7",
    ];
    for pattern in patterns {
        let prog = product(pattern);
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

// What:    The realistic AWS-key complement shape must stay tiny and decide keys exactly.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function aws_key_complement_stays_small(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn aws_key_complement_stays_small() {
    let prog = product("(?:AKIA[A-Z2-7]{16}) & ~(AKIA2{16})");
    // What:    One positive operand (5 elements) and one negative operand (5 elements).
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(prog.positives.len(), 1);
    assert_eq!(prog.negatives.len(), 1);
    assert_eq!(prog.positives[0].elements.len(), 5);
    assert_eq!(prog.negatives[0].elements.len(), 5);
    let bytes = bincode::serialize(&prog).expect("serializes");
    assert!(
        bytes.len() < 2_000,
        "product program serialized to {} bytes, expected under 2 KB",
        bytes.len()
    );
}

// What:    The complement is exact: a real key passes, only the all-`2` placeholder fails.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function aws_key_complement_is_exact(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn aws_key_complement_is_exact() {
    let prog = product("(?:AKIA[A-Z2-7]{16}) & ~(AKIA2{16})");
    // What:    A realistic key: AKIA then sixteen [A-Z2-7] that are not all '2'.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(prog.is_match(b"AKIAABCDEFGHIJKLMNO\x50"));
    assert!(prog.is_match(b"AKIA234567234567ABC\x44"));
    assert!(prog.is_match(b"prefix AKIAABCDEFGHIJKLMNO\x50 suffix"));
    // What:    The exact placeholder AKIA + sixteen '2' is the complement target: rejected.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!prog.is_match(b"AKIA222222222222222\x32"));
    // What:    One byte off the placeholder is a real key again: only the exact span fails.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(prog.is_match(b"AKIA222222222222222\x33"));
    // What:    Too few trailing bytes: the positive operand cannot match at all.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!prog.is_match(b"AKIA222222222222222"));
}

// What:    A decoded product program runs on untrusted input, so validate must reject a
//          program with no positive operand (it would accept everywhere) or a corrupt operand.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function validate_rejects_an_unsafe_program(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn validate_rejects_an_unsafe_program() {
    let empty = ProductProgram { positives: Vec::new(), negatives: Vec::new() };
    assert!(empty.validate().is_err());

    let good = product("(?:AKIA[A-Z2-7]{4}) & ~(AKIA2222)");
    assert!(good.validate().is_ok());

    let mut corrupt = product("(?:AKIA[A-Z2-7]{4}) & ~(AKIA2222)");
    corrupt.positives[0].elements.clear();
    assert!(corrupt.validate().is_err());
}

// What:    The full AWS rule with an alternation prefix stays small and decides exactly.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function aws_rule_with_alternation_stays_small(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
fn aws_rule_with_alternation_stays_small() {
    let prog = product(
        "(?:\\b(?:(?:A3T[A-Z0-9])|(?:AKIA)|(?:ASIA))[A-Z2-7]{16}\\b) & ~(AKIA2{16})",
    );
    let bytes = bincode::serialize(&prog).expect("serializes");
    assert!(
        bytes.len() < 4_000,
        "product program serialized to {} bytes, expected under 4 KB",
        bytes.len()
    );
    // What:    Real keys via each prefix branch, all 20-byte spans, none the placeholder.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(prog.is_match(b" AKIAABCDEFGHIJKLMNO\x50 "));
    assert!(prog.is_match(b" ASIAABCDEFGHIJKLMNO\x50 "));
    assert!(prog.is_match(b" A3TXABCDEFGHIJKLMNO\x50 "));
    // What:    The exact AKIA + sixteen '2' placeholder is vetoed by the complement.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!prog.is_match(b" AKIA222222222222222\x32 "));
    // What:    The same all-'2' tail under a different prefix is not the placeholder.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(prog.is_match(b" ASIA222222222222222\x32 "));
}
