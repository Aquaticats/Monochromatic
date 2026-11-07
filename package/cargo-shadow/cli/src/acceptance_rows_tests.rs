//! What:
//! Round 8 tests of acceptance rows that no other test file of the round holds:
//! the actual completion rename and sync at the journal level (A3,
//! A4),
//! the kept-files warning (K1),
//! repeated recovery (V5),
//! and the acknowledge command through the command line.
//! Why:
//! Contract `doc/planning/cargo-shadow-guarantees.md`,
//! "Acceptance list".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('acceptance rows', () => { it('K1: the threshold is exact', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::crates::LockSnapshot;
use crate::failpoints::{Point, arm};
use crate::journal::{JOURNAL_DIR, KEPT_FILES_WARNING_COUNT, QUARANTINE_DIR, begin, recover};
use crate::journal_support::{journal_dirs, plant};
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/a/one";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the given lockfile.
fn repository(lock: &str) -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return root;
}

/// A judge that refuses the live lockfile.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// Acceptance row A3,
/// A4:
/// the real rename of the completion marker fails:
/// the error is a mandatory one,
/// and the journal keeps its active name.
#[test]
fn a_failing_completion_rename_is_mandatory_and_leaves_the_journal_active() {
    let root = repository("original");
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    arm(Point::CompletionRename);

    let error = transaction.finish().expect_err("the rename failed");

    assert!(!error.cleanup_only(), "{error}");
    let journals: Vec<PathBuf> = journal_dirs(root.path());
    assert_eq!(journals.len(), 1);
    assert!(!journals[0].to_string_lossy().ends_with(".done"), "the journal was renamed although the rename failed");
}

/// Acceptance row A3,
/// A4:
/// the sync of the directory that lists the marker fails after the real rename:
/// the error is a mandatory one,
/// the journal is already named `.done`,
/// and a later recovery removes it without judging the run again.
#[test]
fn a_failing_completion_directory_sync_is_mandatory_and_a_later_recovery_only_removes_the_journal() {
    let root = repository("final output of the run");
    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");
    arm(Point::CompletionSync);

    let error = transaction.finish().expect_err("the sync failed");
    let marked: Vec<PathBuf> = journal_dirs(root.path());
    let messages: Vec<String> = recover(root.path(), &refusing);

    assert!(!error.cleanup_only(), "{error}");
    assert_eq!(marked.len(), 1, "the finished journal was removed before the failed barrier was handled, or was never made: {marked:?}");
    assert!(marked[0].to_string_lossy().ends_with(".done"), "{marked:?}");
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "final output of the run", "a finished run was judged again: {messages:?}");
    assert!(journal_dirs(root.path()).is_empty(), "the finished journal was not removed");
}

/// Acceptance row K1:
/// the warning point is exact:
/// nothing at or below the count,
/// a notice above it,
/// the same on repeated runs,
/// and nothing is refused or deleted.
#[test]
fn the_kept_files_warning_point_is_exact_and_repeated_runs_agree() {
    for number in [KEPT_FILES_WARNING_COUNT - 1, KEPT_FILES_WARNING_COUNT, KEPT_FILES_WARNING_COUNT + 1] {
        let root = repository("original");
        let kept = root.path().join(QUARANTINE_DIR);
        fs::create_dir_all(&kept).expect("kept directory");
        for index in 0..number {
            fs::write(kept.join(format!("Cargo.lock.displaced.{index}")), "kept").expect("a kept file");
        }
        for _run in 0..2 {
            let transaction = begin(root.path(), &[snapshot("original")], false).expect("a kept folder never refuses");

            let notice: Option<String> = transaction.kept_notice().map(str::to_string);

            assert_eq!(notice.is_some(), number > KEPT_FILES_WARNING_COUNT, "{number} files: {notice:?}");
            if let Some(text) = notice {
                assert!(text.contains(&format!("{number} files")) && text.contains(&format!("{} bytes", number * 4)) && text.contains(&kept.display().to_string()), "{text}");
            }
            drop(transaction);
        }
        assert_eq!(fs::read_dir(&kept).expect("lists").count(), number, "a kept file was deleted");
    }
}

/// Acceptance row K1 (inventory):
/// kept names beside the lockfile and stranded temporaries count toward the warning too,
/// with the central folder,
/// so repeated beside-target retention cannot hide below the count.
#[test]
fn kept_names_beside_the_lockfile_and_stranded_temporaries_count_toward_the_warning() {
    let root = repository("original");
    let kept = root.path().join(QUARANTINE_DIR);
    fs::create_dir_all(&kept).expect("kept directory");
    let crate_root = root.path().join(CRATE_DIR);
    for index in 0..600 {
        fs::write(kept.join(format!("Cargo.lock.displaced.{index}")), "kept").expect("a kept file");
    }
    for index in 0..400 {
        fs::write(crate_root.join(format!("Cargo.lock.displaced.{index}")), "kept").expect("a name beside the target");
    }
    fs::write(crate_root.join(".Cargo.lock.shadow-abc.tmp"), "kept").expect("a stranded temporary");
    fs::write(crate_root.join("Cargo.lock.refused"), "kept").expect("a refused copy");
    fs::write(crate_root.join("Cargo.toml"), "not kept").expect("an ordinary file");

    // Two transactions in a row (certification group 3): the notice is the same each time, and nothing is deleted or counted twice.
    let mut notices: Vec<String> = Vec::new();
    for _run in 0..2 {
        let transaction = begin(root.path(), &[snapshot("original")], false).expect("never refuses");
        notices.push(transaction.kept_notice().map(str::to_string).expect("1002 kept files are past the count"));
        drop(transaction);
    }

    // 1002 files of 4 bytes each: the combined size covers the central folder and the entries beside the target together,
    // and the folder named is the central one.
    for notice in &notices {
        assert!(notice.contains("1002 files") && notice.contains("402 more are kept beside"), "{notice}");
        assert!(notice.contains("4008 bytes"), "the size is not the combined size of the mixed locations: {notice}");
        assert!(notice.contains(&kept.display().to_string()), "the folder is not named: {notice}");
    }
    assert_eq!(notices[0], notices[1], "repeated transactions disagree");
    // The inventory after the runs: every kept name beside the target, the stranded temporary and the refused copy are still there.
    let names: Vec<String> = fs::read_dir(&crate_root).expect("lists").flatten().map(|entry| { return entry.file_name().to_string_lossy().to_string(); }).collect();
    assert_eq!(names.iter().filter(|name| { return name.starts_with("Cargo.lock.displaced."); }).count(), 400, "a name beside the target was deleted: {names:?}");
    assert!(names.contains(&".Cargo.lock.shadow-abc.tmp".to_string()) && names.contains(&"Cargo.lock.refused".to_string()), "the stranded temporary or the refused copy was deleted: {names:?}");
    assert_eq!(fs::read_dir(&kept).expect("lists").count(), 600, "a central kept file was deleted");
}

/// Acceptance row V5:
/// a recovery is repeatable:
/// the second pass finds nothing to do and changes nothing,
/// and the refused leftover saved by the first pass is not overwritten.
#[test]
fn a_second_recovery_pass_changes_nothing_and_keeps_the_refused_copy() {
    let root = repository("unchecked output of the killed run");
    let _journal = plant(root.path(), &[snapshot("original")], &[]);

    let first: Vec<String> = recover(root.path(), &refusing);
    let after_first: String = fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock");
    let refused: String = fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.refused")).expect("the refused copy");
    let second: Vec<String> = recover(root.path(), &refusing);

    assert!(!first.is_empty());
    assert_eq!(after_first, "original");
    assert_eq!(refused, "unchecked output of the killed run");
    assert!(second.is_empty(), "{second:?}");
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "original");
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.refused")).expect("the refused copy"), "unchecked output of the killed run");
    assert!(journal_dirs(root.path()).is_empty());
    assert!(!root.path().join(JOURNAL_DIR).join("anything").exists());
}

/// Acceptance row "Weaker path":
/// the advertised command works through the command line:
/// it exits 0 and acknowledges the journal named by the refusal.
#[test]
fn the_acknowledge_subcommand_acknowledges_the_journal_the_refusal_named() {
    let root = repository("original");
    let journal: PathBuf = plant(root.path(), &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), "pgroup starting\n").expect("record");
    let named: String = crate::journal_recover::survivor_note(&journal).expect("blocked");
    assert!(named.contains(&format!("cargo-shadow acknowledge {}", journal.display())), "{named}");

    let refused = crate::cli::run_cli(vec!["cargo-shadow".to_string(), "acknowledge".to_string(), root.path().join("nowhere").display().to_string()]);
    let code = crate::cli::run_cli(vec!["cargo-shadow".to_string(), "acknowledge".to_string(), journal.display().to_string()]);

    assert_ne!(refused, 0, "a path that is no journal was acknowledged");
    assert_eq!(code, 0);
    assert_eq!(crate::journal_recover::survivor_note(&journal), None);
    let _ = Path::new("");
}

/// Acceptance row K1 (certification,
/// third pass;
/// "Warn past a count,
/// never refuse"):
/// kept entries with huge sparse lengths do not
/// make the inventory panic or wrap:
/// the total is the exact sum and the transaction is admitted.
/// Three entries of `i64::MAX` bytes exceed `u64::MAX` together;
/// no content is written or read,
/// only the length is set.
/// A file system that refuses a length that large cannot show the schedule;
/// the test says so and stops (btrfs,
/// this host,
/// allows it).
#[test]
fn kept_entries_with_huge_sparse_lengths_are_summed_exactly_and_never_refuse() {
    let root = repository("original");
    let kept = root.path().join(QUARANTINE_DIR);
    fs::create_dir_all(&kept).expect("kept directory");
    let huge: u64 = u64::try_from(i64::MAX).expect("positive");
    let big_names = ["Cargo.lock.displaced.huge-a", "Cargo.lock.displaced.huge-b", "Cargo.lock.displaced.huge-c"];
    for name in big_names {
        let file = fs::File::create(kept.join(name)).expect("a kept entry");
        if let Err(error) = file.set_len(huge) {
            eprintln!("skipped: this file system refuses a length of {huge} bytes ({error}), so the overflow schedule cannot be built here");
            return;
        }
    }
    for index in 0..KEPT_FILES_WARNING_COUNT {
        fs::write(kept.join(format!("Cargo.lock.displaced.{index}")), "kept").expect("a small kept file");
    }
    let expected_total: u128 = u128::from(huge) * u128::try_from(big_names.len()).expect("small") + u128::try_from(KEPT_FILES_WARNING_COUNT).expect("small") * 4;
    assert!(expected_total > u128::from(u64::MAX), "the schedule must overflow a 64-bit sum");

    let transaction = begin(root.path(), &[snapshot("original")], false).expect("a kept folder never refuses, whatever its size");

    let notice: String = transaction.kept_notice().map(str::to_string).expect("past the count");
    assert!(notice.contains(&format!("{expected_total} bytes in all")), "the total is not the exact sum: {notice}");
    drop(transaction);
}

/// Acceptance row K1 (certification,
/// fourth pass;
/// "Warn past a count,
/// never refuse"):
/// an inventory that could not be completed is
/// never taken as small.
/// The central folder lists but its entries cannot be examined (a directory without the search permission lists
/// its names and refuses `lstat` of each),
/// so the number kept there is unknown although far fewer than the threshold were counted:
/// the warning says the inventory is unknown,
/// and the transaction is admitted with every kept file retained.
#[test]
fn an_inventory_that_could_not_be_read_is_reported_as_unknown_and_never_taken_as_small() {
    use std::os::unix::fs::PermissionsExt;
    let root = repository("original");
    let kept = root.path().join(QUARANTINE_DIR);
    fs::create_dir_all(&kept).expect("kept directory");
    for index in 0..3 {
        fs::write(kept.join(format!("Cargo.lock.displaced.{index}")), "kept").expect("a kept file");
    }
    fs::set_permissions(&kept, fs::Permissions::from_mode(0o444)).expect("list but do not search");
    let examinable: bool = fs::symlink_metadata(kept.join("Cargo.lock.displaced.0")).is_ok();
    if examinable {
        fs::set_permissions(&kept, fs::Permissions::from_mode(0o755)).expect("restore");
        eprintln!("skipped: permissions do not stop this user from examining the entries, so the unreadable inventory cannot be built this way");
        return;
    }

    let admitted = begin(root.path(), &[snapshot("original")], false);
    fs::set_permissions(&kept, fs::Permissions::from_mode(0o755)).expect("restore");

    let transaction = admitted.expect("a kept folder never refuses, however unreadable");
    let notice: String = transaction.kept_notice().map(str::to_string).expect("an inventory that could not be completed is not silence");
    assert!(notice.contains("could not all be read") && notice.contains("unknown"), "{notice}");
    assert_eq!(fs::read_dir(&kept).expect("lists").count(), 3, "a kept file was removed");
    drop(transaction);
}
