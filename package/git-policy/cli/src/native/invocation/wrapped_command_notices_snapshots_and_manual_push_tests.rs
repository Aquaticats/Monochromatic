//! Controls for what every command says about the worktree's pushes:
//! the held ones,
//! whose notice must reach the person without any notice directory existing yet,
//! and the words it uses.

use super::*;
use crate::push_failure_notice::CommitNamer;
use crate::test_support::{fixture, remove};
use crate::work_item::{WorkItem, WorkKind};

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
    let said: Vec<String> = crate::test_support::shown_notices(push_notices_for(git_dir.as_path(), git_dir.as_path(), &AsIs));
    assert_eq!(said.len(), 1, "one line: {said:?}");
    assert!(said[0].contains(oid.as_str()), "{}", said[0]);
    assert!(said[0].contains("next push"), "{}", said[0]);
    // The words claim only what is known: the push is held, and the next push of its branch carries it.
    assert!(!said[0].contains("stopped"), "the line does not claim the command stopped: {}", said[0]);
    assert!(!said[0].contains("checks"), "and not that its checks ran or not: {}", said[0]);
    let again: Vec<String> = crate::test_support::shown_notices(push_notices_for(git_dir.as_path(), git_dir.as_path(), &AsIs));
    assert!(again.is_empty(), "said once: {again:?}");
    assert!(
        crate::work_item::state_directory(work_dir.as_path(), crate::work_item::HELD).join(item.id()).exists(),
        "the record stays held: a notice is not a release"
    );
    remove(root.as_path());
}

/// A worktree with nothing held and no notice says nothing at all.
#[test]
fn a_worktree_with_nothing_held_says_nothing() {
    let root: PathBuf = fixture("notices-held-push-quiet");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    assert!(push_notices_for(git_dir.as_path(), git_dir.as_path(), &AsIs).is_empty(), "a worktree with nothing held says nothing");
    assert!(!notice_directory(git_dir.as_path()).is_dir(), "the check made no directory either");
    remove(root.as_path());
}
