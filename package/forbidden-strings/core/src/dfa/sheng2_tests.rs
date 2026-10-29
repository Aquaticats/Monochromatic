// What:  unit tests for the two-byte composed Sheng kernel.
// Why:     This file groups the sheng2 test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("sheng2", () => {
//   // test cases below
// });
// ```

use crate::ast::node::Node;
use crate::ast::smart::concat;
use crate::dfa::table::Dfa;
use crate::dfa::{build_dfa_within, minimize};
use crate::parse::parse;

// What:    Builds the same unanchored search DFA an engine would for `pattern`.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function search_dfa(pattern: string): Dfa {
//   // Rust body below is the implementation.
// }
// ```
fn search_dfa(pattern: &str) -> Dfa {
    let node = concat(vec![Node::Top, parse(pattern).expect("pattern parses")]);
    return minimize(&build_dfa_within(node, 10_000).expect("builds under cap"))
}

// What:    Asserts the two-byte kernel agrees with the per-line scalar oracle on `lines`.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function sheng2_agrees(pattern: string, lines: Uint8Array[]): void {
//   // Rust body below is the implementation.
// }
// ```
fn sheng2_agrees(pattern: &str, lines: &[&[u8]]) {
    let dfa = search_dfa(pattern);
    let oracle: Vec<bool> = lines.iter().map(|line| return dfa.is_match(line)).collect();
    let mut out = vec![false; lines.len()];
    dfa.is_match_batch_sheng2(lines, &mut out);
    assert_eq!(out, oracle, "sheng2 disagrees for {pattern}");
}

#[test]
fn hex_run_even_and_odd_lengths() {
    let lines: &[&[u8]] = &[
        b"deadbeefdeadbeefdeadbeefdeadbeef",
        b"deadbeefdeadbeefdeadbeefdeadbee",
        b"x deadbeefdeadbeefdeadbeefdeadbeef",
        b"xx deadbeefdeadbeefdeadbeefdeadbeef",
        b"not hex",
        b"",
        b"a",
    ];
    sheng2_agrees("[0-9a-f]{32}", lines);
}

#[test]
fn literal_and_class_pattern() {
    let lines: &[&[u8]] = &[
        b"AKIA2345",
        b"AKIA23456",
        b"no secret here at all",
        b"prefix-AKIAZ7Q9",
        b"AKIA",
    ];
    sheng2_agrees("AKIA[A-Z2-7]{4}", lines);
}

#[test]
fn position_dependent_pattern_falls_back_correctly() {
    // What:    `\bcat\b` has context-dependent acceptance, so the kernel must fall back to
    //          scalar and still return correct verdicts.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let lines: &[&[u8]] = &[b"cat", b"category", b"a cat sat", b"scatter", b"cat!"];
    sheng2_agrees(r"\bcat\b", lines);
}

#[test]
fn a_match_ending_at_an_odd_offset_is_accumulated() {
    // What:    The pair table folds BOTH bytes' acceptance per two-byte step. A match that
    //          completes at an ODD offset (e.g. "cat" at offset 3 of "catx") is detected only
    //          by the pair's second-byte term, then lost as the residual drops the empty match
    //          on the next byte; if the pair combiner were AND instead of OR, that match would
    //          vanish. Odd- and even-offset matches must all agree with the scalar oracle.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let lines: &[&[u8]] = &[b"catx", b"xcat", b"cat", b"xcatx", b"dogx", b"acaty"];
    sheng2_agrees("cat", lines);
}

// What:    A self-looping DFA with `num_states` states and `nclasses` classes, accepting
//          nothing: the minimal shape for exercising build_sheng2's state- and class-count
//          eligibility gates at chosen sizes (kernel correctness on real DFAs is covered by
//          the agreement tests above).
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function flat_dfa(num_states: number, nclasses: number): Dfa {
//   // Rust body below is the implementation.
// }
// ```
fn flat_dfa(num_states: u16, nclasses: u32) -> Dfa {
    let nc = nclasses as usize;
    let class_map: Vec<u8> = (0..256usize).map(|b| return (b % nc) as u8).collect();
    return Dfa::from_parts(
        nclasses,
        class_map,
        vec![false; nc],
        vec![false; nc],
        vec![0u16; num_states as usize * nc],
        vec![0u8; num_states as usize],
        0,
        num_states,
    )
}

#[test]
fn build_sheng2_eligibility_bounds_states_classes_and_acceptance() {
    // What:    The composed kernel needs <=64 states, <=16 classes, and position-independent
    //          acceptance. A qualifying pattern builds; the state and class ceilings are
    //          inclusive (64 and 16 build, 65 does not); both must be checked (the guard is an
    //          OR), so a 65-state DFA is rejected regardless of its class count.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(search_dfa("[0-9a-f]{8}").build_sheng2().is_some(), "a small hex run qualifies");
    assert!(flat_dfa(64, 2).build_sheng2().is_some(), "64 states is the inclusive maximum");
    assert!(flat_dfa(10, 16).build_sheng2().is_some(), "16 classes is the inclusive maximum");
    assert!(flat_dfa(65, 2).build_sheng2().is_none(), "65 states is over the limit");
    assert!(flat_dfa(10, 17).build_sheng2().is_none(), "17 classes is over the limit");
}
