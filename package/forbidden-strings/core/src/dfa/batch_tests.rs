// What:  unit tests for the batched multi-line DFA kernels.
// Why:     This file groups the batch test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("batch", () => {
//   // test cases below
// });
// ```

use super::LANES;
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

// What:    Asserts every kernel agrees with the per-line scalar oracle on `lines`.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function kernels_agree(pattern: string, lines: Uint8Array[]): void {
//   // Rust body below is the implementation.
// }
// ```
fn kernels_agree(pattern: &str, lines: &[&[u8]]) {
    let dfa = search_dfa(pattern);
    let oracle: Vec<bool> = lines.iter().map(|line| return dfa.is_match(line)).collect();

    let mut scalar = vec![false; lines.len()];
    dfa.is_match_batch_scalar(lines, &mut scalar);
    assert_eq!(scalar, oracle, "scalar batch disagrees for {pattern}");

    let mut interleaved = vec![false; lines.len()];
    dfa.is_match_batch_interleaved(lines, &mut interleaved);
    assert_eq!(interleaved, oracle, "interleaved batch disagrees for {pattern}");
}

#[test]
fn literal_and_class_pattern_across_varied_lines() {
    // What:    A leading-literal rule with a bounded class tail: matches as a substring, dies
    //          fast on most lines (exercises the per-lane dead early-exit divergence).
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let lines: &[&[u8]] = &[
        b"AKIA2345",
        b"no secret here",
        b"prefix-AKIAZ7Q9-suffix",
        b"",
        b"AKIA",
        b"AKIA!!!!",
        b"a",
        b"AKIA67QZ tail bytes that keep going for a while",
        b"another miss",
        b"xxxxxxxxxxxxxxxxxxxxAKIA2222",
        b"short",
        b"AKIAAAAA",
        b"zzz",
    ];
    kernels_agree("AKIA[A-Z2-7]{4}", lines);
}

#[test]
fn word_boundary_and_anchors_at_chunk_and_remainder_sizes() {
    // What:    `\b` acceptance can fire at end of input and mid-line transitions; run both a
    //          chunk-aligned count (2 * LANES) and a ragged count so both code paths execute.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let base: Vec<&[u8]> = vec![
        b"cat",
        b"a cat sat",
        b"category",
        b"the cat.",
        b"scatter",
        b"cat!",
        b"dog",
        b" cat ",
        b"concat",
        b"cat\tcat",
    ];
    let aligned: Vec<&[u8]> = base.iter().cycle().take(LANES * 2).copied().collect();
    kernels_agree(r"\bcat\b", &aligned);
    let ragged: Vec<&[u8]> = base.iter().cycle().take(LANES * 2 + 3).copied().collect();
    kernels_agree(r"\bcat\b", &ragged);
}

#[test]
fn line_start_and_line_end_anchors() {
    // What:    `^`/`$` resolve against the one-line input's position zero and end of input.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let lines: &[&[u8]] = &[
        b"#deny this",
        b"deny that",
        b"  #deny indented",
        b"trailing deny",
        b"#deny",
        b"nope",
        b"#den",
        b"#deny extra",
        b"x",
    ];
    kernels_agree("^#deny", lines);
}

#[test]
fn empty_line_set_is_a_noop() {
    // What:    No lines: every kernel must leave the (empty) output untouched without panic.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    kernels_agree("AKIA[A-Z2-7]{4}", &[]);
}

#[test]
fn tight_kernel_matches_oracle_on_equal_length_lines() {
    // What:    The branchless tight kernel assumes every line shares one length (exact
    //          bucket); on such a set its accumulated verdict must equal the scalar oracle.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dfa = search_dfa("AKIA[A-Z2-7]{4}");
    let lines: &[&[u8]] = &[
        b"AKIA2345", b"nomatch1", b"AKIAZ7Q9", b"xxxxxxxx", b"AKIA0000",
        b"abcdefgh", b"AKIA6QZ7", b"zzzzzzzz", b"AKIA9999", b"plain---",
    ];
    let oracle: Vec<bool> = lines.iter().map(|line| return dfa.is_match(line)).collect();
    let mut out = vec![false; lines.len()];
    dfa.is_match_batch_tight_w::<8>(lines, &mut out);
    assert_eq!(out, oracle);
}

#[test]
fn end_of_input_acceptance_fires_on_a_line_end_anchor() {
    // What:    A `$`-anchored match accepts only at the end-of-input boundary, whose mask bit
    //          differs from a mid-line accept, so the kernels' end-of-input check must use
    //          that exact bit. The tight kernel needs equal-length lines and at least a full
    //          lane chunk to run its branchless body (not the scalar remainder), so a `cat$`
    //          match at a line's end exercises the end-of-input bit directly.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dfa = search_dfa("cat$");
    let lines: &[&[u8]] = &[b"cat", b"dog", b"cat", b"act", b"cat", b"tac", b"cat", b"xyz", b"cat"];
    let oracle: Vec<bool> = lines.iter().map(|line| return dfa.is_match(line)).collect();
    let mut out = vec![false; lines.len()];
    dfa.is_match_batch_tight_w::<8>(lines, &mut out);
    assert_eq!(out, oracle, "the end-of-input check must use the line-end accept bit");
}

#[test]
fn interleaved_width_hook_fills_every_lane() {
    // What:    The benchmark width hook must actually run the kernel and fill the output, not
    //          no-op: its verdicts must equal the scalar oracle across a ragged line set.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dfa = search_dfa("AKIA[A-Z2-7]{4}");
    let lines: &[&[u8]] = &[b"AKIA2345", b"miss", b"x AKIAZ7Q9", b"", b"AKIA9999", b"nope", b"AKIA0000"];
    let oracle: Vec<bool> = lines.iter().map(|line| return dfa.is_match(line)).collect();
    let mut out = vec![false; lines.len()];
    dfa.is_match_batch_interleaved_w::<16>(lines, &mut out);
    assert_eq!(out, oracle, "the width hook must fill every verdict");
}

#[test]
fn tight_kernel_accumulates_a_match_that_ends_before_the_line() {
    // What:    When a match completes mid-line, the residual drops the empty match on the next
    //          byte, so the end-of-input check no longer sees it: only the per-position
    //          accumulation catches it. Equal-length lines whose `cat` match ends at offset 3
    //          (before the trailing byte) must still be reported, which fails if that
    //          accumulation is dropped.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dfa = search_dfa("cat");
    let lines: &[&[u8]] = &[b"catx", b"dogx", b"caty", b"zzzz", b"acat", b"catz"];
    let oracle: Vec<bool> = lines.iter().map(|line| return dfa.is_match(line)).collect();
    let mut out = vec![false; lines.len()];
    dfa.is_match_batch_tight_w::<4>(lines, &mut out);
    assert_eq!(out, oracle, "a mid-line match must be accumulated, not only the end boundary");
}
