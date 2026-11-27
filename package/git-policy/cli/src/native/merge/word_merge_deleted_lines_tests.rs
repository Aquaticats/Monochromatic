//! Controls for a region each side only deleted lines from:
//! the union of the two deletions,
//! and a generated search for a correct merge of lines that repeat
//! that the rule keeping a line one side deleted out of a merge refuses.

use super::{
    keeps_a_deleted_line,
    line_counts,
};
use crate::merge::word_merge::merged_word_text;
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
