//! What:
//!  A lost `index-lock-1.json` next to a held `index.lock`, which once left every commit stopped on an unproven owner,
//!  and the controls that keep the stop where recovery cannot prove the lock is gone with its creator.
//! Why:
//!  A lock without its record is released when the PID file names the dead owner and the dates, the process table and
//!  the open files agree. Every control below starts a real stand-in for a live process, or a lock of unknown origin, and
//!  requires the lock to stay. The lost owner name is in `binary_recovery_tearing_converge_tests.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await loseName('landing-locked', 'index-lock-1.json'); expect(await classify('landing-locked')).toBe('discards');
//! ```

use super::converge::status;
use super::names::{Outcome, lose_name, settle, transaction_directory};
use super::{Paused, Probe, pause_at, probe};
use crate::commit::remove_settled;
use crate::support::Sleeper;
use git_policy_cli::process_identity::process_birth_identity;
use git_policy_cli::transaction_owner::{encode_transaction_owner, parse_transaction_owner};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// A lost `index-lock-1.json` converges: the PID file names the dead owner, the lock is empty and dated between the
///  owner record and the PID file, no process has the owner's number, and nobody holds the lock, so recovery releases
///  it and the next commit lands.
#[test]
fn a_lost_index_lock_name_converges() {
    for phase in ["landing-locked", "objects-migrated"] {
        let paused: Paused = pause_at(phase);
        let directory: PathBuf = transaction_directory(paused.repo.join(".git").as_path());
        lose_name(directory.as_path(), "index-lock-1.json");
        let outcome: Outcome = settle(paused);
        assert_eq!(outcome.cell, "discards", "{phase}: {outcome:?}");
    }
}

/// What:
///  A paused repository whose lock record is lost, and the paths the controls change.
/// Why:
///   Every control of the lock release starts from the same lost record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lost = await pausedWithoutLockRecord('landing-locked');
/// ```
struct LostLock {
    /// The paused repository.
    paused: Paused,
    /// Its transaction directory.
    directory: PathBuf,
    /// The real index lock.
    lock: PathBuf,
    /// The PID file beside it.
    pid_file: PathBuf,
}

/// Pause at `landing-locked` and lose the lock record.
fn lost_lock() -> LostLock {
    let paused: Paused = pause_at("landing-locked");
    let git_dir: PathBuf = paused.repo.join(".git");
    let directory: PathBuf = transaction_directory(git_dir.as_path());
    lose_name(directory.as_path(), "index-lock-1.json");
    let lock: PathBuf = git_dir.join("index.lock");
    let pid_file: PathBuf = git_dir.join("index~pid.lock");
    assert!(lock.exists() && pid_file.exists(), "the killed commit left its lock and PID file");
    return LostLock { paused, directory, lock, pid_file };
}

/// What:
///  The result of a commit after the first status, which recovers.
/// Why:
///   The stop of an unproven lock is reported by the commit that waits for it, not by the status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await run('git commit --message=after a.txt')
/// ```
fn commit_after_status(lost: &LostLock) -> Probe {
    let first: Probe = status(&lost.paused);
    assert_eq!(first.code, Some(0), "status works whatever the lock: {}", first.stderr);
    std::fs::write(lost.paused.repo.join("a.txt"), b"one\ntwo\nthree\n").expect("another edit");
    return probe(&lost.paused.fixture, lost.paused.repo.as_path(), &["commit", "--message=after", "a.txt"]);
}

/// A lock of unknown origin keeps the stop: with the PID file gone nothing shows which process created the lock, so the
///  commit exits 2, the lock stays, and the message names the exact path, how to check that nothing holds it, the
///  command that removes it and the condition under which that is safe.
#[test]
fn a_lock_of_unknown_origin_still_stops_every_commit_and_names_the_way_out() {
    let lost: LostLock = lost_lock();
    std::fs::remove_file(lost.pid_file.as_path()).expect("the PID file is announced after the record");
    let commit: Probe = commit_after_status(&lost);
    let context: String = commit.stderr.clone();
    assert_eq!(commit.code, Some(2), "{context}");
    assert!(lost.lock.exists(), "{context}");
    assert!(commit.stderr.contains(lost.lock.to_string_lossy().as_ref()), "the lock is named: {context}");
    assert!(commit.stderr.contains(format!("rm {}", lost.lock.display()).as_str()), "the removal is exact: {context}");
    assert!(commit.stderr.contains(format!("lsof {}", lost.lock.display()).as_str()), "the check is exact: {context}");
    assert!(commit.stderr.contains("Removal is safe when no process holds the lock"), "the condition is named: {context}");
    assert!(commit.stderr.contains("index-lock-unproven-owner") || commit.stderr.contains("unprovenOwnerTimeoutMs"), "{context}");
    remove_settled(&lost.paused.fixture);
}

/// A process that holds the lock open keeps it, though the PID file names the dead owner: the sleeper has the lock as
///  its standard input, the stand-in for a Git that took the lock after the wrapper's was released.
#[test]
fn a_process_holding_the_lock_open_keeps_it() {
    let lost: LostLock = lost_lock();
    let holder: Sleeper = Sleeper::start(lost.paused.repo.as_path(), Some(lost.lock.as_path()));
    let first: Probe = status(&lost.paused);
    assert!(lost.lock.exists(), "{}", first.stderr);
    assert!(first.stderr.contains(format!("process {} holds it open", holder.pid()).as_str()), "{}", first.stderr);
    drop(holder);
    remove_settled(&lost.paused.fixture);
}

/// A lock dated after the PID file was written is another process's, taken once the owner's was gone: it stays.
#[test]
fn a_lock_newer_than_the_pid_file_keeps_it() {
    let lost: LostLock = lost_lock();
    let announced: SystemTime = std::fs::metadata(lost.pid_file.as_path()).and_then(|metadata| { return metadata.modified(); }).expect("PID file date");
    std::fs::File::open(lost.lock.as_path())
        .expect("open the lock")
        .set_modified(announced + Duration::from_secs(60))
        .expect("date the lock");
    let first: Probe = status(&lost.paused);
    assert!(lost.lock.exists(), "{}", first.stderr);
    assert!(first.stderr.contains("another process took it"), "{}", first.stderr);
    remove_settled(&lost.paused.fixture);
}

/// What:
///  Rewrite the owner record to name another process, keeping the date it had, and the PID file to name it too.
/// Why:
///   The record's date is one of the proofs, so a control that changes the owner must not change it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(owner, bytes); await utimes(owner, before, before);
/// ```
fn name_owner(lost: &LostLock, pid: i64, identity: &str) {
    let path: PathBuf = lost.directory.join("owner.json");
    let before: SystemTime = std::fs::metadata(&path).and_then(|metadata| { return metadata.modified(); }).expect("owner date");
    let mut owner = parse_transaction_owner(std::fs::read(&path).expect("owner record").as_slice()).expect("owner parses");
    owner.owner_pid = pid;
    owner.owner_identity = String::from(identity);
    std::fs::write(&path, encode_transaction_owner(&owner)).expect("rewrite the owner");
    std::fs::File::open(&path).expect("open").set_modified(before).expect("restore the date");
    std::fs::write(lost.pid_file.as_path(), format!("pid {pid}\n")).expect("rewrite the PID file");
}

/// A process numbered like the owner keeps the lock: the owner record's identity says the owner is gone, but a process
///  with its number runs, so the PID file may name that process and the lock may be its own.
#[test]
fn a_reused_process_number_keeps_the_lock() {
    let lost: LostLock = lost_lock();
    let stand_in: Sleeper = Sleeper::start(lost.paused.repo.as_path(), None);
    name_owner(&lost, stand_in.pid(), "linux:1");
    let first: Probe = status(&lost.paused);
    assert!(lost.lock.exists(), "{}", first.stderr);
    assert!(first.stderr.contains("may have been reused"), "{}", first.stderr);
    drop(stand_in);
    remove_settled(&lost.paused.fixture);
}

/// A live creator leaves everything alone: with the owner record naming the sleeper and its true birth identity the
///  transaction is live, so neither the lock, the PID file nor the directory is touched, by the status or by the commit
///  after it, and that commit does not succeed (it waits for the holder until its bound, or stops).
#[test]
fn a_live_creator_keeps_the_lock_the_pid_file_and_the_directory_through_a_commit() {
    let lost: LostLock = lost_lock();
    let stand_in: Sleeper = Sleeper::start(lost.paused.repo.as_path(), None);
    let identity: String = process_birth_identity(stand_in.pid()).expect("identity readable").expect("the sleeper runs");
    name_owner(&lost, stand_in.pid(), identity.as_str());
    let commit: Probe = commit_after_status(&lost);
    assert_ne!(commit.code, Some(0), "a commit beside a live creator does not land: {}", commit.stderr);
    assert!(lost.lock.exists() && lost.pid_file.exists() && lost.directory.exists(), "{}", commit.stderr);
    drop(stand_in);
    remove_settled(&lost.paused.fixture);
}
