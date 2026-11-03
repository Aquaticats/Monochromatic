//! The tests of the third review of the replace protocol and the journal (3b): data is never unlinked while it can still be
//! open elsewhere, a failed root is never defaulted to a restoration, and a disappearance is left alone.

use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::crates::LockSnapshot;
use crate::journal::{JOURNAL_DIR, KeepAs, ReplaceOutcome, begin, recover, replace_if_holds, replace_if_holds_with_hooks};
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

/// A pid no process can have, to pose a journal as the one of a killed run.
const DEAD_PID: u32 = 4_194_305;

/// Makes the journal this process began look like the journal of a killed run.
fn pose_as_killed(root: &Path) {
    let ours = root.join(JOURNAL_DIR).join(std::process::id().to_string());
    fs::rename(&ours, root.join(JOURNAL_DIR).join(DEAD_PID.to_string())).expect("pose as a killed process");
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
    begin(root.path(), &[snapshot("first original")]).expect("first begin");
    begin(root.path(), &[snapshot("second original")]).expect("second begin");

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
    begin(root.path(), &[snapshot("original")]).expect("begin");
    let edit = ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "manifest before".to_string(), produced: None };
    crate::journal::add_manifests(root.path(), &[edit]).expect("journal the manifest");
    pose_as_killed(root.path());
    let crate_dir = root.path().join(CRATE_DIR);
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o555)).expect("make the lockfile directory read-only");

    let messages = recover(root.path(), &|_pid, _identity| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o755)).expect("writable again");
    assert_eq!(fs::read_to_string(&manifest).expect("manifest"), "manifest rewritten by the killed run", "{messages:?}");
}
