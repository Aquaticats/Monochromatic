// What:     Unit tests for `unplayable_notification_logic.rs`, pulled in by
//           `#[cfg(test)] #[path = "unplayable_notification_logic_tests.rs"] mod tests;`.
// Why:      The wording names the file, a batch collapses to a count, the title is fixed and private-data free, and a
//           repeated report posts nothing.

use super::*;

fn paths(names: &[&str]) -> Vec<PathBuf> {
    return names.iter().map(|name| return PathBuf::from("/library/Album").join(name)).collect();
}

#[test]
fn one_unplayable_file_is_named_without_its_folder() {
    assert_eq!(unplayable_message(&[PathBuf::from("/lib/Broken/01 Corrupt.wav")]), "01 Corrupt.wav can't be played.");
}

#[test]
fn several_unplayable_files_collapse_to_a_count_naming_the_first() {
    let report = [PathBuf::from("/a/one.wav"), PathBuf::from("/a/two.wav"), PathBuf::from("/a/three.wav")];
    assert_eq!(unplayable_message(&report), "3 files can't be played, including one.wav.");
}

#[test]
fn no_unplayable_files_give_no_message() {
    assert_eq!(unplayable_message(&[]), "");
}

#[test]
fn a_path_without_a_file_name_is_shown_whole() {
    assert_eq!(file_name_text(Path::new("/")), "/");
}

#[test]
fn the_title_is_fixed_plain_wording_that_names_nothing() {
    assert_eq!(SUMMARY, "Cannot play a file");
}

#[test]
fn one_report_becomes_one_notice_with_the_fixed_title_and_the_message_as_body() {
    let notice = Ledger::new().admit(&paths(&["Example Track.flac"])).expect("a new file makes a notice");
    assert_eq!(notice, Notice { summary: "Cannot play a file", body: "Example Track.flac can't be played.".to_string() });
    assert!(!notice.summary.contains("Example"), "the title carries no file name");
    assert!(!notice.summary.contains("Album"), "the title carries no folder");
}

#[test]
fn several_files_in_one_report_become_one_notice_with_a_count() {
    let notice = Ledger::new().admit(&paths(&["one.wav", "two.wav", "three.wav"])).expect("new files make a notice");
    assert_eq!(notice.body, "3 files can't be played, including one.wav.");
}

#[test]
fn an_empty_report_makes_no_notice() {
    assert_eq!(Ledger::new().admit(&[]), None);
}

#[test]
fn the_same_report_twice_makes_one_notice() {
    let mut ledger = Ledger::new();
    let report = paths(&["Example Track.flac"]);
    assert!(ledger.admit(&report).is_some());
    assert_eq!(ledger.admit(&report), None, "a repeat posts nothing");
}

#[test]
fn a_report_that_adds_one_file_announces_only_the_new_file() {
    let mut ledger = Ledger::new();
    assert!(ledger.admit(&paths(&["a.wav", "b.wav"])).is_some());
    let notice = ledger.admit(&paths(&["b.wav", "c.wav"])).expect("c.wav is new");
    assert_eq!(notice.body, "c.wav can't be played.", "b.wav was told before and is left out");
}

#[test]
fn a_file_named_twice_in_one_report_counts_once() {
    let notice = Ledger::new().admit(&paths(&["a.wav", "a.wav"])).expect("a.wav is new");
    assert_eq!(notice.body, "a.wav can't be played.");
}

#[test]
fn two_files_with_the_same_name_in_different_folders_are_both_counted() {
    let report = [PathBuf::from("/one/dup.wav"), PathBuf::from("/two/dup.wav")];
    let notice = Ledger::new().admit(&report).expect("both are new");
    assert_eq!(notice.body, "2 files can't be played, including dup.wav.");
}
