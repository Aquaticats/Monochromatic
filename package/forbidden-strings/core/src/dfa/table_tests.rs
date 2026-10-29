// What:  unit tests for the compiled DFA table: the acceptance-bit encoding, the match
//        loop on a built automaton, the first-byte set used by the line-start reject,
//        decode validation, and that minimization preserves matching.
// Why:     This file groups the table test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("table", () => {
//   // test cases below
// });
// ```

use super::{Dfa, accept_bit};
use crate::charset::ByteSet;
use crate::dfa::{build_dfa_within, minimize};
use crate::parse::parse;

// What:    Builds a minimized anchored DFA for a pattern (no Sigma* prefix, matches a prefix).
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function anchored_dfa(pattern: string): Dfa {
//   // Rust body below is the implementation.
// }
// ```
fn anchored_dfa(pattern: &str) -> Dfa {
    let node = parse(pattern).expect("test pattern parses");
    return minimize(&build_dfa_within(node, 10_000).expect("builds under cap"))
}

#[test]
fn accept_bit_encodes_four_distinct_contexts() {
    assert_eq!(accept_bit(false, false), 1);
    assert_eq!(accept_bit(true, false), 2);
    assert_eq!(accept_bit(false, true), 4);
    assert_eq!(accept_bit(true, true), 8);
}

#[test]
fn anchored_dfa_matches_a_prefix_only() {
    let dfa = anchored_dfa("abc");
    assert!(dfa.is_match(b"abc"));
    // What:    the prefix "abc" matches.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(dfa.is_match(b"abcd"));
    // What:    incomplete.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(!dfa.is_match(b"ab"));
    // What:    anchored: nothing matches at offset 0.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(!dfa.is_match(b"xabc"));
    assert!(!dfa.is_match(b""));
}

#[test]
fn class_repetition_dfa_counts_correctly() {
    let dfa = anchored_dfa("[a-z]{3}");
    assert!(dfa.is_match(b"abc"));
    // What:    first three match.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(dfa.is_match(b"abcd"));
    assert!(!dfa.is_match(b"ab"));
    assert!(!dfa.is_match(b"a1c"));
}

#[test]
fn mark_first_bytes_reports_only_viable_starts() {
    let dfa = anchored_dfa("abc");
    let mut set = ByteSet::empty();
    dfa.mark_first_bytes(&mut set);
    assert!(set.contains(b'a'));
    assert!(!set.contains(b'b'));
    assert!(!set.contains(b'x'));
}

#[test]
fn validate_accepts_a_built_dfa() {
    let dfa = anchored_dfa("AKIA[A-Z2-7]{4}");
    assert!(dfa.validate().is_ok());
}

#[test]
fn validate_rejects_each_corruption() {
    // What:    A decoded DFA runs against attacker-influenced input, so validate must reject
    //          any out-of-range or inconsistent field. Corrupt each field of a good DFA in
    //          turn.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let good = anchored_dfa("abc");

    let mut nclasses_zero = good.clone();
    nclasses_zero.nclasses = 0;
    assert!(nclasses_zero.validate().is_err());

    let mut nclasses_big = good.clone();
    nclasses_big.nclasses = 257;
    assert!(nclasses_big.validate().is_err());

    let mut short_class_map = good.clone();
    short_class_map.class_map.truncate(255);
    assert!(short_class_map.validate().is_err());

    let mut bad_class_id = good.clone();
    // What:    class id == nclasses is out of range.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    bad_class_id.class_map[0] = good.nclasses as u8;
    assert!(bad_class_id.validate().is_err());

    let mut short_flags = good.clone();
    short_flags.class_word.pop();
    assert!(short_flags.validate().is_err());

    let mut start_oob = good.clone();
    start_oob.start = good.num_states;
    assert!(start_oob.validate().is_err());

    let mut accept_len = good.clone();
    accept_len.accept.push(0);
    assert!(accept_len.validate().is_err());

    let mut trans_len = good.clone();
    trans_len.trans.push(0);
    assert!(trans_len.validate().is_err());

    let mut trans_target_oob = good.clone();
    trans_target_oob.trans[0] = good.num_states;
    assert!(trans_target_oob.validate().is_err());
}

#[test]
fn validate_rejects_a_forged_dead_state() {
    // What:    A hostile blob could name an ACCEPTING state as the dead sink so the match loop
    //          early-exits false and misses a match; validate must reject that.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let good = anchored_dfa("abc");
    let mut forged = good.clone();
    // What:    Point `dead` at the start state, which is not a non-accepting self-looping
    //          sink.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    forged.dead = good.start;
    assert!(forged.validate().is_err());

    let mut out_of_range = good.clone();
    out_of_range.dead = good.num_states + 1;
    assert!(out_of_range.validate().is_err());
}

// What:    A consistent self-looping single-state DFA over `nclasses` classes (all bytes class
//          0), the minimal well-formed automaton for exercising the decode-validation bounds
//          checks.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function uniform_dfa(nclasses: number): Dfa {
//   // Rust body below is the implementation.
// }
// ```
fn uniform_dfa(nclasses: u32) -> Dfa {
    let nc = nclasses as usize;
    return Dfa::from_parts(
        nclasses,
        // What:    every byte maps to class 0.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        vec![0u8; 256],
        // What:    per-class word flags.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        vec![false; nc],
        // What:    per-class newline flags.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        vec![false; nc],
        // What:    one state, self-looping on every class.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        vec![0u16; nc],
        // What:    the single state accepts nothing.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        vec![0u8; 1],
        // What:    start.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        0,
        // What:    num_states.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        1,
    )
}

#[test]
fn mark_first_bytes_reads_the_start_states_row() {
    // What:    A hand-built DFA whose start state is NOT state 0: byte `a` (class 0) steps to
    //          a live state, every other byte (class 1) steps to the dead sink.
    //          `mark_first_bytes` must index the START state's transition row (`start *
    //          nclasses + class`); a wrong index (e.g. `start / nclasses`) reads a different
    //          row and reports the wrong first-byte set.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut class_map = vec![1u8; 256];
    class_map[b'a' as usize] = 0;
    let dfa = Dfa::from_parts(
        2,
        class_map,
        vec![false; 2],
        vec![false; 2],
        // What:    state0 (dead sink) self-loops; state1 is live; state2 (start): a->state1,
        //          else->dead.
        // Why:     The test uses this setup or assertion to pin the behavior named by the test
        //          function.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        vec![0, 0, 0, 0, 1, 0],
        vec![0u8; 3],
        // What:    start = state 2.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        2,
        // What:    num_states.
        // Why:     The nearby assertion or value needs this note so the test records the
        //          exact behavior being pinned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same assertion or value, with the important expectation named above.
        // ```
        3,
    );
    let mut set = ByteSet::empty();
    dfa.mark_first_bytes(&mut set);
    assert!(set.contains(b'a'), "byte a steps off the dead sink from the start row");
    assert!(!set.contains(b'b'), "byte b steps straight to the dead sink");
}

#[test]
fn validate_class_count_bounds_are_inclusive_of_256_and_exclusive_above() {
    // What:    256 classes is the inclusive maximum (a full byte alphabet), so a consistent
    //          256-class DFA validates; 300 classes is over the limit and must be rejected
    //          even when every other field is internally consistent (so only the nclasses
    //          range check can catch it).
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(uniform_dfa(256).validate().is_ok(), "256 classes is the inclusive maximum");
    assert!(uniform_dfa(300).validate().is_err(), "more than 256 classes is out of range");
    assert!(uniform_dfa(1).validate().is_ok());
}

#[test]
fn a_built_dfa_with_a_real_dead_sink_validates_and_the_sink_is_well_formed() {
    // What:    An anchored literal dies on the first non-matching byte, so its minimized DFA
    //          has a genuine dead sink (`dead != num_states`). validate must accept it (its
    //          dead-sink self-loop check reads `trans[dead * nclasses + class]`), and the sink
    //          `find_dead` located must really be non-accepting and fully self-looping.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dfa = anchored_dfa("abc");
    let dead = dfa.dead as usize;
    let nc = dfa.nclasses as usize;
    assert_ne!(dfa.dead, dfa.num_states, "an anchored literal must have a real dead sink");
    assert!(dfa.validate().is_ok(), "a DFA with a sound dead sink must validate");
    assert_eq!(dfa.accept[dead], 0, "the dead sink must be non-accepting");
    for class in 0..nc {
        assert_eq!(dfa.trans[dead * nc + class] as usize, dead, "the dead sink must self-loop");
    }
}

#[test]
fn minimization_preserves_matching() {
    // What:    Multi-byte alternation branches must each be wrapped as a single atom.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let raw = build_dfa_within(parse("(?:(?:ab)|(?:ac))d").expect("parses"), 10_000).expect("builds");
    let min = minimize(&raw);
    for input in [&b"abd"[..], b"acd", b"abc", b"ad", b"abdx"] {
        assert_eq!(raw.is_match(input), min.is_match(input), "input {input:?}");
    }
    // What:    The minimized automaton is no larger than the raw one.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(min.num_states <= raw.num_states);
}
