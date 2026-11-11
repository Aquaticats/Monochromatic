//! Controls for reading `Cargo.lock` before a clean and putting it back afterwards,
//! and for the bytes a clean displaced:
//! a lock another command wrote must never be overwritten without a copy of it being kept,
//! and a lock no clean touched must not be written again.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// The bytes the checkout left,
/// which the person reads as the lock.
const CHECKED_OUT: &[u8] = b"# the checked-out lock\n";

/// The bytes a command that is not the clean wrote while the clean ran.
const ANOTHER_COMMAND: &[u8] = b"# a lock another command wrote\n";

/// What:
///  The Git directory these controls keep displaced bytes in.
/// Why:
///   The repair is handed the Git directory of the worktree it repairs,
/// and a copy kept there is outside the worktree,
///  so the checked-out tree stays clean and the copy still belongs to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDir = join(root, 'git');
/// ```
fn git_directory(root: &Path) -> PathBuf {
    return root.join("git");
}

/// What:
///  The copy of a displaced lock,
///  which is the only one a single displacement makes.
/// Why:
///   The directory is the one the product names the lock's path by,
/// and one displacement puts one file in it,
///  so a control that reads the directory's only entry reads the copy the product made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copies = readdirSync(copiesDirectory(gitDir, lock)); assert.equal(copies.length, 1); return join(directory, copies[0]);
/// ```
fn the_kept_copy(git_dir: &Path, lock: &Path) -> PathBuf {
    let directory: PathBuf = copies_directory(git_dir, lock);
    let mut entries: Vec<PathBuf> = std::fs::read_dir(directory.as_path())
        .expect("the copies directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .collect();
    assert_eq!(entries.len(), 1, "one displacement makes one copy: {entries:?}");
    return entries.remove(0);
}

/// What is kept when the clean displaced bytes nobody can attribute to it:
/// the bytes themselves,
/// and the checked-out lock back in the worktree.
/// Why:
///   A comparison at restoration time cannot tell Cargo's own rewrite of a lock that no longer matches
/// its manifest from a command or person's newer write,
///  so the bytes that stand there are kept before the checked-out ones are put back:
/// nothing of the person's is discarded,
///  and the worktree is not left dirty by a clean.
/// The line names the copy,
/// because a person looking for a lock their own command wrote has to be able to find it.
#[test]
fn a_lock_another_command_wrote_is_kept_before_the_checked_out_one_comes_back() {
    let root: PathBuf = fixture("cargo-lock-displaced-by-another-command");
    let lock: PathBuf = root.join(LOCK_FILE);
    let git_dir: PathBuf = git_directory(root.as_path());
    std::fs::write(lock.as_path(), CHECKED_OUT).expect("the checked-out lock");
    let before: Option<Vec<u8>> = read_lock(root.as_path());
    // A command that is not this clean writes the lock while the clean runs.
    std::fs::write(lock.as_path(), ANOTHER_COMMAND).expect("a newer lock");
    let said: Option<String> = restore_lock(root.as_path(), before.as_deref(), Some(git_dir.as_path()));
    assert_eq!(std::fs::read(lock.as_path()).expect("the lock"), CHECKED_OUT, "the lock is back as checked out");
    let kept: PathBuf = the_kept_copy(git_dir.as_path(), lock.as_path());
    assert_eq!(std::fs::read(kept.as_path()).expect("the kept bytes"), ANOTHER_COMMAND, "the bytes that stood there are kept whole");
    let words: String = said.expect("the person is told what was kept and where");
    assert!(words.contains(kept.display().to_string().as_str()), "{words}: the line names the copy the person can read");
    remove(root.as_path());
}

/// What a clean that changed nothing does:
/// nothing,
/// and says nothing.
/// Why:
///   An untouched lock keeps its file time,
/// and a worktree whose build state needed no rewrite is not written to by the repair at all.
#[test]
fn a_lock_no_clean_touched_is_not_written_again() {
    let root: PathBuf = fixture("cargo-lock-untouched");
    let lock: PathBuf = root.join(LOCK_FILE);
    std::fs::write(lock.as_path(), CHECKED_OUT).expect("the checked-out lock");
    let before: Option<Vec<u8>> = read_lock(root.as_path());
    let time: std::time::SystemTime = std::fs::metadata(lock.as_path()).expect("the lock").modified().expect("a change time");
    assert_eq!(restore_lock(root.as_path(), before.as_deref(), Some(git_directory(root.as_path()).as_path())), None, "nothing changed, so nothing is said");
    assert_eq!(std::fs::metadata(lock.as_path()).expect("the lock").modified().expect("a change time"), time, "the file keeps its time");
    remove(root.as_path());
}

/// What a clean that removed the lock does:
/// puts the checked-out lock back,
/// and says nothing,
/// because no bytes of anyone's were displaced.
/// Why:
///   A lock the checkout had must come back after a clean that removed it,
/// and there is nothing to keep when nothing stood there.
#[test]
fn a_lock_the_clean_removed_is_put_back_as_checked_out() {
    let root: PathBuf = fixture("cargo-lock-removed-by-the-clean");
    let lock: PathBuf = root.join(LOCK_FILE);
    std::fs::write(lock.as_path(), CHECKED_OUT).expect("the checked-out lock");
    let before: Option<Vec<u8>> = read_lock(root.as_path());
    std::fs::remove_file(lock.as_path()).expect("a clean that removed the lock");
    let said: Option<String> = restore_lock(root.as_path(), before.as_deref(), Some(git_directory(root.as_path()).as_path()));
    assert_eq!(std::fs::read(lock.as_path()).expect("the lock"), CHECKED_OUT, "the checked-out lock is back");
    assert!(said.is_none(), "no bytes were displaced, so the person has nothing to be told");
    remove(root.as_path());
}

/// What:
///  A lock whose displaced bytes cannot be kept is left exactly as it was found,
///  and the person is told.
/// Why:
///   The checked-out bytes are in the branch,
/// and the bytes standing there are nowhere else,
///  so putting the branch's bytes back without having kept them would be the step that lost the only copy of the others.
#[test]
fn a_lock_whose_bytes_cannot_be_kept_is_left_as_it_was_found() {
    let root: PathBuf = fixture("cargo-lock-cannot-be-kept");
    let lock: PathBuf = root.join(LOCK_FILE);
    let git_dir: PathBuf = git_directory(root.as_path());
    std::fs::write(lock.as_path(), CHECKED_OUT).expect("the checked-out lock");
    let before: Option<Vec<u8>> = read_lock(root.as_path());
    std::fs::write(lock.as_path(), ANOTHER_COMMAND).expect("a newer lock");
    // A Git directory that is a file cannot hold a copy of anything.
    std::fs::write(git_dir.as_path(), b"not a directory\n").expect("a Git directory that cannot be written in");
    let said: Option<String> = restore_lock(root.as_path(), before.as_deref(), Some(git_dir.as_path()));
    assert_eq!(std::fs::read(lock.as_path()).expect("the lock"), ANOTHER_COMMAND, "the bytes that could not be kept are still there");
    let words: String = said.expect("the person is told what happened");
    assert!(words.contains(lock.display().to_string().as_str()), "{words}: the line names the file it left alone");
    remove(root.as_path());
}

/// What:
///  A clean given no Git directory to keep anything in leaves the lock it displaced alone.
/// Why:
///   A repair that does not know the worktree's Git directory has nowhere to keep the bytes,
/// and a copy inside the worktree would make the new worktree dirty,
///  so the file is left as the clean left it and the line says so.
#[test]
fn a_lock_displaced_with_no_place_to_keep_it_is_left_as_it_was_found() {
    let root: PathBuf = fixture("cargo-lock-no-place-to-keep-it");
    let lock: PathBuf = root.join(LOCK_FILE);
    std::fs::write(lock.as_path(), CHECKED_OUT).expect("the checked-out lock");
    let before: Option<Vec<u8>> = read_lock(root.as_path());
    std::fs::write(lock.as_path(), ANOTHER_COMMAND).expect("a newer lock");
    let said: Option<String> = restore_lock(root.as_path(), before.as_deref(), None);
    assert_eq!(std::fs::read(lock.as_path()).expect("the lock"), ANOTHER_COMMAND, "the lock is left as the clean left it");
    assert!(said.is_some(), "the person is told why the worktree shows a difference");
    remove(root.as_path());
}

/// What:
///  A second displacement of one lock keeps a copy of its own,
///  and the first copy stays as it was.
/// Why:
///   The first copy is the oldest statement of what a clean found,
/// so replacing it would lose the bytes of an earlier repair,
///  and a person looking at a worktree that was repaired twice must find both.
#[test]
fn a_second_displacement_of_one_lock_keeps_its_own_copy() {
    let root: PathBuf = fixture("cargo-lock-displaced-twice");
    let lock: PathBuf = root.join(LOCK_FILE);
    let git_dir: PathBuf = git_directory(root.as_path());
    std::fs::write(lock.as_path(), CHECKED_OUT).expect("the checked-out lock");
    let before: Option<Vec<u8>> = read_lock(root.as_path());
    std::fs::write(lock.as_path(), ANOTHER_COMMAND).expect("a newer lock");
    let _first: Option<String> = restore_lock(root.as_path(), before.as_deref(), Some(git_dir.as_path()));
    // A later clean of the same workspace finds the checked-out lock and a command's newer write again.
    std::fs::write(lock.as_path(), b"# a third lock\n").expect("another newer lock");
    let _second: Option<String> = restore_lock(root.as_path(), before.as_deref(), Some(git_dir.as_path()));
    let directory: PathBuf = copies_directory(git_dir.as_path(), lock.as_path());
    let mut written: Vec<Vec<u8>> = std::fs::read_dir(directory.as_path())
        .expect("the copies directory")
        .flatten()
        .map(|entry| { return std::fs::read(entry.path()).expect("a kept copy"); })
        .collect();
    written.sort();
    assert_eq!(written.len(), 2, "one copy per displacement, and neither is replaced");
    assert!(written.contains(&ANOTHER_COMMAND.to_vec()), "the first displacement's bytes are still there");
    assert!(written.contains(&b"# a third lock\n".to_vec()), "the second displacement's bytes are there too");
    remove(root.as_path());
}

/// What:
///  A lock a clean created where the checkout had none is kept and removed again.
/// Why:
///   The worktree must not hold a file the checkout does not,
/// and the bytes Cargo resolved are the only record of what that clean found.
#[test]
fn a_lock_the_clean_created_is_kept_and_removed_again() {
    let root: PathBuf = fixture("cargo-lock-created-by-the-clean");
    let lock: PathBuf = root.join(LOCK_FILE);
    let git_dir: PathBuf = git_directory(root.as_path());
    let before: Option<Vec<u8>> = read_lock(root.as_path());
    assert!(before.is_none(), "the checkout has no lock");
    std::fs::write(lock.as_path(), ANOTHER_COMMAND).expect("the lock the clean created");
    let said: Option<String> = restore_lock(root.as_path(), before.as_deref(), Some(git_dir.as_path()));
    assert!(!lock.exists(), "the worktree no longer holds a file the checkout does not");
    let kept: PathBuf = the_kept_copy(git_dir.as_path(), lock.as_path());
    assert_eq!(std::fs::read(kept.as_path()).expect("the kept bytes"), ANOTHER_COMMAND, "what the clean resolved is kept");
    assert!(said.is_some(), "the person is told what was kept");
    remove(root.as_path());
}
