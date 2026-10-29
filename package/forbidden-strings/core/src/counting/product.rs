//! Synchronized-product back-end for intersection and complement over counting
//! NFAs.
//!
//! What:
//!  a [`ProductProgram`] holds positive operands (each must match the same
//! span) and negative operands (none may match that span,
//!  the `~(...)` operands),
//! matched by running every operand NFA in lockstep per start position.
//!  Why:
//!  under
//! `Σ*·(A & ~B)` the SAME substring must satisfy `A` and fail `B`,
//!  so the operands
//! cannot be run as independent search automata (they would match different spans);
//! one thread per start keeps them synchronized while each operand's counts still
//! live in a counter-set,
//!  so alternation and bounded repetition never blow up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module product: see exported functions and types below.
//! ```

/// What:
///     Imports the serde derives so a product program can be persisted.
/// Why:
///      The code below uses `Deserialize`,
///  `Serialize` directly;
///  importing from `serde`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Deserialize, Serialize } from "serde";
/// ```
use serde::{Deserialize, Serialize};

/// What:
///     Imports the node algebra the builder reads.
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
///     Imports the boundary context threaded through the closure.
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
///     Imports the NFA builder for each operand.
/// Why:
///      The code below uses `build_nfa` directly;
///  importing from `crate/counting/build`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { build_nfa } from "crate/counting/build";
/// ```
use crate::counting::build::build_nfa;

/// What:
///     Imports the counting NFA each operand compiles to.
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

/// What:
///     Imports the error type for validating a decoded program.
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

/// An intersection of counting-NFA operands,
///  some of them complemented.
///
/// What:
///  the positives that must all match one span and the negatives that must all
/// fail that same span.
///  Why:
///  the serializable,
///  counter-aware back-end for `&`/`~`
/// patterns;
///  its size is linear in the pattern,
///  never in any repetition bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProductProgram = {
///   // fields documented in Rust above
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductProgram {
    /// What:
    ///     Operands that must each match the same span.
    /// Why:
    ///      `positives` stores operands that must each match the same span,
    ///  so matcher code
    ///          reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// positives: CountingNfa[];
    /// ```
    pub positives: Vec<CountingNfa>,
    /// What:
    ///     Operands whose match would veto the span (the `~(...)` operands).
    /// Why:
    ///      `negatives` stores operands whose match would veto the span (the `~(...)`
    ///          operands),
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// negatives: CountingNfa[];
    /// ```
    pub negatives: Vec<CountingNfa>,
}

/// What:
///     Matching and decode validation for a product program.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl ProductProgram {
    /// Reports whether some span satisfies every positive and no negative.
    ///
    /// What:
    ///  runs the synchronized-product simulation.
    ///  Why:
    ///  the boolean answer for an
    /// intersection-with-complement pattern;
    ///  the prefilter lives one level up.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn is_match(&self, line: &[u8]) -> bool {
        return run_product(self, line)
    }

    /// Checks that a decoded product program is safe to run on untrusted input.
    ///
    /// What:
    ///  requires at least one positive operand and validates every operand.
    /// Why:
    ///  a program with no positive would accept on the empty span everywhere,
    /// and each operand allocates counter-sets sized by its decoded bound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function validate(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), CompileError> {
        if self.positives.is_empty() {
            return Err(CompileError::Invalid {
                message: "product program has no positive operand".to_string(),
            });
        }
        for operand in self.positives.iter().chain(&self.negatives) {
            operand.validate()?;
        }
        return Ok(())
    }
}

/// Attempts to express `node` as a product of counting-NFA operands.
///
/// What:
///  a `Node::Inter` whose operands each build into an NFA,
///  splitting `Comp`
/// operands into the negatives and the rest into the positives;
///  returns `None` for
/// anything else or when no positive remains.
///  Why:
///  those shapes need the eager DFA,
/// so the caller falls back to it;
///  this back-end claims only the NFA `&`/`~` cases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_product(node: Node): ProductProgram | null {
///   // Rust body below is the implementation.
/// }
/// ```
///
/// @example
/// ```ignore
/// // (?:\b(?:A3T[A-Z0-9]|AKIA|ASIA)[A-Z2-7]{16}\b) & ~(AKIA2{16})
/// let prog = build_product(&node).unwrap();
/// assert_eq!((prog.positives.len(), prog.negatives.len()), (1, 1));
/// ```
pub fn build_product(node: &Node) -> Option<ProductProgram> {
    let Node::Inter(operands) = node else {
        return None;
    };
    let mut positives: Vec<CountingNfa> = Vec::new();
    let mut negatives: Vec<CountingNfa> = Vec::new();
    for operand in operands {
        match operand {
            Node::Comp(inner) => negatives.push(build_nfa(inner)?),
            other => positives.push(build_nfa(other)?),
        }
    }
    if positives.is_empty() {
        return None;
    }
    return Some(ProductProgram { positives, negatives })
}

/// The per-operand simulation states for one side of a thread at one instant.
///
/// What:
///  one `State` per positive operand and one per negative operand,
///  in their
/// program order.
///  Why:
///  a thread keeps two of these as ping-pong buffers so the byte
/// step writes into the spare and swaps,
///  never allocating per byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Operands = {
///   // fields documented in Rust above
/// };
/// ```
struct Operands {
    /// What:
    ///     Per-positive-operand simulation state,
    ///  in `positives` order.
    /// Why:
    ///      `positives` stores per-positive-operand simulation state,
    ///  in `positives` order,
    ///          so matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// positives: State[];
    /// ```
    positives: Vec<State>,
    /// What:
    ///     Per-negative-operand simulation state,
    ///  in `negatives` order.
    /// Why:
    ///      `negatives` stores per-negative-operand simulation state,
    ///  in `negatives` order,
    ///          so matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// negatives: State[];
    /// ```
    negatives: Vec<State>,
}

/// One in-flight match attempt anchored at a single start position.
///
/// What:
///  the current operand states plus a spare buffer the byte step fills.
///  Why:
/// keeping every operand of one start together enforces the same-span requirement,
/// and reusing the spare keeps the step allocation-free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Thread = {
///   // fields documented in Rust above
/// };
/// ```
struct Thread {
    /// What:
    ///     Live operand states for this start.
    /// Why:
    ///      `cur` stores live operand states for this start,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cur: Operands;
    /// ```
    cur: Operands,
    /// What:
    ///     Spare buffer the byte step writes into,
    ///  then swaps with `cur`.
    /// Why:
    ///      `next` stores spare buffer the byte step writes into,
    ///  then swaps with `cur`,
    ///  so
    ///          matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// next: Operands;
    /// ```
    next: Operands,
}

/// Runs the product across every boundary,
///  returning true on first acceptance.
///
/// What:
///  seed a fresh thread at each boundary (the `Σ*` prefix),
///  close every live
/// thread,
///  test acceptance,
///  then advance all threads by the next byte.
///  Why:
///  a match
/// may begin at any position,
///  and each thread carries the joint operand state for
/// its own start so acceptance tests one shared span.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run_product(prog: ProductProgram, line: Uint8Array): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn run_product(prog: &ProductProgram, line: &[u8]) -> bool {
    let mut threads: Vec<Thread> = Vec::new();
    for i in 0..=line.len() {
        threads.push(new_thread(prog));
        let ctx = boundary_ctx(line, i);
        for thread in &mut threads {
            close_operands(prog, &mut thread.cur, ctx);
        }
        if threads.iter().any(|thread| return accepts(&thread.cur)) {
            return true;
        }
        if i == line.len() {
            break;
        }
        let b = line[i];
        threads.retain_mut(|thread| return advance_thread(prog, thread, b));
    }
    return false
}

/// Builds a thread seeded for a start at the current boundary.
///
/// What:
///  seeded current states plus empty spare buffers,
///  one per operand.
///  Why:
///  each
/// start gets its own anchored run,
///  seeded once;
///  the spare is reused every byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function new_thread(prog: ProductProgram): Thread {
///   // Rust body below is the implementation.
/// }
/// ```
fn new_thread(prog: &ProductProgram) -> Thread {
    return Thread {
        cur: Operands {
            positives: seeded_states(&prog.positives),
            negatives: seeded_states(&prog.negatives),
        },
        next: Operands {
            positives: empty_states(&prog.positives),
            negatives: empty_states(&prog.negatives),
        },
    }
}

/// Builds a seeded state for each operand.
///
/// What:
///  one `State` per operand with its start positions active.
///  Why:
///  the current
/// side of a fresh thread starts poised at every operand's start set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function seeded_states(operands: CountingNfa[]): State[] {
///   // Rust body below is the implementation.
/// }
/// ```
fn seeded_states(operands: &[CountingNfa]) -> Vec<State> {
    return operands
        .iter()
        .map(|nfa| {
            let mut state = State::new(&nfa.elements);
            state.seed(&nfa.start);
            return state
        })
        .collect()
}

/// Builds an empty state for each operand.
///
/// What:
///  one `State::new` sized to each operand's positions.
///  Why:
///  the spare side of
/// a fresh thread is the reusable byte-step destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function empty_states(operands: CountingNfa[]): State[] {
///   // Rust body below is the implementation.
/// }
/// ```
fn empty_states(operands: &[CountingNfa]) -> Vec<State> {
    return operands.iter().map(|nfa| return State::new(&nfa.elements)).collect()
}

/// Takes the zero-width closure of every operand in one side.
///
/// What:
///  closes each positive and negative state under the boundary context.
///  Why:
/// anchors and skippable repetitions must settle before the accept test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function close_operands(prog: ProductProgram, ops: Operands, ctx: Ctx): void {
///   // Rust body below is the implementation.
/// }
/// ```
fn close_operands(prog: &ProductProgram, ops: &mut Operands, ctx: Ctx) {
    for (state, nfa) in ops.positives.iter_mut().zip(&prog.positives) {
        closure(&nfa.elements, &nfa.follow, state, ctx);
    }
    for (state, nfa) in ops.negatives.iter_mut().zip(&prog.negatives) {
        closure(&nfa.elements, &nfa.follow, state, ctx);
    }
}

/// Reports whether a side accepts:
///  all positives match and no negative does.
///
/// What:
///  conjunction over positives with a negation over negatives.
///  Why:
///  this is
/// `A & ~B` evaluated on the one span the thread represents.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function accepts(ops: Operands): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn accepts(ops: &Operands) -> bool {
    return ops.positives.iter().all(State::accepts) && ops.negatives.iter().all(|state| return !state.accepts())
}

/// Advances one thread by a byte,
///  reporting whether it stays alive.
///
/// What:
///  steps positives into the spare and prunes if any died,
///  then steps negatives
/// and swaps the buffers.
///  Why:
///  a dead positive can never match again so the thread
/// is dropped,
///  which bounds the live-thread count by the longest positive;
///  negatives
/// are kept even when dead (a dead negative means `~B` holds).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function advance_thread(prog: ProductProgram, thread: Thread, b: number): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn advance_thread(prog: &ProductProgram, thread: &mut Thread, b: u8) -> bool {
    step_states(&prog.positives, &thread.cur.positives, &mut thread.next.positives, b);
    if thread.next.positives.iter().any(State::is_dead) {
        return false;
    }
    step_states(&prog.negatives, &thread.cur.negatives, &mut thread.next.negatives, b);
    std::mem::swap(&mut thread.cur, &mut thread.next);
    return true
}

/// Steps every operand state of one side from `src` into `dst`.
///
/// What:
///  runs the byte step per operand,
///  source to reused destination.
///  Why:
///  the
/// shared advance for both the positive and negative lists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function step_states(operands: CountingNfa[], src: State[], dst: State[], b: number): void {
///   // Rust body below is the implementation.
/// }
/// ```
fn step_states(operands: &[CountingNfa], src: &[State], dst: &mut [State], b: u8) {
    for ((nfa, source), destination) in operands.iter().zip(src).zip(dst.iter_mut()) {
        step_into(&nfa.elements, &nfa.follow, source, b, destination);
    }
}

/// Differential tests against the eager DFA plus a serialized-size proof.
///
/// What:
///  lives in a separate `*_tests.rs` file (exempt from the line and rustdoc
/// budgets).
///  Why:
///  keeps the product file within budget while proving it against the
/// trusted oracle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./tests";
/// ```
#[cfg(test)]
#[path = "product_tests.rs"]
mod tests;
