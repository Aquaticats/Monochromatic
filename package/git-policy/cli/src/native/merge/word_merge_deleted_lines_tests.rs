//! Controls for a region each side only deleted lines from:
//! the union of the two deletions,
//! a generated search for a correct merge of lines that repeat
//! that the rule keeping a line one side deleted out of a merge refuses,
//! and a generated search for an answer past what both sides hold,
//! with the cases the alpha.49 release review found it in.

use super::{
    keeps_a_deleted_line,
    line_body,
    line_counts,
    lines,
};
use crate::merge::word_merge::{
    WordMerge,
    keep_both,
    merge_by_word,
    merged_word_text,
};
use std::collections::BTreeMap;

/// The records a generated region is written from,
/// without their line ends.
/// Why:
///   Three records over at most four lines write one twice often,
///  which is the shape `merged_deletions` leaves to the word pass,
///  and the shape the rule over a deleted line reads by how many copies it finds.
const REPEATED_RECORDS: &[&str] = &["alpha", "beta", "gamma"];

/// The seed of the generated regions,
/// so a failure repeats.
const SEED: u64 = 20_261_127;

/// How many generated regions the search reads.
const CASES: usize = 600;

/// How many lines a generated region holds,
/// at fewest.
const LEAST_LINES: usize = 2;

/// How many lines a generated region holds,
/// at most.
const MOST_LINES: usize = 4;

/// The multiplier and increment of the linear congruential generator (Numerical Recipes).
const MULTIPLIER: u64 = 1_664_525;
const INCREMENT: u64 = 1_013_904_223;

/// The next number below `bound`.
fn next(
    state: &mut u64,
    bound: usize,
) -> usize {
    *state = (state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT)) & u64::from(u32::MAX);
    return usize::try_from(*state >> 8).expect("fits") % bound;
}

/// What:
///  The places in the base one side of a generated region keeps,
///  in the base's order.
/// Why:
///   A side that only deletes holds some of the base's lines in their order;
///  at least one line is kept and at least one is gone,
///  so both sides change the region and the union rule has two deletions to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptPlaces(state, size) { const mask = 1 + next(state, (1 << size) - 2); return [...Array(size).keys()].filter((place) => mask & (1 << place)); }
/// ```
fn kept_places(
    state: &mut u64,
    size: usize,
) -> Vec<usize> {
    let masks: usize = (1 << size) - 2;
    let mask: usize = 1 + next(state, masks);
    let mut places: Vec<usize> = Vec::new();
    for place in 0..size {
        if mask & (1 << place) != 0 {
            places.push(place);
        }
    }
    return places;
}

/// What:
///  The lines `places` keeps of `records`,
///  joined with the line end every line but the last has.
/// Why:
///   A region at the end of a file whose last line has no line end holds exactly this run,
///  which is where the answer and the file differ by that byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runOfRecords = (records, places) => places.map((place) => records[place]).join('\n');
/// ```
fn run_of_records(
    records: &[&str],
    places: &[usize],
) -> String {
    let kept: Vec<&str> = places.iter().map(|place| return records[*place]).collect();
    return kept.join("\n");
}

/// What:
///  Whether `merged` holds as many copies of each line as the union of the two sides' deletions can leave.
/// Why:
///   Each side deleted some copies of a line,
/// and one copy can be one both sides deleted:
/// a sound answer holds as few as both deletions together leave,
/// and as many as the larger deletion alone leaves.
/// A count outside those bounds is no union of these two deletions,
/// so only an answer inside them is what this control asks the rule about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isUnion = (base, ours, theirs, merged) => countLines(base).every((line, inBase) => { const o = countLines(ours).get(line) ?? 0; const t = countLines(theirs).get(line) ?? 0; const held = countLines(merged).get(line) ?? 0; return held >= Math.max(0, o + t - inBase) && held <= Math.min(o, t); });
/// ```
fn is_union_of_the_deletions(
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
        let ours_held: usize = ours_counts.get(line).copied().unwrap_or(0);
        let theirs_held: usize = theirs_counts.get(line).copied().unwrap_or(0);
        let fewest: usize = (ours_held + theirs_held).saturating_sub(*in_base);
        let most: usize = ours_held.min(theirs_held);
        let held: usize = merged_counts.get(line).copied().unwrap_or(0);
        if held < fewest || held > most {
            return false;
        }
    }
    return true;
}

/// What:
///  No merge that is the union of the two sides' deletions of a region of repeated lines
///  is refused by the rule that keeps a line one side deleted out of a merge.
/// Why:
///   The rule asked whether the merge holds such a line at all,
/// and a region whose line repeats can hold a copy of it and still be right,
/// because the side that deleted a line of its own still holds the other copy.
/// So the search draws regions of repeated records,
/// whose last line has no line end,
/// deletes records from each side,
/// and reads the merge the word pass would take before that rule:
/// seed `SEED`,
/// 600 cases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let case_ = 0; case_ < CASES; case_ += 1) { const base = draw(); const ours = drop(base); const theirs = drop(base); const merged = mergedWordText(base, ours, theirs); if (!merged || !isUnion(base, ours, theirs, merged)) continue; expect(keepsADeletedLine(base, ours, theirs, merged)).toBe(false); }
/// ```
#[test]
fn no_union_of_repeated_lines_is_refused() {
    let mut state: u64 = SEED;
    let mut correct: usize = 0;
    let mut repeating: usize = 0;
    for case in 0..CASES {
        let size: usize = LEAST_LINES + next(&mut state, MOST_LINES - LEAST_LINES + 1);
        let records: Vec<&str> = (0..size)
            .map(|_| {
                return REPEATED_RECORDS[next(&mut state, REPEATED_RECORDS.len())];
            })
            .collect();
        let base: String = records.join("\n");
        let ours: String =
            run_of_records(records.as_slice(), kept_places(&mut state, size).as_slice());
        let theirs: String =
            run_of_records(records.as_slice(), kept_places(&mut state, size).as_slice());
        let Some(merged): Option<Vec<u8>> =
            merged_word_text(base.as_bytes(), ours.as_bytes(), theirs.as_bytes())
        else {
            // The refusal that keeps invented text out of a region answered this one, so the rule under
            // test never reads it.
            continue;
        };
        if !is_union_of_the_deletions(
            base.as_bytes(),
            ours.as_bytes(),
            theirs.as_bytes(),
            merged.as_slice(),
        ) {
            continue;
        }
        correct += 1;
        if records.iter().enumerate().any(|(place, record)| {
            return records[..place].contains(record);
        }) {
            repeating += 1;
        }
        assert!(
            !keeps_a_deleted_line(
                base.as_bytes(),
                ours.as_bytes(),
                theirs.as_bytes(),
                merged.as_slice(),
            ),
            "case {case} of seed {SEED}: the union of these two deletions was refused: \
             base {base:?}, ours {ours:?}, theirs {theirs:?}, merged {:?}",
            String::from_utf8_lossy(merged.as_slice())
        );
    }
    assert!(
        correct > 0,
        "the search found no region whose merge is the union of the two deletions"
    );
    assert!(
        repeating > 0,
        "the search found no such region whose base writes a line twice"
    );
}
/// What:
///  A merge of a region of repeated lines that holds the copies the deleting side still holds is not refused.
/// Why:
///   This is the union the generated search found refused (its case 50):
///  the base writes `beta` twice,
///  this side keeps both,
///  the other side deleted one and still holds the other,
///  so a merge that writes one `beta` wrote a copy both sides hold.
/// The old test was whether the merge holds the line at all,
/// and it refused this one.
#[test]
fn a_union_that_keeps_the_copy_a_side_still_holds_is_not_refused() {
    let base: &[u8] = b"alpha\nbeta\nalpha\nbeta";
    let ours: &[u8] = b"beta\nalpha\nbeta";
    let theirs: &[u8] = b"alpha\nbeta";
    assert!(
        !keeps_a_deleted_line(base, ours, theirs, b"beta"),
        "one beta is the copy the second side still holds"
    );
}

/// What:
///  A merge of the same region that writes more copies of the line than the deleting side holds is refused.
/// Why:
///   The second side deleted one `beta` and kept one,
/// so a merge that writes two wrote back the copy it deleted,
/// which is the record issue 6529 reports coming back.
#[test]
fn a_copy_the_deleting_side_lost_is_still_refused() {
    let base: &[u8] = b"alpha\nbeta\nalpha\nbeta";
    let ours: &[u8] = b"beta\nalpha\nbeta";
    let theirs: &[u8] = b"alpha\nbeta";
    assert!(
        keeps_a_deleted_line(base, ours, theirs, b"beta\nbeta"),
        "two betas are one more than the second side holds"
    );
}

/// The seed of the search over two sides that only delete lines,
/// so a failure names the region it read.
const PAST_BOTH_SIDES_SEED: u64 = 20_261_128;

/// How many generated regions that search reads.
const PAST_BOTH_SIDES_CASES: usize = 400;

/// What:
///  The records `places` keeps of `records`,
///  joined with a line end after each,
///  so the run ends its file with one.
/// Why:
///   The search below reads whole files of records whose last line ends,
///  which is the shape an allow list or a manifest has;
///  a region that ends a file with no line end is `run_of_records`'s own case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runOfRecordsEnding = (records, places) => `${runOfRecords(records, places)}\n`;
/// ```
fn run_of_records_ending(
    records: &[&str],
    places: &[usize],
) -> String {
    return format!("{}\n", run_of_records(records, places));
}

/// What:
///  How many copies of each line `text` writes,
///  keyed by `line_body`,
///  which is what this control reads a line by.
/// Why:
///   The control counts the records a person reads,
/// and a file whose lines end with a carriage return and a line feed holds the same records
/// as one that lost its last line end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const countBodies = (text: Uint8Array): Map<Uint8Array, number> => /* one entry per distinct line body */;
/// ```
fn count_bodies(text: &[u8]) -> BTreeMap<&[u8], usize> {
    let mut counts: BTreeMap<&[u8], usize> = BTreeMap::new();
    for line in lines(text) {
        *counts.entry(line_body(line)).or_insert(0) += 1;
    }
    return counts;
}

/// What:
///  Whether the lines of `merged` are some of the lines of `side`,
///  in the order `side` writes them,
/// keyed by `line_body`.
/// Why:
///   A side that only deleted lines keeps its own lines in their order,
/// so an answer that is the union of two deletions is a run of each side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isARunOf = (merged: Uint8Array, side: Uint8Array): boolean => /* one walk over the side's lines */;
/// ```
fn is_a_run_of(
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
///  The first way `merged` writes past what both sides hold,
///  or nothing when it does not.
/// Why:
///   A side that only deleted lines holds some of them,
///  so an answer that is a union of the two deletions holds no more copies of a line
///  than either side holds of it,
///  and holds its lines in an order both sides hold them in.
/// The two bounds are what the alpha.49 release review judged answers by,
/// and what the search below asserts on every resolved answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pastWhatBothSidesHold(ours: Uint8Array, theirs: Uint8Array, merged: Uint8Array): string | undefined;
/// ```
fn past_what_both_sides_hold(
    ours: &[u8],
    theirs: &[u8],
    merged: &[u8],
) -> Option<String> {
    let ours_counts: BTreeMap<&[u8], usize> = count_bodies(ours);
    let theirs_counts: BTreeMap<&[u8], usize> = count_bodies(theirs);
    for (line, held) in count_bodies(merged) {
        let both_hold: usize = ours_counts
            .get(line)
            .copied()
            .unwrap_or(0)
            .min(theirs_counts.get(line).copied().unwrap_or(0));
        if held > both_hold {
            return Some(format!(
                "the line {:?} appears {held} times, more than the {both_hold} both sides hold",
                String::from_utf8_lossy(line)
            ));
        }
    }
    if !is_a_run_of(merged, ours) {
        return Some(String::from(
            "the answer's lines are not in the order this side holds them",
        ));
    }
    if !is_a_run_of(merged, theirs) {
        return Some(String::from(
            "the answer's lines are not in the order the other side holds them",
        ));
    }
    return None;
}

/// What:
///  No answer the merge resolves for two sides that only deleted lines
///  writes past what both sides hold.
/// Why:
///   A resolved answer replaces a conflict a person would have read,
///  and the owner asked on 2026-11-10 for wrong text to land less often
/// (`src/native/merge/word_merge.rs`,
/// the module's own words).
/// A base that writes a line twice is where the merge cannot tell which copy a side deleted,
/// so that is where an answer can hold a copy a side removed,
/// or hold two lines in an order one side does not.
/// The search draws such files,
/// deletes records from each side,
/// and reads every answer the merge resolves:
/// seed `PAST_BOTH_SIDES_SEED`,
/// 400 cases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const { base, ours, theirs } of draw(SEED, CASES)) { const merged = mergeByWord(base, ours, theirs); if (merged.kind !== 'resolved') continue; expect(pastWhatBothSidesHold(ours, theirs, merged.merged)).toBeUndefined(); }
/// ```
#[test]
fn no_answer_writes_past_what_both_sides_hold() {
    let mut state: u64 = PAST_BOTH_SIDES_SEED;
    let mut resolved: usize = 0;
    for case in 0..PAST_BOTH_SIDES_CASES {
        let size: usize = LEAST_LINES + next(&mut state, MOST_LINES - LEAST_LINES + 1);
        let records: Vec<&str> = (0..size)
            .map(|_| {
                return REPEATED_RECORDS[next(&mut state, REPEATED_RECORDS.len())];
            })
            .collect();
        let base: String = format!("{}\n", records.join("\n"));
        let ours: String =
            run_of_records_ending(records.as_slice(), kept_places(&mut state, size).as_slice());
        let theirs: String =
            run_of_records_ending(records.as_slice(), kept_places(&mut state, size).as_slice());
        let WordMerge::Resolved {
            merged,
            ..
        } = merge_by_word(base.as_bytes(), ours.as_bytes(), theirs.as_bytes())
        else {
            continue;
        };
        resolved += 1;
        assert_eq!(
            past_what_both_sides_hold(ours.as_bytes(), theirs.as_bytes(), merged.as_slice()),
            None,
            "case {case} of seed {PAST_BOTH_SIDES_SEED}: the answer wrote past what both sides hold: \
             base {base:?}, ours {ours:?}, theirs {theirs:?}, merged {:?}",
            String::from_utf8_lossy(merged.as_slice())
        );
    }
    assert!(
        resolved > 0,
        "the search resolved no answer of two sides that only deleted lines"
    );
}

/// What:
///  The smallest case where both sides only deleted a line the base wrote twice keeps both sides.
/// Why:
///   The base writes `a` twice,
///  this side deleted one `a` and the other deleted both,
///  so no union of the two deletions holds an `a` at all.
/// The word pass joined one side's deleted `a` with the other side's `b` and resolved the file to `a`,
/// which is the alpha.49 release review's smallest case
/// (`merge2-demonstrations.txt`,
///  `union-deleted-lines-repeated-both-sides-delete`).
#[test]
fn the_smallest_pair_of_deletions_of_a_repeated_line_keeps_both_sides() {
    let base: &[u8] = b"b\na\na\n";
    let ours: &[u8] = b"a\n";
    let theirs: &[u8] = b"b\n";
    assert_eq!(
        merge_by_word(base, ours, theirs),
        WordMerge::Conflict {
            unresolved: 1
        },
        "the other side deleted every a, so no answer holds one"
    );
}

/// What:
///  The region the alpha.49 release review found the rule's own shape in,
///  counted over the whole file,
///  keeps both sides.
/// Why:
///   The file is the review's case `case-00423` of its seed 1:
///  the whole file puts this side's `b` once against the base's and the other side's twice,
///  which is the shape the rule refuses,
///  while the region Git left holds the other side's `b` once and the base's twice,
///  so the rule counted inside the region and did not fire.
///   The region's own texts are below as well,
///  because that is what the second pass hands the rule.
#[test]
fn the_file_the_rule_s_own_shape_was_counted_past_keeps_both_sides() {
    let base: &[u8] = b"a\na\na\na\nb\nb\na";
    let ours: &[u8] = b"a\na\nb\na\n";
    let theirs: &[u8] = b"a\na\nb\nb";
    assert_eq!(
        merge_by_word(base, ours, theirs),
        WordMerge::Conflict {
            unresolved: 1
        },
        "the answer wrote the other side's second b, which this side deleted"
    );
    // What Git's `--zdiff3` line merge left as the region of that file, and what it settled.
    let region_base: &[u8] = b"a\na\nb\nb\na\n";
    let region_ours: &[u8] = b"a\n";
    let region_theirs: &[u8] = b"b\n";
    assert_eq!(
        merge_by_word(region_base, region_ours, region_theirs),
        WordMerge::Conflict {
            unresolved: 1
        },
        "the region's answer wrote a b, which this side holds none of"
    );
}

/// What:
///  An answer whose lines are in an order one side does not hold them in keeps both sides.
/// Why:
///   The base writes `c` twice and `a` three times,
///  this side kept `a` then `c`,
///  the other side kept `c` then `a`,
///  and the word pass wrote `a` then `c`,
///  which is no run the other side holds
/// (`merge2-fuzz-seed7-all-line-ends.json`,
///  `case-00014`).
#[test]
fn an_answer_in_an_order_one_side_does_not_hold_keeps_both_sides() {
    let base: &[u8] = b"c\na\na\na\nc\nc\n";
    let ours: &[u8] = b"a\nc\n";
    let theirs: &[u8] = b"c\na\n";
    assert_eq!(
        merge_by_word(base, ours, theirs),
        WordMerge::Conflict {
            unresolved: 1
        },
        "the other side holds c before a"
    );
}

/// What:
///  A region whose lines end with a carriage return and a line feed,
///  and whose other side lost its last line end,
///  keeps both sides when the answer holds more copies of a line than both sides hold.
/// Why:
///   This is the region Git's line merge left for the file of `case-00463` of the alpha.49 release review's seed 1:
///  the answer wrote `b` twice,
///  while the side whose last line has no line end holds it once,
///  and the bytes that side's last line ends with are no part of the record either way.
#[test]
fn a_region_whose_side_lost_its_last_line_end_keeps_both_sides() {
    let base: &[u8] = b"a\r\nb\r\nb\r\nb\r\na\r\na\r\n";
    let ours: &[u8] = b"b\r\nb\r\na\r\n";
    let theirs: &[u8] = b"a\r\nb\r\na\r\na";
    assert!(
        matches!(
            merge_by_word(base, ours, theirs),
            WordMerge::Conflict { .. }
        ),
        "the b this side wrote once cannot appear twice"
    );
}

/// The seed of the generated regions in which one side also writes a line,
/// so a failure names the region it read.
const AN_ADDED_LINE_SEED: u64 = 20_261_129;

/// How many generated regions that search reads.
const AN_ADDED_LINE_CASES: usize = 400;

/// The records a side may write of its own:
/// two the base may hold, so a merge may hold a copy the other side does not,
/// and one the base never holds.
const WRITTEN_RECORDS: &[&str] = &["alpha", "beta", "delta"];

/// What:
///  The run `places` keeps of `records`,
///  with `written` written at `at`.
/// Why:
///   A side that deleted records and wrote one of its own is the shape the review found a wrong answer in:
///  the copy it wrote may be a record the base wrote several times,
///  which is what lifts one line's count above the other side's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runWithAWrittenRecord = (records, places, at, written) => { const kept = places.map((place) => records[place]); kept.splice(at, 0, written); return kept.join('\n'); };
/// ```
fn run_with_a_written_record(
    records: &[&str],
    places: &[usize],
    at: usize,
    written: &str,
) -> String {
    let mut kept: Vec<&str> = places.iter().map(|place| return records[*place]).collect();
    kept.insert(at, written);
    return kept.join("\n");
}

/// What:
///  The first line `merged` writes more copies of than the two sides leave,
///  or nothing when it writes none.
/// Why:
///   This is the bound the alpha.50 release review states:
///  a line at most as often as the side that deleted copies of it holds it,
///  while at most one side wrote more copies of that line than the base held.
/// A side that wrote copies of its own is where no bound over the two sides is right,
/// and a line the base never held is no line of the base to count.
/// The counts are read over the whole answer,
/// which is the unit the merge resolves:
/// a region that holds more copies than its own two sides leave can be right
/// when another region holds fewer of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pastWhatTheSidesLeave(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array, merged: Uint8Array): string | undefined;
/// ```
fn past_what_the_sides_leave(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
    merged: &[u8],
) -> Option<String> {
    let base_counts: BTreeMap<&[u8], usize> = count_bodies(base);
    let ours_counts: BTreeMap<&[u8], usize> = count_bodies(ours);
    let theirs_counts: BTreeMap<&[u8], usize> = count_bodies(theirs);
    for (line, held) in count_bodies(merged) {
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
            return Some(format!(
                "the line {line:?} appears {held} times, where the two sides leave {most}"
            ));
        }
    }
    return None;
}

/// What:
///  No answer the merge resolves for a region in which one side also wrote a line
///  writes more copies of a base line than the two sides leave.
/// Why:
///   A resolved answer replaces a conflict a person would have read,
///  so an answer that holds a copy of a line one side deleted lands with no marker
/// (`src/native/merge/word_merge.rs`,
/// the module's own words,
/// the owner's request of 2026-11-10 that wrong text land less often).
/// The search draws regions whose base writes records more than once,
/// deletes records from each side,
/// and writes one record of a side's own,
/// so the shape that escaped the rule over two sides that only deleted lines is what it reads:
/// seed `AN_ADDED_LINE_SEED`,
/// 400 cases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let case_ = 0; case_ < CASES; case_ += 1) { const { base, ours, theirs } = draw(); const merged = mergeByWord(base, ours, theirs); if (merged.kind !== 'resolved') continue; expect(pastWhatTheSidesLeave(base, ours, theirs, merged.bytes)).toBeUndefined(); }
/// ```
#[test]
fn no_answer_of_a_region_with_a_written_line_writes_past_the_copies_the_sides_leave() {
    let mut state: u64 = AN_ADDED_LINE_SEED;
    let mut resolved: usize = 0;
    let mut conflicted: usize = 0;
    let mut written_beside = 0_usize;
    let mut written_of_its_own = 0_usize;
    for case in 0..AN_ADDED_LINE_CASES {
        let size: usize = LEAST_LINES + next(&mut state, MOST_LINES - LEAST_LINES + 1);
        let records: Vec<&str> = (0..size)
            .map(|_| {
                return REPEATED_RECORDS[next(&mut state, REPEATED_RECORDS.len())];
            })
            .collect();
        let base: String = records.join("\n");
        let ours: String =
            run_of_records(records.as_slice(), kept_places(&mut state, size).as_slice());
        let places: Vec<usize> = kept_places(&mut state, size);
        let at: usize = next(&mut state, places.len() + 1);
        let written: &str = WRITTEN_RECORDS[next(&mut state, WRITTEN_RECORDS.len())];
        let theirs: String =
            run_with_a_written_record(records.as_slice(), places.as_slice(), at, written);
        if records.contains(&written) {
            written_beside += 1;
        } else {
            written_of_its_own += 1;
        }
        let WordMerge::Resolved {
            merged,
            ..
        } = merge_by_word(base.as_bytes(), ours.as_bytes(), theirs.as_bytes())
        else {
            conflicted += 1;
            continue;
        };
        resolved += 1;
        assert_eq!(
            past_what_the_sides_leave(
                base.as_bytes(),
                ours.as_bytes(),
                theirs.as_bytes(),
                merged.as_slice(),
            ),
            None,
            "case {case} of seed {AN_ADDED_LINE_SEED}: the answer wrote past what the two sides leave: \
             base {base:?}, ours {ours:?}, theirs {theirs:?}, merged {:?}",
            String::from_utf8_lossy(merged.as_slice())
        );
    }
    assert!(
        resolved > 0,
        "the search resolved no region with a written line"
    );
    assert!(
        conflicted > 0,
        "the search left every region with a written line, so the bound it reads refused nothing"
    );
    assert!(
        written_beside > 0 && written_of_its_own > 0,
        "the search wrote no copy of a record the base held beside one it does not: \
         {written_beside} beside, {written_of_its_own} of its own"
    );
}

/// What:
///  The smallest region the alpha.50 release review found a wrong answer in,
///  where one side also wrote a line,
///  keeps both sides.
/// Why:
///   The base writes `a` four times,
///  this side kept three of them and dropped a `b`,
///  the other side kept one `a` and wrote a second `b`,
///  so at most one `a` is left:
/// no copy of that line is a copy both sides kept.
/// The word pass wrote three and both the alpha.49 and the alpha.50 release resolved it
/// (`merge2-minimal-with-an-added-line.json`,
/// `min3`,
/// of the alpha.50 release review).
#[test]
fn the_smallest_region_with_a_written_line_keeps_no_copy_a_side_deleted() {
    let base: &[u8] = b"c\nb\nb\na\na\na\na\nb\n";
    let ours: &[u8] = b"c\na\na\na\nb\n";
    let theirs: &[u8] = b"b\nb\nb\na\nb\n";
    assert_eq!(
        merge_by_word(base, ours, theirs),
        WordMerge::Conflict {
            unresolved: 1
        },
        "three copies of a are not the one copy the other side kept"
    );
}

/// What:
///  The text a carried layer writes for a file the whole-answer rule refuses
///  holds both sides of every region the two sides disagree about.
/// Why:
///   `merge_by_word` reads one walk's count and `keep_both` reads the same walk's bytes,
///  so a refusal the second walk could not carry would leave the refused answer in the file
///  with no marker in it
/// (`src/native/merge/word_merge.rs`,
/// `walk`).
/// The file here is the one the rule refuses for the order of its lines,
/// and the walk that resolves it settles every region,
/// so the text without the second walk carries no marker at all.
#[test]
fn the_text_a_refused_file_keeps_holds_both_sides() {
    let base: &[u8] = b"c\na\na\na\nc\nc\n";
    let ours: &[u8] = b"a\nc\n";
    let theirs: &[u8] = b"c\na\n";
    assert!(
        matches!(
            merge_by_word(base, ours, theirs),
            WordMerge::Conflict { .. }
        ),
        "the answer is no run the other side holds"
    );
    let kept: Vec<u8> = keep_both(base, ours, theirs, ("ours", "theirs"));
    let text: String = String::from_utf8_lossy(kept.as_slice()).into_owned();
    assert!(
        text.contains("<<<<<<< ours\n") && text.contains(">>>>>>> theirs\n"),
        "the file a carried layer keeps shows both sides: {text}"
    );
}

/// What:
///  An answer whose lines are in an order the first side does not hold them in keeps both sides.
/// Why:
///   The base writes `c` twice and `a` three times,
///  this side kept `c` then `a`,
///  the other side kept `a` then `c`,
///  and the word pass wrote `a` then `c`,
///  which is a run of the other side and no run this side holds:
/// the answer is wrong for this side alone, so only this side's order can refuse it.
#[test]
fn an_answer_in_an_order_the_first_side_does_not_hold_keeps_both_sides() {
    let base: &[u8] = b"c\na\na\na\nc\nc\n";
    let ours: &[u8] = b"c\na\n";
    let theirs: &[u8] = b"a\nc\n";
    assert_eq!(
        merge_by_word(base, ours, theirs),
        WordMerge::Conflict {
            unresolved: 1
        },
        "this side holds c before a"
    );
}

/// What:
///  A file whose last line has no line end resolves when a side lost the line end of the record
///  that is the same record.
/// Why:
///   A person reads `b` and `b` with a carriage return and a line feed before it as the same record,
/// so a side that lost its last line end deleted no record,
/// and the merge of the two sides is right.
/// A check that kept the carriage return counted the base's `b` and that side's `b` as two records,
/// held the answer's one copy against the side that holds none of the base's spelling,
/// and refused a correct merge
/// (`merge2-replay-of-own-seeds` of the alpha.50 release review,
/// `crlf-noeol-seed606-00437`).
#[test]
fn a_region_whose_side_lost_its_last_line_end_reads_a_record_by_its_text() {
    let base: &[u8] = b"a\r\nb\r\na\r\na\r\nb\r\nb";
    let ours: &[u8] = b"b";
    let theirs: &[u8] = b"b\r\n";
    let WordMerge::Resolved {
        merged,
        ..
    } = merge_by_word(base, ours, theirs)
    else {
        panic!("both sides kept the one b, so the merge of the file resolves")
    };
    assert_eq!(
        String::from_utf8_lossy(merged.as_slice()),
        "b\r\n",
        "the merge writes the bytes the base had"
    );
}

/// What:
///  A file whose regions each leave one side as it was keeps both sides
///  when the answer those regions compose writes more copies of a line than the two sides leave.
/// Why:
///   No region of this file is one the word pass had to merge:
///  every one of them is a region one side left as it was,
///  and the answer the walk settles them to still holds one `b` more than the side that kept fewer of
///  them holds.
/// A refusal the second walk carried only over the regions the word pass would have merged
/// would leave that answer in the file with no marker in it
/// (`src/native/merge/word_merge.rs`,
/// `walk`).
/// The file is `del-add-seed202-00045` of the alpha.50 release review's seeds.
#[test]
fn the_answer_of_regions_a_side_left_as_it_was_keeps_both_sides() {
    let base: &[u8] = b"a\nb\nb\nb\nc\nc\nb\nb\nc\n";
    let ours: &[u8] = b"b\nb\nc\nc\nb\nc\n";
    let theirs: &[u8] = b"b\nb\nc\nc\nb\nb\nb\nc\n";
    assert!(
        matches!(
            merge_by_word(base, ours, theirs),
            WordMerge::Conflict { .. }
        ),
        "the answer holds four copies of b where this side kept three"
    );
    let kept: Vec<u8> = keep_both(base, ours, theirs, ("ours", "theirs"));
    let text: String = String::from_utf8_lossy(kept.as_slice()).into_owned();
    assert!(
        text.contains("<<<<<<< ours\n") && text.contains(">>>>>>> theirs\n"),
        "the file a carried layer keeps shows both sides: {text}"
    );
}
