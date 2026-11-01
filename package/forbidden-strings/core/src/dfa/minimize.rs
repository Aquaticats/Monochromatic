//! What:
//!     DFA state minimization by Moore partition refinement.
//! Why:
//!      This file is the Rust module that groups the minimize implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module minimize: see exported functions and types below.
//! ```

/// What:
///     Imports the hash map used to assign colors from signatures.
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
///     Imports the table type being minimized.
/// Why:
///      The code below uses `Dfa` directly;
///  importing from `crate/dfa/table` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Dfa } from "crate/dfa/table";
/// ```
use crate::dfa::table::Dfa;

/// Returns a behaviorally equivalent DFA with the fewest states.
///
/// What:
///  Moore's algorithm:
///  color states by acceptance mask,
///  then repeatedly
/// refine so two states share a color only if every class transition leads to the
/// same color,
///  until the partition is stable;
///  rebuild one state per color.
///  Why:
/// the unanchored derivative construction over-distinguishes overlapping partial
/// matches,
///  so the raw DFA has far more states than the language needs;
///  the
/// minimal DFA is smaller,
///  faster (better cache behavior),
///  and smaller to
/// serialize.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function minimize(dfa: Dfa): Dfa {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn minimize(dfa: &Dfa) -> Dfa {
    let n = dfa.num_states as usize;
    let nc = dfa.nclasses as usize;
    // What: seed colors by accept mask. Why: two states are immediately
    // distinguishable when they accept in different boundary contexts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut color = initial_colors(dfa, n);
    let mut count = distinct_count(&color);
    loop {
        let (next_color, next_count) = refine(dfa, &color, n, nc);
        // What: stop when refinement adds no new groups. Why: the partition is
        // then stable and equals the Myhill-Nerode classes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        if next_count == count {
            color = next_color;
            break;
        }
        color = next_color;
        count = next_count;
    }
    return rebuild(dfa, &color, count, nc)
}

/// Assigns each state an initial color from its acceptance mask.
///
/// What:
///  equal masks share a color.
///  Why:
///  the coarsest partition consistent with
/// observable acceptance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initial_colors(dfa: Dfa, n: number): number[] {
///   // Rust body below is the implementation.
/// }
/// ```
fn initial_colors(dfa: &Dfa, n: usize) -> Vec<u32> {
    let mut map: HashMap<u8, u32> = HashMap::new();
    let mut color = vec![0u32; n];
    for (state, slot) in color.iter_mut().enumerate() {
        let next = map.len() as u32;
        *slot = *map.entry(dfa.accept[state]).or_insert(next);
    }
    return color
}

/// Refines a coloring once,
///  splitting groups by transition signature.
///
/// What:
///  a state's signature is its color plus the colors of all its class
/// successors;
///  distinct signatures get distinct new colors.
///  Why:
///  one round of
/// Moore refinement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refine(dfa: Dfa, color: number[], n: number, nc: number): (Vec<u32>, usize) {
///   // Rust body below is the implementation.
/// }
/// ```
fn refine(dfa: &Dfa, color: &[u32], n: usize, nc: usize) -> (Vec<u32>, usize) {
    let mut map: HashMap<Vec<u32>, u32> = HashMap::new();
    let mut next = vec![0u32; n];
    for (state, slot) in next.iter_mut().enumerate() {
        // What: build the signature vector. Why: captures the state's class plus
        // where each transition currently leads.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let mut signature = Vec::with_capacity(nc + 1);
        signature.push(color[state]);
        for class in 0..nc {
            signature.push(color[dfa.trans[state * nc + class] as usize]);
        }
        let id = map.len() as u32;
        *slot = *map.entry(signature).or_insert(id);
    }
    let count = map.len();
    return (next, count)
}

/// Counts the distinct colors in a coloring.
///
/// What:
///  size of the set of color values.
///  Why:
///  the loop stops when this stops
/// growing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function distinct_count(color: number[]): number {
///   // Rust body below is the implementation.
/// }
/// ```
fn distinct_count(color: &[u32]) -> usize {
    let mut seen: Vec<u32> = color.to_vec();
    seen.sort_unstable();
    seen.dedup();
    return seen.len()
}

/// What:
///     Unit tests for minimization helpers,
///  in a sidecar (max-lines exempt).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./tests";
/// ```
#[cfg(test)]
#[path = "minimize_tests.rs"]
mod tests;

/// Rebuilds a DFA with one state per color.
///
/// What:
///  picks a representative state per color and remaps its transitions and
/// acceptance through the coloring.
///  Why:
///  produces the minimized table while
/// keeping the byte-class layout unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rebuild(dfa: Dfa, color: number[], count: number, nc: number): Dfa {
///   // Rust body below is the implementation.
/// }
/// ```
fn rebuild(dfa: &Dfa, color: &[u32], count: usize, nc: usize) -> Dfa {
    // What: first state seen for each color is its representative. Why: any member
    // of a color has identical behavior, so the first works.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut rep: Vec<usize> = vec![usize::MAX; count];
    for (state, &c) in color.iter().enumerate() {
        if rep[c as usize] == usize::MAX {
            rep[c as usize] = state;
        }
    }
    // What: emit the remapped ids at the `u16` width `from_parts` now takes. Why: each
    // color id is below `count`, itself at most the input DFA's state count (capped at
    // 65534), so every `color[..]` fits `u16` and no narrowing pass is needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut trans = vec![0u16; count * nc];
    let mut accept = vec![0u8; count];
    for c in 0..count {
        let state = rep[c];
        accept[c] = dfa.accept[state];
        for class in 0..nc {
            let target = dfa.trans[state * nc + class] as usize;
            trans[c * nc + class] = color[target] as u16;
        }
    }
    return Dfa::from_parts(
        nc as u32,
        dfa.class_map.clone(),
        dfa.class_word.clone(),
        dfa.class_newline.clone(),
        trans,
        accept,
        color[dfa.start as usize] as u16,
        count as u16,
    )
}
