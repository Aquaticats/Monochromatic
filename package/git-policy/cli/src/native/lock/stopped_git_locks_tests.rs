//! Controls for the sweep after a stopped Git child:
//! it removes a lock only on a proof about the holder,
//! and keeps every other file.
//! Why:
//!  The sweep deletes files in a repository,
//! and a stopped child's lifetime,
//! a lock born in it and an empty file all fit a live
//!  writer of another Git as well (finding S-1 of the cross-vendor review of 2026-11-03T11:33:50-05:00).
//! Each schedule that
//!  deleted such a lock is pinned,
//! on disposable repositories and real short-lived processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ledger.sweep(tree)).toEqual([lockWhoseHolderIsProvenGone]); expect(existsSync(lockOfAnotherGit)).toBe(true);
//! ```
#![cfg(unix)]

use super::*;
use crate::test_support::{fixture, remove};
use crate::process_tree::TreeMember;
use std::process::Child;
use std::time::Duration;

/// A repository directory with the ref directory the incident's locks sat in.
struct Repo {
    /// The fixture directory,
    /// removed at the end.
    root: PathBuf,
    /// Its `.git` directory.
    git_dir: PathBuf,
}

/// Build the repository.
fn repo(name: &str) -> Repo {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join(".git");
    std::fs::create_dir_all(git_dir.join("refs").join("remotes").join("origin")).expect("ref directory");
    return Repo { root, git_dir };
}

/// The lock path of a tracking ref.
fn lock_of(repo: &Repo, reference: &str) -> PathBuf {
    return repo.git_dir.join("refs").join("remotes").join("origin").join(format!("{reference}.lock"));
}

/// The pid file beside a lock of a tracking ref.
fn pid_file_of(repo: &Repo, reference: &str) -> PathBuf {
    return repo.git_dir.join("refs").join("remotes").join("origin").join(format!("{reference}{PID_LOCK_SUFFIX}"));
}

/// A ledger for a command that runs in the repository,
/// judging as the real judge does but taking every lock as old enough.
fn ledger_of(repo: &Repo) -> LockLedger {
    let mut command: Command = Command::new("git");
    command.current_dir(repo.root.as_path());
    return LockLedger::before_start(&command).with_judge(crate::stale_ref_lock_killed_git::judge());
}

/// A tree member that has ended:
/// a short-lived process,
/// recorded while it ran and reaped.
fn ended_member() -> TreeMember {
    let mut child: Child = Command::new("sleep").arg("0.3").spawn().expect("start sleep");
    let member: TreeMember = crate::process_tree::member_of(i32::try_from(child.id()).expect("pid")).expect("a running process");
    child.wait().expect("reap sleep");
    return member;
}

/// A tree member that still runs,
/// and its child to end afterwards.
fn running_member() -> (TreeMember, Child) {
    let child: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("30")).expect("start sleep");
    let member: TreeMember = crate::process_tree::member_of(i32::try_from(child.id()).expect("pid")).expect("a running process");
    return (member, child);
}

/// Write a pid file the way Git does.
fn write_pid_file(path: &Path, pid: i32) {
    std::fs::write(path, format!("pid {pid}\n")).expect("pid file");
}

/// A lock whose pid file names an ended member of the stopped tree stays with its pid file:
/// the number is a number the wrapper's own namespace gave its child,
/// and a Git of a sibling namespace may have written the same one,
/// so the pid file proves nothing about this lock.
#[test]
fn a_lock_named_by_an_ended_member_stays_with_its_pid_file() {
    let repo: Repo = repo("stopped-locks-named");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("lock with content");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists() && pid_file_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A lock whose pid file names a process that runs is that process's and stays,
/// whether or not it belongs to the tree.
#[test]
fn a_lock_named_by_a_running_process_stays() {
    let repo: Repo = repo("stopped-locks-running");
    let ledger: LockLedger = ledger_of(&repo);
    let (member, mut child): (TreeMember, Child) = running_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("lock with content");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    child.kill().expect("end sleep");
    child.wait().expect("reap sleep");
    remove(repo.root.as_path());
}

/// A lock with content and no pid file may be another Git's work in progress and stays.
#[test]
fn a_lock_with_content_and_no_pid_file_stays() {
    let repo: Repo = repo("stopped-locks-content");
    let ledger: LockLedger = ledger_of(&repo);
    std::fs::write(lock_of(&repo, "main"), "0123456789012345678901234567890123456789\n").expect("lock with content");
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// An empty lock whose pid file names a running process is that process's.
#[test]
fn an_empty_lock_with_a_pid_file_is_decided_by_the_pid_file() {
    let repo: Repo = repo("stopped-locks-empty-pid");
    let ledger: LockLedger = ledger_of(&repo);
    std::fs::write(lock_of(&repo, "main"), "").expect("empty lock");
    write_pid_file(pid_file_of(&repo, "main").as_path(), i32::try_from(std::process::id()).expect("pid"));
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// Files that are not locks,
/// and a lock-named directory,
/// are left alone.
#[test]
fn only_regular_lock_files_are_candidates() {
    let repo: Repo = repo("stopped-locks-other-files");
    let ledger: LockLedger = ledger_of(&repo);
    let plain: PathBuf = repo.git_dir.join("refs").join("remotes").join("origin").join("main");
    std::fs::write(plain.as_path(), "").expect("empty ref");
    std::fs::create_dir(lock_of(&repo, "directory")).expect("directory named like a lock");
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(plain.exists());
    assert!(lock_of(&repo, "directory").is_dir());
    remove(repo.root.as_path());
}

/// A ledger that finds nothing removes nothing.
#[test]
fn the_empty_ledger_removes_nothing() {
    let repo: Repo = repo("stopped-locks-none");
    let ledger: LockLedger = LockLedger::none();
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("lock with content");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// The repository of a command comes from a `gitdir:` file,
/// from `GIT_DIR`,
/// and from the directory above the working one.
#[test]
fn the_repository_is_found_from_the_working_directory_or_the_environment() {
    let repo: Repo = repo("stopped-locks-roots");
    let nested: PathBuf = repo.root.join("a").join("b");
    std::fs::create_dir_all(nested.as_path()).expect("nested");
    let mut command: Command = Command::new("git");
    command.current_dir(nested.as_path());
    assert_eq!(LockLedger::before_start(&command).roots, vec![repo.git_dir.clone()]);
    let linked: PathBuf = repo.root.join("linked");
    std::fs::create_dir_all(linked.as_path()).expect("linked");
    let admin: PathBuf = repo.git_dir.join("worktrees").join("linked");
    std::fs::create_dir_all(admin.as_path()).expect("admin");
    std::fs::write(admin.join("commondir"), "../..\n").expect("commondir");
    std::fs::write(linked.join(".git"), format!("gitdir: {}\n", admin.display())).expect("gitdir file");
    let mut in_linked: Command = Command::new("git");
    in_linked.current_dir(linked.as_path());
    let roots: Vec<PathBuf> = LockLedger::before_start(&in_linked).roots;
    assert_eq!(roots.first(), Some(&admin));
    assert_eq!(roots.len(), 2, "{roots:?}");
    let mut named: Command = Command::new("git");
    named.current_dir(linked.as_path()).env("GIT_DIR", repo.git_dir.as_os_str());
    assert_eq!(LockLedger::before_start(&named).roots, vec![repo.git_dir.clone()]);
    remove(repo.root.as_path());
}

//region Locks of a live foreign writer, which the sweep must never remove
// Cross-vendor review of 2026-11-03T11:33:50-05:00, finding S-1: every schedule below ended with a live writer's lock deleted.

/// The reviewer's schedule:
/// a foreign writer without `core.lockfilePid` creates the lock while the child lives,
/// its lock
///  descriptor is closed,
/// and it keeps the lock empty for as long as it likes while the child's tree has ended.
#[test]
fn a_live_foreign_writers_empty_lock_stays() {
    let repo: Repo = repo("stopped-locks-foreign-writer");
    let ledger: LockLedger = ledger_of(&repo);
    // The foreign writer takes the lock after the child started, and never writes a pid file.
    std::fs::write(lock_of(&repo, "main"), "").expect("the foreign writer's empty lock");
    let removed: Vec<PathBuf> = ledger.sweep();
    assert_eq!(removed, Vec::<PathBuf>::new(), "a lock nothing ties to the stopped child was removed");
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A pid file that is older than the lock names an earlier holder;
/// the lock beside it is another writer's,
/// whose own pid
///  file Git could not create because this one was in the way.
#[test]
fn a_lock_made_after_a_pid_file_that_names_an_ended_member_stays() {
    let repo: Repo = repo("stopped-locks-older-pid-file");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    std::thread::sleep(Duration::from_millis(50));
    std::fs::write(lock_of(&repo, "main"), "").expect("the foreign writer's lock, made after the pid file");
    let removed: Vec<PathBuf> = ledger.sweep();
    assert_eq!(removed, Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists() && pid_file_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// Locks of other protocols are never the sweep's to remove,
/// whatever the child did.
#[test]
fn locks_that_are_not_reference_locks_stay() {
    let repo: Repo = repo("stopped-locks-not-references");
    let ledger: LockLedger = ledger_of(&repo);
    let others: Vec<PathBuf> = vec![
        repo.git_dir.join("index.lock"),
        repo.git_dir.join("shallow.lock"),
        repo.git_dir.join("objects").join("pack").join("pack.lock"),
    ];
    for path in &others {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
        std::fs::write(path, "").expect("empty lock");
    }
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    for path in &others {
        assert!(path.exists(), "{path:?}");
    }
    remove(repo.root.as_path());
}

//endregion

//region The wrapper's own reaped child, in a process table that is not the machine's
// The container gate showed `git_stop_lock_tests::a_stop_without_grace_sweeps_the_locks_on_a_thread_of_its_own` failing
// there: a container has a PID namespace of its own, so the judge cannot tell a gone number from one of another namespace.

/// A judge with the view a container has:
/// the process numbers it reads are not known to be the machine's.
fn container_judge() -> crate::stale_ref_lock_judge::Judge {
    return crate::stale_ref_lock_judge::Judge { ..crate::stale_ref_lock_killed_git::judge() };
}

/// A pid file holds the number its writer had in the writer's own PID namespace,
/// and Git records no namespace:
/// a number that matches a member of the stopped tree,
/// on a lock born after that member started,
/// may be a Git of a sibling namespace that shares the repository.
/// The wrapper never saw this file held,
/// so nothing proves the writer ended and the lock stays.
#[test]
fn a_lock_whose_pid_file_names_an_ended_childs_number_stays_without_a_claim() {
    let repo: Repo = repo("stopped-locks-container-view");
    let mut command: Command = Command::new("git");
    command.current_dir(repo.root.as_path());
    let ledger: LockLedger = LockLedger::before_start(&command).with_judge(container_judge());
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("the lock the child held");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists() && pid_file_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A ledger for a command in the repository whose judge has the view of a container.
fn container_ledger_of(repo: &Repo) -> LockLedger {
    let mut command: Command = Command::new("git");
    command.current_dir(repo.root.as_path());
    return LockLedger::before_start(&command).with_judge(container_judge());
}

/// An old pid file in front of a newer lock names an earlier holder:
/// the newer lock is another writer's,
/// in any view.
#[test]
fn in_a_container_view_a_pid_file_older_than_the_lock_vouches_for_nothing() {
    let repo: Repo = repo("stopped-locks-container-older-pid");
    let ledger: LockLedger = container_ledger_of(&repo);
    let member: TreeMember = ended_member();
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    std::thread::sleep(Duration::from_millis(50));
    std::fs::write(lock_of(&repo, "main"), "").expect("the foreign writer's lock, made after the pid file");
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A lock that was there before the child started is not the child's,
/// whatever its pid file says,
/// and neither is one made during the child's life:
/// the number is all there is,
/// and it is not the wrapper's own proof.
#[test]
fn in_a_container_view_a_lock_named_by_an_ended_member_stays() {
    let repo: Repo = repo("stopped-locks-container-older-lock");
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("the lock");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    std::thread::sleep(Duration::from_millis(50));
    let ledger: LockLedger = container_ledger_of(&repo);
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A number that is not a process of the stopped tree is not the wrapper's to vouch for,
/// though nothing runs under it.
#[test]
fn in_a_container_view_a_number_outside_the_tree_proves_nothing() {
    let repo: Repo = repo("stopped-locks-container-outside");
    let ledger: LockLedger = container_ledger_of(&repo);
    let outsider: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("the lock");
    write_pid_file(pid_file_of(&repo, "main").as_path(), outsider.pid);
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A member of the tree that still runs holds its lock.
#[test]
fn in_a_container_view_a_running_member_keeps_its_lock() {
    let repo: Repo = repo("stopped-locks-container-running");
    let ledger: LockLedger = container_ledger_of(&repo);
    let (member, mut child): (TreeMember, Child) = running_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("the lock");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    child.kill().expect("end sleep");
    child.wait().expect("reap sleep");
    remove(repo.root.as_path());
}

/// A lock with no pid file is another writer's in any view.
#[test]
fn in_a_container_view_a_lock_without_a_pid_file_stays() {
    let repo: Repo = repo("stopped-locks-container-no-pid");
    let ledger: LockLedger = container_ledger_of(&repo);
    std::fs::write(lock_of(&repo, "main"), "").expect("the lock");
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A lock of the stopped child waits out the margin like any other:
/// nothing the wrapper saw makes it judge a young lock sooner,
/// and a patient margin leaves it,
/// while the same lock past the margin is decided by the judgement alone.
#[test]
fn a_young_lock_of_a_stopped_child_waits_out_the_margin() {
    let repo: Repo = repo("stopped-locks-young");
    let mut command: Command = Command::new("git");
    command.current_dir(repo.root.as_path());
    let patient = crate::stale_ref_lock_judge::Judge { margin: Duration::from_secs(3600), ..container_judge() };
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("the child's lock");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    let ledger: LockLedger = LockLedger::before_start(&command).with_judge(patient);
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new(), "a lock inside the margin is left without a word");
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

//endregion

/// A lock whose pid file names a running member of the stopped tree is not that member's.
/// A Git of a sibling PID namespace that shares this repository may have written that number,
/// and nothing the wrapper sees of its own child distinguishes the two,
/// so no claim is made from a number in a file:
/// the lock stays and is named.
#[test]
fn a_lock_whose_pid_file_names_a_running_member_is_not_claimed() {
    let repo: Repo = repo("sweep-a-number-is-not-a-claim");
    let lock: PathBuf = lock_of(&repo, "main");
    std::fs::write(&lock, b"0123456789abcdef0123456789abcdef01234567\n").expect("another Git's lock");
    let (member, mut child) = running_member();
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    let ledger: LockLedger = ledger_of(&repo);
    // The cut: the pid file is removed and the member is killed before it removes the lock.
    std::fs::remove_file(pid_file_of(&repo, "main")).expect("the pid file goes first");
    child.kill().expect("kill the member");
    child.wait().expect("reap the member");
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new(), "a number in a pid file is no claim");
    assert!(lock.exists());
    remove(repo.root.as_path());
}

/// A lock whose pid file went first is left:
/// Git's handler removes a lock's pid file and then the lock,
/// so a kill between the two leaves a lock nothing names,
/// and nothing else the wrapper saw says whose it was.
#[test]
fn a_lock_whose_pid_file_went_first_stays_and_is_named() {
    let repo: Repo = repo("sweep-lock-without-its-pid-file");
    let lock: PathBuf = lock_of(&repo, "main");
    std::fs::write(&lock, b"0123456789abcdef0123456789abcdef01234567\n").expect("lock");
    let (member, mut child) = running_member();
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    let ledger: LockLedger = ledger_of(&repo);
    std::fs::remove_file(pid_file_of(&repo, "main")).expect("the pid file goes first");
    child.kill().expect("kill the holder");
    child.wait().expect("reap the holder");
    assert_eq!(ledger.sweep(), Vec::<PathBuf>::new());
    assert!(lock.exists());
    remove(repo.root.as_path());
}
