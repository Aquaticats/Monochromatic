//! What:
//!  Controls for the clearing of reference locks that a killed Git left behind.
//! Why:
//!  The clearing deletes files in a repository,
//! so each condition that lets a lock go is pinned on both sides:
//! the lock of a
//!  Git killed with `SIGKILL` goes,
//! and the lock of a Git that runs,
//! of a process that holds it open,
//! of a holder that has
//!  not been shown gone,
//! and a lock that is too new,
//! stays.
//! The locks come from real Git in disposable repositories.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! held.kill('SIGKILL'); expect(clearStaleRefLocks(roots, judge).cleared).toHaveLength(2); expect(lockFiles(gitDir)).toEqual([]);
//! ```
#![cfg(unix)]

use crate::wrapper_names_old_and_new::PROGRAM;

/// The bytes of a command name that `/proc/<pid>/comm` keeps.
const KERNEL_COMMAND_NAME_BYTES: usize = 15;
use super::*;
use crate::stale_ref_lock_killed_git as killed;
use crate::stale_ref_lock_killed_git::{Held, Helper, Stage, GONE_REF};
use crate::test_support::remove;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};

/// A judge that sees the processes of this kernel completely,
/// takes any lock as old enough,
/// and trusts local disks,
/// scanning for open holders only among the processes the control names.
/// A scan of every process of the machine would let any other program's Git,
/// or one this account cannot look into,
/// veto the clearance a control expects
/// (the handover records that failure of 2026-11-08);
/// the veto has its own controls,
/// which name the holder they plant.
fn judge() -> Judge {
    return Judge { margin: Duration::ZERO, boot: None, scan_scope: Some(Vec::new()), ..Judge::here() };
}

/// The same judge,
/// for a control that plants a live process holding a file open:
/// that process is the only one its scan may see.
fn judge_seeing(pid: i64) -> Judge {
    return Judge { scan_scope: Some(vec![pid]), ..judge() };
}

/// A process number that no running process has:
/// one that has just ended.
fn ended_pid() -> i64 {
    let mut child: Child = Command::new("true").spawn().expect("start true");
    let pid: i64 = i64::from(child.id());
    child.wait().expect("reap true");
    return pid;
}

/// A long-running process whose command name is not `git`.
fn sleeper() -> Helper {
    return Helper(Command::new("sleep").arg("120").stdin(Stdio::null()).spawn().expect("start sleep"));
}

/// Write an empty lock and,
/// when `pid` is given,
/// the pid file Git would have written beside it after the lock.
fn plant_lock(stage: &Stage, reference: &str, pid: Option<i64>) {
    killed::write_file(stage.lock_of(reference).as_path(), "");
    if let Some(number) = pid {
        std::thread::sleep(Duration::from_millis(20));
        killed::write_file(stage.pid_file_of(reference).as_path(), &format!("pid {number}\n"));
    }
}

/// The reference the planted locks belong to.
const PLANTED_REF: &str = "refs/heads/planted";

/// Every file below the Git directory whose name ends in `.lock` or is the packed staging file,
/// sorted.
fn leftovers(stage: &Stage) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![stage.git_dir.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory.as_path()).expect("read the directory").flatten() {
            let path: PathBuf = entry.path();
            if entry.file_type().expect("file type").is_dir() {
                pending.push(path);
            } else if path.to_string_lossy().ends_with(".lock") || path.ends_with("packed-refs.new") {
                found.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// The verdict a pass gave one lock,
/// wherever it landed.
fn verdict_of(outcome: &Outcome, lock: &Path) -> Option<Verdict> {
    return outcome.cleared.iter().chain(outcome.left.iter()).find(|found| { return found.lock == lock; }).map(|found| { return found.verdict.clone(); });
}

/// Locks that a fetch killed with `SIGKILL` left,
/// with the pid files that name it,
/// stay with their staging file:
/// a number in a pid file is not proof that this lock's writer is gone,
/// so each lock is named with the command that clears it.
#[test]
fn locks_left_by_a_killed_fetch_stay_and_are_named() {
    let stage: Stage = killed::stage("stale-refs-killed-fetch", true);
    let mut held: Held = killed::hold_fetch(&stage);
    assert!(stage.lock_of(GONE_REF).exists() && stage.packed_lock().exists() && stage.packed_staging().exists(), "the fetch holds its locks");
    assert_eq!(std::fs::metadata(stage.lock_of(GONE_REF)).expect("lock").len(), 0, "a deleted reference's lock is empty");
    held.kill();
    let before: Vec<PathBuf> = leftovers(&stage);
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "a lock nothing proves stale was removed: {outcome:?}");
    assert_eq!(leftovers(&stage), before);
    assert!(matches!(verdict_of(&outcome, stage.packed_lock().as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    assert!(outcome.notice().contains("rm -- "), "the person is given the command that clears it: {}", outcome.notice());
    stage.finish();
}

/// Without `core.lockfilePid` nothing names the holder:
/// the locks stay and are reported,
/// never removed.
#[test]
fn locks_of_a_killed_fetch_with_no_pid_file_stay() {
    let stage: Stage = killed::stage("stale-refs-no-pid-file", false);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    let before: Vec<PathBuf> = leftovers(&stage);
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert_eq!(leftovers(&stage), before);
    assert!(matches!(verdict_of(&outcome, stage.lock_of(GONE_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    stage.finish();
}

/// A Git that runs holds locks with no descriptor open,
/// and the pid file that names it proves nothing,
/// so every one of them is left and named.
#[test]
fn locks_of_a_running_git_stay_though_no_descriptor_holds_them() {
    let stage: Stage = killed::stage("stale-refs-running", true);
    let held: Held = killed::hold_transaction(&stage, &["delete refs/remotes/origin/gone", "delete refs/remotes/origin/main"]);
    let before: Vec<PathBuf> = leftovers(&stage);
    assert!(before.contains(&stage.packed_lock()) && before.contains(&stage.lock_of(GONE_REF)), "{before:?}");
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert_eq!(leftovers(&stage), before);
    assert!(matches!(verdict_of(&outcome, stage.packed_lock().as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    drop(held);
    stage.finish();
}

/// The same locks with no pid file are not shown gone either.
#[test]
fn locks_of_a_running_git_with_no_pid_file_stay() {
    let stage: Stage = killed::stage("stale-refs-running-no-pid", false);
    let held: Held = killed::hold_transaction(&stage, &["delete refs/remotes/origin/gone"]);
    let before: Vec<PathBuf> = leftovers(&stage);
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert_eq!(leftovers(&stage), before);
    drop(held);
    stage.finish();
}

/// A process that has the lock open is its holder,
/// whatever the pid file says.
#[test]
fn a_lock_that_a_process_holds_open_stays_though_its_pid_file_names_a_gone_process() {
    let stage: Stage = killed::stage("stale-refs-open", true);
    plant_lock(&stage, PLANTED_REF, Some(ended_pid()));
    let lock: PathBuf = stage.lock_of(PLANTED_REF);
    let holder: Helper = Helper(Command::new("sh").arg("-c").arg("exec 3<>\"$1\"; exec sleep 120").arg("sh").arg(lock.as_os_str()).spawn().expect("start the holder"));
    let holder_pid: i64 = holder.pid();
    let descriptors: PathBuf = PathBuf::from(format!("/proc/{holder_pid}/fd"));
    let started: std::time::Instant = std::time::Instant::now();
    while !std::fs::read_dir(descriptors.as_path()).expect("fds").flatten().any(|entry| { return std::fs::read_link(entry.path()).is_ok_and(|target| { return target == lock; }); }) {
        assert!(started.elapsed() < killed::HOLD_PATIENCE, "the holder opened the lock");
        std::thread::sleep(Duration::from_millis(20));
    }
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge_seeing(holder_pid));
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(lock.exists() && stage.pid_file_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, lock.as_path()), Some(Verdict::Held(_))), "{outcome:?}");
    drop(holder);
    stage.finish();
}

/// A lock younger than the margin is not judged:
/// a live Git may have just made it,
/// and once it is older nothing about it is proven either.
#[test]
fn a_lock_inside_the_margin_stays_and_is_judged_once_it_is_older() {
    let stage: Stage = killed::stage("stale-refs-margin", true);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    let before: Vec<PathBuf> = leftovers(&stage);
    let strict: Judge = Judge { margin: Duration::from_secs(3600), ..judge() };
    let young: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &strict);
    assert!(!young.cleared_any() && young.left.is_empty(), "a lock inside the margin is left without a word: {young:?}");
    assert_eq!(leftovers(&stage), before);
    let aged: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!aged.cleared_any(), "a lock nothing proves stale was removed: {aged:?}");
    assert!(!aged.left.is_empty(), "a lock past the margin is judged and named: {aged:?}");
    stage.finish();
}

/// A pid file born before the lock was written by an earlier holder;
/// it does not name this lock's holder.
#[test]
fn a_pid_file_born_before_the_lock_is_not_evidence() {
    let stage: Stage = killed::stage("stale-refs-older-pid-file", true);
    killed::write_file(stage.pid_file_of(PLANTED_REF).as_path(), &format!("pid {}\n", ended_pid()));
    std::thread::sleep(Duration::from_millis(50));
    killed::write_file(stage.lock_of(PLANTED_REF).as_path(), "");
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    stage.finish();
}

/// A number that a program other than Git now has,
/// having started after the lock was made,
/// proves nothing:
/// a Git that runs under another name looks exactly the same,
/// and only the start time could tell them apart,
/// which a step of the clock moves.
/// The lock stays and the notice names the program and gives the command that clears it.
#[test]
fn a_number_that_another_program_now_holds_is_not_proof() {
    let stage: Stage = killed::stage("stale-refs-reused", true);
    killed::write_file(stage.lock_of(PLANTED_REF).as_path(), "");
    std::thread::sleep(Duration::from_millis(400));
    let other: Helper = sleeper();
    std::thread::sleep(Duration::from_millis(50));
    killed::write_file(stage.pid_file_of(PLANTED_REF).as_path(), &format!("pid {}\n", other.pid()));
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(text)) if text.contains("is running")), "{outcome:?}");
    assert!(outcome.notice().contains("rm -- "), "the person is given the command that clears it: {}", outcome.notice());
    drop(other);
    stage.finish();
}

/// The wrapper's one executable is also the commit service,
/// and a process of that name is not Git:
/// a number it now holds is a running program's like any other's,
/// so the lock stays and is reported.
/// The service's former name,
/// a different file,
///  was not Git's either,
/// so merging the two executables changes no verdict.
#[test]
fn a_process_named_like_the_one_executable_is_not_git() {
    // The executable's current name and the name an older install gave it: neither starts with Git's.
    for program in [PROGRAM.current, PROGRAM.earlier] {
        a_process_named_like_this_program_is_not_git(program);
    }
}

/// One program name of the control above.
fn a_process_named_like_this_program_is_not_git(program: &str) {
    let stage: Stage = killed::stage(format!("stale-refs-named-{program}").as_str(), true);
    let named: PathBuf = stage.git_dir.join(program);
    std::fs::copy(Path::new("/usr/bin/sleep"), &named).expect("a copy of sleep under the executable's name");
    killed::write_file(stage.lock_of(PLANTED_REF).as_path(), "");
    std::thread::sleep(Duration::from_millis(400));
    let other: Helper = Helper(Command::new(&named).arg("120").stdin(Stdio::null()).spawn().expect("start the copy"));
    let comm: String = std::fs::read_to_string(format!("/proc/{}/comm", other.pid())).expect("comm").trim().to_string();
    // The kernel reports at most fifteen bytes of a command name.
    assert_eq!(comm, program.chars().take(KERNEL_COMMAND_NAME_BYTES).collect::<String>());
    std::thread::sleep(Duration::from_millis(50));
    killed::write_file(stage.pid_file_of(PLANTED_REF).as_path(), &format!("pid {}\n", other.pid()));
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(text)) if text.contains("running")), "{outcome:?}");
    drop(other);
    stage.finish();
}

/// A running program that is not named Git may be a Git under another name,
/// whether it started before the lock was made or after:
/// it is
/// taken for the holder and the lock stays.
#[test]
fn a_running_program_that_is_not_named_git_keeps_the_lock_whatever_its_start() {
    let stage: Stage = killed::stage("stale-refs-renamed-git", true);
    let other: Helper = sleeper();
    std::thread::sleep(Duration::from_millis(400));
    plant_lock(&stage, PLANTED_REF, Some(other.pid()));
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    drop(other);
    stage.finish();
}

/// A running Git that the pid file names is not the holder by that number:
/// the pid file decides nothing,
/// so the lock is unproven and named.
#[test]
fn a_running_git_named_by_the_pid_file_is_not_proven_the_holder() {
    let stage: Stage = killed::stage("stale-refs-running-git", true);
    let idle: Held = killed::idle_git(&stage);
    plant_lock(&stage, PLANTED_REF, Some(i64::from(idle.pid())));
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(text)) if text.contains("is running")), "{outcome:?}");
    drop(idle);
    stage.finish();
}

/// A lock whose times are earlier than the boot is no proof of a gone holder,
/// whichever way the times fell:
/// a correction of the
/// clock makes a live holder's lock look the same.
/// The notice still says what the times suggest.
#[test]
fn a_lock_older_than_the_boot_is_left_and_said_to_be_unproven() {
    let stage: Stage = killed::stage("stale-refs-boot", false);
    plant_lock(&stage, PLANTED_REF, None);
    let after: Judge = Judge { boot: Some(SystemTime::now() + A_DAY), ..judge() };
    let before: Judge = Judge { boot: Some(SystemTime::now() - A_DAY), ..judge() };
    let younger: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &before);
    assert!(!younger.cleared_any(), "{younger:?}");
    assert!(!matches!(verdict_of(&younger, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(text)) if text.contains("start of this boot")), "a lock younger than the boot is not said to look older: {younger:?}");
    let older: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &after);
    assert!(!older.cleared_any(), "{older:?}");
    assert!(matches!(verdict_of(&older, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(text)) if text.contains("start of this boot") && text.contains("correction of the clock")), "{older:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(older.notice().contains("rm -- "), "the person is given the command that clears it: {}", older.notice());
    stage.finish();
}

/// A running Git named by a pid file keeps its lock,
/// whatever the boot time says.
#[test]
fn the_boot_time_does_not_override_a_pid_file_that_names_a_running_git() {
    let stage: Stage = killed::stage("stale-refs-boot-vs-pid", true);
    let idle: Held = killed::idle_git(&stage);
    plant_lock(&stage, PLANTED_REF, Some(i64::from(idle.pid())));
    let after: Judge = Judge { boot: Some(SystemTime::now() + A_DAY), ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &after);
    assert!(!outcome.cleared_any(), "{outcome:?}");
    drop(idle);
    stage.finish();
}

/// A view of the processes that may be another namespace's proves nothing about a pid file.
#[test]
fn a_view_of_processes_that_may_be_partial_proves_nothing() {
    let stage: Stage = killed::stage("stale-refs-partial-view", true);
    plant_lock(&stage, PLANTED_REF, Some(ended_pid()));
    let partial: Judge = Judge { ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &partial);
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    stage.finish();
}

/// A lock on a file system that is not on the local list may be held by a Git on another machine.
#[test]
fn a_file_system_off_the_local_list_proves_nothing() {
    let stage: Stage = killed::stage("stale-refs-remote-fs", true);
    plant_lock(&stage, PLANTED_REF, Some(ended_pid()));
    let nowhere: Judge = Judge { local_file_systems: Vec::new(), boot: Some(SystemTime::now() + A_DAY), ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &nowhere);
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    stage.finish();
}

/// A process table that cannot be listed shows no holder,
/// which is not the same as none.
#[test]
fn an_unreadable_process_table_proves_nothing() {
    let stage: Stage = killed::stage("stale-refs-no-proc", true);
    plant_lock(&stage, PLANTED_REF, Some(ended_pid()));
    let blind: Judge = Judge { proc_root: stage.root.join("no-such-proc"), ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &blind);
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    stage.finish();
}

/// Locks of other protocols,
/// and pid files alone,
/// are never touched,
/// whatever else is proven.
#[test]
fn locks_of_other_protocols_are_never_candidates() {
    let stage: Stage = killed::stage("stale-refs-other-locks", false);
    let others: Vec<PathBuf> = vec![
        stage.git_dir.join("index.lock"),
        stage.git_dir.join("config.lock"),
        stage.git_dir.join("shallow.lock"),
        stage.git_dir.join("objects").join("pack").join("tmp.lock"),
        stage.git_dir.join("refs").join("heads").join("alone~pid.lock"),
    ];
    for path in &others {
        killed::write_file(path.as_path(), "");
    }
    let after: Judge = Judge { boot: Some(SystemTime::now() + A_DAY), ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &after);
    assert!(!outcome.cleared_any() && outcome.left.is_empty(), "{outcome:?}");
    for path in &others {
        assert!(path.exists(), "{path:?}");
    }
    stage.finish();
}

/// A reference named with shell and path characters is quoted in the notice as the literal name it is,
/// and nothing of it is run.
#[test]
fn a_reference_with_shell_and_path_characters_is_named_literally() {
    let stage: Stage = killed::stage("stale-refs-adversarial", true);
    let name: &str = "refs/heads/it's$(touch${IFS}pwned);`id`&x|(y)";
    let mut held: Held = killed::hold_transaction(&stage, &[format!("create {name} HEAD").as_str()]);
    assert!(stage.lock_of(name).exists(), "Git holds the lock of the odd name");
    held.kill();
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(outcome.notice().contains("rm -- '"), "the name is quoted for a shell: {}", outcome.notice());
    assert!(!stage.repo.join("pwned").exists() && !Path::new("pwned").exists(), "nothing was run");
    stage.finish();
}

/// A symbolic link is never followed and never removed,
/// whether it stands for a directory or for a lock.
#[test]
fn symbolic_links_are_neither_followed_nor_removed() {
    let stage: Stage = killed::stage("stale-refs-symlinks", true);
    let outside: PathBuf = stage.root.join("outside");
    killed::write_file(outside.join("x.lock").as_path(), "");
    killed::write_file(outside.join("target.lock").as_path(), "");
    let heads: PathBuf = stage.git_dir.join("refs").join("heads");
    std::os::unix::fs::symlink(outside.as_path(), heads.join("linked-directory")).expect("link a directory");
    std::os::unix::fs::symlink(outside.join("target.lock"), heads.join("linked.lock")).expect("link a lock");
    let after: Judge = Judge { boot: Some(SystemTime::now() + A_DAY), ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &after);
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(outside.join("x.lock").exists() && outside.join("target.lock").exists());
    assert!(std::fs::symlink_metadata(heads.join("linked.lock")).is_ok());
    remove(stage.root.as_path());
}

/// A lock that cannot be set aside is left and reported,
/// not retried and not an error.
#[test]
fn a_lock_in_a_directory_that_cannot_be_written_is_left() {
    use std::os::unix::fs::PermissionsExt as _;
    let stage: Stage = killed::stage("stale-refs-unwritable", true);
    plant_lock(&stage, PLANTED_REF, Some(ended_pid()));
    let directory: PathBuf = stage.git_dir.join("refs").join("heads");
    std::fs::set_permissions(directory.as_path(), std::fs::Permissions::from_mode(0o555)).expect("make read-only");
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    std::fs::set_permissions(directory.as_path(), std::fs::Permissions::from_mode(0o755)).expect("make writable");
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    stage.finish();
}

/// A path that a live Git took again after the judgement is put back whole,
/// never removed.
#[test]
fn a_lock_taken_again_after_the_judgement_is_put_back() {
    let stage: Stage = killed::stage("stale-refs-retaken", true);
    plant_lock(&stage, PLANTED_REF, Some(ended_pid()));
    let lock: PathBuf = stage.lock_of(PLANTED_REF);
    let judged = crate::stale_ref_lock_judge::identity_of(lock.as_path()).expect("identity");
    std::fs::remove_file(lock.as_path()).expect("the stale lock is removed by another wrapper");
    killed::write_file(lock.as_path(), "live holder's content\n");
    let mut outcome: Outcome = Outcome::default();
    let removed = crate::stale_ref_lock_retire::remove_if_unchanged(stage.git_dir.as_path(), lock.as_path(), &judged, &mut outcome);
    assert_eq!(removed, Ok(false));
    assert_eq!(std::fs::read_to_string(lock.as_path()).expect("the live lock"), "live holder's content\n");
    assert!(outcome.displaced.is_empty(), "{outcome:?}");
    stage.finish();
}

/// A file that is not the judged one when it is taken is put back whole,
/// nothing stays under the private name,
/// and a lock
/// that is there again by then is never written over.
#[test]
fn a_lock_that_differs_from_the_judged_one_when_taken_is_put_back_whole() {
    let stage: Stage = killed::stage("stale-refs-taken-then-differs", true);
    plant_lock(&stage, PLANTED_REF, None);
    let lock: PathBuf = stage.lock_of(PLANTED_REF);
    let judged = crate::stale_ref_lock_judge::identity_of(lock.as_path()).expect("identity");
    std::fs::remove_file(lock.as_path()).expect("the stale lock is removed by another wrapper");
    killed::write_file(lock.as_path(), "live holder's content\n");
    let mut outcome: Outcome = Outcome::default();
    let removed = crate::stale_ref_lock_retire::take_and_check(stage.git_dir.as_path(), lock.as_path(), &judged, &mut outcome);
    assert_eq!(removed, Ok(false));
    assert_eq!(std::fs::read_to_string(lock.as_path()).expect("the live lock"), "live holder's content\n");
    assert!(outcome.displaced.is_empty(), "{outcome:?}");
    let aside: Vec<String> = std::fs::read_dir(stage.git_dir.as_path())
        .expect("read the Git directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name.starts_with("cli-git-stale-lock"); })
        .collect();
    assert!(aside.is_empty(), "nothing is left under the private name: {aside:?}");
    stage.finish();
}

/// The judged file itself,
/// taken the same way,
/// is removed.
#[test]
fn the_judged_lock_is_removed_when_taken() {
    let stage: Stage = killed::stage("stale-refs-taken-judged", true);
    plant_lock(&stage, PLANTED_REF, None);
    let lock: PathBuf = stage.lock_of(PLANTED_REF);
    let judged = crate::stale_ref_lock_judge::identity_of(lock.as_path()).expect("identity");
    let mut outcome: Outcome = Outcome::default();
    let removed = crate::stale_ref_lock_retire::take_and_check(stage.git_dir.as_path(), lock.as_path(), &judged, &mut outcome);
    assert_eq!(removed, Ok(true));
    assert!(!lock.exists());
    stage.finish();
}

/// The pid file that goes with a removed lock is removed only if it is the file that was there when the lock was judged.
#[test]
fn a_pid_file_that_a_live_writer_made_after_the_judgement_is_not_removed_with_the_lock() {
    let stage: Stage = killed::stage("stale-refs-pid-file-replaced", true);
    plant_lock(&stage, PLANTED_REF, Some(ended_pid()));
    let pid_file: PathBuf = stage.pid_file_of(PLANTED_REF);
    let judged = crate::stale_ref_lock_judge::identity_of(pid_file.as_path()).expect("identity");
    std::fs::remove_file(pid_file.as_path()).expect("the dead holder's pid file is removed by another wrapper");
    killed::write_file(pid_file.as_path(), &format!("pid {}\n", std::process::id()));
    let mut outcome: Outcome = Outcome::default();
    let removed = crate::stale_ref_lock_retire::remove_if_unchanged(stage.git_dir.as_path(), pid_file.as_path(), &judged, &mut outcome);
    assert_eq!(removed, Ok(false));
    assert!(pid_file.exists());
    stage.finish();
}

/// The Git directories of a command come from its working directory,
/// without starting Git.
#[test]
fn the_roots_of_a_command_are_found_without_git() {
    let stage: Stage = killed::stage("stale-refs-roots", false);
    let mut command: Command = Command::new("git");
    command.current_dir(stage.repo.as_path());
    assert_eq!(crate::git_directory_roots::roots_of_command(&command), stage.roots());
    stage.finish();
}

/// The number of the one process of a fake table.
const FAKE_PROCESS: i64 = 4242;

/// A process table with one process of this account whose descriptor record cannot be read,
/// as `systemd --user` is on a host;
/// the judge of it scans that process and no other.
fn table_with_unreadable(stage: &Stage, command: &str) -> Judge {
    use std::os::unix::fs::PermissionsExt as _;
    let table: PathBuf = stage.root.join("proc");
    let process: PathBuf = table.join(FAKE_PROCESS.to_string());
    let descriptors: PathBuf = process.join("fdinfo");
    std::fs::create_dir_all(descriptors.as_path()).expect("fake process");
    std::fs::write(process.join("comm"), format!("{command}\n")).expect("comm");
    std::fs::set_permissions(descriptors.as_path(), std::fs::Permissions::from_mode(0o000)).expect("hide the descriptors");
    return Judge { proc_root: table, scan_scope: Some(vec![FAKE_PROCESS]), ..judge() };
}

/// Make the fake table removable again.
fn reveal(table: &Path) {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(table.join(FAKE_PROCESS.to_string()).join("fdinfo"), std::fs::Permissions::from_mode(0o700)).expect("reveal");
}

/// A process of this account that is not Git and cannot be looked into (the user's service manager) cannot hold a Git lock,
/// so it is not the reason a lock is left:
/// the reason is what the pid file says.
#[test]
fn an_unreadable_process_that_is_not_git_is_not_the_veto() {
    let stage: Stage = killed::stage("stale-refs-unreadable-service", true);
    let gone: i64 = ended_pid();
    plant_lock(&stage, PLANTED_REF, Some(gone));
    let listed: Judge = table_with_unreadable(&stage, "systemd");
    let revealed: PathBuf = listed.proc_root.clone();
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &listed);
    reveal(revealed.as_path());
    let verdict: Option<Verdict> = verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path());
    assert!(matches!(verdict, Some(Verdict::Unproven(text)) if text.contains(&format!("names process {gone}"))), "{outcome:?}");
    stage.finish();
}

/// A Git process of this account that cannot be looked into may hold the lock,
/// so nothing is proven.
#[test]
fn an_unreadable_git_process_stops_a_proof() {
    let stage: Stage = killed::stage("stale-refs-unreadable-git", true);
    plant_lock(&stage, PLANTED_REF, Some(ended_pid()));
    let listed: Judge = table_with_unreadable(&stage, "git");
    let revealed: PathBuf = listed.proc_root.clone();
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &listed);
    reveal(revealed.as_path());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    stage.finish();
}

/// A process of this account that the scan cannot look into,
/// named like Git:
/// the shape a sandboxed or privileged program of the same account leaves beside a control,
/// which denies its descriptors to its own account (`prctl(2)`,
/// `PR_SET_DUMPABLE`).
struct NuisanceGit {
    /// The process number.
    pid: i64,
}

impl NuisanceGit {
    /// Start one,
    /// or `None` where this account can look into every process it runs (the kernel lets the root account read the
    /// descriptors of a process that is not dumpable).
    fn start() -> Option<Self> {
        // SAFETY: between `fork` and the end of the child the code runs only `prctl` and `pause`, which take no pointer
        // into this process's memory but the name below, and touch no lock.
        let forked: libc::pid_t = unsafe { libc::fork() };
        if forked == 0 {
            // The child: named like Git and not dumpable, so its own account cannot read `/proc/<pid>/fd`.
            let name: &[u8] = b"git-nuisance\0";
            // SAFETY: the name is a terminated string that outlives the call.
            unsafe {
                let _named = libc::prctl(libc::PR_SET_NAME, name.as_ptr(), 0, 0, 0);
                let _hidden = libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
                loop {
                    let _paused = libc::pause();
                }
            }
        }
        assert!(forked > 0, "fork the process of another program");
        let nuisance: Self = Self { pid: i64::from(forked) };
        let started: std::time::Instant = std::time::Instant::now();
        while !nuisance.named_and_hidden() {
            assert!(started.elapsed() < killed::HOLD_PATIENCE, "the process of another program took its name and hid its descriptors");
            std::thread::sleep(Duration::from_millis(5));
        }
        return nuisance.named_and_hidden().then_some(nuisance);
    }

    /// Whether the scan of this account meets it as a Git process it cannot look into.
    fn named_and_hidden(&self) -> bool {
        let process: PathBuf = PathBuf::from("/proc").join(self.pid.to_string());
        let named: bool = std::fs::read_to_string(process.join("comm")).is_ok_and(|text| { return text.trim().starts_with("git"); });
        let hidden: bool = std::fs::read_dir(process.join("fd")).is_err_and(|error| { return error.kind() == std::io::ErrorKind::PermissionDenied; });
        return named && hidden;
    }
}

/// Whatever the control did,
/// the process of another program is gone.
impl Drop for NuisanceGit {
    fn drop(&mut self) {
        // SAFETY: the number is the own child this control forked, and reaping it here takes its only record.
        unsafe {
            let _killed = libc::kill(libc::pid_t::try_from(self.pid).unwrap_or_default(), libc::SIGKILL);
            let _reaped = libc::waitpid(libc::pid_t::try_from(self.pid).unwrap_or_default(), std::ptr::null_mut(), 0);
        }
    }
}

/// Another program's Git process of this account that cannot be looked into is no holder of this fixture's locks:
/// the judgement of one fixture looks at the processes that fixture started,
/// so a sandboxed or privileged Git of the same session elsewhere does not keep a stale lock in place.
#[test]
fn an_unreadable_git_of_another_program_does_not_stop_a_fixtures_clearance() {
    let stage: Stage = killed::stage("stale-refs-unreadable-elsewhere", true);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    let Some(nuisance) = NuisanceGit::start() else {
        eprintln!("skipped: this account can look into every process it runs");
        stage.finish();
        return;
    };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(GONE_REF).exists());
    drop(nuisance);
    stage.finish();
}

/// The staging file of a killed packed-references update stays when its lock was removed by hand;
/// it goes,
/// and the next
/// packed-references update works.
#[test]
fn a_staging_file_with_no_lock_beside_it_is_cleared() {
    let stage: Stage = killed::stage("stale-refs-orphan-staging", true);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    std::fs::remove_file(stage.packed_lock()).expect("the lock is removed by hand");
    std::fs::remove_file(stage.git_dir.join("packed-refs~pid.lock")).expect("its pid file too");
    assert!(stage.packed_staging().exists());
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!stage.packed_staging().exists(), "{outcome:?}");
    assert_eq!(verdict_of(&outcome, stage.packed_staging().as_path()), Some(Verdict::Stale(Proof::OrphanStaging)), "{outcome:?}");
    // The reference lock the killed fetch left is named, not removed: only the staging file has a proof.
    assert!(stage.lock_of(GONE_REF).exists());
    assert!(outcome.notice().contains("left the reference lock"), "{}", outcome.notice());
    stage.finish();
}

/// While the lock is there,
/// the staging file is part of a live or killed update that the lock's own judgement decides.
#[test]
fn a_staging_file_with_its_lock_beside_it_is_left_to_the_lock() {
    let stage: Stage = killed::stage("stale-refs-live-staging", true);
    let held: Held = killed::hold_transaction(&stage, &["delete refs/remotes/origin/gone"]);
    assert!(stage.packed_staging().exists(), "the update holds its staging file");
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.packed_staging().exists());
    drop(held);
    stage.finish();
}

/// A staging file younger than the margin may be a live update's whose lock a hand removed a moment ago:
/// it stays.
#[test]
fn a_staging_file_inside_the_margin_stays() {
    let stage: Stage = killed::stage("stale-refs-young-staging", true);
    killed::write_file(stage.packed_staging().as_path(), "# pack-refs with:\n");
    let strict: Judge = Judge { margin: Duration::from_secs(3600), ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &strict);
    assert!(!outcome.cleared_any() && outcome.left.is_empty(), "{outcome:?}");
    assert!(stage.packed_staging().exists());
    stage.finish();
}

/// A Git program started after the lock was made,
/// named by the pid file,
/// is a reading the pid file alone cannot prove either.
#[test]
fn a_git_that_started_after_the_lock_is_not_proven_the_holder() {
    let stage: Stage = killed::stage("stale-refs-later-git", true);
    killed::write_file(stage.lock_of(PLANTED_REF).as_path(), "");
    std::thread::sleep(Duration::from_millis(400));
    let later: Held = killed::idle_git(&stage);
    std::thread::sleep(Duration::from_millis(50));
    killed::write_file(stage.pid_file_of(PLANTED_REF).as_path(), &format!("pid {}\n", later.pid()));
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    drop(later);
    stage.finish();
}

/// A staging file that a process has open is in use,
/// though no lock is beside it.
#[test]
fn a_staging_file_that_a_process_holds_open_stays() {
    let stage: Stage = killed::stage("stale-refs-open-staging", true);
    killed::write_file(stage.packed_staging().as_path(), "# pack-refs with:\n");
    let holder: Helper = Helper(Command::new("sh").arg("-c").arg("exec 3<>\"$1\"; exec sleep 120").arg("sh").arg(stage.packed_staging().as_os_str()).spawn().expect("start the holder"));
    let descriptors: PathBuf = PathBuf::from(format!("/proc/{}/fd", holder.pid()));
    let staging: PathBuf = stage.packed_staging();
    let started: std::time::Instant = std::time::Instant::now();
    while !std::fs::read_dir(descriptors.as_path()).expect("fds").flatten().any(|entry| { return std::fs::read_link(entry.path()).is_ok_and(|target| { return target == staging; }); }) {
        assert!(started.elapsed() < killed::HOLD_PATIENCE, "the holder opened the staging file");
        std::thread::sleep(Duration::from_millis(20));
    }
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge_seeing(holder.pid()));
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.packed_staging().exists());
    assert!(matches!(verdict_of(&outcome, stage.packed_staging().as_path()), Some(Verdict::Held(_))), "{outcome:?}");
    drop(holder);
    stage.finish();
}

//region Another ref backend's lock (audit of 2026-11-09, finding 1)

/// The reftable backend keeps its references,
/// and the lock of its table list,
/// in the `reftable` directory of the Git directory,
/// so no `packed-refs.lock`,
/// `HEAD.lock` or `refs/**/*.lock` exists beside them.
/// A killed Git's lock of that list is a candidate,
/// and the pid file beside it names the dead Git without proving anything,
/// so the lock stays and the notice gives the file and the command that clears it.
#[test]
fn a_reftable_tables_list_lock_left_by_a_killed_git_stays_and_is_named() {
    let stage: Stage = killed::reftable_stage("stale-refs-reftable-killed", true);
    let mut held: Held = killed::hold_transaction(&stage, &["create refs/heads/planted HEAD"]);
    assert!(stage.tables_lock().exists() && stage.tables_pid_file().exists(), "the transaction holds the lock of the reftable table list");
    held.kill();
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.tables_lock().exists() && stage.tables_pid_file().exists());
    assert!(matches!(verdict_of(&outcome, stage.tables_lock().as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    assert!(outcome.notice().contains("tables.list.lock") && outcome.notice().contains("rm -- "), "the person is told which file stands in the way: {}", outcome.notice());
    stage.finish();
}

/// A reftable Git that runs holds that lock with the pid file beside it,
/// so the lock stays.
#[test]
fn a_live_reftable_gits_tables_list_lock_stays() {
    let stage: Stage = killed::reftable_stage("stale-refs-reftable-live", true);
    let held: Held = killed::hold_transaction(&stage, &["create refs/heads/planted HEAD"]);
    assert!(stage.tables_lock().exists(), "the transaction holds the lock");
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.tables_lock().exists() && stage.tables_pid_file().exists());
    assert!(matches!(verdict_of(&outcome, stage.tables_lock().as_path()), Some(Verdict::Unproven(text)) if text.contains("is running")), "{outcome:?}");
    drop(held);
    stage.finish();
}

/// Nothing proves the holder of that lock gone when no pid file names one:
/// it is left and the notice gives the file and the command that clears it.
#[test]
fn a_reftable_lock_that_nothing_proves_gone_is_left_and_named() {
    let stage: Stage = killed::reftable_stage("stale-refs-reftable-unproven", false);
    let mut held: Held = killed::hold_transaction(&stage, &["create refs/heads/planted HEAD"]);
    held.kill();
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.tables_lock().exists());
    assert!(matches!(verdict_of(&outcome, stage.tables_lock().as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    assert!(outcome.notice().contains("tables.list.lock") && outcome.notice().contains("rm -- "), "the person is told which file stands in the way: {}", outcome.notice());
    stage.finish();
}

//endregion

//region Evidence that is not proof (cross-vendor review of 2026-11-03T11:33:50-05:00, finding S-2)

/// A day,
/// far over the hour a clock correction is usually made of.
const A_DAY: Duration = Duration::from_secs(86_400);

/// A forward correction of the wall clock during the boot moves `btime`,
/// so a lock a live Git made before it looks older
///  than the boot.
/// The time of the file is then no evidence,
/// and with nothing else the lock stays and is reported.
#[test]
fn a_lock_that_a_clock_step_puts_before_the_boot_is_not_judged_stale() {
    let stage: Stage = killed::stage("stale-refs-clock-step", false);
    plant_lock(&stage, PLANTED_REF, None);
    let stepped: Judge = Judge { boot: Some(SystemTime::now() + A_DAY), ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &stepped);
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    stage.finish();
}

/// A process table with one process of a nested PID namespace:
/// the table numbers it `outer`,
/// and it numbers itself `inner`.
fn table_with_nested_process(stage: &Stage, outer: i64, inner: i64) -> PathBuf {
    let table: PathBuf = stage.root.join("proc-nested");
    let process: PathBuf = table.join(outer.to_string());
    std::fs::create_dir_all(process.join("fd")).expect("fake process");
    std::fs::write(process.join("comm"), "sh\n").expect("comm");
    std::fs::write(process.join("status"), format!("Name:\tsh\nNSpid:\t{outer}\t{inner}\n")).expect("status");
    return table;
}

/// A pid file written inside a nested PID namespace holds the number that namespace gave the writer.
/// No process of the
///  initial namespace has that number,
/// yet the writer runs,
/// so the number being absent proves nothing.
#[test]
fn a_pid_number_that_a_nested_namespace_gave_its_writer_is_not_judged_gone() {
    let stage: Stage = killed::stage("stale-refs-nested-number", true);
    let inner: i64 = ended_pid();
    plant_lock(&stage, PLANTED_REF, Some(inner));
    let table: PathBuf = table_with_nested_process(&stage, inner + 1, inner);
    let nested: Judge = Judge { proc_root: table, ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &nested);
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    stage.finish();
}

/// The same,
/// with a real nested namespace:
/// a process whose number in its namespace was set high enough to be free on this host.
#[test]
#[cfg(target_os = "linux")]
fn a_writer_in_a_real_nested_namespace_is_not_judged_gone() {
    let stage: Stage = killed::stage("stale-refs-real-namespace", true);
    let Some(inside) = NestedProcess::start(FREE_LOCAL_NUMBER) else {
        eprintln!("skipped: this host does not let an unprivileged process make a PID namespace");
        stage.finish();
        return;
    };
    assert!(!Path::new("/proc").join(inside.local.to_string()).exists(), "the number {} is free on this host", inside.local);
    plant_lock(&stage, PLANTED_REF, Some(inside.local));
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "{outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    assert!(matches!(verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    drop(inside);
    stage.finish();
}

/// The number the next process of the nested namespace gets,
/// set high so that this host has no process with it.
const FREE_LOCAL_NUMBER: i64 = 3_000_000;

/// A process in a PID namespace of its own,
/// and the number that namespace gave it.
#[cfg(target_os = "linux")]
struct NestedProcess {
    /// The `unshare` process;
    /// its standard input is what keeps the namespace's init waiting.
    child: Child,
    /// The number the namespace gave the sleeping process.
    local: i64,
}

#[cfg(target_os = "linux")]
impl NestedProcess {
    /// Start `sleep` as process number `first + 1` of a new PID namespace;
    /// nothing when the host refuses the namespace.
    fn start(first: i64) -> Option<Self> {
        use std::io::BufRead as _;
        let script: String = format!("echo {first} > /proc/sys/kernel/ns_last_pid; sleep 120 & echo $!; read _; kill $!");
        let mut child: Child = Command::new("unshare")
            .args(["--user", "--map-root-user", "--pid", "--fork", "--mount-proc", "sh", "-c", script.as_str()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let output = child.stdout.take()?;
        let mut line: String = String::new();
        std::io::BufReader::new(output).read_line(&mut line).ok()?;
        let local: i64 = line.trim().parse::<i64>().ok()?;
        return Some(Self { child, local });
    }
}

/// Whatever the control did,
/// closing the input ends the namespace's init,
/// which ends the sleeper with it.
#[cfg(target_os = "linux")]
impl Drop for NestedProcess {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _waited = self.child.wait();
    }
}

//endregion

//region The wall clock decides nothing about a holder (cross-vendor review of 2026-11-09, finding 1)

/// A process table whose uptime is zero,
/// so the start this host derives for `pid` is the whole uptime later than it really is:
/// the reading a forward correction of the wall clock produces,
/// which moves the start derived from `/proc/<pid>/stat` and `/proc/uptime` while the lock's birth stays where it is.
fn table_after_a_forward_clock_step(stage: &Stage, pid: i64) -> PathBuf {
    let table: PathBuf = stage.root.join("proc-stepped");
    let process: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(process.as_path()).expect("the process of the table");
    let stat: String = std::fs::read_to_string(Path::new("/proc").join(pid.to_string()).join("stat")).expect("the running process's stat");
    std::fs::write(process.join("stat"), stat).expect("stat");
    std::fs::write(process.join("comm"), "sleep\n").expect("comm");
    std::fs::write(table.join("uptime"), "0.00 0.00\n").expect("uptime");
    return table;
}

/// A running program of this account that is not named like Git,
/// whose number the pid file names,
/// and a clock step that puts its start after the lock was made:
/// a Git that holds the lock under another name looks exactly the same,
/// so the lock stays and the clock is named as what moved.
#[test]
fn a_forward_clock_step_does_not_prove_a_live_holder_reused_the_number() {
    let stage: Stage = killed::stage("stale-refs-clock-step-live-holder", true);
    let other: Helper = sleeper();
    std::thread::sleep(Duration::from_millis(400));
    plant_lock(&stage, PLANTED_REF, Some(other.pid()));
    let stepped: Judge = Judge { proc_root: table_after_a_forward_clock_step(&stage, other.pid()), ..judge() };
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &stepped);
    assert!(!outcome.cleared_any(), "the lock of a running process was removed: {outcome:?}");
    assert!(stage.lock_of(PLANTED_REF).exists());
    let verdict: Option<Verdict> = verdict_of(&outcome, stage.lock_of(PLANTED_REF).as_path());
    assert!(matches!(verdict, Some(Verdict::Unproven(text)) if text.contains("correction of the clock")), "{outcome:?}");
    drop(other);
    stage.finish();
}

//endregion

//region The pid file authorizes nothing (cross-vendor review of 2026-11-09, finding 3)

/// A lock whose file was made before the pid file beside it,
/// which names a process that has ended:
/// the two stored births say the pid file was written after the lock,
/// which is the reading a backward correction of the clock leaves in front of a live writer's new lock.
/// Birth order is the only thing that could tie that number to this lock,
/// so the lock stays.
#[test]
fn a_pid_file_whose_births_say_after_the_lock_proves_nothing() {
    let stage: Stage = killed::stage("stale-refs-backward-step", true);
    let lock: PathBuf = stage.lock_of(PLANTED_REF);
    // The lock's file is made first and left to age, so its birth is earlier than the pid file's: a lock stamped before
    // the pid file beside it.
    let aged: PathBuf = stage.root.join("aged");
    killed::write_file(aged.as_path(), "");
    std::thread::sleep(Duration::from_millis(50));
    killed::write_file(stage.pid_file_of(PLANTED_REF).as_path(), &format!("pid {}\n", ended_pid()));
    std::fs::rename(aged.as_path(), lock.as_path()).expect("the live writer's lock takes the path");
    let outcome: Outcome = clear_stale_ref_locks(stage.roots().as_slice(), &judge());
    assert!(!outcome.cleared_any(), "a live lock was removed on a pid file an earlier holder left: {outcome:?}");
    assert!(lock.exists());
    assert!(matches!(verdict_of(&outcome, lock.as_path()), Some(Verdict::Unproven(_))), "{outcome:?}");
    stage.finish();
}

//endregion
