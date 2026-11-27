//! What:
//!  A three-way merge of text by line,
//! and by word for the line regions that both sides changed,
//! in process.
//!  A text is merged only when every such region resolves;
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
//! The second pass gives this function one of Git's `--zdiff3` conflict regions at a time
//! (`src/native/merge/merge_second_pass.rs`);
//! the lines and words a text is cut into are `word_merge_tokens.rs`.
//! the controls give it whole files,
//! as the corpus stores them,
//! which is why it merges lines itself as well.
//! The diff is the `similar` crate's Myers
//!  algorithm;
//! the controls keep the results equal to Jujutsu 0.46.0's on the stored corpus
//!  (`src/native/merge/word_merge_corpus/`,
//! `doc/planning/cli-git-merge-auto-resolution.md`).
//!  The pass can produce text neither side wrote (two edits of different arguments of one call),
//! which the owner accepted on 2026-11-03;
//! nothing either side added or kept is dropped (the generated control holds it),
//! and a region whose merge would do more than that keeps both sides instead:
//! a word written twice in a row where no side writes it twice,
//! or,
//! in a region that spans lines on some side,
//! a line no side holds.
//! A region both sides only deleted lines from is not read by word at all:
//! it resolves to the
//! lines both sides kept,
//! which is the union of the two deletions
//! (`src/native/merge/word_merge_deleted_lines.rs`,
//! issue 6529).
//! A region one side also wrote a line into is read by word,
//! and a word merge that would write back more copies of a line one side deleted
//! than that side still holds is refused there.
//! A merge of two sides that only deleted lines is refused as a whole
//! when its answer writes more copies of a line than both sides hold,
//! or writes the lines in an order one side does not hold them in,
//! which no union of the two deletions does.
//! A region both sides only deleted lines from keeps both sides as well
//! when its word merge would write more copies of a line than both sides hold,
//! or write the lines in an order one side does not hold them in.
//! A region's own text is the bytes its stages hold,
//! so a region that ends a file keeps the file's own ending:
//! a file whose last line has no line end keeps none,
//! and one whose last line has a line end keeps it.
//! The owner asked on 2026-11-10 for wrong text to land less often;
//! the controls for both refusals are
//!  `src/native/merge/word_merge_refusals/`
//! and merge `ef43afe9f9` (`doc/planning/cli-git-merge-auto-resolution.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const merged = mergeByWord({ base, ours, theirs }); // { kind: 'resolved', merged, parts } | { kind: 'conflict', unresolved }
//! ```

/// The answer for a region both sides only deleted lines from,
/// and the checks that keep a line one side deleted out of a merged region.
use crate::merge::word_merge_deleted_lines::{
    keeps_a_deleted_line,
    merged_deletions,
    writes_past_the_union_of_two_deletions,
};
/// The checks that keep a word merge from writing text no side wrote.
use crate::merge::word_merge_invented_text::refuses_word_merge;
/// The lines and words a text is cut into,
/// and the algorithm the diff of two lists of them reads.
use crate::word_merge_tokens::{
    ALGORITHM,
    LINE_END,
    Level,
    slices_of,
    token_ranges,
};
/// The diff of two token lists.
use similar::{
    DiffOp,
    capture_diff_slices,
};
/// A byte range of a text.
use std::ops::Range;

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
fn matched_tokens(
    base: &[&[u8]],
    other: &[&[u8]],
) -> Vec<Option<usize>> {
    let mut matches: Vec<Option<usize>> = vec![None; base.len()];
    for operation in capture_diff_slices(ALGORITHM, base, other) {
        if let DiffOp::Equal {
            old_index,
            new_index,
            len,
        } = operation
        {
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
fn align<'a>(
    base: &'a [u8],
    ours: &'a [u8],
    theirs: &'a [u8],
    level: Level,
) -> Vec<Hunk<'a>> {
    let whole: Gap<'a> = Gap {
        base,
        ours,
        theirs,
        starts: [0, 0, 0],
    };
    if base.is_empty() || ours.is_empty() || theirs.is_empty() {
        return vec![Hunk::Gap(whole)];
    }
    let base_ranges: Vec<Range<usize>> = token_ranges(base, level);
    let ours_ranges: Vec<Range<usize>> = token_ranges(ours, level);
    let theirs_ranges: Vec<Range<usize>> = token_ranges(theirs, level);
    let base_tokens: Vec<&[u8]> = slices_of(base, &base_ranges);
    let to_ours: Vec<Option<usize>> = matched_tokens(&base_tokens, &slices_of(ours, &ours_ranges));
    let to_theirs: Vec<Option<usize>> =
        matched_tokens(&base_tokens, &slices_of(theirs, &theirs_ranges));
    let mut hunks: Vec<Hunk<'a>> = Vec::new();
    let mut positions: [usize; 3] = [0, 0, 0];
    for (index, base_range) in base_ranges.iter().enumerate() {
        let (Some(ours_index), Some(theirs_index)) = (to_ours[index], to_theirs[index]) else {
            continue;
        };
        let (ours_range, theirs_range): (&Range<usize>, &Range<usize>) =
            (&ours_ranges[ours_index], &theirs_ranges[theirs_index]);
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

///  Re-merge one line region:
/// by the union of two sides' deletions where each side only deleted lines,
/// otherwise by word and then by the bytes between the words;
/// `None` unless every part resolves.
/// Why:
///   A result that merged some parts and left others in conflict would mix contexts and be hard to review,
/// so a region
///  is taken whole or not at all.
/// A region each side only deleted lines from has one answer no side has to judge,
/// and reading it word by word loses which line is which
/// (`src/native/merge/word_merge_deleted_lines.rs`,
/// issue 6529).
/// A word merge of such a region that writes back a copy of a line one side deleted keeps both sides instead,
/// and one whose whole answer writes past what both sides hold is kept by `walk`,
/// which is where the answer's own bounds can be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeGapByWord(gap: Gap): Uint8Array | undefined;
/// ```
fn merge_gap_by_word(gap: &Gap<'_>) -> Option<Vec<u8>> {
    if let Some(deleted) = merged_deletions(gap.base, gap.ours, gap.theirs) {
        return Some(deleted);
    }
    let merged: Vec<u8> = merged_word_text(gap.base, gap.ours, gap.theirs)?;
    if keeps_a_deleted_line(gap.base, gap.ours, gap.theirs, merged.as_slice()) {
        return None;
    }
    return Some(merged);
}

/// What:
///  The merge of one line region by word and then by the bytes between the words,
///  or nothing when it would write text no side wrote.
/// Why:
///   The rule that keeps a line one side deleted out of a merge judges this text,
/// and its own control asks what that rule refuses,
/// so the text is one answer of its own
/// (`src/native/merge/word_merge_deleted_lines_tests.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergedWordText(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array): Uint8Array | undefined;
/// ```
pub(super) fn merged_word_text(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
) -> Option<Vec<u8>> {
    let mut merged: Vec<u8> = Vec::new();
    for hunk in align(base, ours, theirs, Level::Word) {
        match hunk {
            Hunk::Same(text) => merged.extend_from_slice(text),
            Hunk::Gap(word_gap) => match trivial(&word_gap) {
                Some(text) => merged.extend_from_slice(text),
                None => {
                    for part in align(
                        word_gap.base,
                        word_gap.ours,
                        word_gap.theirs,
                        Level::NonWord,
                    ) {
                        match part {
                            Hunk::Same(text) => merged.extend_from_slice(text),
                            Hunk::Gap(byte_gap) => merged.extend_from_slice(trivial(&byte_gap)?),
                        }
                    }
                }
            },
        }
    }
    if refuses_word_merge(base, ours, theirs, merged.as_slice()) {
        return None;
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
fn line_span(
    text: &[u8],
    start: usize,
    length: usize,
) -> LineSpan {
    let mut before: usize = 0;
    for byte in &text[..start] {
        if *byte == LINE_END {
            before += 1;
        }
    }
    let count: usize = token_ranges(&text[start..start + length], Level::Line).len();
    return LineSpan {
        first: before + 1,
        count,
    };
}

/// What:
///  Merge `ours` and `theirs`,
/// both derived from `base`,
/// by line,
/// and by word where both sides changed a line region.
/// Why:
///   See the module comment.
/// The result is `Resolved` only when no region keeps two different changes of the same words,
/// and the whole answer writes no copy and no order that a union of two sides that only deleted lines
///  cannot leave
/// (`writes_past_the_union_of_two_deletions`).
/// The answer is judged whole because a region's own counts cannot say what the merge as a whole holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeByWord(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array): WordMerge;
/// ```
pub fn merge_by_word(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
) -> WordMerge {
    let (merged, parts, unresolved) = walk(base, ours, theirs, None);
    if unresolved == 0 {
        return WordMerge::Resolved {
            merged,
            parts,
        };
    }
    return WordMerge::Conflict {
        unresolved,
    };
}

/// The width of a conflict marker,
/// which is Git's default for `merge.conflictStyle` (`<<<<<<<`).
const MARKER_WIDTH: usize = 7;

/// What:
///  The text of a marker line:
/// the repeated character,
/// a space,
/// then the label.
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
fn marker_line(
    character: u8,
    label: &str,
) -> Vec<u8> {
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
/// An answer `merge_by_word` refuses for writing past what both sides hold keeps both sides of every region
///  here as well,
/// so the text it would have written lands with a marker rather than as the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepBoth(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array, labels: [string, string]): Uint8Array;
/// ```
pub fn keep_both(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
    labels: (&str, &str),
) -> Vec<u8> {
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
/// An answer that writes past what both sides hold is no merge the pass may take,
/// so it walks again and keeps both sides of every region it would have resolved:
/// a refusal this walk only counts would leave `keep_both` writing the refused text with no marker in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walk(base, ours, theirs, conflict: [string, string] | null): { merged: Uint8Array; parts: MergedPart[]; unresolved: number } {
///   const taken = walkTheRegions(base, ours, theirs, conflict, false);
///   if (taken.unresolved !== 0 || !writesPastTheUnionOfTwoDeletions(base, ours, theirs, taken.merged)) return taken;
///   return walkTheRegions(base, ours, theirs, conflict, true);
/// };
/// ```
fn walk(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
    conflict: Option<(&str, &str)>,
) -> (Vec<u8>, Vec<MergedPart>, usize) {
    let (merged, parts, unresolved) = walk_the_regions(base, ours, theirs, conflict, false);
    if unresolved != 0
        || !writes_past_the_union_of_two_deletions(base, ours, theirs, merged.as_slice())
    {
        return (merged, parts, unresolved);
    }
    return walk_the_regions(base, ours, theirs, conflict, true);
}

/// What:
///  One walk over the line regions,
/// resolving each region unless `everything_kept` says a region keeps both sides.
/// Why:
///   `walk` reads the whole answer before it decides,
/// and a region's own answer cannot say whether the merge as a whole is past what both sides hold:
/// a merge of two sides that only deleted lines can put a line two sides kept in one region while another
///  region holds none of them,
/// which is still the union.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walkTheRegions(base, ours, theirs, conflict: [string, string] | null, everythingKept: boolean): { merged: Uint8Array; parts: MergedPart[]; unresolved: number };
/// ```
fn walk_the_regions(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
    conflict: Option<(&str, &str)>,
    everything_kept: bool,
) -> (Vec<u8>, Vec<MergedPart>, usize) {
    let mut merged: Vec<u8> = Vec::with_capacity(ours.len().max(theirs.len()));
    let mut parts: Vec<MergedPart> = Vec::new();
    let mut unresolved: usize = 0;
    for hunk in align(base, ours, theirs, Level::Line) {
        match hunk {
            Hunk::Same(text) => merged.extend_from_slice(text),
            Hunk::Gap(gap) => match trivial(&gap) {
                Some(text) => merged.extend_from_slice(text),
                None => match if everything_kept {
                    None
                } else {
                    merge_gap_by_word(&gap)
                } {
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
