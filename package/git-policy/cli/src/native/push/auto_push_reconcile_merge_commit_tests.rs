//! Controls for the answer ceilings of the second pass's commands:
//! the line merge's answer covers a measured answer of many regions,
//! and every command the pass runs has a ceiling.

use super::*;

/// The bytes of the answer Git 2.56.0 printed for the round-198 fixture,
/// whose three stages are 7,605,777 bytes each and hold 103,000 regions:
/// measured on 2026-11-11 by running the pass's own `merge-file` form over those stages.
/// Why a control pins it:
///   the answer of a merge whose stages the pass takes is what the ceiling has to cover,
/// and no fixture of that size is cheap enough to run in every gate.
const MEASURED_MANY_REGION_ANSWER_BYTES: u64 = 37_871_557;

/// What:
///  The line merge's answer ceiling covers a measured answer of a hundred and three thousand regions.
/// Why:
///  The ceiling was four stages plus the push report's cap,
///  37,748,736 bytes,
///  which is 122,821 bytes short of that answer:
///  the count of a merge's regions is bounded by the lines of its stages and not by a margin of their bytes,
///  so a merge whose stages the pass takes and whose regions all resolve was refused as a flood and nothing was pushed
///  (cross-vendor review of round 198,
///  finding 2).
/// The content of an answer is at most the three stages and the text Git settled around them,
/// so the markers alone are what carried this answer past four stages.
#[test]
fn the_line_merges_answer_ceiling_covers_a_measured_many_region_answer() {
    // The words of one invocation, without the labels and the stages, which the ceiling does not read.
    let cap: u64 = answer_cap(&["merge-file", "--stdout", "--object-id", "--zdiff3"], None);
    assert!(
        cap >= MEASURED_MANY_REGION_ANSWER_BYTES,
        "the line merge's answer ceiling covers the measured answer: {cap} against {MEASURED_MANY_REGION_ANSWER_BYTES}"
    );
    // The ceiling this control was written against, kept here so the failure it prevented stays readable.
    let four_stages_and_the_push_cap: u64 = 4 * LARGEST_STAGE_BYTES + MAX_PUSH_OUTPUT;
    assert!(
        four_stages_and_the_push_cap < MEASURED_MANY_REGION_ANSWER_BYTES,
        "the measured answer is the one that ceiling refused: {four_stages_and_the_push_cap} against {MEASURED_MANY_REGION_ANSWER_BYTES}"
    );
    // The content of the answer is under four stages, so its markers are the bytes the ceiling has to add.
    let four_stages: u64 = 4 * LARGEST_STAGE_BYTES;
    assert!(MEASURED_MANY_REGION_ANSWER_BYTES > four_stages, "the answer is past four stages of content");
}

/// Every command of the pass has a ceiling,
/// and the line merge's is the largest of them.
#[test]
fn every_command_of_the_pass_has_an_answer_ceiling() {
    let line_merge: u64 = answer_cap(&["merge-file", "--stdout"], None);
    assert_eq!(line_merge, MERGE_FILE_ANSWER_BYTES, "the line merge's ceiling is the one the module names");
    // `cat-file` is given one name per line of its request, and its ceiling grows with them.
    let names: usize = 3;
    let mut request: Vec<u8> = Vec::new();
    for _ in 0..names {
        request.extend_from_slice(b"aa\n");
    }
    let cat_file: u64 = answer_cap(&["cat-file", "--batch"], Some(request.as_slice()));
    assert_eq!(cat_file, u64::try_from(names).expect("a count") * (LARGEST_STAGE_BYTES + CAT_FILE_HEADER_BYTES), "one stage and a header per name asked for");
    assert!(line_merge > cat_file, "the line merge's answer holds three stages and the settled text, so its ceiling is the larger");
    // Every other command prints a line or nothing.
    for command in ["check-attr", "hash-object", "read-tree", "update-index", "write-tree", "merge-tree", "rev-parse"] {
        assert_eq!(answer_cap(&[command], None), MAX_PUSH_OUTPUT, "{command} keeps the push report's cap");
    }
}
