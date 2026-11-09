//! Controls for the removal of an `index.lock` that no process holds:
//! the proof the owner named (a complete scan that finds no holder,
//! no Git lock PID file),
//! the cases that must never be removed (a holder in another process,
//! a holder inside this one,
//! a PID file,
//! a scan that did not finish,
//! a lock another program put in its place),
//! the words of the journal and of the person,
//! and the acquisition that goes on to land over it.

use super::*;
use crate::index_lock_holders::{ScanLimit, scan_platform_holders_bounded, scan_proc_fd_holders_stopping};
use crate::service_fault::hooks;
use crate::service_index_lock::{Announce, LockFailure, acquire_on, acquire_within, lock_path_of};
use crate::service_index_marker::{marker_text, own_identity};
use crate::service_index_placement::{Platform, SYSTEM};
use crate::service_protocol::FailureCode;
use crate::test_support::{fixture, remove};
use std::sync::Mutex;
use std::time::{Duration, Instant};

fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let index: PathBuf = root.join("index");
    std::fs::write(&index, "old index").expect("index");
    return (root, index);
}

/// A scan that looked at every process and found no holder.
fn complete_and_empty(_lock: &Path, _device: u64, _inode: u64) -> HolderEvidence {
    return HolderEvidence { method: "proc-fd", holders: Vec::new(), partial: Vec::new() };
}

/// A child of this process that has the file at `lock` open as its standard input,
/// and no other descriptor of this process names it.
fn holder_of(lock: &Path) -> std::process::Child {
    let file: std::fs::File = std::fs::File::open(lock).expect("the lock");
    let mut command: std::process::Command = std::process::Command::new("sleep");
    command.arg("120").stdin(std::process::Stdio::from(file));
    let child: std::process::Child = command.spawn().expect("a holder");
    // The command held the parent's copy of the descriptor until here.
    drop(command);
    return child;
}

/// Ends a holder started by `holder_of`.
fn release(mut child: std::process::Child) {
    child.kill().expect("the holder ends");
    child.wait().expect("reaped");
}

/// The number of a process that has exited and been reaped.
fn dead_pid() -> i64 {
    let mut child = std::process::Command::new("true").spawn().expect("a short process");
    let pid: i64 = i64::from(child.id());
    child.wait().expect("reaped");
    return pid;
}

/// An empty lock nothing holds,
/// with no PID file beside it,
/// as the one of the incident of 2026-11-07,
/// is removed on a proof that names the file as it was opened:
/// its device,
/// inode and size,
/// and how many processes the scan could not read.
#[test]
fn an_empty_unheld_lock_with_no_pid_file_is_removed_and_the_proof_names_the_file() {
    let (root, index) = index_in("unheld-removed");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, "").expect("empty foreign lock");
    let before: std::fs::Metadata = std::fs::metadata(&lock).expect("lock");
    let outcome: Unheld = remove_if_unheld(&SYSTEM, &index, &complete_and_empty);
    let Unheld::Removed(proof) = outcome else { panic!("an unheld lock was not removed: {outcome:?}") };
    assert_eq!(proof, Proof { lock: lock.clone(), device: before.dev(), inode: before.ino(), size: before.len(), unreadable: 0 });
    assert!(!lock.exists());
    remove(root.as_path());
}

/// A lock with content is never removed,
/// though nothing holds it that the scan can see:
/// a native `git commit --all` (and `--only`,
/// and one with paths) keeps `index.lock` with the whole index written into it through its editor,
/// with no descriptor open on it and no PID file unless `core.lockfilePid` is on,
/// so a lock with content that nothing holds is what a person's commit in its editor looks like,
/// and removing it would make that commit fail when the editor closes.
/// The control runs real Git:
/// it waits in its editor,
/// the scan finds nothing holding the lock,
/// the removal leaves it,
/// and the commit then succeeds.
#[test]
fn a_lock_with_content_is_never_removed_and_a_real_commit_in_its_editor_is_why() {
    let root: PathBuf = fixture("unheld-real-editor");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    crate::test_support::git(repo.as_path(), &["add", "a.txt"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    // The editor gives the commit its message, says it started, and waits (a bounded number of looks) for the control to release it.
    let editor: PathBuf = root.join("editor");
    crate::test_support::executable(
        editor.as_path(),
        b"#!/bin/sh\necho message > \"$1\"\ntouch \"$0.started\"\nlooks=0\nwhile [ ! -e \"$0.release\" ] && [ $looks -lt 1200 ]; do sleep 0.05; looks=$((looks + 1)); done\n",
    );
    let mut commit: std::process::Child = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(&repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_EDITOR", &editor)
        .args(["commit", "--all", "--quiet"])
        .spawn()
        .expect("a real commit");
    let started: PathBuf = root.join("editor.started");
    let give_up: Instant = Instant::now() + Duration::from_secs(60);
    while !started.exists() && Instant::now() < give_up {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(started.exists(), "the real commit never reached its editor");
    let index: PathBuf = repo.join(".git").join("index");
    let lock: PathBuf = lock_path_of(&index);
    let meta: std::fs::Metadata = std::fs::metadata(&lock).expect("the editor phase holds the lock");
    assert!(meta.len() > 0, "the lock of an editor-phase commit holds the index");
    let seen: HolderEvidence = scan_platform_holders_bounded(&lock, meta.dev(), meta.ino());
    assert!(seen.holders.is_empty(), "the scan sees a holder the measurement says there is none: {seen:?}");
    let outcome: Unheld = remove_if_unheld(&SYSTEM, &index, &scan_platform_holders_bounded);
    std::fs::write(root.join("editor.release"), "").expect("release");
    let status: std::process::ExitStatus = commit.wait().expect("the commit ends");
    assert_eq!(outcome, Unheld::Kept);
    assert!(status.success(), "the commit failed after its lock was judged: {status}");
    assert!(!lock.exists(), "Git released its own lock");
    remove(root.as_path());
}

/// A lock a live process in this namespace has open is never removed:
/// the scan finds it,
/// and the lock stands.
/// A positive control first shows the scan does find that process,
/// so the stay is for that reason and no other.
#[test]
fn a_lock_a_live_process_has_open_is_never_removed() {
    let (root, index) = index_in("unheld-live-holder");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, "").expect("empty foreign lock");
    let holder: std::process::Child = holder_of(&lock);
    let meta: std::fs::Metadata = std::fs::metadata(&lock).expect("lock");
    let seen: HolderEvidence = scan_platform_holders_bounded(&lock, meta.dev(), meta.ino());
    assert!(seen.holders.iter().any(|found| { return found.pid == i64::from(holder.id()); }), "the scan does not see the holder: {seen:?}");
    let outcome: Unheld = remove_if_unheld(&SYSTEM, &index, &scan_platform_holders_bounded);
    release(holder);
    assert_eq!(outcome, Unheld::Kept);
    assert_eq!(std::fs::read_to_string(&lock).expect("the lock stands"), "");
    remove(root.as_path());
}

/// A lock whose Git lock PID file names a live process is never removed,
/// however empty the scan.
#[test]
fn a_lock_with_a_pid_file_naming_a_live_process_is_never_removed() {
    let (root, index) = index_in("unheld-pid-file-live");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, "").expect("empty foreign lock");
    std::fs::write(crate::recovery_files::lock_pid_path(&index), format!("pid {}\n", std::process::id())).expect("PID file");
    assert_eq!(remove_if_unheld(&SYSTEM, &index, &complete_and_empty), Unheld::Kept);
    assert_eq!(std::fs::read_to_string(&lock).expect("the lock stands"), "");
    remove(root.as_path());
}

/// "There is no Git PID file" means no file at all:
/// a PID file that is damaged,
/// or that names a process that is gone,
/// is evidence the removal does not judge,
/// so the lock stands for the wait to name.
#[test]
fn a_lock_with_any_pid_file_at_all_is_never_removed() {
    for (name, text) in [("malformed", String::from("not a pid line\n")), ("gone", format!("pid {}\n", dead_pid()))] {
        let (root, index) = index_in(format!("unheld-pid-file-{name}").as_str());
        let lock: PathBuf = lock_path_of(&index);
        std::fs::write(&lock, "").expect("empty foreign lock");
        std::fs::write(crate::recovery_files::lock_pid_path(&index), text).expect("PID file");
        assert_eq!(remove_if_unheld(&SYSTEM, &index, &complete_and_empty), Unheld::Kept, "{name}");
        assert!(lock.exists(), "{name}");
        remove(root.as_path());
    }
}

/// A lock replaced between the scan and the unlink is never removed:
/// the scan was of the file that was opened,
/// and the name now refers to another file whose holders nobody looked for.
#[test]
fn a_lock_replaced_between_the_scan_and_the_unlink_is_never_removed() {
    let (root, index) = index_in("unheld-replaced");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, "").expect("empty foreign lock");
    let (hook_lock, hook_root) = (lock.clone(), root.clone());
    hooks::set("unheld-scanned", std::rc::Rc::new(move || {
        // The scanned lock goes and another program's lock takes its name, as Git's own `index.lock` would.
        std::fs::remove_file(&hook_lock).expect("the scanned lock is removed by someone else");
        std::fs::write(hook_root.join("staged-foreign"), "a foreign program's lock").expect("staged");
        std::fs::rename(hook_root.join("staged-foreign"), &hook_lock).expect("placed");
    }));
    assert_eq!(remove_if_unheld(&SYSTEM, &index, &complete_and_empty), Unheld::Changed);
    assert_eq!(std::fs::read_to_string(&lock).expect("the other program's lock stands"), "a foreign program's lock");
    remove(root.as_path());
}

/// A lock that is gone asks for another placement,
/// not for a wait.
#[test]
fn a_lock_that_is_gone_asks_for_another_placement() {
    let (root, index) = index_in("unheld-gone");
    assert_eq!(remove_if_unheld(&SYSTEM, &index, &complete_and_empty), Unheld::Changed);
    remove(root.as_path());
}

/// A scan that did not look at every process proves nothing:
/// one cut at its time bound,
/// and one that could not list the processes at all,
/// leave the lock.
/// A process whose descriptors were refused is an unseen process the owner accepted as the limit,
/// so it is counted in the proof and does not stop the removal.
/// The scan's own wording is used,
/// so the classification follows what the scan really says.
#[test]
fn only_a_scan_that_finished_may_prove_a_lock_unheld() {
    let proc: PathBuf = fixture("unheld-fake-proc");
    std::fs::create_dir_all(proc.join("9")).expect("a process");
    let cut = |_lock: &Path, mount: u64, inode: u64| { return scan_proc_fd_holders_stopping(&proc, mount, inode, 0, ScanLimit { deadline: None, processes: Some(0) }); };
    let missing_proc: PathBuf = proc.join("no-such-proc-root");
    let unlistable = |_lock: &Path, mount: u64, inode: u64| { return scan_proc_fd_holders_stopping(&missing_proc, mount, inode, 0, ScanLimit::default()); };
    // The process 9 has a descriptor record that is not a directory, which the scan reports as that process's descriptors being unreadable.
    std::fs::write(proc.join("9").join("fdinfo"), "").expect("descriptor records");
    let refused = |_lock: &Path, mount: u64, inode: u64| { return scan_proc_fd_holders_stopping(&proc, mount, inode, 0, ScanLimit::default()); };
    for (name, scan, removed) in [
        ("cut at its bound", &cut as &dyn Fn(&Path, u64, u64) -> HolderEvidence, false),
        ("no process list", &unlistable as &dyn Fn(&Path, u64, u64) -> HolderEvidence, false),
        ("one process refused", &refused as &dyn Fn(&Path, u64, u64) -> HolderEvidence, true),
    ] {
        let (root, index) = index_in(format!("unheld-scan-{}", name.replace(' ', "-")).as_str());
        let lock: PathBuf = lock_path_of(&index);
        std::fs::write(&lock, "").expect("empty foreign lock");
        let outcome: Unheld = remove_if_unheld(&SYSTEM, &index, scan);
        match (removed, outcome) {
            (true, Unheld::Removed(proof)) => assert_eq!(proof.unreadable, 1, "{name}"),
            (false, Unheld::Kept) => assert!(lock.exists(), "{name}"),
            (_, other) => panic!("{name}: {other:?}"),
        }
        remove(root.as_path());
    }
    remove(proc.as_path());
}

/// A lock this process itself holds open (another landing of the service,
/// placing a lock on a file system that makes no file whole) is never removed,
/// though the scan leaves this process out;
/// it is removable once that descriptor is closed.
#[test]
fn a_lock_a_thread_of_this_process_holds_open_is_never_removed() {
    let (root, index) = index_in("unheld-own-descriptor");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, "").expect("empty foreign lock");
    let held: std::fs::File = std::fs::File::open(&lock).expect("another thread's descriptor");
    assert_eq!(remove_if_unheld(&SYSTEM, &index, &complete_and_empty), Unheld::Kept);
    assert!(lock.exists());
    drop(held);
    assert!(matches!(remove_if_unheld(&SYSTEM, &index, &complete_and_empty), Unheld::Removed(_)), "closing the descriptor makes it removable");
    remove(root.as_path());
}

/// A lock the service itself marked is judged by the stale-lock removal,
/// never here,
/// whoever holds it.
#[test]
fn a_lock_with_the_services_marker_is_not_judged_here() {
    let (root, index) = index_in("unheld-marked");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, marker_text(7, &own_identity).expect("an identity")).expect("a marked lock");
    assert_eq!(remove_if_unheld(&SYSTEM, &index, &complete_and_empty), Unheld::Kept);
    assert!(lock.exists());
    remove(root.as_path());
}

/// A link named `index.lock` is left where it is and what it points at is never touched:
/// the lock is opened without following links.
#[test]
fn a_link_named_as_the_lock_is_left_and_its_target_is_untouched() {
    let (root, index) = index_in("unheld-link");
    let lock: PathBuf = lock_path_of(&index);
    let target: PathBuf = root.join("target");
    std::fs::write(&target, "the person's file").expect("target");
    std::os::unix::fs::symlink(&target, &lock).expect("link");
    assert_eq!(remove_if_unheld(&SYSTEM, &index, &complete_and_empty), Unheld::Kept);
    assert!(std::fs::symlink_metadata(&lock).expect("the link stands").file_type().is_symlink());
    assert_eq!(std::fs::read_to_string(&target).expect("target"), "the person's file");
    remove(root.as_path());
}

/// The journal's line names every part of the proof,
/// and the person's words add what the scan cannot see and what to do about it.
#[test]
fn the_journal_and_the_person_are_given_the_proof_and_its_limit() {
    let proof: Proof = Proof { lock: PathBuf::from("/repo/.git/index.lock"), device: 2049, inode: 9183, size: 0, unreadable: 3 };
    let journal: String = journal_words(&proof);
    for part in ["/repo/.git/index.lock", "device 2049", "inode 9183", "0 bytes", "no Git lock PID file", "unreadable processes: 3"] {
        assert!(journal.contains(part), "{part}: {journal}");
    }
    let told: String = told_words(&proof);
    for part in ["/repo/.git/index.lock", "removed the empty index lock", "another user namespace", "run its command again"] {
        assert!(told.contains(part), "{part}: {told}");
    }
    assert!(told.ends_with('\n'));
}

/// What:
///  The acquisition that meets an unheld lock removes it on its first look,
///  takes its own lock instead of waiting a notice interval,
///  and tells the person what was removed.
/// Why:
///   The first look is what the removal is decided at,
/// and a wait for a lock is what the person hears while there is one;
/// so the removal being the only line,
/// and the first line,
/// is the event this control means.
/// A clock cannot say it:
/// on 2026-11-09 this control failed alone at 11 to 40 s against its own 8 s bound while the host's scan of 1,253
/// processes was cut at its budget,
/// which the removal cannot act on,
/// and the acquisition then waited a notice interval for a lock nothing held.
/// The scan is the platform's,
/// so the control gives it a scan that finished and found no holder,
/// and the host's process table stops deciding what this controls.
#[test]
fn a_landing_over_an_unheld_lock_removes_it_and_goes_on() {
    let (root, index) = index_in("unheld-acquire");
    std::fs::write(lock_path_of(&index), "").expect("empty foreign lock");
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let unheld: Platform = Platform { scan: complete_and_empty, ..SYSTEM };
    let lock = acquire_on(&unheld, &index, 3, &|note| { heard.lock().expect("lock").push(note); }).expect("taken over the unheld lock");
    let said: Vec<Announce> = heard.lock().expect("lock").clone();
    assert_eq!(said.len(), 1, "the person hears the removal and no wait for a lock nothing holds: {said:?}");
    assert!(matches!(&said[0], Announce::Waiting(text) if text.contains("removed") && text.contains("index.lock")), "the first line is the removal: {said:?}");
    assert!(std::fs::read_to_string(lock_path_of(&index)).expect("the service's lock").starts_with("cli-git-service "));
    drop(lock);
    remove(root.as_path());
}

/// The acquisition never removes a lock a live process holds:
/// it waits to its ceiling,
/// is refused,
/// and the lock stands as it was.
#[test]
fn a_landing_never_removes_a_lock_a_live_process_holds() {
    let (root, index) = index_in("unheld-acquire-held");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, "").expect("empty foreign lock");
    let holder: std::process::Child = holder_of(&lock);
    let refused: Result<_, LockFailure> = acquire_within(&SYSTEM, &index, 4, Duration::from_millis(1500), &|_| {});
    release(holder);
    assert_eq!(refused.expect_err("refused").code, FailureCode::IndexInUse);
    assert_eq!(std::fs::read_to_string(&lock).expect("the lock stands"), "");
    remove(root.as_path());
}

/// The acquisition never removes a lock whose PID file names a live process.
#[test]
fn a_landing_never_removes_a_lock_whose_pid_file_names_a_live_process() {
    let (root, index) = index_in("unheld-acquire-pid-file");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, "").expect("empty foreign lock");
    std::fs::write(crate::recovery_files::lock_pid_path(&index), format!("pid {}\n", std::process::id())).expect("PID file");
    let refused: Result<_, LockFailure> = acquire_within(&SYSTEM, &index, 5, Duration::from_millis(1500), &|_| {});
    assert_eq!(refused.expect_err("refused").code, FailureCode::IndexInUse);
    assert_eq!(std::fs::read_to_string(&lock).expect("the lock stands"), "");
    remove(root.as_path());
}
