// What:  unit tests for the per-pattern Engine: the prefilter gate in front of the
//        back-end, the prefilter-skipping match the set gate uses, seed reporting, and
//        the first-byte set.
// Why:     This file groups the engine test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("engine", () => {
//   // test cases below
// });
// ```

use super::{Engine, EngineKind};
use crate::dfa::{build_dfa_within, minimize};
use crate::parse::parse;

// What:    A Table engine over a pattern's anchored DFA, carrying the given seeds.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function table_engine(pattern: string, seeds: Uint8Array[]): Engine {
//   // Rust body below is the implementation.
// }
// ```
fn table_engine(pattern: &str, seeds: &[&[u8]]) -> Engine {
    let dfa = minimize(&build_dfa_within(parse(pattern).expect("parses"), 10_000).expect("builds"));
    let owned_seeds = seeds.iter().map(|s| return s.to_vec()).collect();
    return Engine::new(EngineKind::Table(dfa), owned_seeds)
}

#[test]
fn is_match_rejects_when_the_seed_is_absent() {
    // What:    The back-end would match "abc", but the seed "zzz" is not present, so the
    //          prefilter short-circuits to a non-match.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let engine = table_engine("abc", &[b"zzz"]);
    assert!(!engine.is_match(b"abc"));
}

#[test]
fn matches_only_skips_the_prefilter() {
    // What:    Same engine: matches_only ignores the prefilter and runs the back-end, which
    //          matches the "abc" prefix.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let engine = table_engine("abc", &[b"zzz"]);
    assert!(engine.matches_only(b"abc"));
    assert!(!engine.matches_only(b"xbc"));
}

#[test]
fn is_match_runs_the_back_end_when_the_seed_is_present() {
    let engine = table_engine("abc", &[b"abc"]);
    assert!(engine.is_match(b"abc"));
    assert!(!engine.is_match(b"abx"));
}

#[test]
fn empty_seeds_make_the_prefilter_always_allow() {
    let engine = table_engine("abc", &[]);
    assert_eq!(engine.seeds(), None);
    assert!(engine.is_match(b"abc"));
    assert!(!engine.is_match(b"xyz"));
}

#[test]
fn seeds_are_reported_when_present() {
    let engine = table_engine("abc", &[b"abc"]);
    assert_eq!(engine.seeds(), Some(vec![b"abc".to_vec()]));
}

#[test]
fn validate_and_prepare_keep_a_built_engine_working() {
    let mut engine = table_engine("AKIA[A-Z2-7]{4}", &[b"AKIA"]);
    assert!(engine.validate().is_ok());
    // What:    prepare rebuilds the prefilter searchers; matching still works afterwards.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    engine.prepare();
    assert!(engine.is_match(b"AKIAB2C7"));
}

#[test]
fn prepare_rebuilds_the_prefilter_dropped_by_decode() {
    // What:    The prefilter is `#[serde(skip)]`, so a decoded engine starts with none and
    //          must be rebuilt by `prepare`. A match verdict cannot see this (an empty
    //          prefilter just allows every line, still sound), so pin it on the searcher
    //          count: zero after decode, one per seed after prepare. This proves `prepare` is
    //          not a no-op.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let engine = table_engine("AKIA[A-Z2-7]{4}", &[b"AKIA"]);
    let bytes = bincode::serialize(&engine).expect("engine serializes");
    let mut decoded: Engine = bincode::deserialize(&bytes).expect("engine deserializes");
    assert_eq!(decoded.prefilter.len(), 0, "a decoded engine has no prefilter until prepared");
    decoded.prepare();
    assert_eq!(decoded.prefilter.len(), 1, "prepare rebuilds one searcher per seed");
}

#[test]
fn validate_rejects_a_corrupt_back_end() {
    // What:    Engine::validate must delegate into the back-end so a hostile decoded table is
    //          caught before it runs; corrupt the inner DFA and confirm rejection.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut engine = table_engine("abc", &[]);
    if let EngineKind::Table(dfa) = &mut engine.kind {
        // What:    out-of-range start state.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        dfa.start = dfa.num_states + 1;
    }
    assert!(engine.validate().is_err());
}

#[test]
fn mark_first_bytes_delegates_to_the_table() {
    let engine = table_engine("abc", &[]);
    let mut set = crate::charset::ByteSet::empty();
    engine.mark_first_bytes(&mut set);
    assert!(set.contains(b'a'));
    assert!(!set.contains(b'b'));
}

#[test]
fn table_dfa_is_some_only_for_a_table_back_end() {
    // What:    The batch kernels run directly on a `Dfa`, so the routing must hand back the
    //          table for a Table engine and `None` otherwise; a Table engine here must yield
    //          its DFA.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let engine = table_engine("abc", &[]);
    let dfa = engine.table_dfa().expect("a Table engine exposes its DFA");
    assert!(dfa.is_match(b"abc"), "the returned DFA is the real matcher");
}
