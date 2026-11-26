//! What:
//!     End-to-end tests of how one line is routed through a ruleset:
//!  which rule ids a line
//!  reports,
//!  the `^` rules checked at line start,
//!  the weak inner seeds,
//!  and a rule the
//!  union group's state cap cannot cover.
//! Why:
//!      This file is the Rust module that groups the routing tests,
//!  so the test binary gives
//!          them one namespace and the shared key stays in `integration.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe("routing a line through a ruleset", ...) with the cases below.
//! ```

/// What:
///     Imports the key each AWS-shaped rule is checked against,
///  defined once for the whole binary.
/// Why:
///      The tests below assert against that one literal;
///  importing it keeps one definition
///          instead of a copy per test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AKIA_KEY } from "../integration";
/// ```
use super::AKIA_KEY;

/// What:
///     Imports the compiled-ruleset type these tests build and match against.
/// Why:
///      The code below calls `RegexSet::new`,
///  `is_match` and `matches` directly;
///  importing
///          the type keeps each test focused on the routing instead of the full path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegexSet } from "forbidden_regex";
/// ```
use forbidden_regex::RegexSet;
#[test]
fn from_ruleset_splits_on_delimiter() {
    let text = "AKIA[A-Z2-7]{16}\n---\nghp_[A-Za-z0-9]{36}\n";
    let set = RegexSet::from_ruleset(text, "---").expect("build a regex set from ruleset text");
    assert_eq!(set.len(), 2);
    assert!(set.is_match(AKIA_KEY.as_bytes()));
}

#[test]
fn line_start_marker_rule_matches_only_at_line_start() {
    // What:    A `^`-anchored marker (short codes) routes to the line-start check, not the
    //          gate.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let set = RegexSet::new(&["^(?:(?:PR)|(?:TS))[0-9]:"])
        .expect("build a regex set from the line-start marker pattern");
    assert!(set.is_match(b"PR5: a note"));
    assert!(set.is_match(b"TS0:"));
    // What:    not at line start.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(!set.is_match(b"xPR5:"));
    assert!(!set.is_match(b"PR: no digit"));
    assert!(!set.is_match(b"nothing"));
    let hits: Vec<usize> = set.matches(b"PR5:").collect();
    assert_eq!(hits, vec![0]);
}

#[test]
fn input_is_one_line_so_caret_is_position_zero() {
    // What:    Contract: the matcher is called with ONE content line per call, so `^` means
    //          position zero. A marker only at the start of the (single-line) input matches.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let set = RegexSet::new(&["^(?:(?:PR)|(?:TS))[0-9]:"])
        .expect("build a regex set from the line-start marker pattern");
    assert!(set.is_match(b"PR9: at the start"));
    // What:    Mid-input occurrence (no real line break in one-line input) does not match.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!set.is_match(b"prefix PR9: not at start"));
}

#[test]
fn weak_inner_seed_rule_matches_on_its_short_literal() {
    // What:    The only required literal is the 2-byte "Q~", below the default seed floor; the
    //          fold gates on it (weak inner seed) and runs the full engine on a hit.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let set = RegexSet::new(&["[a-z]{3}Q\\~[a-z]{3}"])
        .expect("build a regex set from the weak-inner-seed pattern");
    assert!(set.is_match(b"junk abcQ~def junk"));
    assert!(!set.is_match(b"abcXdef"));
    assert!(!set.is_match(b"no marker here"));
}

#[test]
fn mixed_ruleset_routes_each_rule_correctly() {
    // What:    A line-start marker, a leading-literal rule, and a weak-inner-seed rule
    //          together.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let set = RegexSet::new(&[
        "^(?:(?:PR)|(?:TS))[0-9]:",
        "AKIA[A-Z2-7]{16}",
        "[a-z]{3}Q\\~[a-z]{3}",
    ])
    .expect("build a regex set from a mixed ruleset");
    assert_eq!(set.len(), 3);
    assert_eq!(set.matches(b"PR1:").collect::<Vec<_>>(), vec![0]);
    assert_eq!(
        set.matches(AKIA_KEY.as_bytes()).collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(set.matches(b"abcQ~def").collect::<Vec<_>>(), vec![2]);
    assert!(!set.is_match(b"unrelated content"));
}

#[test]
fn seedless_rule_above_the_group_state_cap_still_matches() {
    // What:    A literal-free rule whose own search DFA needs more states than one union group
    //          may hold must still be covered by the boolean fast path, so `is_match` agrees
    //          with `matches` and both report the rule. `[0-9].{11}` has exactly that shape: a
    //          digit followed by eleven bytes needs more DFA states than the group state cap
    //          admits (src/group_tests.rs pins that window against the cap itself) while the
    //          rule still builds as its own table engine.
    // Why:     Grouping the literal-free rules into union DFAs is a throughput lever, so
    //          exceeding its state cap may cost an extra pass but must never lose coverage: a
    //          dropped rule is a silent false negative in a deny-rule scanner.
    let pattern = "[0-9].{11}";
    let line: &[u8] = b"id 123456789012 ok";
    let set = RegexSet::new(&[pattern]).expect("the rule builds its own engine");
    assert_eq!(
        set.is_match(line),
        set.matches(line).next().is_some(),
        "is_match must agree with matches for every line",
    );
    assert!(
        set.is_match(line),
        "the boolean fast path must report the rule"
    );
    assert_eq!(set.matches(line).collect::<Vec<usize>>(), vec![0]);
    // What:    The batch boolean kernels read the same union engines, so they must report the
    //          rule on the line that holds it and stay false on the line that does not;
    //          `is_match_batch_concat` sweeps the joined buffer and resolves each line through
    //          the batch `resolve_line` path.
    // Why:     A verdict that disagrees between the per-line and batch APIs is the same
    //          false-negative defect one call site over.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert.deepEqual(set.isMatchBatch(lines), [true, false]);
    // assert.deepEqual(set.isMatchBatchConcat(lines), [true, false]);
    // ```
    let lines: [&[u8]; 2] = [line, b"no digit run here"];
    assert_eq!(
        set.is_match_batch(&lines),
        vec![true, false],
        "batch kernels keep the rule"
    );
    assert_eq!(
        set.is_match_batch_concat(&lines),
        vec![true, false],
        "the concatenated sweep keeps the rule",
    );
}
