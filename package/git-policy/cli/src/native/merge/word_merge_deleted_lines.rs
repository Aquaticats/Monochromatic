//! What:
//!  The answer for a line region that both sides only deleted lines from,
//! and the check that keeps a word merge from writing back a line a side deleted.
//! Why:
//!  A file that holds one record per line (an allow list,
//! a manifest of one entry per line)
//!  is merged by the union of the two sides' deletions when each side only deletes.
//! A word merge of such a region loses which line is which,
//!  so it pairs one side's deleted line with the other side's kept line and keeps it
//! (issue 6529,
//!  merge `b928c5fa6999`,
//!  whose allow-list region kept the `direct_fix_install` lines `main` had deleted,
//!  and whose resolution `24f8c1947b` corrected by hand).
//! A region one side also wrote a line into is read by word,
//! and there the same pairing can still happen,
//! so a word merge that writes a line the base held and one side deleted is refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = mergedDeletions(base, ours, theirs); // Uint8Array | undefined
//! const kept = keepsADeletedLine(base, ours, theirs, merged); // boolean
//! ```

/// What:
///  `BTreeMap` and `BTreeSet` are sorted maps and sets keyed by their entry,
///  so a lookup is a walk down a tree rather than a scan of every entry.
/// Why:
///   A region may hold a great many lines,
/// and a scan per line would make the pass quadratic in the region's lines;
/// `HashMap` and `HashSet` are the siblings that would answer in constant time instead,
///  and the sorted ones are chosen because `&[u8]` is their key without a hasher to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // `Map` and `Set`.
/// ```
use std::collections::{
    BTreeMap,
    BTreeSet,
};

/// What:
///  The lines of `text`,
/// each with its line end,
/// and no empty line after a final line end.
/// Why:
///   A line region's answer is written from the base's own lines,
/// so each line is kept with the bytes it had.
/// `&[u8]` is the sibling of `Vec<u8>`,
/// a borrowed slice that copies nothing,
/// and it is borrowed because the text outlives the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = (text: Uint8Array): Uint8Array[] => text.length === 0 ? [] : /* split, each line keeping its "\n" */ [];
/// ```
fn lines(text: &[u8]) -> Vec<&[u8]> {
    let mut found: Vec<&[u8]> = Vec::new();
    let mut start: usize = 0;
    for (index, byte) in text.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        found.push(&text[start..index + 1]);
        start = index + 1;
    }
    if start < text.len() {
        found.push(&text[start..]);
    }
    return found;
}

/// What:
///  The bytes of one line without its line end.
/// Why:
///   A merge may end a file with or without its last line end,
/// and a person reads those as the same line,
/// so this check tells a line from another by its text alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutLineEnd = (line: Uint8Array): Uint8Array => line[line.length - 1] === 0x0a ? line.subarray(0, -1) : line;
/// ```
fn without_line_end(line: &[u8]) -> &[u8] {
    if let Some((last, head)) = line.split_last()
        && *last == b'\n'
    {
        return head;
    }
    return line;
}

/// What:
///  How many times each line of `text` is written,
/// keyed by the line's bytes without its line end.
/// Why:
///   A line a side wrote more copies of than the base held is a line that side did not leave
/// untouched,
/// so the number of copies is what tells "left as it was" from "written again".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lineCounts = (text: Uint8Array): Map<Uint8Array, number> => /* one entry per distinct line body */;
/// ```
fn line_counts(text: &[u8]) -> BTreeMap<&[u8], usize> {
    let mut counts: BTreeMap<&[u8], usize> = BTreeMap::new();
    for line in lines(text) {
        let body: &[u8] = without_line_end(line);
        *counts.entry(body).or_insert(0) += 1;
    }
    return counts;
}

/// What:
///  Where each line of `side` stands in `base`,
/// in the order the side writes them,
/// or `None` when one line is not a line of `base` or the order is not the base's.
/// Why:
///   A side that only deleted lines holds a run of the base's lines,
/// so its places must climb;
/// a side that wrote a line of its own or moved one is not a side of deletions.
/// `base` is a map of lines to places,
/// which only a caller that proved every base line different may build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placesIn = (base: Map<Uint8Array, number>, side: Uint8Array[]): number[] => { let last = -1; return side.map((line) => { const place = base.get(line); if (place === undefined || place <= last) throw new Error('not a deletion'); last = place; return place; }); };
/// ```
fn places_in(
    base: &BTreeMap<&[u8], usize>,
    side: &[u8],
) -> Option<Vec<usize>> {
    let mut places: Vec<usize> = Vec::new();
    let mut last: Option<usize> = None;
    for line in lines(side) {
        let place: usize = *base.get(line)?;
        if let Some(before) = last
            && place <= before
        {
            return None;
        }
        places.push(place);
        last = Some(place);
    }
    return Some(places);
}

/// What:
///  The answer for a line region that both sides only deleted lines from:
/// every line of the base that both sides kept,
/// in the base's own bytes.
/// Why:
///   The union of two sides' deletions is what a person means when each side deletes
/// records from a file of one record per line,
/// and the base's bytes are the only bytes no side had to write.
/// `None` when a side wrote a line the base does not hold,
/// when a side's lines are not in the base's order,
/// or when the base writes one line twice:
///  with a repeated line the pass cannot tell which copy a side deleted,
/// so such a region is left to the word pass.
/// The line end counts as part of the line here,
/// because the answer is written from the base's lines alone:
/// a side that ends a line differently deleted no line of the base,
/// and the word pass judges that region instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergedDeletions(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array): Uint8Array | undefined;
/// ```
pub(super) fn merged_deletions(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
) -> Option<Vec<u8>> {
    let base_lines: Vec<&[u8]> = lines(base);
    let mut distinct: BTreeSet<&[u8]> = BTreeSet::new();
    let mut places: BTreeMap<&[u8], usize> = BTreeMap::new();
    for (place, line) in base_lines.iter().enumerate() {
        if !distinct.insert(line) {
            return None;
        }
        places.insert(line, place);
    }
    let kept_by_ours: BTreeSet<usize> = places_in(&places, ours)?.into_iter().collect();
    let kept_by_theirs: BTreeSet<usize> = places_in(&places, theirs)?.into_iter().collect();
    let mut merged: Vec<u8> = Vec::new();
    for (place, line) in base_lines.iter().enumerate() {
        if !kept_by_ours.contains(&place) || !kept_by_theirs.contains(&place) {
            continue;
        }
        merged.extend_from_slice(line);
    }
    return Some(merged);
}

/// What:
///  Whether `merged` writes a line the base held that one side deleted while the other
/// side left it as it was.
/// Why:
///   Such a line is text neither side holds:
/// the side that kept it did not ask for it,
/// and the side that deleted it asked for it to go,
///  so a merge that writes it has finished a conflict by itself with a line one side deleted
/// (`package/git-policy/cli/SPEC.md`,
/// "Word-level second merge pass").
/// A side that wrote more copies of a line than the base held did not leave that line as it was,
/// so only a side that holds it as often as the base did,
/// beside a side that holds it less often,
/// is the shape refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepsADeletedLine(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array, merged: Uint8Array): boolean;
/// ```
pub(super) fn keeps_a_deleted_line(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
    merged: &[u8],
) -> bool {
    let base_counts: BTreeMap<&[u8], usize> = line_counts(base);
    let ours_counts: BTreeMap<&[u8], usize> = line_counts(ours);
    let theirs_counts: BTreeMap<&[u8], usize> = line_counts(theirs);
    let merged_counts: BTreeMap<&[u8], usize> = line_counts(merged);
    for (line, in_base) in &base_counts {
        if merged_counts.get(line).copied().unwrap_or(0) == 0 {
            continue;
        }
        let ours_held: usize = ours_counts.get(line).copied().unwrap_or(0);
        let theirs_held: usize = theirs_counts.get(line).copied().unwrap_or(0);
        if ours_held < *in_base && theirs_held == *in_base {
            return true;
        }
        if theirs_held < *in_base && ours_held == *in_base {
            return true;
        }
    }
    return false;
}

/// The union of two sides' deletions,
/// and the rule over a generated region of repeated lines.
#[cfg(test)]
#[path = "word_merge_deleted_lines_tests.rs"]
mod tests;
