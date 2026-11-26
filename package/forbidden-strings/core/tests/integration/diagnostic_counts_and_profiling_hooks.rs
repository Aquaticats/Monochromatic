//! What:
//!     End-to-end tests of what a built set reports about itself:
//!  the structural counts,
//!  the
//!  bench's profiling hooks,
//!  and the two diagnostics outside the set.
//! Why:
//!      This file is the Rust module that groups the diagnostic tests,
//!  so the test binary
//!          gives them one namespace and the shared key stays in `tests/integration/main.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe("diagnostic counts and profiling hooks", ...) with the cases below.
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
///     Imports the compiled-ruleset type whose counts and hooks these tests read.
/// Why:
///      The code below builds a `RegexSet` and calls its accessors directly;
///  importing the
///          type keeps each test focused on the reported value instead of the full path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegexSet } from "forbidden_regex";
/// ```
use forbidden_regex::RegexSet;
#[test]
fn diagnostic_accessors_report_structure() {
    // What:    Two leading-literal rules: both gated and anchored, none seedless.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let anchored = RegexSet::new(&["AKIA[A-Z2-7]{16}", "ghp_[A-Za-z0-9]{36}"])
        .expect("build a regex set with two anchored rules");
    assert_eq!(anchored.len(), 2);
    assert!(!anchored.is_empty());
    assert_eq!(anchored.anchored_count(), 2);
    assert_eq!(anchored.seedless_count(), 0);
    assert_eq!(anchored.line_start_count(), 0);
    assert_eq!(anchored.seedless_group_count(), 0);
    assert_eq!(anchored.seedless_union_size(), 0);

    // What:    Two class-only rules: no seed, no leading literal, so truly seedless and
    //          grouped.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let seedless = RegexSet::new(&["[a-z]{20}", "[0-9]{18}"])
        .expect("build a regex set with two seedless rules");
    assert_eq!(seedless.seedless_count(), 2);
    assert!(seedless.seedless_group_count() >= 1);
    assert!(seedless.seedless_union_size() >= 2);

    // What:    One line-start marker rule.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let marker = RegexSet::new(&["^(?:(?:PR)|(?:TS))[0-9]:"])
        .expect("build a regex set from the line-start marker pattern");
    assert_eq!(marker.line_start_count(), 1);

    assert!(
        RegexSet::new::<&str>(&[])
            .expect("build an empty regex set")
            .is_empty()
    );
}

#[test]
fn profiling_hooks_track_their_components() {
    let set = RegexSet::new(&["AKIA[A-Z2-7]{16}", "[a-z]{20}"])
        .expect("build a regex set with an anchored and a seedless rule");
    let key = AKIA_KEY.as_bytes();
    let twenty = b"abcdefghijklmnopqrst";

    // What:    Gate / prefilter / candidate hooks see the seeded AKIA rule, not a bare line.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(set.gate_only_is_match(key));
    assert!(!set.gate_only_is_match(b"nothing seeded here"));
    assert!(set.prefilter_only_is_match(key));
    assert!(!set.prefilter_only_is_match(b"nothing seeded here"));
    // What:    candidates_only does the enumeration work but the stubbed predicate makes it
    //          always return false (a timing harness), so it must report false either way.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!set.candidates_only_is_match(key));
    assert!(!set.candidates_only_is_match(b"nothing seeded here"));
    assert!(set.gate_anchored_only_is_match(key));

    // What:    Seedless / counting-union hooks see the class-only rule, not the seeded one.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(set.seedless_only_is_match(twenty));
    assert!(!set.seedless_only_is_match(key));
    assert!(set.csa_only_is_match(twenty));
    assert!(!set.csa_only_is_match(b"short"));

    // What:    The anchored-only gate hook sees AKIA (anchored) but not a seedless line.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!set.gate_anchored_only_is_match(b"nothing seeded here"));
    assert!(!set.gate_anchored_only_is_match(twenty));
}

#[test]
fn debug_seedless_reports_only_literal_free_patterns() {
    // What:    The diagnostic returns the node dump for a pattern with no usable seed (no
    //          leading literal AND no required inner literal), and None once either kind of
    //          seed exists. A class repetition is seedless; a leading or inner literal makes
    //          it seeded.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dump = forbidden_regex::debug_seedless("[a-z]{5}").expect("a class repetition is seedless");
    assert!(
        dump.len() > 10,
        "the dump must be the real parsed node, got {dump:?}"
    );
    assert_ne!(dump, "xyzzy");
    assert!(
        forbidden_regex::debug_seedless("[a-z]{3}abc").is_none(),
        "an inner literal is a seed"
    );
    assert!(
        forbidden_regex::debug_seedless("AKIA[A-Z2-7]{16}").is_none(),
        "a leading literal is a seed"
    );
}

#[test]
fn try_combined_dfa_reports_a_real_state_count() {
    // What:    The diagnostic builds one combined search DFA over all patterns and returns its
    //          state count; that must be the genuine count (more than a couple of states), not
    //          a constant.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let count = forbidden_regex::try_combined_dfa(&["abc", "def", "[0-9]{3}"]).expect("builds");
    assert!(
        count > 2,
        "a three-rule combined DFA has many states, got {count}"
    );
}
