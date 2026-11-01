//! What:
//!     Eager determinization:
//!  turn a node into a `Dfa` by enumerating derivative states.
//! Why:
//!      This file is the Rust module that groups the build implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module build: see exported functions and types below.
//! ```

/// What:
///     Imports the hash map used to intern states.
/// Why:
///      The code below uses `HashMap` directly;
///  importing from `std/collections` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { HashMap } from "std/collections";
/// ```
use std::collections::HashMap;

/// What:
///     Imports the node algebra being determinized.
/// Why:
///      The code below uses `Node` directly;
///  importing from `crate/ast/node` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Node } from "crate/ast/node";
/// ```
use crate::ast::node::Node;

/// What:
///     Imports the boundary context driving derivatives and acceptance.
/// Why:
///      The code below uses `Ctx` directly;
///  importing from `crate/context` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Ctx } from "crate/context";
/// ```
use crate::context::Ctx;

/// What:
///     Imports the byte derivative.
/// Why:
///      The code below uses `derivative` directly;
///  importing from `crate/derivative` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { derivative } from "crate/derivative";
/// ```
use crate::derivative::derivative;

/// What:
///     Imports the error type for the state cap.
/// Why:
///      The code below uses `CompileError` directly;
///  importing from `crate/error` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CompileError } from "crate/error";
/// ```
use crate::error::CompileError;

/// What:
///     Imports nullability for acceptance masks.
/// Why:
///      The code below uses `nullable` directly;
///  importing from `crate/nullable` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { nullable } from "crate/nullable";
/// ```
use crate::nullable::nullable;

/// What:
///     Imports the byte-class computation.
/// Why:
///      The code below uses `compute_classes` directly;
///  importing from `crate/dfa/classes`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { compute_classes } from "crate/dfa/classes";
/// ```
use crate::dfa::classes::compute_classes;

/// What:
///     Imports the table type and its acceptance-bit helper.
/// Why:
///      The code below uses `Dfa`,
///  `accept_bit` directly;
///  importing from `crate/dfa/table`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Dfa, accept_bit } from "crate/dfa/table";
/// ```
use crate::dfa::table::{Dfa, accept_bit};

/// A determinization state:
///  a residual node plus its incoming boundary bits.
///
/// What:
///  the residual regex together with whether the previous byte was a
/// newline (`at_line_start`) and a word byte (`prev_word`).
///  Why:
///  those two bits
/// are the parts of the boundary context that a residual cannot recover on its
/// own,
///  so they must be part of the state's identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StateKey = {
///   // fields documented in Rust above
/// };
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
struct StateKey {
    /// What:
    ///     The residual regex after the bytes that lead to this state.
    /// Why:
    ///      `node` stores the residual regex after the bytes that lead to this state,
    ///  so
    ///          matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// node: Node;
    /// ```
    node: Node,
    /// What:
    ///     Whether this position is a line start (previous byte was a newline).
    /// Why:
    ///      `at_line_start` stores whether this position is a line start (previous byte was
    ///          a newline),
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// at_line_start: boolean;
    /// ```
    at_line_start: bool,
    /// What:
    ///     Whether the previous byte was a word byte.
    /// Why:
    ///      `prev_word` stores whether the previous byte was a word byte,
    ///  so matcher code
    ///          reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prev_word: boolean;
    /// ```
    prev_word: bool,
}

/// Largest number of sub-nodes a single derivative residual may hold before the
/// build bails.
///
/// What:
///  a ceiling on one residual regex's node count.
///  Why:
///  a real rule's residuals
/// stay small (tens to a few hundred nodes,
///  since bounded class repetition unrolls
/// into STATES,
///  not large residual NODES);
///  only pathological nested repetition or
/// complement-over-repetition grows a residual past this,
///  and that explodes memory and
/// time before the state-count cap fires,
///  so a tight ceiling aborts it fast (to the
/// counting back-end or a clean error) while leaving every real rule on the DFA.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RESIDUAL_NODE_CAP: number = 2_000;
/// ```
const RESIDUAL_NODE_CAP: usize = 2_000;

/// Reports whether a node holds more than [`RESIDUAL_NODE_CAP`] sub-nodes.
///
/// What:
///  a bounded structural walk that stops as soon as the budget is spent,
///  so it
/// is cheap on the common small residual and never itself walks an unbounded tree.
/// Why:
///  the build uses it to abort before a giant residual is cloned into a state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function residual_too_large(node: Node): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn residual_too_large(node: &Node) -> bool {
    fn spend(node: &Node, budget: &mut usize) -> bool {
        if *budget == 0 {
            return true;
        }
        *budget -= 1;
        match node {
            Node::Concat(parts) | Node::Alt(parts) | Node::Inter(parts) => {
                return parts.iter().any(|child| return spend(child, budget))
            }
            Node::Comp(inner) => return spend(inner, budget),
            Node::Repeat { node: body, .. } => return spend(body, budget),
            _ => return false,
        }
    }
    let mut budget = RESIDUAL_NODE_CAP;
    return spend(node, &mut budget)
}

/// What:
///     Unit tests for the DFA build and its residual guard,
///  in a sidecar (max-lines
///          exempt).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./tests";
/// ```
#[cfg(test)]
#[path = "build_tests.rs"]
mod tests;

/// Builds a DFA from a (search-wrapped) node,
///  abandoning past `cap` states.
///
/// What:
///  BFS over derivative states,
///  computing per-class transitions and a
/// per-state acceptance mask,
///  until no new state appears or `cap` is hit.
///  Why:
/// eager determinization yields a flat table that matches with no per-byte
/// allocation and no lazy-cache lock;
///  a caller selecting a back-end passes a small
/// `cap` so a blowup fails fast and it can fall back to the counting engine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_dfa_within(root: Node, cap: number): Dfa {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn build_dfa_within(root: Node, cap: usize) -> Result<Dfa, CompileError> {
    // What: never let the build exceed what a u16 state id can address. Why: the table
    // stores ids as u16 for cache density, so a state count past 65534 would truncate;
    // clamping here makes any caller's larger cap (e.g. the oracle) fail fast as StateCap
    // instead, and every real rule is far below this (the engine cap is 20000).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const MAX_U16_STATES: number = 65_534;
    // ```
    const MAX_U16_STATES: usize = 65_534;
    // The clamped budget gets its own name so it never shadows the caller's `cap`
    // parameter; every enforcement below spends the clamped value.
    let state_cap = cap.min(MAX_U16_STATES);
    let classes = compute_classes(&root);
    let nc = classes.nclasses;
    let mut index: HashMap<StateKey, u32> = HashMap::new();
    let mut states: Vec<StateKey> = Vec::new();
    let mut trans: Vec<u32> = Vec::new();
    let mut accept: Vec<u8> = Vec::new();
    // What: the start sits at a line start with no preceding word byte. Why:
    // position zero of the input has nothing before it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let start = intern(
        &mut index,
        &mut states,
        StateKey {
            node: root,
            at_line_start: true,
            prev_word: false,
        },
    );
    // What: process states in discovery order; interning may append more. Why:
    // appending in id order keeps `trans`/`accept` aligned with state ids.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut i = 0usize;
    while i < states.len() {
        let node = states[i].node.clone();
        // What: bail when a residual regex grows past the node budget. Why: nested
        // bounded repetition (and complement over it) makes derivative residuals
        // explode in SIZE before the state COUNT cap fires, so without this the build
        // exhausts memory; bailing as a StateCap lets the caller fall back to the
        // counting back-end or reject, never OOM (found by the roundtrip/differential
        // fuzz targets on deeply nested patterns).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        if residual_too_large(&node) {
            return Err(CompileError::StateCap { limit: state_cap });
        }
        let at_ls = states[i].at_line_start;
        let pw = states[i].prev_word;
        accept.push(compute_accept(&node, at_ls, pw));
        for class in 0..nc {
            let b = classes.reps[class];
            let word_after = classes.class_word[class];
            let line_end = classes.class_newline[class];
            let ctx = Ctx {
                line_start: at_ls,
                line_end,
                word_before: pw,
                word_after,
            };
            let next_node = derivative(&node, b, ctx);
            let next = intern(
                &mut index,
                &mut states,
                StateKey {
                    node: next_node,
                    at_line_start: line_end,
                    prev_word: word_after,
                },
            );
            trans.push(next);
        }
        if states.len() > state_cap {
            return Err(CompileError::StateCap { limit: state_cap });
        }
        i += 1;
    }
    // What: narrow the builder's `u32` ids to the `u16` the table stores. Why: the
    // build clamps the state cap to 65534 (MAX_U16_STATES) above, so every id and the
    // state count fit `u16` here; the table holds them at half a `u32` table's width
    // for cache density, and this is the one narrowing point now that `from_parts`
    // takes `u16` directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let trans_u16: Vec<u16> = trans.into_iter().map(|target| return target as u16).collect();
    return Ok(Dfa::from_parts(
        nc as u32,
        classes.class_map,
        classes.class_word,
        classes.class_newline,
        trans_u16,
        accept,
        start as u16,
        states.len() as u16,
    ))
}

/// Returns the id of `key`,
///  interning it on first sight.
///
/// What:
///  hash-map lookup with append-on-miss.
///  Why:
///  equal residual states must
/// collapse to one id so the automaton stays finite.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function intern(index: HashMap<StateKey, u32>, states: StateKey[], key: StateKey): number {
///   // Rust body below is the implementation.
/// }
/// ```
fn intern(index: &mut HashMap<StateKey, u32>, states: &mut Vec<StateKey>, key: StateKey) -> u32 {
    if let Some(&id) = index.get(&key) {
        return id;
    }
    let id = states.len() as u32;
    index.insert(key.clone(), id);
    states.push(key);
    return id
}

/// Computes the 4-bit acceptance mask of a state.
///
/// What:
///  tests nullability under each `(word_after, line_end)` boundary,
///  with the
/// state's own `line_start`/`word_before` fixed.
///  Why:
///  acceptance depends on the
/// upcoming byte,
///  so the matcher needs all four answers precomputed per state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compute_accept(node: Node, at_line_start: boolean, prev_word: boolean): number {
///   // Rust body below is the implementation.
/// }
/// ```
fn compute_accept(node: &Node, at_line_start: bool, prev_word: bool) -> u8 {
    // What: fold the four boundary contexts into a bitmask. Why: one mask-and at
    // match time then replaces four nullability checks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut mask = 0u8;
    for line_end in [false, true] {
        for word_after in [false, true] {
            let ctx = Ctx {
                line_start: at_line_start,
                line_end,
                word_before: prev_word,
                word_after,
            };
            if nullable(node, ctx) {
                mask |= accept_bit(word_after, line_end);
            }
        }
    }
    return mask
}
