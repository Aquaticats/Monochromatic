//! The unit tests of compare-before-replace and of recovery from a killed run.

use std::fs;
use std::path::Path;

use crate::crates::LockSnapshot;
use crate::journal::{JOURNAL_DIR, begin, finish, recover, restore_if_unchanged};
use crate::test_scratch::scratch;

const CRATE_DIR: &str = "package/j/one";

fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

fn repository(lock: &str) -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return root;
}

fn lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("read");
}

#[test]
fn a_lock_that_still_holds_what_the_run_produced_is_replaced_by_the_snapshot() {
    let root = repository("produced");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert!(warnings.is_empty());
    assert_eq!(lock(root.path()), "original");
}

#[test]
fn a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning() {
    let root = repository("written by someone else");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert_eq!(lock(root.path()), "written by someone else");
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains(CRATE_DIR) && warnings[0].contains("another process"));
}

#[test]
fn a_lock_that_already_equals_the_snapshot_needs_nothing_and_no_temporary_file_is_left() {
    let root = repository("original");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert!(warnings.is_empty());
    let leftovers: Vec<String> = fs::read_dir(root.path().join(CRATE_DIR))
        .expect("list")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().to_string(); })
        .collect();
    assert_eq!(leftovers, vec!["Cargo.lock".to_string()]);
}

/// A process id that cannot exist on Linux (above the kernel's maximum).
const DEAD_PID: u32 = 4_194_305;

fn plant_journal(root: &Path, original: &str) {
    begin(root, &[snapshot(original)]).expect("begin");
    let ours = root.join(JOURNAL_DIR).join(std::process::id().to_string());
    fs::rename(&ours, root.join(JOURNAL_DIR).join(DEAD_PID.to_string())).expect("pose as a killed process");
}

#[test]
fn a_killed_run_whose_lock_holds_refused_versions_is_saved_aside_and_the_snapshot_goes_back() {
    let root = repository("unchecked output of the killed run");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(lock(root.path()), "original");
    assert_eq!(
        fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.refused")).expect("the saved lock"),
        "unchecked output of the killed run"
    );
    assert_eq!(messages.len(), 1);
    assert!(!root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists());
}

#[test]
fn a_killed_run_whose_lock_passes_the_policy_is_kept() {
    let root = repository("good output");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

    assert_eq!(lock(root.path()), "good output");
    assert!(!root.path().join(CRATE_DIR).join("Cargo.lock.refused").exists());
    assert_eq!(messages.len(), 1);
}

#[test]
fn a_journal_of_a_running_process_and_our_own_journal_are_not_touched() {
    let root = repository("live lock");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return true; }, &|_before, _live| { return vec!["never asked".to_string()]; });

    assert!(messages.is_empty());
    assert_eq!(lock(root.path()), "live lock");
    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists());
}

#[test]
fn finish_removes_this_processs_journal() {
    let root = repository("x");
    begin(root.path(), &[snapshot("x")]).expect("begin");

    finish(root.path());

    assert!(!root.path().join(JOURNAL_DIR).join(std::process::id().to_string()).exists());
}

/// Review finding (recovery destroys the remaining recovery copies):
/// a second refusal must not overwrite the first saved
/// lockfile,
/// so every saved copy gets its own name.
#[test]
fn a_second_refused_lock_is_saved_under_its_own_name_and_the_first_copy_survives() {
    let root = repository("first unchecked output");
    plant_journal(root.path(), "original");
    let refuse = |_before: &[LockSnapshot], _live: &[LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; };
    recover(root.path(), &|_pid| { return false; }, &refuse);
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "second unchecked output").expect("the second killed run");
    plant_journal(root.path(), "original");

    recover(root.path(), &|_pid| { return false; }, &refuse);

    let mut saved: Vec<String> = Vec::new();
    for entry in fs::read_dir(root.path().join(CRATE_DIR)).expect("list").flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("Cargo.lock.refused") {
            saved.push(fs::read_to_string(entry.path()).expect("read a saved copy"));
        }
    }
    saved.sort();
    assert_eq!(saved, vec!["first unchecked output".to_string(), "second unchecked output".to_string()]);
}

/// Review finding:
/// a recovery that could not finish keeps the journal,
/// because the journal holds the only copy of the
/// original lockfile.
/// Here the crate directory cannot be written,
/// so neither the refused copy nor the restore can happen.
#[test]
fn a_recovery_that_fails_keeps_the_journal() {
    use std::os::unix::fs::PermissionsExt;
    let root = repository("unchecked output of the killed run");
    plant_journal(root.path(), "original");
    let crate_dir = root.path().join(CRATE_DIR);
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o555)).expect("make the directory read-only");

    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o755)).expect("make the directory writable again");
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists(), "the journal of a failed recovery was deleted");
}

/// Review finding:
/// a live lockfile that cannot be read keeps the journal.
#[test]
fn an_unreadable_live_lock_keeps_the_journal() {
    let root = repository("whatever");
    plant_journal(root.path(), "original");
    fs::remove_file(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("remove the live lock");

    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists(), "the journal was deleted with the live lock unread");
}

/// Review finding:
/// a journal that cannot be read in full keeps its directory.
#[test]
fn an_unreadable_journal_is_kept() {
    let root = repository("whatever");
    plant_journal(root.path(), "original");
    fs::remove_file(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).join("0.lock")).expect("damage the journal");

    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists(), "a damaged journal was deleted");
}

/// Review finding (compare-before-replace):
/// the policy judgement is slow,
/// and another writer can publish a lockfile while
/// it runs.
/// Recovery must compare again right before it replaces the file.
#[test]
fn recovery_leaves_a_lock_that_another_writer_published_while_the_policy_was_judged() {
    let root = repository("unchecked output of the killed run");
    plant_journal(root.path(), "original");
    let lock_path = root.path().join(CRATE_DIR).join("Cargo.lock");

    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| {
        fs::write(&lock_path, "published by another writer meanwhile").expect("the other writer");
        return vec!["dep@1.0.1 refused".to_string()];
    });

    assert_eq!(lock(root.path()), "published by another writer meanwhile");
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("another process"), "{messages:?}");
}

/// Review finding (unknown ownership):
/// a lockfile this run never produced is not restored.
#[test]
fn a_lock_the_run_never_produced_is_left_alone_even_when_it_differs_from_the_snapshot() {
    let root = repository("written by someone else");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[]).expect("restores");

    assert_eq!(lock(root.path()), "written by someone else");
    assert_eq!(warnings.len(), 1);
}

/// Delegated pick 5 (2026-11-03):
/// a restore compares and replaces in one atomic step (`renameat2` with `RENAME_EXCHANGE`) and
/// undoes the swap when the file it displaced is not the one it expected.
/// Another writer publishes a lockfile in the gap
/// between the comparison and the replacement;
/// that lockfile must survive.
#[test]
fn a_lockfile_published_between_the_comparison_and_the_replacement_is_not_lost() {
    use crate::journal::restore_if_unchanged_with;
    let root = repository("produced");
    let publish = |path: &Path| { fs::write(path, "published by another writer in the gap").expect("the other writer"); };

    let warnings = restore_if_unchanged_with(root.path(), &[snapshot("original")], &[snapshot("produced")], &publish).expect("restores");

    assert_eq!(lock(root.path()), "published by another writer in the gap");
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

/// The recovery of a killed run replaces the same way:
/// a lockfile published after it was judged survives.
#[test]
fn recovery_does_not_replace_a_lock_published_in_the_gap_after_the_last_comparison() {
    use crate::journal::recover_with;
    let root = repository("unchecked output of the killed run");
    plant_journal(root.path(), "original");
    let lock_path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let publish = move |_path: &Path| { fs::write(&lock_path, "published by another writer in the gap").expect("the other writer"); };

    let messages = recover_with(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; }, &publish);

    assert_eq!(lock(root.path()), "published by another writer in the gap");
    assert!(messages.iter().any(|message| { return message.contains("another process"); }), "{messages:?}");
}

#[test]
fn a_compare_and_replace_that_finds_the_expected_text_replaces_it_and_leaves_no_temporary_file() {
    use crate::journal::{ReplaceOutcome, replace_if_holds};
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");

    let outcome = replace_if_holds(&path, "expected", "new text", &|_path| {}).expect("replaces");

    assert_eq!(outcome, ReplaceOutcome::Replaced);
    assert_eq!(lock(root.path()), "new text");
    let names: Vec<String> = fs::read_dir(root.path().join(CRATE_DIR)).expect("list").flatten().map(|entry| { return entry.file_name().to_string_lossy().to_string(); }).collect();
    assert_eq!(names, vec!["Cargo.lock".to_string()]);
}

#[test]
fn a_compare_and_replace_that_finds_other_text_or_no_file_leaves_things_as_they_are() {
    use crate::journal::{ReplaceOutcome, replace_if_holds};
    let root = repository("somebody else's");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");

    assert_eq!(replace_if_holds(&path, "expected", "new text", &|_path| {}).expect("answers"), ReplaceOutcome::LeftAlone);
    assert_eq!(lock(root.path()), "somebody else's");
    fs::remove_file(&path).expect("remove");
    assert_eq!(replace_if_holds(&path, "expected", "new text", &|_path| {}).expect("answers"), ReplaceOutcome::LeftAlone);
    assert!(!path.exists());
}

fn plant_breaking_journal(root: &Path, original_lock: &str, original_manifest: &str) {
    use crate::manifest_edit::ManifestEdit;
    let manifest = root.join(CRATE_DIR).join("Cargo.toml");
    begin(root, &[snapshot(original_lock)]).expect("begin");
    crate::journal::add_manifests(root, &[ManifestEdit { path: manifest, before: original_manifest.to_string(), produced: None }]).expect("journals the manifest");
    let ours = root.join(JOURNAL_DIR).join(std::process::id().to_string());
    fs::rename(&ours, root.join(JOURNAL_DIR).join(DEAD_PID.to_string())).expect("pose as a killed process");
}

fn manifest(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.toml")).expect("read the manifest");
}

/// Delegated pick 8 (2026-11-03):
/// a killed `--breaking` run is recovered for the manifests it rewrote.
/// When the lockfile it left
/// holds refused versions,
/// the lockfile and the manifest both go back,
/// and the text that was replaced is kept aside.
#[test]
fn a_killed_breaking_run_whose_lock_is_refused_gets_its_manifest_back_too() {
    let root = repository("unchecked output of the killed run");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "edited by cargo").expect("the edit");
    plant_breaking_journal(root.path(), "original", "manifest before");

    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(lock(root.path()), "original");
    assert_eq!(manifest(root.path()), "manifest before");
    let kept = fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.toml.refused")).expect("the replaced manifest is kept aside");
    assert_eq!(kept, "edited by cargo");
    assert!(!root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists());
}

/// When the lockfile it left passes the policy,
/// the manifest edit that goes with it is kept.
#[test]
fn a_killed_breaking_run_whose_lock_passes_keeps_its_manifest_edit() {
    let root = repository("good output");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "edited by cargo").expect("the edit");
    plant_breaking_journal(root.path(), "original", "manifest before");

    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

    assert_eq!(lock(root.path()), "good output");
    assert_eq!(manifest(root.path()), "edited by cargo");
}

/// A run killed before cargo wrote the lockfile but after it rewrote a manifest:
/// the lockfile is unchanged,
/// the manifest goes back.
#[test]
fn a_killed_breaking_run_that_only_rewrote_the_manifest_gets_it_back() {
    let root = repository("original");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "edited by cargo").expect("the edit");
    plant_breaking_journal(root.path(), "original", "manifest before");

    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["never asked".to_string()]; });

    assert_eq!(manifest(root.path()), "manifest before");
}

/// A file displaced twice:
/// one writer publishes in the gap before the exchange,
/// another between the exchange and its undo.
/// Neither file is lost:
/// the first is back at the path,
/// the second is kept as `Cargo.lock.displaced`.
#[test]
fn a_file_displaced_twice_is_kept_as_displaced_and_the_first_writers_file_is_put_back() {
    use crate::journal::{ReplaceOutcome, replace_if_holds_with_hooks};
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let first_writer = |target: &Path| { fs::write(target, "first writer").expect("the first writer"); };
    let second_writer = |target: &Path| { fs::write(target, "second writer").expect("the second writer"); };

    let outcome = replace_if_holds_with_hooks(&path, "expected", "new text", &first_writer, &second_writer).expect("answers");

    let kept = root.path().join(CRATE_DIR).join("Cargo.lock.displaced");
    assert_eq!(outcome, ReplaceOutcome::Preserved(kept.clone()));
    assert_eq!(lock(root.path()), "first writer");
    assert_eq!(fs::read_to_string(&kept).expect("the displaced file is kept"), "second writer");
}

/// A second displaced file does not replace the first kept one.
#[test]
fn a_second_displaced_file_gets_its_own_name() {
    use crate::journal::{ReplaceOutcome, replace_if_holds_with_hooks};
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock.displaced"), "an earlier displaced file").expect("earlier");
    let first_writer = |target: &Path| { fs::write(target, "first writer").expect("the first writer"); };
    let second_writer = |target: &Path| { fs::write(target, "second writer").expect("the second writer"); };

    let outcome = replace_if_holds_with_hooks(&path, "expected", "new text", &first_writer, &second_writer).expect("answers");

    let kept = root.path().join(CRATE_DIR).join("Cargo.lock.displaced.1");
    assert_eq!(outcome, ReplaceOutcome::Preserved(kept.clone()));
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.displaced")).expect("kept"), "an earlier displaced file");
    assert_eq!(fs::read_to_string(&kept).expect("kept"), "second writer");
}
