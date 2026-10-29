//! What:
//!     Counting-NFA search loop under unanchored matching.
//! Why:
//!      This file is the Rust module that groups the run implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module run: see exported functions and types below.
//! ```

/// What:
///     Imports the counting NFA being run.
/// Why:
///      The code below uses `CountingNfa` directly;
///  importing from `crate/counting/nfa`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CountingNfa } from "crate/counting/nfa";
/// ```
use crate::counting::nfa::CountingNfa;

/// What:
///     Imports the shared simulation core.
/// Why:
///      The code below uses `State`,
///  `boundary_ctx`,
///  `closure`,
///  `step_into` directly;
///          importing from `crate/counting/sim` keeps each call site focused on the matcher
///          logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   State,
///   boundary_ctx,
///   closure,
///   step_into,
/// } from "crate/counting/sim";
/// ```
use crate::counting::sim::{State, boundary_ctx, closure, step_into};

/// Runs the NFA across every boundary,
///  returning true on first acceptance.
///
/// What:
///  at each boundary seed the start positions afresh (the `Σ*` search prefix),
/// take the zero-width closure,
///  test acceptance,
///  then consume the next byte into the
/// spare buffer and swap.
///  Why:
///  a match may start at any position and end at any
/// boundary,
///  so both seeding and the accept test happen at every boundary;
///  the two
/// buffers are reused so the byte step never allocates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(nfa: CountingNfa, line: Uint8Array): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn run(nfa: &CountingNfa, line: &[u8]) -> bool {
    let mut cur = State::new(&nfa.elements);
    let mut next = State::new(&nfa.elements);
    for i in 0..=line.len() {
        // What: the search prefix keeps a fresh start live at every boundary.
        // Why: a match may begin at this position.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        cur.seed(&nfa.start);
        let ctx = boundary_ctx(line, i);
        closure(&nfa.elements, &nfa.follow, &mut cur, ctx);
        if cur.accepts() {
            return true;
        }
        if i == line.len() {
            break;
        }
        step_into(&nfa.elements, &nfa.follow, &cur, line[i], &mut next);
        std::mem::swap(&mut cur, &mut next);
    }
    return false
}

/// Differential tests against the eager DFA plus a serialized-size proof.
///
/// What:
///  lives in a separate `*_tests.rs` file (exempt from the line and rustdoc
/// budgets).
///  Why:
///  keeps the simulation file within its budget while still proving
/// the counting back-end against the trusted oracle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./tests";
/// ```
#[cfg(test)]
#[path = "run_tests.rs"]
mod tests;
