//! Controls for the recovery of isolated merges, each on a repository left in one of the states a killed merge
//! leaves.

use super::*;
use crate::merge_isolation_journal::{intent_path, write_merge_intent};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_owner::{TransactionOwner, current_transaction_owner};

/// A repository where `main` and `feature` have diverged and the merge commit exists but is not applied.
struct World {
    /// The fixture directory removed at the end.
    parent: PathBuf,
    /// The worktree.
    repo: PathBuf,
    /// The Git directory.
    git_dir: PathBuf,
    /// `main` before the merge.
    old_oid: String,
    /// The merge commit.
    new_oid: String,
}

/// Build the world: `feature` changes `f.txt`, adds `dir/new.txt` and deletes `gone.txt`; `main` changes `h.txt`.
fn world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for file in ["u.txt", "f.txt", "gone.txt", "h.txt"] {
        std::fs::write(repo.join(file), format!("{file} base\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--", "u.txt", "f.txt", "gone.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/new.txt"), b"new\n").expect("file");
    git(repo.as_path(), &["rm", "--quiet", "--", "gone.txt"]);
    git(repo.as_path(), &["add", "--", "f.txt", "dir/new.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]);
    let new_oid: String = git_text(
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    );
    let git_dir: PathBuf = repo.join(".git");
    return World { parent, repo, git_dir, old_oid, new_oid };
}

/// The recovery place for a world.
fn place(world: &World) -> RecoveryPlace {
    return RecoveryPlace { git: git_context(), cwd: world.repo.clone(), common_dir: world.git_dir.clone() };
}

/// An owner that is certainly gone: this process's PID with a birth identity no process has.
fn dead_owner(id: &str) -> TransactionOwner {
    let mut owner: TransactionOwner = current_transaction_owner(id, "2026-10-29T00:00:00.000Z").expect("owner");
    owner.owner_identity = String::from("linux:1");
    return owner;
}

/// Write the record of a merge of the world, owned by `owner`, and return its file.
fn record_of(world: &World, owner: TransactionOwner, target_ref: Option<&str>) -> PathBuf {
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: world.repo.to_string_lossy().into_owned(),
        target_ref: target_ref.map(String::from),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
    };
    return write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
}

/// Everything about the staged entries and the worktree that another session would notice.
fn snapshot(world: &World) -> (String, String) {
    return (
        git_text(world.repo.as_path(), &["ls-files", "--stage"]),
        git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]),
    );
}

/// A merge killed before it touched anything is discarded, and the checkout is as it was.
#[test]
fn a_merge_that_never_began_is_discarded() {
    let world: World = world("merge-recovery-never-began");
    std::fs::write(world.repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "u.txt"]);
    let before = snapshot(&world);
    let record: PathBuf = record_of(&world, dead_owner("11111111-0000-4000-8000-000000000001"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect("recovery");
    assert_eq!(outcomes, vec![MergeRecoveryOutcome { record: record.clone(), action: MergeRecoveryAction::Discarded }]);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    assert_eq!(snapshot(&world), before);
    remove(world.parent.as_path());
}

/// A record whose owner still runs is left alone and is not a reason to take the landing lock.
#[test]
fn a_live_owners_record_is_left_alone() {
    let world: World = world("merge-recovery-live");
    let live: TransactionOwner =
        current_transaction_owner("22222222-0000-4000-8000-000000000002", "2026-10-29T00:00:00.000Z").expect("owner");
    let record: PathBuf = record_of(&world, live, Some("refs/heads/main"));
    assert!(!merge_intents_to_recover(world.git_dir.as_path()).expect("scan"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::OwnerActive);
    assert!(record.exists());
    remove(world.parent.as_path());
}

/// A dead owner's record, a malformed record and no record are told apart without a lock or a process.
#[test]
fn only_dead_or_unreadable_records_need_recovery() {
    let world: World = world("merge-recovery-scan");
    assert!(!merge_intents_to_recover(world.git_dir.as_path()).expect("no directory"));
    let record: PathBuf = record_of(&world, dead_owner("33333333-0000-4000-8000-000000000003"), None);
    assert!(merge_intents_to_recover(world.git_dir.as_path()).expect("dead owner"));
    std::fs::write(record.as_path(), b"{").expect("damage");
    assert!(merge_intents_to_recover(world.git_dir.as_path()).expect("malformed"));
    let error = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect_err("fails closed");
    assert!(error.0.contains("malformed") && error.0.contains(record.to_string_lossy().as_ref()), "{error}");
    assert!(record.exists(), "a record that cannot be read is kept");
    remove(world.parent.as_path());
}

/// A branch that already names the merge needs only the record removed.
#[test]
fn a_landed_merge_only_loses_its_record() {
    let world: World = world("merge-recovery-landed");
    git(world.repo.as_path(), &["merge", "--ff-only", "--quiet", world.new_oid.as_str()]);
    let record: PathBuf = record_of(&world, dead_owner("44444444-0000-4000-8000-000000000004"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(world.parent.as_path());
}

/// A branch that moved to something else is not touched, and the record is kept with the way out named.
#[test]
fn a_branch_that_moved_elsewhere_is_left_and_named() {
    let world: World = world("merge-recovery-moved");
    std::fs::write(world.repo.join("h.txt"), b"h.txt again\n").expect("file");
    git(world.repo.as_path(), &["commit", "--quiet", "--all", "--message=elsewhere"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let record: PathBuf = record_of(&world, dead_owner("55555555-0000-4000-8000-000000000005"), Some("refs/heads/main"));
    let error = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect_err("fails closed");
    assert!(error.0.contains(moved.as_str()) && error.0.contains(world.old_oid.as_str()), "{error}");
    assert!(error.0.contains("Nothing was changed"), "{error}");
    assert!(record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved);
    remove(world.parent.as_path());
}

/// Git updated the worktree and the index and died before it moved the branch: recovery moves the branch, and
/// another session's staged entry is exactly as it was.
#[test]
fn a_merge_killed_after_the_checkout_update_is_completed() {
    let world: World = world("merge-recovery-after-checkout");
    std::fs::write(world.repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "u.txt"]);
    let staged_entry: String = git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]);
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    let record: PathBuf = record_of(&world, dead_owner("66666666-0000-4000-8000-000000000006"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt");
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]), staged_entry);
    assert!(!world.repo.join("gone.txt").exists());
    assert_eq!(std::fs::read(world.repo.join("dir/new.txt")).expect("added file"), b"new\n");
    remove(world.parent.as_path());
}

/// Git wrote some files and died before it wrote the index: recovery finishes the files and the index.
#[test]
fn a_merge_killed_between_the_worktree_and_the_index_is_completed() {
    let world: World = world("merge-recovery-before-index");
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(world.repo.join("gone.txt")).expect("the file Git removed");
    let record: PathBuf = record_of(&world, dead_owner("77777777-0000-4000-8000-000000000007"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]), "");
    assert_eq!(std::fs::read(world.repo.join("dir/new.txt")).expect("added file"), b"new\n");
    remove(world.parent.as_path());
}

/// A path that holds neither its old nor its merged content belongs to someone else: nothing is changed.
#[test]
fn a_foreign_edit_stops_the_completion() {
    let world: World = world("merge-recovery-foreign");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    std::fs::write(world.repo.join("f.txt"), b"someone else's edit\n").expect("foreign edit");
    let record: PathBuf = record_of(&world, dead_owner("88888888-0000-4000-8000-000000000008"), Some("refs/heads/main"));
    let error = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect_err("fails closed");
    assert!(error.0.contains("f.txt") && error.0.contains("Nothing was changed"), "{error}");
    assert!(record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("file"), b"someone else's edit\n");
    remove(world.parent.as_path());
}

/// A change made by someone else before the merge began is not evidence that it began.
#[test]
fn an_unrelated_edit_does_not_make_a_merge_that_never_began_complete() {
    let world: World = world("merge-recovery-edit-only");
    std::fs::write(world.repo.join("f.txt"), b"an edit made while the merge was dead\n").expect("edit");
    let record: PathBuf = record_of(&world, dead_owner("99999999-0000-4000-8000-000000000009"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Discarded);
    assert!(!record.exists());
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("file"), b"an edit made while the merge was dead\n");
    remove(world.parent.as_path());
}

/// Another session's staged change to a path the merge changes is not evidence that the merge began.
#[test]
fn a_staged_change_of_a_merged_path_is_not_a_started_merge() {
    let world: World = world("merge-recovery-staged-same-path");
    std::fs::write(world.repo.join("f.txt"), b"f.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    let before = snapshot(&world);
    let record: PathBuf = record_of(&world, dead_owner("cccccccc-0000-4000-8000-00000000000c"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Discarded);
    assert!(!record.exists());
    assert_eq!(snapshot(&world), before);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    remove(world.parent.as_path());
}

/// A detached `HEAD` moves by `--no-deref`.
#[test]
fn a_detached_head_is_completed_too() {
    let world: World = world("merge-recovery-detached");
    git(world.repo.as_path(), &["switch", "--quiet", "--detach", "HEAD"]);
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = record_of(&world, dead_owner("aaaaaaaa-0000-4000-8000-00000000000a"), None);
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(world.parent.as_path());
}

/// The merge's own process settles a failure the same way, whatever its owner.
#[test]
fn a_failed_merge_settles_without_a_liveness_check() {
    let world: World = world("merge-recovery-settle");
    let live: TransactionOwner =
        current_transaction_owner("bbbbbbbb-0000-4000-8000-00000000000b", "2026-10-29T00:00:00.000Z").expect("owner");
    let record: PathBuf = record_of(&world, live.clone(), Some("refs/heads/main"));
    let intent: MergeIntent = parse_merge_intent(std::fs::read(record.as_path()).expect("read").as_slice()).expect("intent");
    assert_eq!(intent_path(world.git_dir.as_path(), live.transaction_id.as_str()), record);
    let action = settle_failed_merge(&place(&world), record.as_path(), &intent).expect("settled");
    assert_eq!(action, MergeRecoveryAction::Discarded);
    assert!(!record.exists());
    remove(world.parent.as_path());
}
