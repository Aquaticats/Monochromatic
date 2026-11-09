//! Controls for the one-sided extension check,
//! on zero-context hunks built by hand
//!  over the base lines `a b c d e`.

use super::*;
use crate::replay_patch::PatchLine;

/// The base lines of every case,
/// each with its newline.
fn base() -> Vec<Vec<u8>> {
    return ["a\n", "b\n", "c\n", "d\n", "e\n"]
        .iter()
        .map(|line| { return line.as_bytes().to_vec(); })
        .collect();
}

/// A zero-context hunk from `(marker, text)` lines,
/// each text with its newline.
fn hunk(old_start: i64, new_start: i64, lines: &[(u8, &str)]) -> PatchHunk {
    return PatchHunk {
        old_start,
        new_start,
        lines: lines
            .iter()
            .map(|(op, text)| { return PatchLine {
                op: *op,
                text: text.as_bytes().to_vec(),
            }; })
            .collect(),
    };
}

/// The landed replacement of `b` by `B`.
fn landed_b() -> Vec<PatchHunk> {
    return vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "B\n")])];
}

/// The landed change is present exactly when the prepared bytes hold it as they are.
#[test]
fn an_exact_match_is_contained() {
    let prepared: Vec<PatchHunk> = landed_b();
    assert!(contains_landed_change(&landed_b(), &prepared, &base()));
}

/// An own line after the landed line,
/// on one side only,
/// is a one-sided extension.
#[test]
fn an_own_line_after_the_landed_line_is_contained() {
    let prepared: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "B\n"), (b'+', "X\n")])];
    assert!(contains_landed_change(&landed_b(), &prepared, &base()));
}

/// An own line before the landed line,
/// on one side only,
/// is a one-sided extension too.
#[test]
fn an_own_line_before_the_landed_line_is_contained() {
    let prepared: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "X\n"), (b'+', "B\n")])];
    assert!(contains_landed_change(&landed_b(), &prepared, &base()));
}

/// Own lines on both sides of the landed line are an own edit on both sides and are not subsumed.
#[test]
fn own_lines_on_both_sides_are_not_contained() {
    let prepared: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "X\n"), (b'+', "B\n"), (b'+', "Y\n")])];
    assert!(!contains_landed_change(&landed_b(), &prepared, &base()));
}

/// A prepared change elsewhere does not cover the landed change,
/// so the path merges three-way.
#[test]
fn a_far_apart_prepared_change_does_not_contain_the_landed_change() {
    let prepared: Vec<PatchHunk> = vec![hunk(4, 4, &[(b'-', "d\n"), (b'+', "D\n")])];
    assert!(!contains_landed_change(&landed_b(), &prepared, &base()));
}

/// A prepared hunk that replaces the landed lines with other text is a rewrite,
/// not an extension.
#[test]
fn a_prepared_rewrite_of_a_landed_line_is_not_contained() {
    let prepared: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "Z\n")])];
    assert!(!contains_landed_change(&landed_b(), &prepared, &base()));
}

/// A prepared extension that keeps the landed start and adds lines at its end is a prefix match.
#[test]
fn a_prefix_extension_is_contained() {
    let landed: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'-', "c\n"), (b'+', "B\n"), (b'+', "C\n")])];
    let prepared: Vec<PatchHunk> = vec![hunk(
        2,
        2,
        &[(b'-', "b\n"), (b'-', "c\n"), (b'-', "d\n"), (b'+', "B\n"), (b'+', "C\n"), (b'+', "X\n")],
    )];
    assert!(contains_landed_change(&landed, &prepared, &base()));
}

/// A prepared extension that keeps the landed end and adds lines before it is a suffix match.
#[test]
fn a_suffix_extension_is_contained() {
    let landed: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'-', "c\n"), (b'+', "B\n"), (b'+', "C\n")])];
    let prepared: Vec<PatchHunk> = vec![hunk(
        1,
        1,
        &[(b'-', "a\n"), (b'-', "b\n"), (b'-', "c\n"), (b'+', "X\n"), (b'+', "B\n"), (b'+', "C\n")],
    )];
    assert!(contains_landed_change(&landed, &prepared, &base()));
}

/// Two landed hunks inside one prepared hunk match when the base lines between them are kept.
#[test]
fn two_landed_hunks_inside_one_prepared_hunk_are_contained() {
    let landed: Vec<PatchHunk> = vec![
        hunk(2, 2, &[(b'-', "b\n"), (b'+', "B\n")]),
        hunk(4, 4, &[(b'-', "d\n"), (b'+', "D\n")]),
    ];
    let prepared: Vec<PatchHunk> = vec![hunk(
        2,
        2,
        &[(b'-', "b\n"), (b'-', "c\n"), (b'-', "d\n"), (b'+', "B\n"), (b'+', "c\n"), (b'+', "D\n")],
    )];
    assert!(contains_landed_change(&landed, &prepared, &base()));
    let dropped: Vec<PatchHunk> = vec![hunk(
        2,
        2,
        &[(b'-', "b\n"), (b'-', "c\n"), (b'-', "d\n"), (b'+', "B\n"), (b'+', "D\n")],
    )];
    assert!(!contains_landed_change(&landed, &dropped, &base()));
}

/// A landed deletion that the prepared commit also deletes is contained,
///  and a prepared rewrite of the deleted lines is a modify/delete conflict.
#[test]
fn deletions_are_contained_only_when_both_sides_delete() {
    let landed: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n")])];
    let deleted: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n")])];
    let rewritten: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "X\n")])];
    assert!(contains_landed_change(&landed, &deleted, &base()));
    assert!(!contains_landed_change(&landed, &rewritten, &base()));
}

/// An addition on both sides against an empty base:
///  the prepared file holding every landed line in order is contained,
///  one that dropped a landed line is not.
#[test]
fn additions_against_an_empty_base_keep_every_landed_line() {
    let empty: Vec<Vec<u8>> = Vec::new();
    let landed: Vec<PatchHunk> = vec![hunk(0, 1, &[(b'+', "a\n"), (b'+', "b\n")])];
    let kept: Vec<PatchHunk> = vec![hunk(0, 1, &[(b'+', "a\n"), (b'+', "X\n"), (b'+', "b\n")])];
    let before: Vec<PatchHunk> = vec![hunk(0, 1, &[(b'+', "X\n"), (b'+', "a\n"), (b'+', "b\n")])];
    let dropped: Vec<PatchHunk> = vec![hunk(0, 1, &[(b'+', "a\n"), (b'+', "X\n")])];
    assert!(contains_landed_change(&landed, &kept, &empty));
    assert!(contains_landed_change(&landed, &before, &empty));
    assert!(!contains_landed_change(&landed, &dropped, &empty));
}

/// A landed insertion after the first base line is contained when the prepared hunk ends where it does
///  and keeps the inserted line.
#[test]
fn a_landed_insertion_is_contained_by_a_prepared_hunk_ending_at_it() {
    let landed: Vec<PatchHunk> = vec![hunk(1, 1, &[(b'+', "X\n")])];
    let prepared: Vec<PatchHunk> = vec![hunk(1, 1, &[(b'-', "a\n"), (b'+', "a\n"), (b'+', "X\n")])];
    assert!(contains_landed_change(&landed, &prepared, &base()));
}

/// No landed change is trivially contained,
/// and a non-covering landed change is not.
#[test]
fn no_landed_change_is_contained() {
    assert!(contains_landed_change(&[], &landed_b(), &base()));
    assert!(!contains_landed_change(&landed_b(), &[], &base()));
}

/// A suffix match needs the prepared hunk to end where the landed change ends;
/// a prepared hunk ending later does not qualify.
#[test]
fn a_suffix_match_needs_the_prepared_hunk_to_end_at_the_landed_end() {
    let landed: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'-', "c\n"), (b'+', "B\n"), (b'+', "C\n")])];
    let prepared: Vec<PatchHunk> = vec![hunk(
        1,
        1,
        &[
            (b'-', "a\n"),
            (b'-', "b\n"),
            (b'-', "c\n"),
            (b'-', "d\n"),
            (b'-', "e\n"),
            (b'+', "X\n"),
            (b'+', "Y\n"),
            (b'+', "B\n"),
            (b'+', "C\n"),
        ],
    )];
    assert!(!contains_landed_change(&landed, &prepared, &base()));
}

/// A landed change that is longer than the prepared change sharing its start is not covered by it,
///  even though the prepared lines are a prefix of the landed lines.
#[test]
fn a_prepared_change_shorter_than_the_landed_range_does_not_cover_it() {
    let landed: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'-', "c\n"), (b'-', "d\n"), (b'+', "B\n")])];
    let prepared: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "B\n")])];
    assert!(!contains_landed_change(&landed, &prepared, &base()));
}

/// With only the start shared,
/// own lines between the landed lines are not a one-sided extension:
///  the exact-range allowance needs both ends of the landed range.
#[test]
fn own_lines_between_landed_lines_need_both_ends_anchored() {
    let landed: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "B1\n"), (b'+', "B2\n")])];
    let prepared: Vec<PatchHunk> = vec![hunk(
        2,
        2,
        &[(b'-', "b\n"), (b'-', "c\n"), (b'+', "B1\n"), (b'+', "X\n"), (b'+', "B2\n"), (b'+', "Y\n")],
    )];
    assert!(!contains_landed_change(&landed, &prepared, &base()));
}

/// What:
///  Deciding whether the prepared change contains the landed one looks at each hunk of both
///  sides a bounded number of times,
///  never at every prepared hunk for every landed hunk.
/// Why:
///  `AGENTS.md` `RG2`:
/// a machine-generated change can fill one path's diff with many disjoint
///  hunks,
/// so finding each landed hunk's cover by walking the whole prepared list,
///  and then grouping the covers by filtering the whole covered list per prepared hunk,
///  is quadratic in the two hunk counts.
///  The seam counts the collected hunks a question looks at,
///  which is one per skipped prepared hunk and one per landed hunk when the walk is ordered
///  (`foundation/collected_elements_looked_at_tests.rs` shows a walk over the whole list counts
///  every one of them).
#[test]
fn a_wide_landed_change_reads_each_hunk_once() {
    let width: usize = 512;
    // The base is `width` distinct lines, so both sides' hunks are far apart.
    let lines: Vec<Vec<u8>> = (0..width)
        .map(|number| { return format!("line {number}\n").into_bytes(); })
        .collect();
    let mut landed: Vec<PatchHunk> = Vec::new();
    let mut prepared: Vec<PatchHunk> = Vec::new();
    for number in 0..width {
        let start = i64::try_from(number).expect("fits") + 1;
        let old: String = format!("line {number}\n");
        let new: String = format!("line {number} {number}\n");
        // The prepared commit makes the same edit and adds its own line beside it: a one-sided
        //  extension, so every landed hunk is contained.
        let own: String = format!("own {number}\n");
        landed.push(hunk(start, start, &[(b'-', old.as_str()), (b'+', new.as_str())]));
        prepared.push(hunk(start, start, &[(b'-', old.as_str()), (b'+', new.as_str()), (b'+', own.as_str())]));
    }
    let before: u64 = crate::collected_elements_looked_at::looked_at();
    let contained: bool = contains_landed_change(landed.as_slice(), prepared.as_slice(), lines.as_slice());
    let looked: u64 = crate::collected_elements_looked_at::looked_at() - before;
    assert!(contained, "every landed hunk is the prepared one's prefix");
    let allowed: u64 = (landed.len() + prepared.len()) as u64;
    assert!(looked > 0, "the decision looked at the hunks");
    assert!(
        looked <= allowed,
        "{looked} collected hunks looked at for {} landed and {} prepared hunks; one ordered walk looks at each hunk at most once ({allowed})",
        landed.len(),
        prepared.len()
    );
}
