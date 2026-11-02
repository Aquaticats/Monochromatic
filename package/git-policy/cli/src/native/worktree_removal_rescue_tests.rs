//! Controls for keeping the records of a removed worktree: what is copied before a removal, what a later settle makes of it,
//!  and that a worktree which stays loses nothing and shows nothing.

use super::*;
use crate::push_failure_notice::{record_push_failure, take_orphaned_failures};
use crate::service_work::work_directory;
use crate::test_support::{fixture, remove};
use crate::work_item::enqueue;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER_COMMIT: &str = "89abcdef0123456789abcdef0123456789abcdef";
const BRANCH: &str = "refs/heads/feature";

fn plain(commit: &str) -> String {
    return commit.to_string();
}

/// A common directory with one linked worktree's Git directory in it.
struct Repo {
    root: PathBuf,
    common: PathBuf,
    linked: PathBuf,
}

fn repo(name: &str) -> Repo {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let linked: PathBuf = common.join("worktrees").join("w");
    std::fs::create_dir_all(&linked).expect("linked git directory");
    return Repo { root, common, linked };
}

fn item(kind: WorkKind, commit: &str) -> WorkItem {
    return WorkItem {
        kind,
        repository: String::from("/nowhere"),
        git_dir: String::from("/nowhere/.git"),
        branch_ref: BRANCH.to_string(),
        oid: commit.to_string(),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
}

fn shown(repo: &Repo) -> Vec<String> {
    return take_orphaned_failures(repo.common.as_path(), &plain);
}

/// A failed push nobody saw and a push still pending, both under a worktree that is then removed, are shown by whoever asks
///  next: the failure with Git's words, the pending push as one that may not have finished.
#[test]
fn the_records_of_a_removed_worktree_are_shown_to_the_next_command() {
    let repo: Repo = repo("rescue-removed");
    record_push_failure(repo.linked.as_path(), BRANCH, COMMIT, "refused by test policy").expect("notice");
    enqueue(work_directory(repo.linked.as_path()).as_path(), &item(WorkKind::Push, OTHER_COMMIT)).expect("pending push");
    snapshot_linked_worktrees(repo.common.as_path());
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    settle_removals(repo.common.as_path());
    let texts: Vec<String> = shown(&repo);
    assert_eq!(texts.len(), 2, "{texts:?}");
    assert!(texts.iter().any(|text| { return text.contains("branch feature") && text.contains(COMMIT) && text.contains("refused by test policy"); }), "{texts:?}");
    assert!(
        texts.iter().any(|text| { return text.contains("branch feature") && text.contains(OTHER_COMMIT) && text.contains("removed") && text.contains("may exist only"); }),
        "{texts:?}"
    );
    assert!(shown(&repo).is_empty(), "shown once");
    assert!(!repo.common.join(SNAPSHOT_DIRECTORY).join(instance_of(repo.linked.as_path())).exists(), "the snapshot is gone");
    remove(repo.root.as_path());
}

/// A removal that did not happen (Git refused, or a dry run) leaves the worktree, its records and no notice of loss.
#[test]
fn a_worktree_that_stays_loses_nothing_and_shows_nothing_extra() {
    let repo: Repo = repo("rescue-stays");
    record_push_failure(repo.linked.as_path(), BRANCH, COMMIT, "refused").expect("notice");
    enqueue(work_directory(repo.linked.as_path()).as_path(), &item(WorkKind::Push, OTHER_COMMIT)).expect("pending push");
    snapshot_linked_worktrees(repo.common.as_path());
    settle_removals(repo.common.as_path());
    assert!(shown(&repo).is_empty());
    assert_eq!(files_of(notice_directory(repo.linked.as_path()).as_path()).len(), 1, "the notice is still the worktree's own");
    assert!(!repo.common.join(SNAPSHOT_DIRECTORY).join(instance_of(repo.linked.as_path())).exists(), "the snapshot was discarded");
    remove(repo.root.as_path());
}

/// A pending push of a commit whose failure was already recorded does not replace Git's words with the vaguer ones.
#[test]
fn a_recorded_failure_keeps_its_words_over_the_pending_push_of_the_same_commit() {
    let repo: Repo = repo("rescue-same-commit");
    record_push_failure(repo.linked.as_path(), BRANCH, COMMIT, "refused by test policy").expect("notice");
    enqueue(work_directory(repo.linked.as_path()).as_path(), &item(WorkKind::Push, COMMIT)).expect("pending push");
    snapshot_linked_worktrees(repo.common.as_path());
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    settle_removals(repo.common.as_path());
    let texts: Vec<String> = shown(&repo);
    assert_eq!(texts.len(), 1, "{texts:?}");
    assert!(texts[0].contains("refused by test policy"), "{texts:?}");
    remove(repo.root.as_path());
}

/// A claimed push counts as unfinished, and work of another kind (upkeep, a review) is not a backup and is not reported.
#[test]
fn a_claimed_push_is_reported_and_other_work_is_not() {
    let repo: Repo = repo("rescue-claimed");
    let work: PathBuf = work_directory(repo.linked.as_path());
    enqueue(work.as_path(), &item(WorkKind::Push, COMMIT)).expect("push");
    enqueue(work.as_path(), &item(WorkKind::Maintenance, OTHER_COMMIT)).expect("upkeep");
    let claimed = crate::work_item::claim_next_of(work.as_path(), std::time::SystemTime::now(), &[WorkKind::Push]).expect("claim");
    assert!(claimed.is_some());
    snapshot_linked_worktrees(repo.common.as_path());
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    settle_removals(repo.common.as_path());
    let texts: Vec<String> = shown(&repo);
    assert_eq!(texts.len(), 1, "{texts:?}");
    assert!(texts[0].contains(COMMIT), "{texts:?}");
    remove(repo.root.as_path());
}

/// A worktree with no records is not snapshotted, so the commands that find nothing pay nothing.
#[test]
fn a_worktree_without_records_has_no_snapshot() {
    let repo: Repo = repo("rescue-nothing");
    snapshot_linked_worktrees(repo.common.as_path());
    assert!(!repo.common.join(SNAPSHOT_DIRECTORY).exists());
    settle_removals(repo.common.as_path());
    assert!(shown(&repo).is_empty());
    remove(repo.root.as_path());
}
