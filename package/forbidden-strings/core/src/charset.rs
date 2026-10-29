//! What:
//!     A 256-bit set of bytes,
//!  the leaf alphabet of the engine.
//! Why:
//!      This file is the Rust module that groups the charset implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module charset: see exported functions and types below.
//! ```

/// What:
///     Imports the serde derives so byte sets can be persisted inside a compiled program.
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
///     Number of `u64` words needed to hold one bit per possible byte value (256/64).
/// Why:
///      The program gives this fixed value a name so every caller uses the same setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WORDS: number = 4;
/// ```
const WORDS: usize = 4;

/// What:
///     Bit width of one `u64`,
///  used to split a byte value into word and bit indices.
/// Why:
///      The program gives this fixed value a name so every caller uses the same setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BITS_PER_WORD: number = 64;
/// ```
const BITS_PER_WORD: usize = 64;

/// An immutable-by-convention set of byte values,
///  stored as a 256-bit bitmap.
///
/// What:
///  `[u64; 4]` where bit `b` is set when byte `b` is a member.
///  Why:
///  byte
/// classes,
///  `.`,
///  single literals,
///  and the `\d \w \s` shorthands are all just
/// byte sets;
///  membership and complement are O(1) word operations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ByteSet {
    /// What:
    ///     Bitmap words;
    ///  `words[b / 64]`'s bit `b % 64` is set iff `b` is present.
    /// Why:
    ///      `words` stores bitmap words;
    ///  `words[b / 64]`'s bit `b % 64` is set iff `b` is
    ///          present,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// words: number[];
    /// ```
    words: [u64; WORDS],
}

/// What:
///     Construction,
///  mutation,
///  and queries for a `ByteSet`.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl ByteSet {
    /// Builds the empty set.
    ///
    /// What:
    ///  all words zero.
    ///  Why:
    ///  the starting point every class builds up from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function empty(): Self {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn empty() -> Self {
        return ByteSet { words: [0; WORDS] }
    }

    /// Builds the full set of every byte value.
    ///
    /// What:
    ///  all 256 bits set.
    ///  Why:
    ///  the conservative first-byte set for a back-end
    /// whose first bytes are not enumerated,
    ///  so its line-start check is never skipped.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function all_bytes(): Self {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn all_bytes() -> Self {
        return ByteSet::empty().negate()
    }

    /// Adds a single byte to the set.
    ///
    /// What:
    ///  sets the bit for `b`.
    ///  Why:
    ///  literals and explicit class members.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function insert(b: number): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn insert(&mut self, b: u8) {
        // What: locate the word and bit for `b`, then OR the bit in.
        // Why: standard bitmap set; `as usize` widens the byte for indexing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let idx = b as usize;
        self.words[idx / BITS_PER_WORD] |= 1u64 << (idx % BITS_PER_WORD);
    }

    /// Adds every byte in the inclusive range `lo..=hi` to the set.
    ///
    /// What:
    ///  iterates the range and inserts each.
    ///  Why:
    ///  class ranges like `a-z`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function insert_range(lo: number, hi: number): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn insert_range(&mut self, lo: u8, hi: u8) {
        // What: `for b in lo..=hi` walks the inclusive byte range.
        // Why: small, bounded (at most 256), so a direct loop is clearest.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        for b in lo..=hi {
            self.insert(b);
        }
    }

    /// Merges another set into this one in place.
    ///
    /// What:
    ///  bitwise-ORs each word.
    ///  Why:
    ///  combining shorthands and members inside
    /// one character class.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function union_with(other: ByteSet): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn union_with(&mut self, other: &ByteSet) {
        // What: OR matching words. Why: union of two bitmaps.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        for i in 0..WORDS {
            self.words[i] |= other.words[i];
        }
    }

    /// Returns the complement of this set over all 256 byte values.
    ///
    /// What:
    ///  bitwise-NOT each word.
    ///  Why:
    ///  negated classes `[^...]` and the
    /// negated shorthands `\D \W \S`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function negate(): ByteSet {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn negate(&self) -> ByteSet {
        // What: NOT every word; all 256 bits are meaningful so no masking needed.
        // Why: the universe is exactly 256 bytes, matching the bitmap width.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let mut out = ByteSet::empty();
        for i in 0..WORDS {
            out.words[i] = !self.words[i];
        }
        return out
    }

    /// Reports whether a byte is a member.
    ///
    /// What:
    ///  tests the bit for `b`.
    ///  Why:
    ///  the leaf step of a derivative on a class.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function contains(b: number): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn contains(&self, b: u8) -> bool {
        let idx = b as usize;
        return (self.words[idx / BITS_PER_WORD] >> (idx % BITS_PER_WORD)) & 1 == 1
    }

    /// Reports whether the set has no members.
    ///
    /// What:
    ///  all words zero.
    ///  Why:
    ///  an empty class is the `Fail` language.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_empty(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        return self.words.iter().all(|w| return *w == 0)
    }

    /// Returns the sole member byte when the set holds exactly one.
    ///
    /// What:
    ///  `Some(b)` iff one bit is set across all words,
    ///  else `None`.
    ///  Why:
    ///  a
    /// singleton class is a literal byte,
    ///  which the prefilter chains into a required
    /// literal seed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function as_singleton(): number | null {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn as_singleton(&self) -> Option<u8> {
        let total: u32 = self.words.iter().map(|w| return w.count_ones()).sum();
        if total != 1 {
            return None;
        }
        // What: locate the one set bit. Why: its global index is the byte value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        for (i, &w) in self.words.iter().enumerate() {
            if w != 0 {
                return Some((i * BITS_PER_WORD + w.trailing_zeros() as usize) as u8);
            }
        }
        return None
    }
}

/// Reports whether a byte is an ASCII word character `[A-Za-z0-9_]`.
///
/// What:
///  the predicate shared by `\w`,
///  `\b`,
///  and byte-class equivalence.
///  Why:
/// one definition keeps the shorthand,
///  the boundary assertion,
///  and class
/// signatures consistent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function is_word_byte(b: number): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn is_word_byte(b: u8) -> bool {
    return b.is_ascii_alphanumeric() || b == b'_'
}

/// Builds the set matched by `.`:
///  every byte except the newline.
///
/// What:
///  full universe minus `\n`.
///  Why:
///  dot is non-dotall,
///  matching any single
/// byte that is not a line terminator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dot_set(): ByteSet {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn dot_set() -> ByteSet {
    let mut set = ByteSet::empty();
    set.insert_range(0, 255);
    // What: clear the newline bit. Why: `.` must not cross a line boundary.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut nl = ByteSet::empty();
    nl.insert(b'\n');
    set = intersect_complement(&set, &nl);
    return set
}

/// Returns `a` with every member of `b` removed.
///
/// What:
///  `a AND NOT b` per word.
///  Why:
///  used to carve `\n` out of the dot set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function intersect_complement(a: ByteSet, b: ByteSet): ByteSet {
///   // Rust body below is the implementation.
/// }
/// ```
fn intersect_complement(a: &ByteSet, b: &ByteSet) -> ByteSet {
    let mut out = ByteSet::empty();
    for i in 0..WORDS {
        out.words[i] = a.words[i] & !b.words[i];
    }
    return out
}

/// Builds the `\d` set,
///  ASCII digits `[0-9]`.
///
/// What:
///  inserts the digit range.
///  Why:
///  the `\d` shorthand and its negation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function digit_set(): ByteSet {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn digit_set() -> ByteSet {
    let mut set = ByteSet::empty();
    set.insert_range(b'0', b'9');
    return set
}

/// Builds the `\w` set,
///  ASCII word characters `[A-Za-z0-9_]`.
///
/// What:
///  letters,
///  digits,
///  underscore.
///  Why:
///  the `\w` shorthand and its negation,
/// kept identical to `is_word_byte`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function word_set(): ByteSet {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn word_set() -> ByteSet {
    let mut set = ByteSet::empty();
    set.insert_range(b'A', b'Z');
    set.insert_range(b'a', b'z');
    set.insert_range(b'0', b'9');
    set.insert(b'_');
    return set
}

/// Builds the `\s` set,
///  ASCII whitespace `[ \t\n\r\x0c\x0b]`.
///
/// What:
///  space,
///  tab,
///  newline,
///  carriage return,
///  form feed,
///  vertical tab.
///  Why:
///  the
/// `\s` shorthand and its negation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function space_set(): ByteSet {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn space_set() -> ByteSet {
    let mut set = ByteSet::empty();
    set.insert(b' ');
    set.insert(b'\t');
    set.insert(b'\n');
    set.insert(b'\r');
    set.insert(0x0c);
    set.insert(0x0b);
    return set
}

/// Builds a singleton set holding exactly one byte.
///
/// What:
///  empty set plus `b`.
///  Why:
///  every literal byte becomes a one-member class.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function singleton(b: number): ByteSet {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn singleton(b: u8) -> ByteSet {
    let mut set = ByteSet::empty();
    set.insert(b);
    return set
}

/// What:
///     Unit tests for the byte-set primitives,
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
#[path = "charset_tests.rs"]
mod tests;
