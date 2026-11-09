//! Controls for what every command says about the worktree's pushes:
//! the held ones,
//! whose notice must reach the person without any notice directory existing yet,
//! and the words it uses.

use super::*;
use crate::push_failure_notice::CommitNamer;
use crate::test_support::{fixture, remove};
use crate::work_item::{WorkItem, WorkKind};

/// What:
///  The proof of a repository that is not asked,
/// so these controls read the held records alone.
/// Why:
///   The line is what they are about;
/// whether Git already holds the commit is asked of Git in
///  `held_push_commit_on_the_remote_tests.rs`.
fn nothing_asked() -> crate::held_push_commit_on_the_remote::NothingAsked {
    return crate::held_push_commit_on_the_remote::NothingAsked;
}

/// A commit name as it is,
/// so a control reads the line the record holds.
struct AsIs;

impl CommitNamer for AsIs {
    fn name_of(&self, commit: &str) -> String {
        return commit.to_string();
    }
}

/// A held push alone says its line:
/// no push-failure notice directory and no orphan-notice directory exists,
/// and the recording is what makes the one the notice is read from.
#[test]
fn a_held_push_is_said_with_no_notice_directory() {
    let root: PathBuf = fixture("notices-held-push");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let oid: String = String::from("1".repeat(40));
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: oid.clone(),
        parent: String::new(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    assert!(!notice_directory(git_dir.as_path()).is_dir(), "no notice directory exists before the push is held");
    crate::work_item::hold(work_dir.as_path(), &item).expect("the push is held");
    assert!(!notice_directory(git_dir.as_path()).is_dir(), "the hold wrote no second file: the record is the state");
    let said: Vec<String> = crate::service_handoff::held_push_notices(git_dir.as_path(), &nothing_asked());
    assert_eq!(said.len(), 1, "one line: {said:?}");
    assert!(said[0].contains(oid.as_str()), "{}", said[0]);
    assert!(said[0].contains("next push"), "{}", said[0]);
    // The words claim only what is known: the push is held, and the next push of its branch carries it.
    assert!(!said[0].contains("stopped"), "the line does not claim the command stopped: {}", said[0]);
    assert!(!said[0].contains("checks"), "and not that its checks ran or not: {}", said[0]);
    // The state stands while the record stands, so the line stands with it and cannot disagree with it.
    let again: Vec<String> = crate::service_handoff::held_push_notices(git_dir.as_path(), &nothing_asked());
    assert_eq!(again, said, "the line is the record's: {again:?}");
    assert!(
        crate::work_item::state_directory(work_dir.as_path(), crate::work_item::HELD).join(item.id()).exists(),
        "the record stays held"
    );
    remove(root.as_path());
}

/// A worktree with nothing held and no notice says nothing at all.
#[test]
fn a_worktree_with_nothing_held_says_nothing() {
    let root: PathBuf = fixture("notices-held-push-quiet");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    assert!(crate::service_handoff::held_push_notices(git_dir.as_path(), &nothing_asked()).is_empty(), "a worktree with nothing held says nothing");
    assert!(push_notices_for(git_dir.as_path(), git_dir.as_path(), &AsIs).is_empty(), "and nothing failed either");
    assert!(!notice_directory(git_dir.as_path()).is_dir(), "the check made no directory either");
    remove(root.as_path());
}

/// A released push is not said:
/// the record is gone,
/// so the line is gone with it,
/// and no second file had to be cleared.
#[test]
fn a_released_push_is_not_said() {
    let root: PathBuf = fixture("notices-held-push-released");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("2".repeat(40)),
        parent: String::new(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    crate::work_item::hold(work_dir.as_path(), &item).expect("the push is held");
    assert_eq!(crate::service_handoff::held_push_notices(git_dir.as_path(), &nothing_asked()).len(), 1);
    assert!(crate::work_item::promote(work_dir.as_path(), item.id().as_str()), "the producer released it");
    assert_eq!(crate::service_handoff::held_push_notices(git_dir.as_path(), &nothing_asked()), Vec::<String>::new(), "a released push is not said");
    remove(root.as_path());
}

/// A blocked push is not said:
/// the verdict removed the record,
/// so nothing reports it as held.
#[test]
fn a_blocked_push_is_not_said() {
    let root: PathBuf = fixture("notices-held-push-blocked");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("3".repeat(40)),
        parent: String::new(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    crate::work_item::hold(work_dir.as_path(), &item).expect("the push is held");
    assert_eq!(crate::service_handoff::held_push_notices(git_dir.as_path(), &nothing_asked()).len(), 1);
    crate::work_item::block(work_dir.as_path(), item.id().as_str());
    assert_eq!(crate::service_handoff::held_push_notices(git_dir.as_path(), &nothing_asked()), Vec::<String>::new(), "a blocked push is not said");
    remove(root.as_path());
}
