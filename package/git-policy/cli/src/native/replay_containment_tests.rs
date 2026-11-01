//! Controls for the one-sided extension check, on zero-context hunks built by hand
//!  over the base lines `a b c d e`.

use super::*;

/// The base lines of every case, each with its newline.
fn base() -> Vec<Vec<u8>> {
    return ["a\n", "b\n", "c\n", "d\n", "e\n"]
        .iter()
        .map(|line| { return line.as_bytes().to_vec(); })
        .collect();
}

/// A zero-context hunk from `(marker, text)` lines, each text with its newline.
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

/// An own line after the landed line, on one side only, is a one-sided extension.
#[test]
fn an_own_line_after_the_landed_line_is_contained() {
    let prepared: Vec<PatchHunk> = vec![hunk(2, 2, &[(b'-', "b\n"), (b'+', "B\n"), (b'+', "X\n")])];
    assert!(contains_landed_change(&landed_b(), &prepared, &base()));
}

/// An own line before the landed line, on one side only, is a one-sided extension too.
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

/// A prepared change elsewhere does not cover the landed change, so the path merges three-way.
#[test]
fn a_far_apart_prepared_change_does_not_contain_the_landed_change() {
    let prepared: Vec<PatchHunk> = vec![hunk(4, 4, &[(b'-', "d\n"), (b'+', "D\n")])];
    assert!(!contains_landed_change(&landed_b(), &prepared, &base()));
}

/// A prepared hunk that replaces the landed lines with other text is a rewrite, not an extension.
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

/// No landed change is trivially contained, and a non-covering landed change is not.
#[test]
fn no_landed_change_is_contained() {
    assert!(contains_landed_change(&[], &landed_b(), &base()));
    assert!(!contains_landed_change(&landed_b(), &[], &base()));
}

/// A suffix match needs the prepared hunk to end where the landed change ends; a prepared hunk ending later does not qualify.
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

/// With only the start shared, own lines between the landed lines are not a one-sided extension:
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
