//! Controls for the index lock on a filesystem that makes neither a rename without replacing nor a hard link:
//!  the lock is taken by exclusive creation as Git takes it, the fallback is used only when both calls are refused,
//!  a lock that exists is never removed or replaced by it, and a failure after the creation leaves no lock of ours.

use crate::service_index_lock::*;
use crate::service_index_marker::{marker_text, own_identity};
use crate::service_protocol::FailureCode;
use crate::test_support::{fixture, remove};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long a control waits for a lock that stays: short, because only the ceiling ends that wait.
const SHORT_CEILING: Duration = Duration::from_millis(400);

fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(&git_dir).expect("directory");
    let index: PathBuf = git_dir.join("index");
    std::fs::write(&index, "old index").expect("index");
    return (root, index);
}

fn refusal(code: i32) -> std::io::Error {
    return std::io::Error::from_raw_os_error(code);
}

fn rename_refused(_: &Path, _: &Path) -> std::io::Result<()> {
    return Err(refusal(libc::EINVAL));
}

fn link_refused(_: &Path, _: &Path) -> std::io::Result<()> {
    return Err(refusal(libc::EPERM));
}

/// What the directory of the index holds besides the index.
fn names_beside_the_index(index: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(index.parent().expect("directory"))
        .expect("listing")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name != "index"; })
        .collect();
    names.sort();
    return names;
}

/// The exclusive creation is the fallback of the two refusals, never a first choice:
/// a rename that works, a link that works after a refused rename, and a link that fails for any other reason never create the
/// lock file themselves.
#[test]
fn exclusive_creation_is_used_only_when_both_the_rename_and_the_link_are_refused() {
    let (root, index) = index_in("lock-creation-only-when-both");
    let created: Cell<usize> = Cell::new(0);
    let create = |path: &Path| -> std::io::Result<std::fs::File> {
        created.set(created.get() + 1);
        return std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(path);
    };
    // Rename works.
    let platform: Platform = Platform { create: &create, ..SYSTEM };
    drop(acquire_on(&platform, &index, 1, &|_| {}).expect("taken by a rename"));
    assert_eq!(created.get(), 0, "a rename that works places the lock whole");
    // Rename refused, link works.
    let platform: Platform = Platform { rename: &rename_refused, create: &create, ..SYSTEM };
    drop(acquire_on(&platform, &index, 2, &|_| {}).expect("taken by a link"));
    assert_eq!(created.get(), 0, "a link that works places the lock whole");
    // Rename refused, link fails for another reason: that error is the answer.
    let link_is_storage = |_: &Path, _: &Path| -> std::io::Result<()> { return Err(refusal(libc::EIO)); };
    let platform: Platform = Platform { rename: &rename_refused, link: &link_is_storage, create: &create, ..SYSTEM };
    assert!(try_acquire_on(&platform, &index, 3).is_none());
    assert_eq!(created.get(), 0, "a failing disk is not a filesystem without links");
    // Both refused.
    let platform: Platform = Platform { rename: &rename_refused, link: &link_refused, create: &create, ..SYSTEM };
    drop(acquire_on(&platform, &index, 4, &|_| {}).expect("taken by exclusive creation"));
    assert_eq!(created.get(), 1, "both refusals make the one creation");
    remove(root.as_path());
}

/// The lock made by exclusive creation holds the marker of its holder, whole, by the time the hold begins; a lock another
/// program holds is waited for to the ceiling and stays exactly as it was, whether it is a Git's empty lock (the state a kill
/// between creation and marker leaves) or anything else, and the wait ends with `IndexInUse` and words that name the file and the fix.
#[test]
fn a_lock_that_exists_is_never_removed_or_replaced_by_the_fallback() {
    let (root, index) = index_in("lock-creation-existing");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, b"").expect("a lock without a marker, as plain Git leaves one");
    let platform: Platform = Platform { rename: &rename_refused, link: &link_refused, ..SYSTEM };
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let failure = acquire_within(&platform, &index, 5, SHORT_CEILING, &|note| {
        if let Announce::Waiting(text) = note {
            said.borrow_mut().push(text);
        }
    })
    .expect_err("a held lock ends at the ceiling");
    assert_eq!(failure.code, FailureCode::IndexInUse);
    let said: String = said.borrow().join("\n");
    assert!(
        said.contains(lock.to_string_lossy().as_ref()) && said.contains("removing the file") && said.contains("was not made"),
        "the person is told which file this is and how to clear it: {said}"
    );
    assert_eq!(std::fs::read(&lock).expect("the lock stays"), b"", "the lock without a marker is left as it was");
    assert!(try_acquire_on(&platform, &index, 6).is_none());
    assert!(lock.exists());
    assert_eq!(names_beside_the_index(&index), vec![String::from("index.lock")], "the attempts leave no staging file");
    remove(root.as_path());
}

/// A lock with the whole marker of a service that is gone is stale, yet where no claim file can be placed (the same filesystem)
/// nothing proves it safe to remove: it stays, and the wait ends at the ceiling.
#[test]
fn a_stale_marked_lock_stays_where_no_claim_can_be_placed() {
    let (root, index) = index_in("lock-creation-stale-marked");
    let lock: PathBuf = lock_path_of(&index);
    let dead: i64 = {
        let mut child = std::process::Command::new("true").spawn().expect("a short process");
        let pid: i64 = i64::from(child.id());
        child.wait().expect("reaped");
        pid
    };
    let own: String = marker_text(9, &own_identity).expect("identity");
    let first_field: &str = own.split_whitespace().next().expect("the marker's first word");
    let birth: String = own.split_whitespace().skip(2).collect::<Vec<_>>().join(" ");
    std::fs::write(&lock, format!("{first_field} {dead} {birth}\n")).expect("a dead service's marked lock");
    let platform: Platform = Platform { rename: &rename_refused, link: &link_refused, ..SYSTEM };
    let failure = acquire_within(&platform, &index, 10, SHORT_CEILING, &|_| {}).expect_err("not removed, so it ends at the ceiling");
    assert_eq!(failure.code, FailureCode::IndexInUse);
    assert!(lock.exists(), "a lock nothing proves safe to remove stays");
    remove(root.as_path());
}

/// A marker that cannot be flushed after the lock was created removes the lock again, so a failure here never leaves a lock of
/// ours that nothing could clear; the failure is the answer.
#[test]
fn a_marker_that_cannot_be_flushed_leaves_no_lock() {
    let (root, index) = index_in("lock-creation-sync-fails");
    let failing = |_: &std::fs::File| -> std::io::Result<()> { return Err(refusal(libc::EIO)); };
    // The staged marker is flushed first by the same call, so the flush fails there before any creation; a flush that works
    //  for the staged file and fails for the created one is what this control needs.
    let flushes: Cell<usize> = Cell::new(0);
    let second_fails = |file: &std::fs::File| -> std::io::Result<()> {
        flushes.set(flushes.get() + 1);
        if flushes.get() >= 2 { return failing(file); }
        return Ok(());
    };
    let platform: Platform = Platform { rename: &rename_refused, link: &link_refused, sync: &second_fails, ..SYSTEM };
    assert!(try_acquire_on(&platform, &index, 11).is_none());
    assert_eq!(flushes.get(), 2, "the staged file was flushed, then the created lock");
    assert!(!lock_path_of(&index).exists(), "the lock this attempt created is removed again");
    assert!(names_beside_the_index(&index).is_empty(), "{:?}", names_beside_the_index(&index));
    remove(root.as_path());
}

/// A creation that fails for any reason is the answer, and nothing is left.
#[test]
fn a_creation_that_fails_is_the_answer_and_leaves_nothing() {
    let (root, index) = index_in("lock-creation-refused");
    let failing = |_: &Path| -> std::io::Result<std::fs::File> { return Err(refusal(libc::EIO)); };
    let platform: Platform = Platform { rename: &rename_refused, link: &link_refused, create: &failing, ..SYSTEM };
    let failure = acquire_on(&platform, &index, 12, &|_| {}).expect_err("refused");
    assert_eq!(failure.code, FailureCode::Unavailable);
    assert!(names_beside_the_index(&index).is_empty(), "{:?}", names_beside_the_index(&index));
    remove(root.as_path());
}
