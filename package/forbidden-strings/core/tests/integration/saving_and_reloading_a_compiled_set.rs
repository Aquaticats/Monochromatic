//! What:
//!     End-to-end tests of saving a compiled set to bytes and loading one back:
//!  the round
//!  trips that must reproduce the same verdicts,
//!  and the truncated or hostile bytes a
//!  decode must reject without panicking.
//! Why:
//!      This file is the Rust module that groups the serialization tests,
//!  so the test binary
//!          gives them one namespace and the shared key stays in `tests/integration/main.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe("saving and reloading a compiled set", ...) with the cases below.
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
///     Imports the compiled-ruleset type these tests encode and decode.
/// Why:
///      The code below calls `RegexSet::new`,
///  `to_bytes` and `from_bytes` directly;
///  importing
///          the type keeps each test focused on the codec instead of the full path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegexSet } from "forbidden_regex";
/// ```
use forbidden_regex::RegexSet;

/// What:
///     Imports the one-pattern compiler for the single-pattern round trip.
/// Why:
///      That test compiles a pattern before encoding its engine;
///  importing the function keeps
///          it focused on the round trip instead of the full path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { compile } from "forbidden_regex";
/// ```
use forbidden_regex::compile;
#[test]
fn regexset_matches_and_roundtrip() {
    let set = RegexSet::new(&["AKIA[A-Z2-7]{16}", "ghp_[A-Za-z0-9]{36}"])
        .expect("build a regex set with two anchored rules");
    let ghp = format!("token ghp_{} end", "a".repeat(36));
    assert!(set.is_match(format!("... {AKIA_KEY} ...").as_bytes()));
    let hits: Vec<usize> = set.matches(ghp.as_bytes()).collect();
    assert_eq!(hits, vec![1]);

    let bytes = set.to_bytes().expect("serialize the regex set to bytes");
    let reloaded = RegexSet::from_bytes(&bytes).expect("deserialize the regex set from bytes");
    assert!(reloaded.is_match(format!("x {AKIA_KEY} x").as_bytes()));
    assert!(!reloaded.is_match(b"nothing to see"));
}

#[test]
fn product_rule_survives_roundtrip() {
    // What:    A set-algebra rule uses the product engine; serializing and reloading must run
    //          its decode validation and still decide keys exactly.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let set = RegexSet::new(&[
        "(?:\\b(?:(?:A3T[A-Z0-9])|(?:AKIA)|(?:ASIA))[A-Z2-7]{16}\\b)&~(AKIA2{16})",
        "ghp_[A-Za-z0-9]{36}",
    ])
    .expect("build a regex set for the product-rule pattern");
    let bytes = set
        .to_bytes()
        .expect("serialize the product-rule set to bytes");
    let reloaded =
        RegexSet::from_bytes(&bytes).expect("deserialize the product-rule set from bytes");
    assert!(reloaded.is_match(format!("key {AKIA_KEY} end").as_bytes()));
    // What:    The all-2s placeholder is vetoed by the complement, even after a round-trip.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!reloaded.is_match(b"AKIA222222222222222\x32"));
    let hits: Vec<usize> = reloaded.matches(AKIA_KEY.as_bytes()).collect();
    assert_eq!(hits, vec![0]);
}

// What:    A serialized RegexSet whose parallel vectors disagree (a `seedless_id` past an
//          empty `rules`). libFuzzer minimized this from `fuzz_from_bytes`; before
//          `validate_structure` it panicked with an out-of-bounds index at match time. Now
//          `from_bytes` must REJECT it (return Err), never panic, because the decoded bytes
//          are attacker-influenced. Embedded inline (not a `.bin` fixture, which the root
//          .gitignore excludes).
// Why:     The program gives this fixed value a name so every caller uses the same setting.
//
// In TS you'd write (pseudocode):
// ```ts
// const STRUCTURAL_CRASH: Uint8Array = &[;
// ```
const STRUCTURAL_CRASH: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0,
];

#[test]
fn from_bytes_rejects_inconsistent_structure() {
    assert!(RegexSet::from_bytes(STRUCTURAL_CRASH).is_err());
}

#[test]
fn from_bytes_rejects_garbage_without_panic() {
    // What:    Empty, truncated, and arbitrary bytes must all decode-fail cleanly, never
    //          panic.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(RegexSet::from_bytes(b"").is_err());
    assert!(forbidden_regex::Regex::from_bytes(b"").is_err());
    assert!(RegexSet::from_bytes(b"not a serialized matcher").is_err());
    for len in 0..STRUCTURAL_CRASH.len() {
        // What:    Truncations of the crash input must also reject, not panic.
        // Why:     The test uses this setup or assertion to pin the behavior named by the test
        //          function.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let _ = RegexSet::from_bytes(&STRUCTURAL_CRASH[..len]);
    }
}

#[test]
fn single_regex_round_trips_through_bytes() {
    let regex = compile("AKIA[A-Z2-7]{16}").expect("compile a leading-literal AKIA pattern");
    let bytes = regex.to_bytes().expect("serialize the regex to bytes");
    assert!(!bytes.is_empty());
    let reloaded =
        forbidden_regex::Regex::from_bytes(&bytes).expect("deserialize the regex from bytes");
    assert_eq!(
        regex.is_match(AKIA_KEY.as_bytes()),
        reloaded.is_match(AKIA_KEY.as_bytes())
    );
    assert!(reloaded.is_match(AKIA_KEY.as_bytes()));
    assert!(!reloaded.is_match(b"not a key"));
}

#[test]
fn from_bytes_roundtrips_a_built_set() {
    let set = RegexSet::new(&["AKIA[A-Z2-7]{16}", "ghp_[A-Za-z0-9]{36}"])
        .expect("build a regex set with two anchored rules");
    let reloaded = RegexSet::from_bytes(&set.to_bytes().expect("serialize the regex set to bytes"))
        .expect("deserialize the regex set from bytes");
    assert!(reloaded.is_match(AKIA_KEY.as_bytes()));
    assert!(!reloaded.is_match(b"nothing here"));
}
