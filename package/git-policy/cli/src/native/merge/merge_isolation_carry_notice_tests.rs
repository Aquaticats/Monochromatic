//! Controls for the lines a carried merge prints:
//! every path it names,
//! what stands there now,
//! and what the person can do with it.

use super::*;

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
/// and how to take the merge's version.
#[test]
fn a_kept_line_names_the_file_its_reason_and_the_way_back() {
    let report: Report = Report { kept: vec![(b"gone.txt".to_vec(), "the merge deletes it".into())], ..Report::default() };
    let lines: Vec<String> = report_lines("feature", &report);
    assert!(lines[0].contains("gone.txt") && lines[0].contains("the merge deletes it"), "{}", lines[0]);
    assert!(lines[0].contains("git restore -- gone.txt"), "{}", lines[0]);
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
    assert!(lines[1].contains("a.txt") && lines[1].contains("was written again"), "{}", lines[1]);
}

/// An empty report says nothing.
#[test]
fn nothing_to_say_is_no_line() {
    assert_eq!(report_lines("feature", &Report::default()), Vec::<String>::new());
}
