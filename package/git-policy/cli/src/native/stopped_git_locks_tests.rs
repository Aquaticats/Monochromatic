//! Controls for the sweep of lock files a stopped Git child left: what it removes, and every file it must keep.
//! Why:
//!  The sweep deletes files in a repository, so each condition that makes a lock the stopped child's is pinned on both
//!  sides, on disposable repositories and real short-lived processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ledger.sweep(tree)).toEqual([lockOfTheStoppedChild]); expect(existsSync(lockOfAnotherGit)).toBe(true);
//! ```
#![cfg(unix)]

use super::*;
use crate::test_support::{fixture, remove};
use std::process::Child;

/// More than the coarse-clock slack, so a file made before the ledger is older than the ledger for certain.
const AGE_PAUSE: Duration = Duration::from_millis(200);

/// A repository directory with the ref directory the incident's locks sat in.
struct Repo {
    /// The fixture directory, removed at the end.
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

/// A ledger for a command that runs in the repository.
fn ledger_of(repo: &Repo) -> LockLedger {
    let mut command: Command = Command::new("git");
    command.current_dir(repo.root.as_path());
    return LockLedger::before_start(&command);
}

/// A tree member that has ended: a short-lived process, recorded while it ran and reaped.
fn ended_member() -> TreeMember {
    let mut child: Child = Command::new("sleep").arg("0.3").spawn().expect("start sleep");
    let member: TreeMember = crate::process_tree::member_of(i32::try_from(child.id()).expect("pid")).expect("a running process");
    child.wait().expect("reap sleep");
    return member;
}

/// A tree member that still runs, and its child to end afterwards.
fn running_member() -> (TreeMember, Child) {
    let child: Child = Command::new("sleep").arg("30").spawn().expect("start sleep");
    let member: TreeMember = crate::process_tree::member_of(i32::try_from(child.id()).expect("pid")).expect("a running process");
    return (member, child);
}

/// Write a pid file the way Git does.
fn write_pid_file(path: &Path, pid: i32) {
    std::fs::write(path, format!("pid {pid}\n")).expect("pid file");
}

/// A lock and its pid file naming an ended member of the tree are both removed.
#[test]
fn a_lock_named_by_an_ended_member_goes_with_its_pid_file() {
    let repo: Repo = repo("stopped-locks-named");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("lock with content");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    let removed: Vec<PathBuf> = ledger.sweep(std::slice::from_ref(&member));
    assert_eq!(removed.len(), 2, "{removed:?}");
    assert!(!lock_of(&repo, "main").exists());
    assert!(!pid_file_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A lock whose pid file names a process outside the tree is another Git's and stays.
#[test]
fn a_lock_named_by_a_process_outside_the_tree_stays() {
    let repo: Repo = repo("stopped-locks-foreign");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("lock with content");
    write_pid_file(pid_file_of(&repo, "main").as_path(), i32::try_from(std::process::id()).expect("pid"));
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    assert!(pid_file_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A member that still runs keeps its lock: it may be about to finish.
#[test]
fn a_lock_named_by_a_running_member_stays() {
    let repo: Repo = repo("stopped-locks-running");
    let ledger: LockLedger = ledger_of(&repo);
    let (member, mut child): (TreeMember, Child) = running_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("lock with content");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    child.kill().expect("end sleep");
    child.wait().expect("reap sleep");
    remove(repo.root.as_path());
}

/// A lock older than the child cannot be the child's, even when its pid file names a member (a reused process number).
#[test]
fn a_lock_older_than_the_child_stays() {
    let repo: Repo = repo("stopped-locks-older");
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123\n").expect("lock with content");
    write_pid_file(pid_file_of(&repo, "main").as_path(), member.pid);
    std::thread::sleep(AGE_PAUSE);
    let ledger: LockLedger = ledger_of(&repo);
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// An empty lock with no pid file, born after the child started, is removed once the whole tree has ended.
#[test]
fn an_empty_lock_without_a_pid_file_goes_when_the_tree_has_ended() {
    let repo: Repo = repo("stopped-locks-empty");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "").expect("empty lock");
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), vec![lock_of(&repo, "main")]);
    assert!(!lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// An empty lock stays while any process of the tree still runs: its creator may be that process.
#[test]
fn an_empty_lock_stays_while_the_tree_runs() {
    let repo: Repo = repo("stopped-locks-tree-running");
    let ledger: LockLedger = ledger_of(&repo);
    let (running, mut child): (TreeMember, Child) = running_member();
    let ended: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "").expect("empty lock");
    assert_eq!(ledger.sweep(&[ended, running]), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    child.kill().expect("end sleep");
    child.wait().expect("reap sleep");
    remove(repo.root.as_path());
}

/// An empty lock with no pid file and no stopped tree is nobody's to remove.
#[test]
fn an_empty_tree_removes_nothing() {
    let repo: Repo = repo("stopped-locks-no-tree");
    let ledger: LockLedger = ledger_of(&repo);
    std::fs::write(lock_of(&repo, "main"), "").expect("empty lock");
    assert_eq!(ledger.sweep(&[]), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// A lock with content and no pid file may be another Git's work in progress and stays.
#[test]
fn a_lock_with_content_and_no_pid_file_stays() {
    let repo: Repo = repo("stopped-locks-content");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "0123456789012345678901234567890123456789\n").expect("lock with content");
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// An empty lock born before the child started stays: it was there already.
#[test]
fn an_empty_lock_older_than_the_child_stays() {
    let repo: Repo = repo("stopped-locks-empty-older");
    std::fs::write(lock_of(&repo, "main"), "").expect("empty lock");
    std::thread::sleep(AGE_PAUSE);
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// An empty lock whose pid file names another process is that process's, even though the tree has ended.
#[test]
fn an_empty_lock_with_a_pid_file_is_decided_by_the_pid_file() {
    let repo: Repo = repo("stopped-locks-empty-pid");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    std::fs::write(lock_of(&repo, "main"), "").expect("empty lock");
    write_pid_file(pid_file_of(&repo, "main").as_path(), i32::try_from(std::process::id()).expect("pid"));
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// Files that are not locks, and a lock-named directory, are left alone.
#[test]
fn only_regular_lock_files_are_candidates() {
    let repo: Repo = repo("stopped-locks-other-files");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    let plain: PathBuf = repo.git_dir.join("refs").join("remotes").join("origin").join("main");
    std::fs::write(plain.as_path(), "").expect("empty ref");
    std::fs::create_dir(lock_of(&repo, "directory")).expect("directory named like a lock");
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), Vec::<PathBuf>::new());
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
    std::fs::write(lock_of(&repo, "main"), "").expect("empty lock");
    assert_eq!(ledger.sweep(std::slice::from_ref(&member)), Vec::<PathBuf>::new());
    assert!(lock_of(&repo, "main").exists());
    remove(repo.root.as_path());
}

/// Locks are found in the top of the Git directory, in nested refs and in the pack and info directories of the object
/// store, but not under loose-object directories or hooks.
#[test]
fn locks_are_searched_where_git_makes_them() {
    let repo: Repo = repo("stopped-locks-places");
    let ledger: LockLedger = ledger_of(&repo);
    let member: TreeMember = ended_member();
    let found: Vec<PathBuf> = vec![
        repo.git_dir.join("index.lock"),
        repo.git_dir.join("packed-refs.lock"),
        repo.git_dir.join("objects").join("pack").join("pack.lock"),
        repo.git_dir.join("objects").join("info").join("commit-graphs.lock"),
        repo.git_dir.join("refs").join("heads").join("feat").join("deep").join("topic.lock"),
    ];
    let skipped: Vec<PathBuf> = vec![repo.git_dir.join("objects").join("ab").join("cd.lock"), repo.git_dir.join("hooks").join("x.lock")];
    for path in found.iter().chain(skipped.iter()) {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
        std::fs::write(path, "").expect("empty lock");
    }
    let mut removed: Vec<PathBuf> = ledger.sweep(std::slice::from_ref(&member));
    removed.sort();
    let mut expected: Vec<PathBuf> = found.clone();
    expected.sort();
    assert_eq!(removed, expected);
    for path in &skipped {
        assert!(path.exists(), "{path:?}");
    }
    remove(repo.root.as_path());
}

/// The repository of a command comes from a `gitdir:` file, from `GIT_DIR`, and from the directory above the working one.
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
