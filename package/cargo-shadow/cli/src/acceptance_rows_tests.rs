//! What:
//! Round 8 tests of acceptance rows that no other test file of the round holds:
//! the actual completion rename and sync at the journal level (A3, A4), the kept-files warning (K1), repeated recovery (V5),
//! and the acknowledge command through the command line.
//! Why:
//! Contract `doc/planning/cargo-shadow-guarantees.md`, "Acceptance list".
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

/// Acceptance row A3, A4:
/// the real rename of the completion marker fails:
/// the error is a mandatory one, and the journal keeps its active name.
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

/// Acceptance row A3, A4:
/// the sync of the directory that lists the marker fails after the real rename:
/// the error is a mandatory one, the journal is already named `.done`,
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
/// the warning point is exact: nothing at or below the count, a notice above it,
/// the same on repeated runs, and nothing is refused or deleted.
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
/// a recovery is repeatable: the second pass finds nothing to do and changes nothing,
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
