//! What:
//!  A three-way merge of text by line,
//! and by word for the line regions that both sides changed,
//! in process.
//!  A file is merged only when every such region resolves;
//! the regions that needed the word pass are reported.
//! Why:
//!  Git's line merge calls two changes that touch (one ends where the other begins) a conflict,
//! so an edit of a line plus
//!  lines inserted right after it stops a merge that nobody needs to judge.
//! The owner chose,
//! on 2026-11-03,
//! a second pass
//!  in the way Jujutsu merges with `merge.hunk-level = "word"`:
//! re-merge the conflicted regions word by word,
//! keep the
//!  result only when every region resolves,
//! and show each region it merged.
//! The diff is the `similar` crate's Myers
//!  algorithm;
//! the controls keep the results equal to Jujutsu 0.46.0's on the stored corpus
//!  (`src/native/merge/word_merge_corpus/`,
//! `doc/planning/cli-git-merge-auto-resolution.md`).
//!  The pass can produce text neither side wrote (two edits of different arguments of one call);
//! it never drops
//!  a byte that either side added or kept.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const merged = mergeByWord({ base, ours, theirs }); // { kind: 'resolved', merged, parts } | { kind: 'conflict', unresolved }
//! ```

/// A byte range of a text.
use std::ops::Range;
/// The diff of two token lists.
use similar::{Algorithm, DiffOp, capture_diff_slices};

/// The diff algorithm.
/// Myers,
/// because the controls found it equal to Jujutsu's results on every corpus input and on
/// every seeded random edit pair,
/// while the Patience algorithm differed in one verdict.
const ALGORITHM: Algorithm = Algorithm::Myers;

/// The first byte of the range that UTF-8 uses for every byte of a multi-byte character;
/// each such byte counts as a
/// letter,
/// so a word is never cut inside a character.
const FIRST_NON_ASCII_BYTE: u8 = 0x80;

/// The byte that ends a line.
const LINE_END: u8 = b'\n';

/// How a text is cut into tokens.
#[derive(Clone, Copy)]
enum Level {
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
fn is_line_end(byte: &u8) -> bool {
    return *byte == LINE_END;
}

/// The pieces of `text` that the ranges name,
/// in order.
fn slices_of<'a>(text: &'a [u8], ranges: &[Range<usize>]) -> Vec<&'a [u8]> {
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
fn token_ranges(text: &[u8], level: Level) -> Vec<Range<usize>> {
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

/// The three texts between two places all three share,
/// and where each of them starts in the text `align` was given.
#[derive(Clone, Copy)]
struct Gap<'a> {
    /// The common ancestor's text.
    base: &'a [u8],
    /// The first side's text.
    ours: &'a [u8],
    /// The second side's text.
    theirs: &'a [u8],
    /// Where `base`,
    /// `ours` and `theirs` start in the texts `align` was given.
    starts: [usize; 3],
}

/// One piece of the three texts:
/// a token all three hold in the same order,
/// or the texts between two such tokens.
enum Hunk<'a> {
    /// A token that all three texts hold.
    Same(&'a [u8]),
    /// Text that differs somewhere.
    Gap(Gap<'a>),
}

/// What:
///  For each base token,
/// the index of the token of `other` that the diff keeps equal to it.
/// Why:
///   A base token with no match was changed or removed by that side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function matchedTokens(base: Uint8Array[], other: Uint8Array[]): (number | undefined)[];
/// ```
fn matched_tokens(base: &[&[u8]], other: &[&[u8]]) -> Vec<Option<usize>> {
    let mut matches: Vec<Option<usize>> = vec![None; base.len()];
    for operation in capture_diff_slices(ALGORITHM, base, other) {
        if let DiffOp::Equal { old_index, new_index, len } = operation {
            for offset in 0..len {
                matches[old_index + offset] = Some(new_index + offset);
            }
        }
    }
    return matches;
}

/// What:
///  Split the three texts into the tokens all three share,
/// in order,
/// and the gaps between them.
/// Why:
///   A gap where one side equals the base,
/// or both sides are equal,
/// resolves by itself;
/// the other gaps are the regions
///  that need a finer look.
///   An empty text has no tokens to share,
/// so the whole of the three is one gap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function align(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array, level: Level): Hunk[];
/// ```
fn align<'a>(base: &'a [u8], ours: &'a [u8], theirs: &'a [u8], level: Level) -> Vec<Hunk<'a>> {
    let whole: Gap<'a> = Gap { base, ours, theirs, starts: [0, 0, 0] };
    if base.is_empty() || ours.is_empty() || theirs.is_empty() {
        return vec![Hunk::Gap(whole)];
    }
    let base_ranges: Vec<Range<usize>> = token_ranges(base, level);
    let ours_ranges: Vec<Range<usize>> = token_ranges(ours, level);
    let theirs_ranges: Vec<Range<usize>> = token_ranges(theirs, level);
    let base_tokens: Vec<&[u8]> = slices_of(base, &base_ranges);
    let to_ours: Vec<Option<usize>> = matched_tokens(&base_tokens, &slices_of(ours, &ours_ranges));
    let to_theirs: Vec<Option<usize>> = matched_tokens(&base_tokens, &slices_of(theirs, &theirs_ranges));
    let mut hunks: Vec<Hunk<'a>> = Vec::new();
    let mut positions: [usize; 3] = [0, 0, 0];
    for (index, base_range) in base_ranges.iter().enumerate() {
        let (Some(ours_index), Some(theirs_index)) = (to_ours[index], to_theirs[index]) else { continue };
        let (ours_range, theirs_range): (&Range<usize>, &Range<usize>) = (&ours_ranges[ours_index], &theirs_ranges[theirs_index]);
        let gap: Gap<'a> = Gap {
            base: &base[positions[0]..base_range.start],
            ours: &ours[positions[1]..ours_range.start],
            theirs: &theirs[positions[2]..theirs_range.start],
            starts: positions,
        };
        if !(gap.base.is_empty() && gap.ours.is_empty() && gap.theirs.is_empty()) {
            hunks.push(Hunk::Gap(gap));
        }
        hunks.push(Hunk::Same(&base[base_range.clone()]));
        positions = [base_range.end, ours_range.end, theirs_range.end];
    }
    let tail: Gap<'a> = Gap {
        base: &base[positions[0]..],
        ours: &ours[positions[1]..],
        theirs: &theirs[positions[2]..],
        starts: positions,
    };
    if !(tail.base.is_empty() && tail.ours.is_empty() && tail.theirs.is_empty()) {
        hunks.push(Hunk::Gap(tail));
    }
    return hunks;
}

/// What:
///  The text of a gap that needs no choice:
/// both sides equal,
/// or one side equal to the base,
/// so the other side wins.
/// Why:
///   Any other gap has two different changes to the same place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trivial(gap: Gap): Uint8Array | undefined;
/// ```
fn trivial<'a>(gap: &Gap<'a>) -> Option<&'a [u8]> {
    if gap.ours == gap.theirs || gap.theirs == gap.base {
        return Some(gap.ours);
    }
    if gap.ours == gap.base {
        return Some(gap.theirs);
    }
    return None;
}

/// What:
///  Re-merge one line region by word,
/// then by the bytes between the words;
/// `None` unless every part resolves.
/// Why:
///   A result that merged some parts and left others in conflict would mix contexts and be hard to review,
/// so a region
///  is taken whole or not at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeGapByWord(gap: Gap): Uint8Array | undefined;
/// ```
fn merge_gap_by_word(gap: &Gap<'_>) -> Option<Vec<u8>> {
    let mut merged: Vec<u8> = Vec::new();
    for hunk in align(gap.base, gap.ours, gap.theirs, Level::Word) {
        match hunk {
            Hunk::Same(text) => merged.extend_from_slice(text),
            Hunk::Gap(word_gap) => match trivial(&word_gap) {
                Some(text) => merged.extend_from_slice(text),
                None => {
                    for part in align(word_gap.base, word_gap.ours, word_gap.theirs, Level::NonWord) {
                        match part {
                            Hunk::Same(text) => merged.extend_from_slice(text),
                            Hunk::Gap(byte_gap) => merged.extend_from_slice(trivial(&byte_gap)?),
                        }
                    }
                }
            },
        }
    }
    return Some(merged);
}

/// A run of lines of one text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineSpan {
    /// The number of the first line,
    /// counting from 1;
    /// the line after the last one for a run with no lines.
    pub first: usize,
    /// How many lines the run holds.
    pub count: usize,
}

/// One line region that both sides changed and that the word pass merged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergedPart {
    /// The region in the common ancestor.
    pub base: LineSpan,
    /// The region in the first side.
    pub ours: LineSpan,
    /// The region in the second side.
    pub theirs: LineSpan,
    /// What the region became.
    pub merged: Vec<u8>,
}

/// How a three-way merge ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WordMerge {
    /// Every region resolved.
    Resolved {
        /// The whole merged text.
        merged: Vec<u8>,
        /// The regions that needed the word pass,
        /// in text order.
        parts: Vec<MergedPart>,
    },
    /// At least one region has two different changes to the same words.
    Conflict {
        /// How many line regions did not resolve.
        unresolved: usize,
    },
}

/// What:
///  The lines of `text` that start at byte `start` and run for `length` bytes,
/// as a span.
/// Why:
///   The report names a region by line numbers so a person can open the file at it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineSpan(text: Uint8Array, start: number, length: number): LineSpan;
/// ```
fn line_span(text: &[u8], start: usize, length: usize) -> LineSpan {
    let mut before: usize = 0;
    for byte in &text[..start] {
        if *byte == LINE_END {
            before += 1;
        }
    }
    let count: usize = token_ranges(&text[start..start + length], Level::Line).len();
    return LineSpan { first: before + 1, count };
}

/// What:
///  Merge `ours` and `theirs`,
/// both derived from `base`,
/// by line,
/// and by word where both sides changed a line region.
/// Why:
///   See the module comment.
/// The result is `Resolved` only when no region keeps two different changes of the same words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeByWord(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array): WordMerge;
/// ```
pub fn merge_by_word(base: &[u8], ours: &[u8], theirs: &[u8]) -> WordMerge {
    let (merged, parts, unresolved) = walk(base, ours, theirs, None);
    if unresolved == 0 {
        return WordMerge::Resolved { merged, parts };
    }
    return WordMerge::Conflict { unresolved };
}

/// The width of a conflict marker,
/// which is Git's default for `merge.conflictStyle` (`<<<<<<<`).
const MARKER_WIDTH: usize = 7;

/// What:
///  The text of a marker line: the repeated character, a space, then the label.
/// Why:
///   Git writes a marker as `<<<<<<< <label>\n` and no label when none is given
/// (`xdl_fill_merge_buffer`,
/// `xdiff/xmerge.c`),
///  so a person's tools read it as they read Git's own markers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markerLine(character: string, label: string): Uint8Array;
/// ```
fn marker_line(character: u8, label: &str) -> Vec<u8> {
    let mut line: Vec<u8> = vec![character; MARKER_WIDTH];
    if !label.is_empty() {
        line.push(b' ');
        line.extend_from_slice(label.as_bytes());
    }
    line.push(LINE_END);
    return line;
}

/// What:
///  Merge `ours` and `theirs` by the same rules as `merge_by_word`,
/// but keep both sides of every region the word pass cannot resolve,
/// between Git's conflict markers.
/// Why:
///   A parsed edit of a file the merge changes is kept beside the merge rather than lost
/// (`package/git-policy/cli/SPEC.md`,
///  "Merges and commits never fail"),
/// and the region that holds two different changes of the same words must keep both
///  sides in the file for the person to judge.
/// Every region the pass does resolve is the bytes `merge_by_word` would write,
/// so
///  a file that resolves has no marker in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepBoth(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array, labels: [string, string]): Uint8Array;
/// ```
pub fn keep_both(base: &[u8], ours: &[u8], theirs: &[u8], labels: (&str, &str)) -> Vec<u8> {
    return walk(base, ours, theirs, Some(labels)).0;
}

/// What:
///  One walk over the line regions,
/// which resolves what it can and either counts or keeps the rest.
/// Why:
///   The verdict of the second pass and the text a person's conflicted file holds must be the same merge,
/// so one
///  walk produces both:
/// `merge_by_word` reads its count,
/// `keep_both` reads its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walk(base, ours, theirs, conflict: [string, string] | null): { merged: Uint8Array; parts: MergedPart[]; unresolved: number };
/// ```
fn walk(base: &[u8], ours: &[u8], theirs: &[u8], conflict: Option<(&str, &str)>) -> (Vec<u8>, Vec<MergedPart>, usize) {
    let mut merged: Vec<u8> = Vec::with_capacity(ours.len().max(theirs.len()));
    let mut parts: Vec<MergedPart> = Vec::new();
    let mut unresolved: usize = 0;
    for hunk in align(base, ours, theirs, Level::Line) {
        match hunk {
            Hunk::Same(text) => merged.extend_from_slice(text),
            Hunk::Gap(gap) => match trivial(&gap) {
                Some(text) => merged.extend_from_slice(text),
                None => match merge_gap_by_word(&gap) {
                    Some(text) => {
                        parts.push(MergedPart {
                            base: line_span(base, gap.starts[0], gap.base.len()),
                            ours: line_span(ours, gap.starts[1], gap.ours.len()),
                            theirs: line_span(theirs, gap.starts[2], gap.theirs.len()),
                            merged: text.clone(),
                        });
                        merged.extend_from_slice(&text);
                    }
                    None => {
                        unresolved += 1;
                        if let Some((ours_label, theirs_label)) = conflict {
                            merged.extend_from_slice(marker_line(b'<', ours_label).as_slice());
                            merged.extend_from_slice(gap.ours);
                            merged.extend_from_slice(marker_line(b'=', "").as_slice());
                            merged.extend_from_slice(gap.theirs);
                            merged.extend_from_slice(marker_line(b'>', theirs_label).as_slice());
                        }
                    }
                },
            },
        }
    }
    return (merged, parts, unresolved);
}

/// Corpus equality,
/// a random-edit differential and the properties of a merge.
#[cfg(test)]
#[path = "word_merge_tests.rs"]
mod tests;
