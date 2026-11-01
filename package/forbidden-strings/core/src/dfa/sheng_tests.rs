// What:  unit tests for the Sheng in-register transition kernel.
// Why:     This file groups the sheng test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("sheng", () => {
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

// What:    Asserts the Sheng kernel agrees with the per-line scalar oracle on `lines`.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function sheng_agrees(pattern: string, lines: Uint8Array[]): void {
//   // Rust body below is the implementation.
// }
// ```
fn sheng_agrees(pattern: &str, lines: &[&[u8]]) {
    let dfa = search_dfa(pattern);
    let oracle: Vec<bool> = lines.iter().map(|line| return dfa.is_match(line)).collect();
    let mut out = vec![false; lines.len()];
    dfa.is_match_batch_sheng(lines, &mut out);
    assert_eq!(out, oracle, "sheng disagrees for {pattern}");
}

#[test]
fn literal_and_class_pattern() {
    let lines: &[&[u8]] = &[
        b"AKIA2345",
        b"no secret here",
        b"prefix-AKIAZ7Q9-suffix",
        b"",
        b"AKIA",
        b"a",
        b"AKIA67QZ and a long tail of bytes that keeps going onward",
    ];
    sheng_agrees("AKIA[A-Z2-7]{4}", lines);
}

#[test]
fn word_boundary_anchors() {
    let lines: &[&[u8]] = &[b"cat", b"a cat sat", b"category", b"scatter", b"cat!", b"dog", b" cat "];
    sheng_agrees(r"\bcat\b", lines);
}

#[test]
fn line_start_anchor() {
    let lines: &[&[u8]] = &[b"#deny this", b"deny that", b"#deny", b"nope", b"#den"];
    sheng_agrees("^#deny", lines);
}

#[test]
fn hex_run_full_scan() {
    // What:    33 states (> 16, <= 64): exercises the wide-permute path, not just the pshufb
    //          range.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let lines: &[&[u8]] = &[
        b"deadbeefdeadbeefdeadbeefdeadbeef",
        b"deadbeefdeadbeefdeadbeefdeadbee",
        b"xx deadbeefdeadbeefdeadbeefdeadbeef xx",
        b"not hex at all",
    ];
    sheng_agrees("[0-9a-f]{32}", lines);
}

// What:    A self-looping DFA with `num_states` states and `nclasses` classes, accepting
//          nothing: the minimal shape for exercising build_sheng's state-count eligibility
//          gate at a chosen size (the kernel's correctness on real DFAs is covered by the
//          agreement tests above).
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
fn build_sheng_eligibility_is_bounded_at_64_states() {
    // What:    The permute table is 64 wide, so build_sheng yields tables only up to 64
    //          states. A normal small DFA and an exactly-64-state DFA both qualify (inclusive
    //          ceiling); a 65-state DFA does not. This pins the bound against off-by-one and
    //          always-None mutants.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(search_dfa("AKIA[A-Z2-7]{4}").build_sheng().is_some(), "a small DFA is eligible");
    assert!(flat_dfa(64, 1).build_sheng().is_some(), "64 states is the inclusive maximum");
    assert!(flat_dfa(65, 1).build_sheng().is_none(), "65 states is over the permute width");
}
