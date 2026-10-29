// What:  unit tests for the RegexSet internals that the public match paths rely on but
//        that no end-to-end verdict pins precisely: the per-line rule-dedup bitset
//        (`CheckedFull`) and the one-byte line-start fast-reject (`line_start_candidate`).
// Why:     This file groups the regex test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("regex", () => {
//   // test cases below
// });
// ```

use super::{CheckedFull, RegexSet};

#[test]
fn checked_full_reports_each_rule_exactly_once() {
    // What:    Each in-range rule is fresh on first sight and deduped after; the ids span
    //          several 64-bit words and bit positions so a wrong word (`/64`) or bit (`<<`,
    //          `%64`) would alias two distinct rules and be caught.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut checked = CheckedFull::new();
    for rule in [0usize, 1, 5, 63, 64, 65, 127, 128, 191, 255] {
        assert!(checked.first_time(rule), "rule {rule} should be fresh on first sight");
        assert!(!checked.first_time(rule), "rule {rule} should be deduped on second sight");
    }
}

#[test]
fn checked_full_keeps_distinct_rules_independent() {
    // What:    Recording one rule must not mark another: rule 64 sits in a different word than
    //          0, and rules 0/1/65 in different bits, so a word/bit miscalculation collapses
    //          them.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut checked = CheckedFull::new();
    assert!(checked.first_time(0));
    assert!(checked.first_time(64), "rule 64 is a different word than rule 0");
    assert!(checked.first_time(1), "rule 1 is a different bit than rule 0");
    assert!(checked.first_time(65), "rule 65 must not alias rule 64");
}

#[test]
fn checked_full_treats_ids_beyond_capacity_as_always_fresh() {
    // What:    The set holds 256 bits; ids at or above that always report fresh (sound: the
    //          caller simply re-runs the whole-line check), and must never index out of
    //          bounds.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut checked = CheckedFull::new();
    assert!(checked.first_time(256));
    assert!(checked.first_time(256));
    assert!(checked.first_time(1000));
}

#[test]
fn line_start_candidate_gates_on_the_first_byte() {
    // What:    A `^`-anchored rule matches only at offset zero, so the one-byte fast reject
    //          must say "candidate" exactly when the first byte is one the line-start rules
    //          can begin with.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let set = RegexSet::new(&["^#deny this"]).expect("compiles");
    assert!(set.line_start_candidate(b"#deny this row"), "'#' begins the line-start rule");
    assert!(!set.line_start_candidate(b"deny this row"), "'d' cannot begin the line-start rule");
    assert!(!set.line_start_candidate(b""), "an empty line has no first byte");
}
