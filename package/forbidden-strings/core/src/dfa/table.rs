//! What:
//!     The serializable DFA table and its allocation-free match loop.
//! Why:
//!      This file is the Rust module that groups the table implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module table: see exported functions and types below.
//! ```

/// What:
///     Imports the serde derives for persisting a compiled automaton.
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
///     Imports the error type for validation of a decoded automaton.
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
///     Imports the byte set used to report a DFA's possible first match bytes.
/// Why:
///      The code below uses `ByteSet` directly;
///  importing from `crate/charset` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ByteSet } from "crate/charset";
/// ```
use crate::charset::ByteSet;

/// Returns the single-bit acceptance mask for one boundary context.
///
/// What:
///  encodes `(word_after, line_end)` into a bit index `0..=3` and shifts a
/// `1` into it.
///  Why:
///  each state stores which of these four boundary contexts make
/// it accepting,
///  so the matcher tests acceptance with one mask-and.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function accept_bit(word_after: boolean, line_end: boolean): number {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn accept_bit(word_after: bool, line_end: bool) -> u8 {
    let index = ((line_end as u8) << 1) | (word_after as u8);
    return 1u8 << index
}

/// A compiled,
///  table-driven deterministic automaton over byte classes.
///
/// What:
///  a byte-to-class map,
///  per-class word/newline flags,
///  a dense transition
/// table,
///  a per-state 4-bit acceptance mask,
///  and the start state.
///  Why:
///  this flat
/// shape serializes directly with serde and matches in a tight per-byte loop with
/// no allocation,
///  which is what beats the lazy,
///  lock-guarded alternatives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Dfa = {
///   // fields documented in Rust above
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dfa {
    /// What:
    ///     Number of distinct byte classes.
    /// Why:
    ///      `nclasses` stores number of distinct byte classes,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nclasses: number;
    /// ```
    pub(crate) nclasses: u32,
    /// What:
    ///     Length-256 map from a byte to its class id.
    /// Why:
    ///      `class_map` stores length-256 map from a byte to its class id,
    ///  so matcher code
    ///          reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// class_map: number[];
    /// ```
    pub(crate) class_map: Vec<u8>,
    /// What:
    ///     Per-class flag:
    ///  is a byte of this class a word byte (for `\b`)?
    /// Why:
    ///      `class_word` stores per-class flag:
    ///  is a byte of this class a word byte (for
    ///          `\b`)?,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// class_word: boolean[];
    /// ```
    pub(crate) class_word: Vec<bool>,
    /// What:
    ///     Per-class flag:
    ///  is a byte of this class a newline (for `^`/`$`)?
    /// Why:
    ///      `class_newline` stores per-class flag:
    ///  is a byte of this class a newline (for
    ///          `^`/`$`)?,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// class_newline: boolean[];
    /// ```
    pub(crate) class_newline: Vec<bool>,
    /// Dense transition table of length `num_states * nclasses`.
    ///
    /// What:
    ///  state ids are `u16`,
    ///  so the table is half the width of a `u32` one.
    ///  Why:
    /// the hot match loop reads one entry per byte,
    ///  so a denser table fits more of the
    /// automaton in cache;
    ///  the build caps states at 65534,
    ///  well above any real rule's
    /// (and the engine cap of 20000),
    ///  so a `u16` id never overflows.
    /// Why:
    ///      `trans` stores state ids are `u16`,
    ///  so the table is half the width of a `u32`
    ///          one.
    ///  Why:
    ///  the hot match loop reads one entry per byte,
    ///  so a denser table fits
    ///          more of the automaton in cache;
    ///  the build caps states at 65534,
    ///  well above any
    ///          real rule's (and the engine cap of 20000),
    ///  so a `u16` id never overflows,
    ///  so
    ///          matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trans: number[];
    /// ```
    pub(crate) trans: Vec<u16>,
    /// What:
    ///     Per-state acceptance mask over the four `(word_after, line_end)` contexts.
    /// Why:
    ///      `accept` stores per-state acceptance mask over the four `(word_after, line_end)`
    ///          contexts,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// accept: number[];
    /// ```
    pub(crate) accept: Vec<u8>,
    /// What:
    ///     Start state id.
    /// Why:
    ///      `start` stores start state id,
    ///  so matcher code reads that precomputed state by
    ///          name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// start: number;
    /// ```
    pub(crate) start: u16,
    /// What:
    ///     Total number of states.
    /// Why:
    ///      `num_states` stores total number of states,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// num_states: number;
    /// ```
    pub(crate) num_states: u16,
    /// A non-accepting self-looping sink,
    ///  or `num_states` when there is none.
    ///
    /// What:
    ///  the dead state the match loop early-exits on;
    ///  `num_states` (no real id)
    /// disables the exit.
    ///  Why:
    ///  an anchored DFA dies on the first non-matching byte,
    ///  so
    /// without this the loop walks the rest of the line for nothing;
    ///  this is the bulk
    /// of the gate's per-hit and line-start cost.
    /// Why:
    ///      `dead` stores the dead state the match loop early-exits on;
    ///  `num_states` (no
    ///          real id) disables the exit.
    ///  Why:
    ///  an anchored DFA dies on the first non-matching
    ///          byte,
    ///  so without this the loop walks the rest of the line for nothing;
    ///  this is
    ///          the bulk of the gate's per-hit and line-start cost,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dead: number;
    /// ```
    pub(crate) dead: u16,
}

/// What:
///     Construction-from-parts and matching for `Dfa`.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl Dfa {
    /// Assembles a `Dfa` from already-built tables.
    ///
    /// What:
    ///  the builder's only constructor;
    ///  takes ownership of each `u16`-id table.
    /// Why:
    ///  fields stay private so a `Dfa` can only arise from the builder or a
    /// validated decode;
    ///  ids are `u16` because the build caps states at 65534,
    ///  so each
    /// caller narrows its own ids and the stored table is half a `u32` table's width.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function from_parts(nclasses: number, class_map: number[], class_word: boolean[], class_newline: boolean[], trans: number[], accept: number[], start: number, num_states: number): Dfa {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        nclasses: u32,
        class_map: Vec<u8>,
        class_word: Vec<bool>,
        class_newline: Vec<bool>,
        trans: Vec<u16>,
        accept: Vec<u8>,
        start: u16,
        num_states: u16,
    ) -> Self {
        let dead = find_dead(&trans, &accept, nclasses, num_states);
        return Dfa {
            nclasses,
            class_map,
            class_word,
            class_newline,
            trans,
            accept,
            start,
            num_states,
            dead,
        }
    }

    /// Reports whether the pattern matches some substring of `line`.
    ///
    /// What:
    ///  walks the byte-class transitions,
    ///  checking the acceptance mask at
    /// each boundary against the upcoming byte's context,
    ///  and returns on the
    /// first match.
    ///  Why:
    ///  this is the hot path;
    ///  it is branch-light and never
    /// allocates,
    ///  and early exit makes a positive answer cheap.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn is_match(&self, line: &[u8]) -> bool {
        let nc = self.nclasses as usize;
        let dead = self.dead as usize;
        let mut state = self.start as usize;
        // What: each iteration tests acceptance at the boundary before `b`, then
        // consumes `b`. Why: a match may end at any boundary; the upcoming byte
        // supplies `word_after` and `line_end` for `$`/`\b`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        for &b in line {
            // What: stop once the residual is the dead sink. Why: an anchored DFA dies
            // on the first non-matching byte, and walking the rest cannot accept.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Same step as the Rust statement below, written with ordinary TS objects/functions.
            // ```
            if state == dead {
                return false;
            }
            let class = self.class_map[b as usize] as usize;
            let mask = accept_bit(self.class_word[class], self.class_newline[class]);
            if self.accept[state] & mask != 0 {
                return true;
            }
            state = self.trans[state * nc + class] as usize;
        }
        // What: the end-of-input boundary has no next byte. Why: `word_after` is
        // false and `line_end` is true there.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        return self.accept[state] & accept_bit(false, true) != 0
    }

    /// Adds to `set` every byte that could begin a match from the start state.
    ///
    /// What:
    ///  marks byte `b` when the start already accepts (an empty match,
    ///  so any byte
    /// qualifies) or its start transition does not go straight to the dead sink.
    ///  Why:
    ///  a
    /// line-start rule is checked only when `line[0]` is one of these,
    ///  so a single byte
    /// test skips the anchored DFA call on almost every line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function mark_first_bytes(set: ByteSet): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn mark_first_bytes(&self, set: &mut ByteSet) {
        let nc = self.nclasses as usize;
        let start = self.start as usize;
        let start_accepts = self.accept[start] != 0;
        for b in 0u8..=u8::MAX {
            let class = self.class_map[b as usize] as usize;
            if start_accepts || self.trans[start * nc + class] as usize != self.dead as usize {
                set.insert(b);
            }
        }
    }

    /// Validates a decoded automaton so the match loop cannot read out of bounds.
    ///
    /// What:
    ///  checks every length and that all class ids,
    ///  transition targets,
    ///  and
    /// the start id are in range.
    ///  Why:
    ///  a serialized DFA may be hostile or
    /// corrupt,
    ///  and it is executed against attacker-influenced input,
    ///  so it must
    /// be proven well-formed before first use.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function validate(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), CompileError> {
        // What: a small helper to build the error. Why: keeps each check terse.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let invalid = |message: &str| return CompileError::Invalid {
            message: message.to_string(),
        };
        let nc = self.nclasses as usize;
        let ns = self.num_states as usize;
        if nc == 0 || nc > 256 {
            return Err(invalid("nclasses out of range"));
        }
        if self.class_map.len() != 256 {
            return Err(invalid("class_map must have length 256"));
        }
        if self.class_word.len() != nc || self.class_newline.len() != nc {
            return Err(invalid("class flag length mismatch"));
        }
        if self.class_map.iter().any(|&c| return c as usize >= nc) {
            return Err(invalid("class id out of range"));
        }
        if ns == 0 || self.accept.len() != ns || self.start as usize >= ns {
            return Err(invalid("state count or start out of range"));
        }
        let expected = ns
            .checked_mul(nc)
            .ok_or_else(|| return invalid("transition table size overflow"))?;
        if self.trans.len() != expected {
            return Err(invalid("transition table length mismatch"));
        }
        if self.trans.iter().any(|&t| return t as usize >= ns) {
            return Err(invalid("transition target out of range"));
        }
        // What: the dead id is either `num_states` (disabled) or a genuine sink. Why:
        // a hostile blob could name an ACCEPTING state as dead, which would make the
        // match loop early-exit false and miss a secret; require it be non-accepting
        // and fully self-looping so the early-exit is provably sound.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let dead = self.dead as usize;
        if dead != ns {
            if dead > ns {
                return Err(invalid("dead state out of range"));
            }
            if self.accept[dead] != 0 || (0..nc).any(|c| return self.trans[dead * nc + c] as usize != dead) {
                return Err(invalid("dead state is not a non-accepting sink"));
            }
        }
        return Ok(())
    }
}

/// What:
///     Unit tests for the DFA table and match loop,
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
#[path = "table_tests.rs"]
mod tests;

/// Finds a non-accepting,
///  fully self-looping sink state,
///  or `num_states` if none.
///
/// What:
///  scans for the first state whose acceptance mask is zero and whose every
/// byte-class transition returns to itself.
///  Why:
///  from such a state no input can ever
/// accept,
///  so the match loop can stop there;
///  minimization collapses all such states
/// into one,
///  and `num_states` (no real id) signals there is none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function find_dead(trans: number[], accept: Uint8Array, nclasses: number, num_states: number): number {
///   // Rust body below is the implementation.
/// }
/// ```
fn find_dead(trans: &[u16], accept: &[u8], nclasses: u32, num_states: u16) -> u16 {
    let nc = nclasses as usize;
    for state in 0..num_states as usize {
        if accept[state] == 0 && (0..nc).all(|c| return trans[state * nc + c] as usize == state) {
            return state as u16;
        }
    }
    return num_states
}
