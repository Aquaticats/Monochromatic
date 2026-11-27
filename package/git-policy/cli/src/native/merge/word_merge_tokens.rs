//! What:
//!  The lines and the words a text is cut into,
//!  and the diff algorithm that reads two lists of them.
//! Why:
//!   The word merge cuts a text by line,
//!  by word and by the bytes between words,
//!  and the check that keeps a merge from writing text no side wrote cuts the same way;
//!  one file holds that cutting,
//!  so the two cannot read a text differently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ranges = tokenRanges(text, Level::Word);
//! ```

/// The diff of two token lists.
use similar::Algorithm;
/// A byte range of a text.
use std::ops::Range;

/// The diff algorithm.
/// Myers,
/// because the controls found it equal to Jujutsu's results on every corpus input and on
/// every seeded random edit pair,
/// while the Patience algorithm differed in one verdict.
pub(super) const ALGORITHM: Algorithm = Algorithm::Myers;

/// The first byte of the range that UTF-8 uses for every byte of a multi-byte character;
/// each such byte counts as a
/// letter,
/// so a word is never cut inside a character.
const FIRST_NON_ASCII_BYTE: u8 = 0x80;

/// The byte that ends a line.
pub(super) const LINE_END: u8 = b'\n';

/// How a text is cut into tokens.
#[derive(Clone, Copy)]
pub(super) enum Level {
    /// Lines,
    /// each with its line end.
    Line,
    /// Runs of letters,
    /// digits,
    /// underscores and non-ASCII bytes.
    Word,
    /// Every byte that is not part of a word,
    /// one token each.
    NonWord,
}

/// Whether the byte belongs to a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isWordByte = (byte: number) => /[A-Za-z0-9_\x80-\xff]/.test(String.fromCharCode(byte));
/// ```
fn is_word_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'_' || byte >= FIRST_NON_ASCII_BYTE;
}

/// Whether one byte ends a line.
/// `&u8` borrows the byte,
/// as `split_inclusive` hands it over.
pub(super) fn is_line_end(byte: &u8) -> bool {
    return *byte == LINE_END;
}

/// The pieces of `text` that the ranges name,
/// in order.
pub(super) fn slices_of<'a>(
    text: &'a [u8],
    ranges: &[Range<usize>],
) -> Vec<&'a [u8]> {
    let mut pieces: Vec<&'a [u8]> = Vec::new();
    for range in ranges {
        pieces.push(&text[range.clone()]);
    }
    return pieces;
}

/// What:
///  The byte ranges of the tokens of `text` at `level`.
/// Why:
///   Words are the runs that `is_word_byte` accepts and the bytes between them are not tokens of their own,
/// so a
///  word-level diff sees only the words;
/// the non-word level then sees only the bytes between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tokenRanges(text: Uint8Array, level: Level): [number, number][];
/// ```
pub(super) fn token_ranges(
    text: &[u8],
    level: Level,
) -> Vec<Range<usize>> {
    match level {
        Level::Line => {
            let mut start: usize = 0;
            let mut ranges: Vec<Range<usize>> = Vec::new();
            for line in text.split_inclusive(is_line_end) {
                let range: Range<usize> = start..start + line.len();
                start = range.end;
                ranges.push(range);
            }
            return ranges;
        }
        Level::Word => {
            let mut ranges: Vec<Range<usize>> = Vec::new();
            let mut word_start: Option<usize> = None;
            for (index, byte) in text.iter().enumerate() {
                match (word_start, is_word_byte(*byte)) {
                    (None, true) => word_start = Some(index),
                    (Some(start), false) => {
                        ranges.push(start..index);
                        word_start = None;
                    }
                    _ => {}
                }
            }
            if let Some(start) = word_start {
                ranges.push(start..text.len());
            }
            return ranges;
        }
        Level::NonWord => {
            let mut ranges: Vec<Range<usize>> = Vec::new();
            for (index, byte) in text.iter().enumerate() {
                if !is_word_byte(*byte) {
                    ranges.push(index..index + 1);
                }
            }
            return ranges;
        }
    }
}
