//! The tests of the third review of the replace protocol and the journal (3b): data is never unlinked while it can still be
//! open elsewhere, a failed root is never defaulted to a restoration, and a disappearance is left alone.

use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::crates::LockSnapshot;
use crate::journal::{JOURNAL_DIR, KeepAs, ReplaceOutcome, begin, recover, replace_if_holds, replace_if_holds_with_hooks};
use crate::journal_support::plant;
use crate::journal_replace::{Seams, replace_with_seams};
use crate::manifest_edit::ManifestEdit;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/j/one";

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

/// Every file name in a directory.
fn names_in(directory: &Path) -> Vec<String> {
    return fs::read_dir(directory).expect("list").flatten().map(|entry| { return entry.file_name().to_string_lossy().to_string(); }).collect();
}

/// Whether some file in the directory holds exactly this text.
fn some_file_holds(directory: &Path, text: &str) -> bool {
    return names_in(directory).iter().any(|name| { return fs::read_to_string(directory.join(name)).map(|found| { return found == text; }).unwrap_or(false); });
}

/// Whether the process may ignore directory permissions, which makes the permission-based failures unreliable.
fn ignores_permissions() -> bool {
    return fs::metadata("/proc/self").map(|metadata| { return metadata.uid() == 0; }).unwrap_or(true);
}

/// Review finding (3b 1): after the exchange was undone the replacement inode has been live, and another writer can hold it open;
/// it must be kept, not unlinked, so a write through that descriptor still has a name.
#[test]
fn a_replacement_that_another_writer_holds_open_is_kept_when_the_exchange_is_undone() {
    let root = repository("expected");
    let directory = root.path().join(CRATE_DIR);
    let path = directory.join("Cargo.lock");
    let first_writer = |target: &Path| { fs::write(target, "first writer").expect("the first writer"); };
    let held: RefCell<Option<File>> = RefCell::new(None);
    let open_replacement = |target: &Path| { *held.borrow_mut() = Some(OpenOptions::new().write(true).open(target).expect("open the live replacement")); };

    let outcome = replace_if_holds_with_hooks(&path, "expected", "new text", &first_writer, &open_replacement).expect("answers");
    held.borrow_mut().as_mut().expect("held").write_all(b"late edit").expect("the late write");

    assert_eq!(fs::read_to_string(&path).expect("live"), "first writer", "{outcome:?}");
    assert!(some_file_holds(&directory, "late edit"), "the write through the open descriptor has no name left: {:?}", names_in(&directory));
}

/// Review finding (3b 2 B): when the expected file was displaced but could not be linked to its kept name, the snapshot is live and
/// the outcome says so, instead of reading like an untouched file.
#[test]
fn a_displaced_file_that_could_not_be_linked_is_reported_with_the_snapshot_live() {
    let root = repository("expected");
    let directory = root.path().join(CRATE_DIR);
    let path = directory.join("Cargo.lock");
    let no_link = |_existing: &Path, _new: &Path| -> std::io::Result<()> { return Err(std::io::Error::other("injected link failure")); };
    let none = |_path: &Path| {};
    let swap = |first: &Path, second: &Path| -> std::io::Result<()> { return crate::journal_replace::exchange_paths(first, second); };
    let seams = Seams { between: &none, after_exchange: &none, swap: &swap, link: &no_link };

    let outcome = replace_with_seams(&path, "expected", "new text", &KeepAs::next_to_target(".displaced"), &seams).expect("answers");

    assert_eq!(fs::read_to_string(&path).expect("live"), "new text");
    match outcome {
        ReplaceOutcome::Displaced(kept) => assert_eq!(fs::read_to_string(kept).expect("the displaced file"), "expected"),
        other => panic!("expected Displaced, got {other:?}"),
    }
}

/// Review finding (3b 10): a file deleted after the comparison stays deleted.
#[test]
fn a_file_deleted_after_the_comparison_stays_deleted() {
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let delete = |target: &Path| { fs::remove_file(target).expect("the deletion"); };

    let outcome = replace_if_holds(&path, "expected", "new text", &delete).expect("answers");

    assert_eq!(outcome, ReplaceOutcome::LeftAlone);
    assert!(!path.exists(), "the deletion was undone");
}

/// Review finding (3b 3): a second transaction of the same process never reuses or truncates the first one's journal.
#[test]
fn a_second_begin_leaves_the_first_journal_intact() {
    let root = repository("lock");
    begin(root.path(), &[snapshot("first original")], false).expect("first begin");
    begin(root.path(), &[snapshot("second original")], false).expect("second begin");

    let journals: Vec<String> = names_in(&root.path().join(JOURNAL_DIR));
    assert_eq!(journals.len(), 2, "{journals:?}");
    let first_kept: bool = journals.iter().any(|name| { return fs::read_to_string(root.path().join(JOURNAL_DIR).join(name).join("0.lock")).map(|text| { return text == "first original"; }).unwrap_or(false); });
    assert!(first_kept, "the first journal's snapshot was overwritten");
}

/// Review finding (3b 2 A): a lockfile whose restoration failed leaves its root's manifest alone; the pair is never split.
#[test]
fn a_failed_lockfile_restore_leaves_the_roots_manifest_alone() {
    if ignores_permissions() {
        return;
    }
    use std::os::unix::fs::PermissionsExt;
    let root = repository("unchecked output of the killed run");
    let manifest_dir: PathBuf = root.path().join("manifests");
    fs::create_dir_all(&manifest_dir).expect("manifest dir");
    let manifest: PathBuf = manifest_dir.join("Cargo.toml");
    fs::write(&manifest, "manifest rewritten by the killed run").expect("manifest");
    let edit = ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "manifest before".to_string(), produced: None };
    plant(root.path(), &[snapshot("original")], &[edit]);
    let crate_dir = root.path().join(CRATE_DIR);
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o555)).expect("make the lockfile directory read-only");

    let messages = recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o755)).expect("writable again");
    assert_eq!(fs::read_to_string(&manifest).expect("manifest"), "manifest rewritten by the killed run", "{messages:?}");
}

/// Review finding (3b 9): a temporary name that already exists is never truncated; the next name is used.
#[test]
fn a_temporary_name_that_exists_is_never_truncated() {
    let root = scratch();
    fs::write(root.path().join(".x.shadow-same.tmp"), "keep me").expect("existing temporary");
    let mut names = ["same", "fresh"].into_iter().map(str::to_string);

    let (created, _device, _inode) = crate::journal_replace::create_exclusive(root.path(), "x", "new", 0o644, &mut || { return names.next().expect("a name"); }).expect("creates");

    assert_eq!(fs::read_to_string(root.path().join(".x.shadow-same.tmp")).expect("existing"), "keep me");
    assert_eq!(fs::read_to_string(created).expect("created"), "new");
}

/// A name generator that never yields a free name ends in an error that names the directory, not in a loop that never ends.
#[test]
fn a_temporary_name_search_ends_with_an_error() {
    let root = scratch();
    fs::write(root.path().join(".x.shadow-same.tmp"), "keep me").expect("existing temporary");

    let result = crate::journal_replace::create_exclusive(root.path(), "x", "new", 0o644, &mut || { return "same".to_string(); });

    let message: String = result.expect_err("no free name").to_string();
    assert!(message.contains(&root.path().display().to_string()), "{message}");
}

/// Review finding (3b 4): the lock the owner holds is the proof that it lives; a record that names a dead process, a renamed
/// directory or an unreadable `/proc` do not change that.
#[test]
fn a_journal_whose_owner_still_holds_its_lock_is_not_recovered() {
    let root = repository("unchecked output of the running owner");
    let _owner = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal = crate::journal_support::journal_dir(root.path());
    fs::rename(&journal, root.path().join(JOURNAL_DIR).join("4194305")).expect("a name that says the process is dead");

    let messages = recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "unchecked output of the running owner", "{messages:?}");
    assert_eq!(crate::journal_support::journal_dirs(root.path()).len(), 1);
}

/// Review finding (3b 5): recovering the journal of a dead run must not restore the inputs of a root a live transaction covers.
#[test]
fn recovery_leaves_the_roots_a_live_transaction_covers() {
    let root = repository("unchecked output of the killed run");
    plant(root.path(), &[snapshot("original of the killed run")], &[]);
    let _live = begin(root.path(), &[snapshot("original of the live run")], false).expect("the live transaction");

    let messages = recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "unchecked output of the killed run", "{messages:?}");
    assert_eq!(crate::journal_support::journal_dirs(root.path()).len(), 2, "the killed run's journal is kept for a later run");
    assert!(messages.iter().any(|message| { return message.contains("live"); }), "{messages:?}");
}

/// Review finding (3b 7): a ready journal whose manifest index is missing is kept, not read as "no manifests".
#[test]
fn a_missing_manifest_index_keeps_the_journal_and_its_snapshots() {
    let root = repository("unchecked output of the killed run");
    let manifest = root.path().join(CRATE_DIR).join("Cargo.toml");
    fs::write(&manifest, "edited by cargo").expect("manifest");
    let edit = ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest, before: "manifest before".to_string(), produced: None };
    let journal = plant(root.path(), &[snapshot("original")], &[edit]);
    fs::remove_file(journal.join("manifests.txt")).expect("lose the manifest index");

    recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert!(journal.join("m0.toml").exists(), "the only copy of the manifest was deleted with the journal");
}

/// A journal whose preparation never finished (its owner was killed before cargo ran) changed nothing, so recovery removes it
/// and does not touch the lockfile, which another writer may have written meanwhile.
#[test]
fn a_journal_that_never_became_ready_is_removed_without_touching_the_lockfile() {
    let root = repository("written by someone else");
    let transaction = begin(root.path(), &[snapshot("original")], true).expect("begin, manifests promised but never journaled");
    drop(transaction);

    recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "written by someone else");
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty(), "an unready journal is removed");
}

/// A journal whose removal was interrupted after the completion marker is finished by the next recovery.
#[test]
fn a_journal_marked_done_is_removed_by_the_next_recovery() {
    let root = repository("good output");
    let journal = plant(root.path(), &[snapshot("original")], &[]);
    let done = journal.with_file_name(format!("{}.done", journal.file_name().expect("name").to_string_lossy()));
    fs::rename(&journal, &done).expect("mark it done");

    recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert!(crate::journal_support::journal_dirs(root.path()).is_empty());
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "good output");
}
