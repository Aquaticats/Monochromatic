//! A bounded set of live repetition counts stored as a bitset.
//!
//! What:
//!  [`CountSet`] holds the set of active counts of one `Counted` element as a
//! bit per count value (`bit i` set means count `i` is live),
//!  bounded by the
//! element's `max`.
//!  Why:
//!  a `Counted` element's counts only ever lie in `[0, max]`,
//! so a fixed-width bitset replaces a heap `BTreeSet`;
//!  entry is one bit-or,
//!  the
//! exit guard is one shift-and-test,
//!  and a matched byte advances every count at
//! once with a single multi-word left shift,
//!  with no per-byte allocation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module countset: see exported functions and types below.
//! ```

/// Live counts of one counted element as a bitset,
///  one bit per count value.
///
/// What:
///  `words` is a little-endian bit array where bit `p` (word `p / 64`,
///  bit
/// `p % 64`) marks count `p` as live.
///  Why:
///  counts stay in `[0, max]`,
///  so the array
/// is sized once and reused;
///  set operations become word ops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CountSet = {
///   // fields documented in Rust above
/// };
/// ```
pub(crate) struct CountSet {
    /// What:
    ///     Bit array of live counts,
    ///  sized to address bit `max + 1`.
    /// Why:
    ///      `words` stores bit array of live counts,
    ///  sized to address bit `max + 1`,
    ///  so
    ///          matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// words: number[];
    /// ```
    words: Vec<u64>,
}

/// What:
///     Construction and the bounded-count set operations the simulation needs.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl CountSet {
    /// Builds an empty set able to hold counts in `[0, max]`.
    ///
    /// What:
    ///  zeroed words sized to address bit `max + 1`.
    ///  Why:
    ///  a left shift can
    /// momentarily set bit `max + 1`,
    ///  which the advance then clears,
    ///  so that bit
    /// must be representable.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function new(max: number): CountSet {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn new(max: usize) -> CountSet {
        return CountSet {
            words: vec![0u64; nwords(max)],
        }
    }

    /// Empties the set in place without reallocating.
    ///
    /// What:
    ///  zeroes every word.
    ///  Why:
    ///  a reused buffer is cleared before the byte
    /// step refills it,
    ///  which is what removes per-byte allocation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function clear(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn clear(&mut self) {
        self.words.iter_mut().for_each(|word| *word = 0);
    }

    /// Adds count zero,
    ///  reporting whether the set changed.
    ///
    /// What:
    ///  sets bit 0.
    ///  Why:
    ///  entering a counted element seeds a fresh repetition
    /// at count 0;
    ///  the changed flag drives the closure fixpoint.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function insert_zero(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn insert_zero(&mut self) -> bool {
        let changed = self.words[0] & 1 == 0;
        self.words[0] |= 1;
        return changed
    }

    /// Reports whether any live count is at least `min`.
    ///
    /// What:
    ///  tests every bit at position `>= min`.
    ///  Why:
    ///  counts never exceed `max`,
    /// so "at least `min`" is exactly the exit guard's `[min, max]` membership.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function has_at_least(min: number): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn has_at_least(&self, min: usize) -> bool {
        let wi = min / 64;
        if wi >= self.words.len() {
            return false;
        }
        if self.words[wi] >> (min % 64) != 0 {
            return true;
        }
        return self.words[wi + 1..].iter().any(|&word| return word != 0)
    }

    /// Reports whether no count is live.
    ///
    /// What:
    ///  every word is zero.
    ///  Why:
    ///  a dead counted element contributes nothing,
    /// part of the thread-prune test.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_empty(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn is_empty(&self) -> bool {
        return self.words.iter().all(|&word| return word == 0)
    }

    /// Sets this set to `src` with every count advanced by one,
    ///  capped at `max`.
    ///
    /// What:
    ///  a multi-word left shift of `src` into `self`,
    ///  then drop the count that
    /// would exceed `max`.
    ///  Why:
    ///  a matched byte advances all live repetitions at
    /// once;
    ///  a count already at `max` has exited and is dropped.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function copy_advanced_from(src: CountSet, max: number): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(crate) fn copy_advanced_from(&mut self, src: &CountSet, max: usize) {
        let mut carry = 0u64;
        for (dst, &word) in self.words.iter_mut().zip(&src.words) {
            *dst = (word << 1) | carry;
            carry = word >> 63;
        }
        self.clear_above(max);
    }

    /// Clears the single stray count at `max + 1` a left shift can create.
    ///
    /// What:
    ///  clears bit `max + 1`.
    ///  Why:
    ///  counts stay in `[0, max]` by induction,
    ///  so
    /// after a shift only that one position can be out of range.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function clear_above(max: number): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn clear_above(&mut self, max: usize) {
        let pos = max + 1;
        let wi = pos / 64;
        if wi < self.words.len() {
            self.words[wi] &= !(1u64 << (pos % 64));
        }
    }
}

/// Returns the word count needed to address bit `max + 1`.
///
/// What:
///  enough 64-bit words for positions `0..=max + 1`.
///  Why:
///  sizing once here
/// keeps every `CountSet` of one element identically shaped for word-wise ops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nwords(max: number): number {
///   // Rust body below is the implementation.
/// }
/// ```
fn nwords(max: usize) -> usize {
    return (max + 1) / 64 + 1
}

/// What:
///     Unit tests for the bounded-count bitset,
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
#[path = "countset_tests.rs"]
mod tests;
