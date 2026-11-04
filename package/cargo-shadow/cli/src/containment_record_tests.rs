//! What:
//! Round 7 tests of what a journal records about containment and what a recovery does with it:
//! the weaker path, an empty journal directory, and the tidying of recorded groups.
//! Why:
//! The coordinator's pick (veto open) is that where no control group can be had a run goes on with a stated weaker guarantee;
//! these tests keep it from being silent and keep a recovery from reading the weaker record as proof of quiescence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('containment record', () => { it('names the missing capability', ...) })
//! ```

use std::fs;
use std::process::Command;
use std::time::Duration;

use crate::bounded_run::{Bound, output_within, use_group_parent_for_test, weaker_notice};
use crate::cgroup_test_support::test_group;
use crate::control_group::{ControlGroup, Record, parse_record};
use crate::crates::LockSnapshot;
use crate::journal::{JOURNAL_DIR, begin, recover};
use crate::journal_support::journal_dirs;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/k/one";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds a lockfile.
fn repository(lock: &str) -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return root;
}

/// A bound for the commands of these tests.
fn bound() -> Bound {
    return Bound { limit: Duration::from_secs(20), what: "the contained command".to_string(), rerun: "contained-command".to_string() };
}

/// A judge that refuses the live lockfile.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// The weaker path:
/// a host that cannot give a control group still runs the command,
/// in a process group,
/// records that group in the journal in the record's weaker form,
/// and says so to the transaction.
#[test]
fn a_host_without_a_control_group_runs_the_command_in_a_process_group_and_records_it() {
    let plain = scratch();
    use_group_parent_for_test(Some(plain.path().to_path_buf()));
    let root = repository("original");
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");

    let result = output_within(Command::new("sh").args(["-c", "echo ok"]), None, &bound());
    use_group_parent_for_test(None);

    let output = result.expect("the weaker path runs the command");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
    assert!(transaction.process_group_only(), "the transaction was not told the guarantee is weaker");
    assert!(!transaction.writers_may_remain());
    let journal = journal_dirs(root.path()).into_iter().next().expect("a journal");
    let record: String = fs::read_to_string(journal.join("groups")).expect("the record");
    let lines: Vec<&str> = record.lines().collect();
    assert_eq!(lines.len(), 2, "{record:?}");
    assert!(matches!(parse_record(lines[1]), Ok(Record::ProcessGroup(_))), "{record:?}");
    drop(transaction);
}

/// The weaker path stops the process group at the bound,
/// as round 5 did.
#[test]
fn the_weaker_path_still_stops_the_process_group_at_the_bound() {
    let plain = scratch();
    use_group_parent_for_test(Some(plain.path().to_path_buf()));
    let marker = plain.path().join("late");
    let script: String = format!("(sleep 2; : > {}) & sleep 30", marker.display());

    let result = output_within(Command::new("sh").args(["-c", &script]), None, &Bound { limit: Duration::from_millis(400), ..bound() });
    use_group_parent_for_test(None);
    std::thread::sleep(Duration::from_millis(2_700));

    assert!(result.is_err(), "the run should have passed its bound");
    assert!(!marker.exists(), "a member of the process group wrote after the stop");
}

/// The notice names the missing capability and says what cannot be shown.
#[test]
fn the_notice_of_the_weaker_path_names_the_missing_capability() {
    let plain = scratch();
    let unavailable = ControlGroup::create_in(plain.path()).expect_err("a plain directory is no control group");

    let notice: String = weaker_notice(&unavailable);

    assert!(notice.contains("cgroup v2") && notice.contains("quiescence is not shown"), "{notice}");
    assert_eq!(notice.lines().count(), 1, "the notice is one line");
}

/// Round 7 (6b2 2):
/// the directory a begin cut off after its creation leaves is empty,
/// no child can have started,
/// and a recovery removes it.
#[test]
fn a_recovery_removes_an_empty_journal_directory() {
    let root = repository("original");
    let cut_off = root.path().join(JOURNAL_DIR).join("cut-off");
    fs::create_dir_all(&cut_off).expect("an empty journal directory");

    let messages: Vec<String> = recover(root.path(), &refusing);

    assert!(!cut_off.exists(), "the empty directory stayed: {messages:?}");
}

/// A recovery that settles a journal tidies the empty control groups it recorded.
#[test]
fn a_recovery_removes_the_empty_control_group_a_killed_run_recorded() {
    let Some(made) = test_group() else {
        return;
    };
    let root = repository("original");
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal = journal_dirs(root.path()).into_iter().next().expect("a journal");
    fs::write(journal.join("groups"), format!("{}\n", made.group.record())).expect("record the group");
    drop(transaction);

    let messages: Vec<String> = recover(root.path(), &refusing);

    assert!(!made.group.directory().exists(), "the empty group was left behind: {messages:?}");
}

/// Advisor finding on the weaker path (the shape of 6b2 2):
/// a wrapper killed after announcing a child and before recording its process group
/// leaves a journal that a recovery must read as a possible survivor.
#[test]
fn a_started_child_whose_process_group_was_never_recorded_is_a_possible_survivor() {
    let root = repository("original");
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal = journal_dirs(root.path()).into_iter().next().expect("a journal");
    fs::write(journal.join("groups"), "pgroup starting\n").expect("the announcement");
    drop(transaction);

    let note = crate::journal_recover::survivor_note(&journal);

    assert!(note.is_some(), "a recovery would restore files a child may still be writing");
}

/// The announcement is followed by the process group in a run that finished the record,
/// and the pair is no survivor once the group is gone.
#[test]
fn an_announced_child_whose_group_is_recorded_and_gone_is_no_survivor() {
    let root = repository("original");
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal = journal_dirs(root.path()).into_iter().next().expect("a journal");
    let gone: i32 = i32::MAX;
    fs::write(journal.join("groups"), format!("pgroup starting\npgroup {gone}\n")).expect("the pair");
    drop(transaction);

    let note = crate::journal_recover::survivor_note(&journal);

    assert_eq!(note, None);
}

/// A run in the weaker path writes the announcement first and the group second.
#[test]
fn the_weaker_path_announces_before_it_records_the_group() {
    let plain = scratch();
    use_group_parent_for_test(Some(plain.path().to_path_buf()));
    let root = repository("original");
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");

    let result = output_within(&mut Command::new("true"), None, &bound());
    use_group_parent_for_test(None);

    result.expect("runs");
    let journal = journal_dirs(root.path()).into_iter().next().expect("a journal");
    let record: String = fs::read_to_string(journal.join("groups")).expect("the record");
    let kinds: Vec<Record> = record.lines().map(|line| { return parse_record(line).expect("a record"); }).collect();
    assert!(matches!(kinds.as_slice(), [Record::ProcessGroupStarting, Record::ProcessGroup(_)]), "{record:?}");
    drop(transaction);
}
