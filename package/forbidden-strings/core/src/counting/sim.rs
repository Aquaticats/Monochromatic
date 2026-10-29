//! Shared counting-NFA simulation primitives over positions and follow sets.
//!
//! What:
//!  the [`State`] (active positions plus per-position count bitsets) and the
//! zero-width [`closure`],
//!  byte [`step_into`],
//!  and boundary helpers that advance it
//! over an element list and its follow sets.
//!  Why:
//!  the single-pattern search and the
//! synchronized product run the same per-operand simulation,
//!  so its core lives here
//! once;
//!  states are reused as ping-pong buffers so the byte step never allocates.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module sim: see exported functions and types below.
//! ```

/// What:
///     Imports the word predicate used to build the boundary context.
/// Why:
///      The code below uses `is_word_byte` directly;
///  importing from `crate/charset` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { is_word_byte } from "crate/charset";
/// ```
use crate::charset::is_word_byte;

/// What:
///     Imports the boundary context that resolves anchors.
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
///     Imports the bitset that holds a counted position's live counts.
/// Why:
///      The code below uses `CountSet` directly;
///  importing from `crate/counting/countset`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CountSet } from "crate/counting/countset";
/// ```
use crate::counting::countset::CountSet;

/// What:
///     Imports the position kind being simulated.
/// Why:
///      The code below uses `Element` directly;
///  importing from `crate/counting/element`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Element } from "crate/counting/element";
/// ```
use crate::counting::element::Element;

/// Live simulation state:
///  which positions are active and each counter's set.
///
/// What:
///  `active[p]` means position `p` is reached,
///  with `p == len` (the virtual
/// accept past the last position) meaning the match is complete;
///  `counts[p]` holds
/// the live repetition counts of a `Counted` position and is empty otherwise.
///  Why:
/// grouping counts into one per-position bitset is the whole device,
///  it keeps
/// overlapping repetitions in one word-addressable set instead of exploding states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = {
///   // fields documented in Rust above
/// };
/// ```
pub(crate) struct State {
    /// What:
    ///     One active-flag per position `0..len` plus the accept index `len`.
    /// Why:
    ///      `active` stores one active-flag per position `0..len` plus the accept index
    ///          `len`,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// active: boolean[];
    /// ```
    active: Vec<bool>,
    /// What:
    ///     One count bitset per position index `0..len`.
    /// Why:
    ///      `counts` stores one count bitset per position index `0..len`,
    ///  so matcher code
    ///          reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// counts: CountSet[];
    /// ```
    counts: Vec<CountSet>,
}

/// What:
///     Construction,
///  reuse,
///  and predicates over a simulation state.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl State {
    /// Builds an empty state shaped for `elements`.
    ///
    /// What:
    ///  `len + 1` active-flags (the extra one is the accept index) and one
    /// count bitset per position,
    ///  each sized to its bound.
    ///  Why:
    ///  sizing the bitsets
    /// once lets them be cleared and reused without reallocation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function new(elements: Element[]): State {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn new(elements: &[Element]) -> State {
        return State {
            active: vec![false; elements.len() + 1],
            counts: elements.iter().map(count_set_for).collect(),
        }
    }

    /// Empties the state in place for reuse as a ping-pong buffer.
    ///
    /// What:
    ///  clears every active-flag and count bitset without reallocating.
    ///  Why:
    /// the byte step fills a cleared destination buffer,
    ///  so no allocation per byte.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function clear(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn clear(&mut self) {
        self.active.iter_mut().for_each(|on| *on = false);
        self.counts.iter_mut().for_each(CountSet::clear);
    }

    /// Activates every start position,
    ///  opening a fresh match attempt.
    ///
    /// What:
    ///  marks each id in `start` active.
    ///  Why:
    ///  the search loop re-seeds the
    /// starts every boundary for the `Σ*` prefix;
    ///  a product thread seeds once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function seed(start: number[]): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn seed(&mut self, start: &[u32]) {
        for &s in start {
            self.active[s as usize] = true;
        }
    }

    /// Reports whether the match has reached the accept position.
    ///
    /// What:
    ///  the active-flag at the accept index (`len`).
    ///  Why:
    ///  reaching past the
    /// last position is acceptance.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function accepts(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn accepts(&self) -> bool {
        return self.active.last().copied().unwrap_or(false)
    }

    /// Reports whether the state can never reach acceptance again.
    ///
    /// What:
    ///  no position is active and every count bitset is empty.
    ///  Why:
    ///  a product
    /// thread whose required operand has gone dead can be dropped,
    ///  bounding the live
    /// thread count by the longest operand.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_dead(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn is_dead(&self) -> bool {
        return self.active.iter().all(|&on| return !on) && self.counts.iter().all(CountSet::is_empty)
    }
}

/// Builds the count bitset for one position,
///  sized to its repetition bound.
///
/// What:
///  a `Counted` position gets a set holding `[0, max]`;
///  any other gets a
/// minimal placeholder never used.
///  Why:
///  only counted positions carry counts,
///  but a
/// uniform per-index vector keeps the simulation's indexing simple.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function count_set_for(element: Element): CountSet {
///   // Rust body below is the implementation.
/// }
/// ```
fn count_set_for(element: &Element) -> CountSet {
    match element {
        Element::Counted { max, .. } => return CountSet::new(*max),
        _ => return CountSet::new(0),
    }
}

/// Builds the boundary context at position `i` between (or at the ends of) `line`.
///
/// What:
///  line starts after a newline or at the front;
///  line ends before a newline or
/// at the back;
///  word flags read the adjacent bytes.
///  Why:
///  mirrors exactly the
/// context the eager DFA uses,
///  so both back-ends agree on anchors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function boundary_ctx(line: Uint8Array, i: number): Ctx {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn boundary_ctx(line: &[u8], i: usize) -> Ctx {
    let len = line.len();
    return Ctx {
        line_start: i == 0 || line[i - 1] == b'\n',
        line_end: i == len || line[i] == b'\n',
        word_before: i > 0 && is_word_byte(line[i - 1]),
        word_after: i < len && is_word_byte(line[i]),
    }
}

/// Expands active positions through every zero-width move to a fixpoint.
///
/// What:
///  enters a `Counted` position by seeding count 0,
///  exits one whose set meets
/// its bound,
///  and passes an anchor when `ctx` permits,
///  following each to its
/// successors;
///  loops until stable.
///  Why:
///  zero-width moves chain (alternation into an
/// anchor,
///  a skippable repetition),
///  so one pass is not enough.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function closure(elements: Element[], follow: number[][], state: State, ctx: Ctx): void {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn closure(elements: &[Element], follow: &[Vec<u32>], state: &mut State, ctx: Ctx) {
    let mut changed = true;
    while changed {
        changed = false;
        for (p, element) in elements.iter().enumerate() {
            if zero_width_move(element, p, follow, state, ctx) {
                changed = true;
            }
        }
    }
}

/// Applies the zero-width moves available at one position this pass.
///
/// What:
///  a reached counted position seeds count 0,
///  any counted position whose set
/// meets `min` follows on,
///  and a reached anchor follows on when `ctx` allows.
///  Why:
/// counted entry depends on the position being freshly active,
///  but counted exit
/// depends only on the carried counts,
///  so both are tested here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function zero_width_move(element: Element, p: number, follow: number[][], state: State, ctx: Ctx): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn zero_width_move(
    element: &Element,
    p: usize,
    follow: &[Vec<u32>],
    state: &mut State,
    ctx: Ctx,
) -> bool {
    match element {
        Element::Class(_) => return false,
        Element::Counted { min, .. } => return counted_move(p, *min, follow, state),
        Element::LineStart => return anchor_move(p, ctx.line_start, follow, state),
        Element::LineEnd => return anchor_move(p, ctx.line_end, follow, state),
        Element::WordBoundary => return anchor_move(p, ctx.word_before != ctx.word_after, follow, state),
    }
}

/// Seeds a freshly entered counted position and follows it once its bound is met.
///
/// What:
///  insert count 0 when active,
///  then follow on when any count is at least
/// `min`.
///  Why:
///  entry is one-shot per activation;
///  exit fires every pass the carried
/// counts allow,
///  independent of activation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function counted_move(p: number, min: number, follow: number[][], state: State): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn counted_move(p: usize, min: usize, follow: &[Vec<u32>], state: &mut State) -> bool {
    let mut changed = false;
    if state.active[p] && state.counts[p].insert_zero() {
        changed = true;
    }
    if state.counts[p].has_at_least(min) && activate(&follow[p], &mut state.active) {
        changed = true;
    }
    return changed
}

/// Follows a reached anchor when its boundary condition holds.
///
/// What:
///  activate the successors when the anchor is active and `cond` is true.
///  Why:
/// anchors are zero-width and pass only in the right boundary context.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anchor_move(p: number, cond: boolean, follow: number[][], state: State): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn anchor_move(p: usize, cond: bool, follow: &[Vec<u32>], state: &mut State) -> bool {
    if state.active[p] && cond {
        return activate(&follow[p], &mut state.active)
    } else {
        return false
    }
}

/// Marks every target active,
///  reporting whether anything changed.
///
/// What:
///  a guarded set over a follow list.
///  Why:
///  the closure fixpoint needs to know
/// when a pass made progress.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function activate(targets: number[], active: boolean[]): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn activate(targets: &[u32], active: &mut [bool]) -> bool {
    let mut changed = false;
    for &t in targets {
        if !active[t as usize] {
            active[t as usize] = true;
            changed = true;
        }
    }
    return changed
}

/// Consumes one byte,
///  writing the next state into a reused destination buffer.
///
/// What:
///  clears `dst`,
///  then a reached class follows on a matching byte and a counted
/// position's live counts each step up by one (capped at `max`) on a matching byte;
/// anchors are zero-width and drop.
///  Why:
///  this is the only place input is consumed;
/// filling a reused buffer keeps the byte step allocation-free and the counted
/// update a single word-level shift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function step_into(elements: Element[], follow: number[][], src: State, b: number, dst: State): void {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn step_into(
    elements: &[Element],
    follow: &[Vec<u32>],
    src: &State,
    b: u8,
    dst: &mut State,
) {
    dst.clear();
    for (p, element) in elements.iter().enumerate() {
        match element {
            Element::Class(set) => {
                if src.active[p] && set.contains(b) {
                    activate(&follow[p], &mut dst.active);
                }
            }
            Element::Counted { set, max, .. } => {
                if set.contains(b) {
                    dst.counts[p].copy_advanced_from(&src.counts[p], *max);
                }
            }
            Element::LineStart | Element::LineEnd | Element::WordBoundary => {}
        }
    }
}

/// What:
///     Unit tests for the counting-set simulation primitives,
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
#[path = "sim_tests.rs"]
mod tests;
