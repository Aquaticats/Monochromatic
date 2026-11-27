//! What:
//!  The answer for a line region that both sides only deleted lines from,
//! and the checks that keep a word merge from writing back a line,
//!  or an order of lines,
//!  that one side's deletions removed.
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
//! A base that writes one line twice is where the pass cannot tell which copy a side deleted,
//!  and the word pass can write a copy a side removed,
//!  or write the two sides' lines in an order neither holds,
//!  so such a region keeps both sides rather than resolve to an answer no union of the deletions leaves.
//! A region one side also wrote a line into is read by word,
//! and there the same pairing can still happen,
//! so an answer that writes more copies of a line the base held
//! than the copies the two sides leave is refused there too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = mergedDeletions(base, ours, theirs); // Uint8Array | undefined
//! const kept = keepsADeletedLine(base, ours, theirs, merged); // boolean
//! const pastCopies = writesPastTheCopiesTheSidesLeave(base, ours, theirs, merged); // boolean
//! const past = writesPastWhatTheSidesLeave(base, ours, theirs, merged); // boolean
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
///  The text of one line without its line end,
/// which is a line feed or a carriage return and a line feed.
/// Why:
///   A file whose lines end with a carriage return and a line feed and a side that lost its last line end
/// are the same records to a person,
/// so the bounds below read a line by this text and not by the bytes it happens to end with.
/// A last line that ends with a bare carriage return keeps it:
/// nothing here says that byte was an ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lineBody = (line: Uint8Array): Uint8Array => /* the line without "\n", and without "\r" when "\n" was there */;
/// ```
fn line_body(line: &[u8]) -> &[u8] {
    let without_feed: &[u8] = without_line_end(line);
    if without_feed.len() == line.len() {
        return without_feed;
    }
    if let Some((last, head)) = without_feed.split_last()
        && *last == b'\r'
    {
        return head;
    }
    return without_feed;
}

/// What:
///  How many times each line of `text` is written,
/// keyed by `line_body`.
/// Why:
///   The bounds over two sides that only deleted lines count the records a person reads,
/// and a carriage return before the line feed is part of the line's ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bodyCounts = (text: Uint8Array): Map<Uint8Array, number> => /* one entry per distinct line body */;
/// ```
fn body_counts(text: &[u8]) -> BTreeMap<&[u8], usize> {
    let mut counts: BTreeMap<&[u8], usize> = BTreeMap::new();
    for line in lines(text) {
        *counts.entry(line_body(line)).or_insert(0) += 1;
    }
    return counts;
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
///  Whether the lines of `merged` are some of the lines of `side`,
///  in the order `side` writes them,
/// keyed by each line's text without its line end.
/// Why:
///   A side that only deleted lines holds a run of the lines it kept,
/// so an answer that is the union of two deletions is a run of each side.
/// A side that ends its last line and a merge that does not (or the reverse)
/// are the same line here,
/// as they are to a person reading the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheLinesInOrder = (merged: Uint8Array, side: Uint8Array): boolean => /* one walk over the side's lines */;
/// ```
fn holds_the_lines_in_order(
    merged: &[u8],
    side: &[u8],
) -> bool {
    let wanted: Vec<&[u8]> = lines(merged);
    let mut at: usize = 0;
    for line in lines(side) {
        if at == wanted.len() {
            break;
        }
        if line_body(line) == line_body(wanted[at]) {
            at += 1;
        }
    }
    return at == wanted.len();
}

/// What:
///  Where each line of `base` sits,
/// keyed by the line's text without its line end.
/// Why:
///   A base may write one line more than once,
///  and a side that deleted copies of it holds a run of the places,
///  so the places of a line are kept in the order the base writes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placesOfLines = (base: Uint8Array): Map<Uint8Array, number[]> => /* one entry per distinct line body */;
/// ```
fn places_of_lines(base: &[u8]) -> BTreeMap<&[u8], Vec<usize>> {
    let mut places: BTreeMap<&[u8], Vec<usize>> = BTreeMap::new();
    for (place, line) in lines(base).into_iter().enumerate() {
        places.entry(line_body(line)).or_default().push(place);
    }
    return places;
}

/// What:
///  The places of `side`'s lines in `base`,
///  in the base's order,
/// or `None` when a line of `side` is not a line of `base`
///  or comes before the place the line before it took.
/// Why:
///   A side that only deleted lines of a base that repeats a line keeps a run of the base's places,
/// and a side that wrote a line of its own,
/// or moved one,
/// is not such a side.
/// A repeated line leaves more than one place to choose,
/// so the earliest place after the one before it is the one a deletion kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placesInARepeatingBase = (places: Map<Uint8Array, number[]>, side: Uint8Array): number[] | undefined => { let after = 0; return sideLines(side).map((line) => { const at = places.get(body(line))?.find((place) => place >= after); if (at === undefined) throw new Error('not a deletion'); after = at + 1; return at; }); };
/// ```
fn places_in_a_repeating_base(
    places: &BTreeMap<&[u8], Vec<usize>>,
    side: &[u8],
) -> Option<Vec<usize>> {
    let mut taken: Vec<usize> = Vec::new();
    let mut after: usize = 0;
    for line in lines(side) {
        let places_of_line: &Vec<usize> = places.get(line_body(line))?;
        let at: usize = places_of_line.partition_point(|place| return *place < after);
        let place: usize = *places_of_line.get(at)?;
        taken.push(place);
        after = place + 1;
    }
    return Some(taken);
}

/// What:
///  Whether `merged` writes more copies of a line the base held
///  than the side that deleted copies of it still holds,
///  while the other side left that line as it was.
/// Why:
///   The copies a side deleted are text that side asked to go,
/// and the other side kept its copies without asking for any:
///  so a merge that writes more copies than the deleting side still holds
///  has finished a conflict by itself with a copy one side deleted
/// (`package/git-policy/cli/SPEC.md`,
/// "Word-level second merge pass").
/// The side that deleted may still hold some copies of a line that repeats,
/// and a merge that writes those is right:
/// the copies that side kept are text it holds,
/// so only the copies past its own count are the ones it deleted.
/// A side that wrote more copies of a line than the base held did not leave that line as it was,
/// so only a side that holds it as often as the base did,
/// beside a side that holds it less often,
/// is the shape judged here.
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
        let held: usize = merged_counts.get(line).copied().unwrap_or(0);
        let ours_held: usize = ours_counts.get(line).copied().unwrap_or(0);
        let theirs_held: usize = theirs_counts.get(line).copied().unwrap_or(0);
        if ours_held < *in_base && theirs_held == *in_base && held > ours_held {
            return true;
        }
        if theirs_held < *in_base && ours_held == *in_base && held > theirs_held {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether `merged` writes more copies of a line the base held
///  than the two sides leave,
///  where at most one of them wrote more copies of that line than the base held.
/// Why:
///   A resolved answer replaces a conflict a person would have read,
/// so an answer that holds a copy of a line one side deleted is text that side asked to go,
/// landing with no marker
/// (`package/git-policy/cli/SPEC.md`,
/// "Word-level second merge pass").
/// The copies of a line two sides leave are at most the copies the side that kept fewer of them holds,
/// and that is the bound whenever neither side wrote a copy of its own:
/// every copy of the answer that neither side deleted is a copy both sides kept.
/// A side that wrote more copies of a line than the base held wrote text of its own,
/// so an answer may hold up to that side's copies;
/// and where both sides wrote copies of the same line no bound over the two sides is right,
/// so that line is not judged here.
/// A line the base never held is no line of the base to count:
/// either side may write it.
/// The bound is read over a whole answer because a region that holds more copies of a line than its own
/// two sides leave can be right when another region holds fewer of them,
/// and the merge that a person would have judged is the whole answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writesPastTheCopiesTheSidesLeave(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array, merged: Uint8Array): boolean;
/// ```
pub(super) fn writes_past_the_copies_the_sides_leave(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
    merged: &[u8],
) -> bool {
    let base_counts: BTreeMap<&[u8], usize> = body_counts(base);
    let ours_counts: BTreeMap<&[u8], usize> = body_counts(ours);
    let theirs_counts: BTreeMap<&[u8], usize> = body_counts(theirs);
    for (line, held) in body_counts(merged) {
        let in_base: usize = base_counts.get(line).copied().unwrap_or(0);
        let ours_held: usize = ours_counts.get(line).copied().unwrap_or(0);
        let theirs_held: usize = theirs_counts.get(line).copied().unwrap_or(0);
        if in_base == 0 || (ours_held > in_base && theirs_held > in_base) {
            continue;
        }
        let most: usize = if ours_held <= in_base && theirs_held <= in_base {
            ours_held.min(theirs_held)
        } else {
            ours_held.max(theirs_held)
        };
        if held > most {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether `merged` writes text no union of the two sides leaves:
///  more copies of a line than the sides leave,
///  or,
///  where each side only deleted lines,
///  its lines in an order one side does not hold them in.
/// Why:
///   A region whose base writes one line twice cannot tell which copy a side deleted,
/// so the word pass pairs one side's deleted line with the other side's kept line and keeps it
/// (issue 6529).
/// `keeps_a_deleted_line` refuses that where the other side left the line as it was;
/// `writes_past_the_copies_the_sides_leave` refuses the copies both sides deleted,
/// and here the word pass can also write the two sides' lines in an order neither side holds.
/// A resolved answer replaces a conflict a person would have read,
/// so a region whose two sides each only deleted lines is kept as a conflict
/// when its answer holds a copy or an order that no union of the two deletions does.
/// The copies bound is read for every shape,
/// a region one side also wrote a line into included:
/// a line that side added only lifts the bound of that line.
/// The order bound is read only where both sides only deleted lines,
/// because a side that wrote a line of its own holds an order of lines the other side does not,
/// so the orders over the two sides say nothing about a merge that is right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writesPastWhatTheSidesLeave(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array, merged: Uint8Array): boolean;
/// ```
pub(super) fn writes_past_what_the_sides_leave(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
    merged: &[u8],
) -> bool {
    if writes_past_the_copies_the_sides_leave(base, ours, theirs, merged) {
        return true;
    }
    let places: BTreeMap<&[u8], Vec<usize>> = places_of_lines(base);
    if places_in_a_repeating_base(&places, ours).is_none()
        || places_in_a_repeating_base(&places, theirs).is_none()
    {
        return false;
    }
    return !holds_the_lines_in_order(merged, ours) || !holds_the_lines_in_order(merged, theirs);
}

/// The union of two sides' deletions,
/// and the rule over a generated region of repeated lines.
#[cfg(test)]
#[path = "word_merge_deleted_lines_tests.rs"]
mod tests;
