//! What:
//!     End-to-end tests exercising the public API across the compilation boundary.
//! Why:
//!      This file is the Rust module that groups the integration implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module integration: see exported functions and types below.
//! ```

use forbidden_regex::{CompileError, RegexSet, compile};

/// What:
///     A valid 20-byte AWS-style key:
///  `AKIA` plus 16 bytes from `[A-Z2-7]`.
/// Why:
///      The program gives this fixed value a name so every caller uses the same setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AKIA_KEY: string = "AKIAABCDEFGHIJKLMNO\x50";
/// ```
const AKIA_KEY: &str = "AKIAABCDEFGHIJKLMNO\x50";

#[test]
fn literal_and_search() {
    let re = compile("AKIA[A-Z2-7]{16}").expect("compile a leading-literal AKIA pattern");
    assert!(re.is_match(b"key=AKIAABCDEFGHIJKLMNO\x50;"));
    assert!(!re.is_match(b"no key here"));
    // What:    Wrong alphabet: '0' and '1' are not in [A-Z2-7].
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!re.is_match(b"AKIA0123456789ABCDEF"));
}

#[test]
fn anchors_and_word_boundary() {
    let start = compile("^abc").expect("compile a start-anchored pattern");
    assert!(start.is_match(b"abcdef"));
    assert!(!start.is_match(b"xabcdef"));

    let end = compile("abc$").expect("compile an end-anchored pattern");
    assert!(end.is_match(b"xxabc"));
    assert!(!end.is_match(b"abcd"));

    let word = compile("\\bcat\\b").expect("compile a word-boundary pattern");
    assert!(word.is_match(b"a cat sat"));
    assert!(!word.is_match(b"category"));
}

#[test]
fn classes_and_shorthands() {
    let re = compile("[a-z]\\d[^0-9]").expect("compile a class-and-shorthand pattern");
    assert!(re.is_match(b"x5!"));
    assert!(!re.is_match(b"x55"));
    let neg = compile("a[^bc]d").expect("compile a negated-class pattern");
    assert!(neg.is_match(b"axd"));
    assert!(!neg.is_match(b"abd"));
}

#[test]
fn repetition_bounds() {
    let re = compile("a{2,4}").expect("compile a bounded-repetition pattern");
    assert!(!re.is_match(b"bab"));
    assert!(re.is_match(b"baab"));
    assert!(re.is_match(b"aaaa"));
    let exact = compile("x{3}").expect("compile an exact-repetition pattern");
    assert!(exact.is_match(b"xxx"));
    assert!(!exact.is_match(b"yxxy"));
}

#[test]
fn unsupported_constructs_rejected() {
    assert!(matches!(compile("a*"), Err(CompileError::Syntax { .. })));
    assert!(matches!(compile("a+"), Err(CompileError::Syntax { .. })));
    assert!(matches!(compile("a{2,}"), Err(CompileError::Syntax { .. })));
    assert!(matches!(compile("(abc)"), Err(CompileError::Syntax { .. })));
    assert!(matches!(compile("\\x41"), Err(CompileError::Syntax { .. })));
    // What:    Multi-atom operand of '&' must be wrapped.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(matches!(compile("ab&cd"), Err(CompileError::Syntax { .. })));
}

#[test]
fn alternation_intersection_complement() {
    // What:    Operands must be single atoms, so multi-letter branches are wrapped.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let alt = compile("(?:(?:cat)|(?:dog))").expect("compile an alternation pattern");
    assert!(alt.is_match(b"a dog"));
    assert!(!alt.is_match(b"a bird"));
    // What:    The unwrapped form is rejected by the grammar.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(matches!(compile("(?:cat|dog)"), Err(CompileError::Syntax { .. })));

    // What:    The canonical set-algebra case, wrapped to the single-atom-operand grammar: an
    //          AWS-style key, excluding the documented all-2s placeholder.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let aws = compile(
        "(?:\\b(?:(?:A3T[A-Z0-9])|(?:AKIA)|(?:ASIA))[A-Z2-7]{16}\\b)&~(AKIA2{16})",
    )
    .expect("compile the AWS set-algebra pattern");
    assert!(aws.is_match(AKIA_KEY.as_bytes()));
    assert!(aws.is_match(format!("prefix {AKIA_KEY} suffix").as_bytes()));
    assert!(!aws.is_match(b"AKIA222222222222222\x32"));
}

#[test]
fn empty_matchable_rejected() {
    assert!(matches!(compile("a?"), Err(CompileError::EmptyMatchable)));
    assert!(matches!(compile("~(abc)"), Err(CompileError::EmptyMatchable)));
}

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
    let bytes = set.to_bytes().expect("serialize the product-rule set to bytes");
    let reloaded = RegexSet::from_bytes(&bytes).expect("deserialize the product-rule set from bytes");
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

#[test]
fn from_ruleset_splits_on_delimiter() {
    let text = "AKIA[A-Z2-7]{16}\n---\nghp_[A-Za-z0-9]{36}\n";
    let set = RegexSet::from_ruleset(text, "---").expect("build a regex set from ruleset text");
    assert_eq!(set.len(), 2);
    assert!(set.is_match(AKIA_KEY.as_bytes()));
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
    assert_eq!(set.matches(AKIA_KEY.as_bytes()).collect::<Vec<_>>(), vec![1]);
    assert_eq!(set.matches(b"abcQ~def").collect::<Vec<_>>(), vec![2]);
    assert!(!set.is_match(b"unrelated content"));
}

#[test]
fn dialect_constructs_have_their_semantics() {
    // What:    `.` is any byte except newline.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dot = compile(".").expect("compile the dot pattern");
    assert!(dot.is_match(b"x"));
    assert!(dot.is_match(b"\t"));
    assert!(!dot.is_match(b"\n"));
    assert!(!dot.is_match(b""));

    // What:    `{n}` is an exact count, not a literal brace.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let three = compile("a{3}").expect("compile an exact-count pattern");
    assert!(three.is_match(b"aaa"));
    // What:    contains a 3-run.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(three.is_match(b"aaaa"));
    assert!(!three.is_match(b"aa"));
    assert!(!three.is_match(b"a{3}"));

    // What:    `{n,m}` is a bounded range.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let range = compile("xa{2,3}").expect("compile a bounded-range pattern");
    assert!(range.is_match(b"xaa"));
    assert!(range.is_match(b"xaaa"));
    assert!(!range.is_match(b"xa"));

    // What:    `?` is zero-or-one of the preceding atom.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let opt = compile("xa?b").expect("compile an optional-atom pattern");
    assert!(opt.is_match(b"xb"));
    assert!(opt.is_match(b"xab"));
    assert!(!opt.is_match(b"xaab"));
}

#[test]
fn single_regex_round_trips_through_bytes() {
    let regex = compile("AKIA[A-Z2-7]{16}").expect("compile a leading-literal AKIA pattern");
    let bytes = regex.to_bytes().expect("serialize the regex to bytes");
    assert!(!bytes.is_empty());
    let reloaded = forbidden_regex::Regex::from_bytes(&bytes).expect("deserialize the regex from bytes");
    assert_eq!(regex.is_match(AKIA_KEY.as_bytes()), reloaded.is_match(AKIA_KEY.as_bytes()));
    assert!(reloaded.is_match(AKIA_KEY.as_bytes()));
    assert!(!reloaded.is_match(b"not a key"));
}

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

    assert!(RegexSet::new::<&str>(&[]).expect("build an empty regex set").is_empty());
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
fn nested_group_repetition_compiles_and_matches() {
    // What:    Reasonable nested group repetition compiles fast and matches correctly.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let triple = compile("(?:ab){3}").expect("compile a repeated-group pattern");
    assert!(triple.is_match(b"ababab"));
    assert!(!triple.is_match(b"abab"));
    let nested = compile("(?:[a-z]{2}){2}").expect("compile a nested-repeated-group pattern");
    assert!(nested.is_match(b"abcd"));
    assert!(!nested.is_match(b"ab1d"));
}

// What:    libFuzzer (roundtrip/differential) found these deeply-nested bounded repetitions
//          (and complement over them) blow up the DFA build's residuals. With the
//          residual-size guard, compile now TERMINATES (rejects or falls back to counting)
//          instead of OOM. Marked #[ignore]: even guarded, the worst case takes seconds (more
//          in a debug build), which would dominate the default suite and per-mutant mutation
//          runs; run on demand with `cargo test -- --ignored`. The engine guard plus the fuzz
//          corpus are the live regression protection.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function pathological_nested_repetition_terminates_without_oom(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
#[ignore = "slow pathological compile; the residual guard is exercised by fuzzing"]
fn pathological_nested_repetition_terminates_without_oom() {
    let _ = compile("(?:(?:(?:(?:[^0-9]a\\|?){3}){2}){3,4}){4,4}");
    let _ = compile("~((?:(?:(?:\\S{4,4}\\d{4,4})a?){4,4}a?){3}a)a");
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
    assert!(dump.len() > 10, "the dump must be the real parsed node, got {dump:?}");
    assert_ne!(dump, "xyzzy");
    assert!(forbidden_regex::debug_seedless("[a-z]{3}abc").is_none(), "an inner literal is a seed");
    assert!(forbidden_regex::debug_seedless("AKIA[A-Z2-7]{16}").is_none(), "a leading literal is a seed");
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
    assert!(count > 2, "a three-rule combined DFA has many states, got {count}");
}
