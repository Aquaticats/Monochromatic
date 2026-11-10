//! What:
//!  What a landing waits for when a reference lock refused its branch write,
//!  and what it says about the locks that are left.
//! Why:
//!  A lock a live process holds lets go on its own,
//!  a lock nothing proves gone is never removed,
//!  and the words must name the lock and the command that clears it
//!  (`SPEC.md`,
//!  "Reference locks left by a Git that was killed").
//!  Every control plants the files it judges in a disposable fixture,
//!  and none of them waits out the real margin.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(await waitOutTheReferenceLocks([gitDir], judge(), now())).toEqual({ blocked: expect.stringContaining('rm -- ') });
//! ```

/// The wait under control.
use super::{REFERENCE_LOCK_POLL, Waiting, wait_out_the_reference_locks};
/// A judgement that takes any lock as old enough and plants the processes it judges.
use crate::stale_ref_lock_killed_git::judge;
/// Its own disposable directory.
use crate::test_support::{fixture, remove};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// A Git directory of this control's own.
fn git_directory(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(git_dir.join("refs/heads")).expect("the Git directory");
    return (root, git_dir);
}

/// The words a `Blocked` answer carries,
/// and a failure when the locks were gone instead.
fn blocked(outcome: Waiting) -> String {
    return match outcome {
        Waiting::Gone => panic!("nothing stands, so the write may be tried again"),
        Waiting::Blocked { words, .. } => words,
    };
}

/// What:
///  A branch lock that nothing proves gone is left where it is and named,
///  with the command that clears it.
/// Why:
///  This is the lock a killed `git update-ref --stdin` leaves,
///  and removing it on a guess would take the lock of a Git that is running.
#[test]
fn a_reference_lock_nothing_proves_gone_is_named_with_the_command_that_clears_it() {
    let (root, git_dir) = git_directory("reference-lock-left");
    let lock: PathBuf = git_dir.join("refs/heads/main.lock");
    std::fs::write(&lock, "").expect("the lock a killed Git leaves");
    let words: String = blocked(wait_out_the_reference_locks(std::slice::from_ref(&git_dir), &judge(), Instant::now()));
    assert!(lock.exists(), "a lock nothing proves gone is never removed: {}", lock.display());
    assert!(words.contains(lock.display().to_string().as_str()), "the lock is named: {words}");
    assert!(words.contains("rm -- "), "the command that clears it is given: {words}");
    assert!(words.contains("Nothing proves that a Git process still holds the lock"), "the words say what is not known: {words}");
    remove(root.as_path());
}

/// What:
///  A lock a live process has open is waited for,
///  and named when the wait runs out,
///  never removed.
/// Why:
///   A descriptor is the one sign that a process is still using the file,
///  so a landing must not take it from that process.
#[test]
fn a_lock_a_live_process_has_open_is_named_and_left() {
    let (root, git_dir) = git_directory("reference-lock-held");
    let lock: PathBuf = git_dir.join("refs/heads/main.lock");
    std::fs::write(&lock, "").expect("the lock");
    // The scan of the real judge ignores this process, so the holder is a child of this control, and the judge is told
    //  to look at that child alone.
    let mut holder: std::process::Child = std::process::Command::new("sh")
        .arg("-c")
        .arg("exec 3<>\"$1\"; exec sleep 120")
        .arg("sh")
        .arg(lock.as_os_str())
        .spawn()
        .expect("a process that has the lock open");
    let holder_pid: i64 = i64::from(holder.id());
    let descriptors: PathBuf = PathBuf::from(format!("/proc/{holder_pid}/fd"));
    let started: Instant = Instant::now();
    while !std::fs::read_dir(descriptors.as_path()).is_ok_and(|entries| {
        return entries.flatten().any(|entry| { return std::fs::read_link(entry.path()).is_ok_and(|target| { return target == lock; }); });
    }) {
        assert!(started.elapsed() < Duration::from_secs(30), "the holder opened the lock");
        std::thread::sleep(Duration::from_millis(20));
    }
    let seeing: crate::stale_ref_locks::Judge = crate::stale_ref_locks::Judge { scan_scope: Some(vec![holder_pid]), ..judge() };
    let words: String = blocked(wait_out_the_reference_locks(std::slice::from_ref(&git_dir), &seeing, Instant::now()));
    assert!(lock.exists(), "the lock of a live holder is never removed: {}", lock.display());
    assert!(words.contains(lock.display().to_string().as_str()), "the lock is named: {words}");
    assert!(words.contains("A program is using the lock"), "the words say a program has it open: {words}");
    let _killed = holder.kill();
    let _reaped = holder.wait();
    remove(root.as_path());
}

/// What:
///  A packed-references staging file with no lock beside it is removed on the proof the judgement owns,
///  and the landing may try its write again.
/// Why:
///  Git makes `packed-refs.new` only while it holds `packed-refs.lock`,
///  so a staging file with no lock has no live maker,
///  and it refuses a later update exactly as a lock does.
#[test]
fn a_staging_file_with_no_lock_is_cleared_and_the_write_may_be_tried_again() {
    let (root, git_dir) = git_directory("reference-staging-orphan");
    let staging: PathBuf = git_dir.join("packed-refs.new");
    std::fs::write(&staging, "").expect("the staging file a killed Git leaves");
    let outcome: Waiting = wait_out_the_reference_locks(std::slice::from_ref(&git_dir), &judge(), Instant::now());
    assert!(matches!(outcome, Waiting::Gone), "the staging file is removed on its proof");
    assert!(!staging.exists(), "the staging file is gone");
    remove(root.as_path());
}

/// What:
///  A repository with no reference lock in the way is not waited for at all.
/// Why:
///   The common case is a write refused by a branch that moved,
///  and the branch is read before this wait:
///  nothing here may add a step to a landing that needs none.
#[test]
fn nothing_standing_costs_no_wait() {
    let (root, git_dir) = git_directory("reference-lock-none");
    let began: Instant = Instant::now();
    let outcome: Waiting = wait_out_the_reference_locks(std::slice::from_ref(&git_dir), &judge(), began);
    assert!(matches!(outcome, Waiting::Gone));
    assert!(began.elapsed() < REFERENCE_LOCK_POLL * 10, "nothing standing is answered at once");
    remove(root.as_path());
}

/// What:
///  A lock younger than the margin is looked at again rather than called unproven,
///  and the wait ends at its own deadline.
/// Why:
///   A live Git may still be filling the lock it just made,
///  and a landing that judged it at once would report a lock that was about to be renamed into place.
#[test]
fn a_young_lock_is_looked_at_again_until_the_deadline_and_then_named() {
    let (root, git_dir) = git_directory("reference-lock-young");
    let lock: PathBuf = git_dir.join("refs/heads/main.lock");
    std::fs::write(&lock, "").expect("the lock a live Git has just made");
    // The judge of this process waits out the real margin before judging; a deadline already past ends the wait at once.
    let began: Instant = Instant::now();
    let outcome: Waiting = wait_out_the_reference_locks(std::slice::from_ref(&git_dir), &crate::stale_ref_locks::Judge::here(), began + Duration::from_millis(60));
    assert!(began.elapsed() < Duration::from_secs(5), "the wait ends at its deadline");
    let words: String = blocked(outcome);
    assert!(words.contains(lock.display().to_string().as_str()), "a lock that is still young at the deadline is named: {words}");
    assert!(lock.exists(), "nothing is removed");
    remove(root.as_path());
}
