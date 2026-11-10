//! Controls for the lines a carried merge prints:
//! every path it names,
//! what stands there now,
//! and what the person can do with it.

use super::*;
use crate::merge_isolation_carry_kept_path::{Holds, MergeHolds};

/// What:
///  A conflict line names the file and says what to do with the markers,
/// because the command exits 0 and has no other way to say so.
#[test]
fn a_conflict_line_names_the_file_and_what_to_do() {
    let report: Report = Report { conflicts: vec![b"dir/a.txt".to_vec()], ..Report::default() };
    let lines: Vec<String> = report_lines("feature", &report);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("git merge feature"), "{}", lines[0]);
    assert!(lines[0].contains("dir/a.txt"), "{}", lines[0]);
    assert!(lines[0].contains("conflict markers"), "{}", lines[0]);
    assert!(lines[0].contains("git add -- dir/a.txt"), "{}", lines[0]);
}

/// What:
///  A path kept whole names the file,
/// the reason,
/// what stands there,
/// and how to reach the merge's version.
#[test]
fn a_kept_line_names_the_file_its_reason_and_the_way_back() {
    let kept: Kept = Kept {
        path: b"gone.txt".to_vec(),
        why: "the merge deletes it",
        worktree: Holds::File,
        merge: MergeHolds::Nothing,
        in_the_index: false,
    };
    let report: Report = Report { kept: vec![kept], ..Report::default() };
    let lines: Vec<String> = report_lines("feature", &report);
    assert!(lines[0].contains("gone.txt") && lines[0].contains("the merge deletes it"), "{}", lines[0]);
    assert!(lines[0].contains("`rm -- gone.txt`"), "{}", lines[0]);
}

/// What:
///  A path another command wrote is named as untouched,
/// and a merged path as merged by word.
#[test]
fn changed_and_merged_paths_get_their_own_lines() {
    let report: Report = Report {
        changed: vec![b"a.txt".to_vec()],
        merged: vec![b"b.txt".to_vec()],
        ..Report::default()
    };
    let lines: Vec<String> = report_lines("feature", &report);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("b.txt") && lines[0].contains("by word"), "{}", lines[0]);
    assert!(lines[1].contains("a.txt") && lines[1].contains("changed while the merge ran"), "{}", lines[1]);
}

/// What:
///  A path another change took is named as left alone,
/// with no claim about what stands there now.
/// Why:
///  A path that changed while the merge ran may have been deleted,
/// replaced by a link or a directory,
///  or rewritten,
/// and a line that says a file holds bytes sends the person to the wrong place when nothing does.
#[test]
fn a_changed_line_claims_nothing_about_what_stands_at_the_path() {
    let report: Report = Report { changed: vec![b"a.txt".to_vec()], ..Report::default() };
    let lines: Vec<String> = report_lines("feature", &report);
    assert!(lines[0].contains("a.txt"), "{}", lines[0]);
    assert!(!lines[0].contains("the file holds what was written then"), "{}", lines[0]);
}

/// What:
///  The line about a path a restore left alone names the path and the change that came after,
/// and no file that holds the person's bytes.
/// Why:
///  The path it names was left alone and may hold nothing at all.
#[test]
fn the_line_about_a_path_a_restore_left_alone_names_no_file() {
    let mut out: Vec<u8> = Vec::new();
    note_left_alone(&mut out, b"a.txt");
    let line: String = String::from_utf8_lossy(out.as_slice()).into_owned();
    assert!(line.contains("a.txt"), "{line}");
    assert!(!line.contains("it holds your bytes"), "{line}");
}

/// An empty report says nothing.
#[test]
fn nothing_to_say_is_no_line() {
    assert_eq!(report_lines("feature", &Report::default()), Vec::<String>::new());
}
