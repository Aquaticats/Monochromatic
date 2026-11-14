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
    let before: Option<Vec<u8>> = read_lock(root.as_path()).expect("the checked-out lock is a regular file");
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
    let before: Option<Vec<u8>> = read_lock(root.as_path()).expect("the checked-out lock is a regular file");
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
    let before: Option<Vec<u8>> = read_lock(root.as_path()).expect("the checked-out lock is a regular file");
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
    let before: Option<Vec<u8>> = read_lock(root.as_path()).expect("the checked-out lock is a regular file");
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
    let before: Option<Vec<u8>> = read_lock(root.as_path()).expect("the checked-out lock is a regular file");
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
    let before: Option<Vec<u8>> = read_lock(root.as_path()).expect("the checked-out lock is a regular file");
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
    let before: Option<Vec<u8>> = read_lock(root.as_path()).expect("the checked-out lock is a regular file");
    assert!(before.is_none(), "the checkout has no lock");
    std::fs::write(lock.as_path(), ANOTHER_COMMAND).expect("the lock the clean created");
    let said: Option<String> = restore_lock(root.as_path(), before.as_deref(), Some(git_dir.as_path()));
    assert!(!lock.exists(), "the worktree no longer holds a file the checkout does not");
    let kept: PathBuf = the_kept_copy(git_dir.as_path(), lock.as_path());
    assert_eq!(std::fs::read(kept.as_path()).expect("the kept bytes"), ANOTHER_COMMAND, "what the clean resolved is kept");
    assert!(said.is_some(), "the person is told what was kept");
    remove(root.as_path());
}

/// The bytes Cargo's own resolution wrote for the clean,
/// which is the lock the restore may replace.
const THE_CLEANS_OWN: &[u8] = b"# the resolution's lock\n";

/// The bytes a person writes while the checked-out lock is being put back.
const WRITTEN_AFTER_THE_READING: &[u8] = b"# the person's newest write\n";

/// What:
///  The copy under the displaced directory that holds `bytes`.
/// Why:
///   A displacement keeps what it found and a restore may stage what it puts back,
/// so a control names the copy by the bytes it holds rather than by the only entry in the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdirSync(copiesDirectory(gitDir, lock)).map(join).find((copy) => readFileSync(copy).equals(bytes));
/// ```
fn the_copy_holding(git_dir: &Path, lock: &Path, bytes: &[u8]) -> PathBuf {
    let directory: PathBuf = copies_directory(git_dir, lock);
    for entry in std::fs::read_dir(directory.as_path()).expect("the copies directory").flatten() {
        if std::fs::read(entry.path()).expect("a kept copy") == bytes {
            return entry.path();
        }
    }
    panic!("no copy holds {}", String::from_utf8_lossy(bytes));
}

/// What:
///  A write that lands after the reading that judged the lock stays in the person's own file,
///  and the checked-out bytes are kept where the notice says.
/// Why:
///  The service's restore runs after the add returned, when the person may be working in the worktree,
/// and the reading that decides whether the lock is the clean's own is a moment of its own.
/// A write between that reading and the step that puts the checked-out bytes back was in neither the preserved
/// copy nor the file afterwards: the copy held the resolution's bytes and the file held the checkout's,
/// which is the repair discarding what a person wrote (never discard user data).
/// The control writes at the seam the restoring step follows, which is exactly the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// betweenTheReadingAndTheRestore(() => writeFileSync(lock, persons)); restoreOnlyTheCleansOwn(...); expect(readFileSync(lock)).toEqual(persons);
/// ```
#[test]
fn a_lock_written_after_the_reading_stays_as_the_person_wrote_it() {
    let root: PathBuf = fixture("cargo-lock-written-during-the-restore");
    let lock: PathBuf = root.join(LOCK_FILE);
    let git_dir: PathBuf = git_directory(root.as_path());
    std::fs::write(lock.as_path(), CHECKED_OUT).expect("the checked-out lock");
    let before: Option<Vec<u8>> = read_lock(root.as_path()).expect("the checked-out lock is a regular file");
    std::fs::write(lock.as_path(), THE_CLEANS_OWN).expect("the lock the clean's resolution wrote");
    let persons: PathBuf = lock.clone();
    BETWEEN_THE_READING_AND_THE_RESTORE.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            std::fs::write(persons.as_path(), WRITTEN_AFTER_THE_READING).expect("the person's write");
        }));
    });
    let said: Option<String> = restore_only_the_cleans_own(root.as_path(), before.as_deref(), Some(git_dir.as_path()), THE_CLEANS_OWN);
    BETWEEN_THE_READING_AND_THE_RESTORE.with(|hook| { *hook.borrow_mut() = None; });
    let standing: Vec<u8> = std::fs::read(lock.as_path()).expect("the lock");
    assert_eq!(standing, WRITTEN_AFTER_THE_READING, "the person's write stays in their own lock");
    let words: String = said.expect("the person is told where the checked-out lock is kept");
    let kept: PathBuf = the_copy_holding(git_dir.as_path(), lock.as_path(), CHECKED_OUT);
    assert!(words.contains(kept.display().to_string().as_str()), "{words}: the line names where the checked-out lock is kept");
    remove(root.as_path());
}

/// What:
///  Make a named pipe at `path`.
/// Why:
///   A named pipe is the one foreign object whose plain open waits for a writer that may never come,
/// which is what the lock's reads must never do at a name anyone can write.
fn named_pipe_at(path: &Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let named: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("a fixture path holds no NUL byte");
    // SAFETY: the string is NUL-terminated and outlives the call, and the name is a fresh path in the fixture.
    let made: libc::c_int = unsafe { libc::mkfifo(named.as_ptr(), 0o600) };
    assert_eq!(made, 0, "make a named pipe at {}: {}", path.display(), std::io::Error::last_os_error());
}

/// The bytes the person wrote first, while the resolution ran.
const THE_PERSONS_FIRST_WRITE: &[u8] = b"# the person wrote this first\n";

/// The bytes the person wrote again while the repair was putting the first write back.
const THE_PERSONS_NEWEST_WRITE: &[u8] = b"# the person wrote this newest\n";

/// What Cargo resolved for the workspace, in the mirror of it.
const THE_RESOLUTION: &[u8] = b"# what the resolution wrote\n";

/// What:
///  Every file whose bytes are `bytes` under the displaced-lock store,
///  which is where a step keeps what it displaced.
/// Why:
///   A control has to find the copy the answer names without depending on the order the names were made in,
/// and the store is the only place this module keeps bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copiesWith = (gitDir, bytes) => walk(join(gitDir, DISPLACED_DIRECTORY)).filter((p) => readFileSync(p).equals(bytes));
/// ```
fn copies_with(git_dir: &Path, bytes: &[u8]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![git_dir.join(DISPLACED_DIRECTORY)];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(directory.as_path()) else { continue };
        for entry in listing.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if std::fs::read(path.as_path()).ok().as_deref() == Some(bytes) {
                found.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// What:
///  A write that lands between the reading of what the first exchange displaced and the rollback is kept,
///  under a name the answer gives,
///  and the answer says what is true.
/// Why:
///   The rollback exchanges the person's earlier write back into the lock,
/// and whatever that exchange displaces is a write that landed after the reading:
/// it is a person's bytes until proven otherwise,
/// so it is kept and named and never removed, as the older `judge_the_displaced` keeps what it displaced.
/// A kill at either moment is covered by the same rule: the resolution is the only bytes this step deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hook(() => writeFileSync(lock, NEWEST)); expect(placeTheResolution(...)).toEqual(gaveWay); expect(readFileSync(lock)).toEqual(FIRST); expect(copiesWith(gitDir, NEWEST).length).toBe(1)
/// ```
#[test]
fn a_write_between_the_reading_and_the_rollback_is_kept_and_named() {
    let root: PathBuf = fixture("cargo-lock-written-during-the-rollback");
    let lock: PathBuf = root.join(LOCK_FILE);
    let git_dir: PathBuf = git_directory(root.as_path());
    // The filling read the checked-out bytes at its start, and the person has written once since, while the resolution ran.
    std::fs::write(lock.as_path(), THE_PERSONS_FIRST_WRITE).expect("the person's first write");
    let planted: PathBuf = lock.clone();
    BETWEEN_THE_READING_AND_THE_ROLLBACK.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            std::fs::write(planted.as_path(), THE_PERSONS_NEWEST_WRITE).expect("the person's newest write");
        }));
    });
    let placed: Placed = place_the_resolution(root.as_path(), THE_RESOLUTION, Some(CHECKED_OUT), Some(git_dir.as_path()));
    BETWEEN_THE_READING_AND_THE_ROLLBACK.with(|hook| { *hook.borrow_mut() = None; });
    let Placed::GaveWay(words) = placed else { panic!("the repair gives way rather than keep the resolution: {placed:?}") };
    assert_eq!(std::fs::read(lock.as_path()).expect("the lock"), THE_PERSONS_FIRST_WRITE, "the person's earlier write is back at the lock: {words}");
    let kept: Vec<PathBuf> = copies_with(git_dir.as_path(), THE_PERSONS_NEWEST_WRITE);
    assert_eq!(kept.len(), 1, "the newer write is kept whole, once: {kept:?}");
    assert!(words.contains(kept[0].display().to_string().as_str()), "the answer names where the newer write is kept: {words}");
    remove(root.as_path());
}

/// What:
///  With no write in that window the person's earlier write is back at the lock,
///  the resolution is gone,
///  and nothing is kept beside it.
/// Why:
///   This is the control of the control:
/// the window must not be the only thing that makes the answer say bytes were kept.
/// The bytes the exchange displaces there are the resolution, which is this step's own, so it is removed as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(placeTheResolution(...)).toEqual(gaveWay); expect(readFileSync(lock)).toEqual(FIRST); expect(copiesWith(gitDir, NEWEST).length).toBe(0)
/// ```
#[test]
fn a_rollback_with_no_write_in_its_window_keeps_nothing_beside_the_lock() {
    let root: PathBuf = fixture("cargo-lock-rolled-back-with-nothing-else");
    let lock: PathBuf = root.join(LOCK_FILE);
    let git_dir: PathBuf = git_directory(root.as_path());
    std::fs::write(lock.as_path(), THE_PERSONS_FIRST_WRITE).expect("the person's first write");
    let placed: Placed = place_the_resolution(root.as_path(), THE_RESOLUTION, Some(CHECKED_OUT), Some(git_dir.as_path()));
    let Placed::GaveWay(words) = placed else { panic!("the repair gives way: {placed:?}") };
    assert_eq!(std::fs::read(lock.as_path()).expect("the lock"), THE_PERSONS_FIRST_WRITE, "the person's write is back at the lock: {words}");
    assert!(copies_with(git_dir.as_path(), THE_PERSONS_NEWEST_WRITE).is_empty(), "no newer write was kept, because there was none");
    assert!(copies_with(git_dir.as_path(), THE_RESOLUTION).is_empty(), "the resolution is this step's own and is removed: {words}");
    remove(root.as_path());
}

/// What:
///  A named pipe at the lock's name refuses the placement's readings at once,
///  and the pipe is put back where it stood without being read, written over or removed.
/// Why:
///   The exchange moves whatever stands at the lock's name into the staged file,
/// and the read that follows would open a pipe and wait for a writer that never comes:
/// no ceiling could expire on the thread (`SPEC.md`, "Every wait has a bound").
/// What stands there is the person's, so the answer is a refusal that names it,
/// and the thing itself is back at the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(lock); expect(within(10_000, () => placeTheResolution(...))).toContain("is not a regular file"); expect(lstatSync(lock).isFIFO()).toBe(true)
/// ```
#[test]
fn a_named_pipe_at_the_lock_never_holds_the_placement() {
    let root: PathBuf = fixture("cargo-lock-pipe-at-the-placement");
    let lock: PathBuf = root.join(LOCK_FILE);
    let git_dir: PathBuf = git_directory(root.as_path());
    named_pipe_at(lock.as_path());
    let covered: PathBuf = root.clone();
    let kept_in: PathBuf = git_dir.clone();
    let placing: Option<Placed> = crate::test_support::finished_within_at(std::time::Duration::from_secs(10), move || {
        return place_the_resolution(root.as_path(), THE_RESOLUTION, Some(CHECKED_OUT), Some(kept_in.as_path()));
    });
    let placed: Placed = placing.expect("placing the resolution must not wait on a named pipe at the lock's name");
    let Placed::GaveWay(words) = placed else { panic!("a named pipe at the lock is refused: {placed:?}") };
    assert!(words.contains("not a regular file"), "the answer names what stands there: {words}");
    let standing: std::fs::Metadata = std::fs::symlink_metadata(lock.as_path()).expect("the pipe is still there");
    {
        use std::os::unix::fs::FileTypeExt as _;
        assert!(standing.file_type().is_fifo(), "the person's pipe is back where it stood, untouched: {}", lock.display());
    }
    remove(covered.as_path());
}

/// What:
///  The filling's first read of the lock refuses a named pipe at once,
///  and names the file and what stands there.
/// Why:
///   That read runs before any bounded step of the filling,
/// so a pipe there would hold the service's task for ever with no ceiling to reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(lock); expect(within(10_000, () => readLock(root))).toContain("is not a regular file")
/// ```
#[test]
fn a_named_pipe_at_the_lock_never_holds_the_read_of_it() {
    let root: PathBuf = fixture("cargo-lock-pipe-at-the-first-read");
    let lock: PathBuf = root.join(LOCK_FILE);
    named_pipe_at(lock.as_path());
    let covered: PathBuf = root.clone();
    let reading: Option<Result<Option<Vec<u8>>, String>> = crate::test_support::finished_within_at(std::time::Duration::from_secs(10), move || return read_lock(root.as_path()));
    let read: Result<Option<Vec<u8>>, String> = reading.expect("the lock's read must not wait on a named pipe at its name");
    let Err(reason) = read else { panic!("a named pipe at the lock is refused: {read:?}") };
    assert!(reason.contains(LOCK_FILE) && reason.contains("not a regular file"), "the refusal names the file and what stands there: {reason}");
    remove(covered.as_path());
}
