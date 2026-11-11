//! Controls for the recovery of isolated merges,
//! each on a repository left in one of the states a killed merge
//! leaves.

use super::*;
use crate::index_holder_wait::IndexWait;
use crate::merge_isolation_changes::{WorktreeState, file_identity, worktree_path, worktree_state};
use crate::merge_isolation_journal::{intent_path, parse_merge_intent, write_merge_intent};
use crate::merge_isolation_recovery_index_lock::may_install;

/// The judgement of one path,
/// for the controls that build a changed path of their own.
use crate::merge_isolation_recovery_steps::{install_entries, put_back_stranded_takes_awaiting, standing};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::{Path, PathBuf};
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

/// Build the world:
/// `feature` changes `f.txt`,
/// adds `dir/new.txt` and deletes `gone.txt`;
/// `main` changes `h.txt`.
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
    return RecoveryPlace { git: git_context(), cwd: world.repo.clone(), common_dir: world.git_dir.clone(), git_dir: world.git_dir.clone() };
}

/// An owner that is certainly gone:
/// this process's PID with a birth identity no process has.
fn dead_owner(id: &str) -> TransactionOwner {
    let mut owner: TransactionOwner = current_transaction_owner(id, "2026-10-29T00:00:00.000Z").expect("owner");
    owner.owner_identity = String::from("linux:1");
    return owner;
}

/// Write the record of a merge of the world,
/// owned by `owner`,
/// and return its file.
fn record_of(world: &World, owner: TransactionOwner, target_ref: Option<&str>) -> PathBuf {
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: world.repo.to_path_buf(),
        target_ref: target_ref.map(String::from),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: None,
        carried: None,
    };
    return write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
}

/// What:
///  A repository where the branch holds `dir/child.txt` and the merge deletes it and adds a file named `dir`,
///  with the merge commit ready and nothing of it in the checkout.
/// Why:
///  That shape makes the worktree state of the added path a directory,
/// which is neither side of the change and used to
///  refuse the merge for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(...); write('dir/child.txt'); commit(); /* feature: rm dir/child.txt, write file 'dir' */ const merge = mergeTreeCommit(...);
/// ```
fn directory_to_file_world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/child.txt"), b"child base\n").expect("file");
    std::fs::write(repo.join("h.txt"), b"h.txt base\n").expect("file");
    git(repo.as_path(), &["add", "--", "dir/child.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    // `git rm` prunes the emptied directory, so the path is free for a file.
    git(repo.as_path(), &["rm", "--quiet", "--", "dir/child.txt"]);
    std::fs::write(repo.join("dir"), b"the merge's file\n").expect("a file where the directory stood");
    git(repo.as_path(), &["add", "--", "dir"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
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

/// What:
///  A repository where the branch holds `dir/sub/child.txt` and the merge deletes it and adds a file named `dir`,
///  with the merge commit ready and nothing of it
///  in the checkout.
/// Why:
///  The tree diff of that merge names the deleted file and no entry for `dir/sub`,
/// so a check of the directory's
///  immediate children needs the whole subtree to judge it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(...); write('dir/sub/child.txt'); commit(); /* feature: rm dir/sub/child.txt, write file 'dir' */ const merge = mergeTreeCommit(...);
/// ```
fn nested_directory_to_file_world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::create_dir_all(repo.join("dir/sub")).expect("directory");
    std::fs::write(repo.join("dir/sub/child.txt"), b"child base\n").expect("file");
    std::fs::write(repo.join("h.txt"), b"h.txt base\n").expect("file");
    git(repo.as_path(), &["add", "--", "dir/sub/child.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    // `git rm` prunes the emptied directories, so the path is free for a file.
    git(repo.as_path(), &["rm", "--quiet", "--", "dir/sub/child.txt"]);
    std::fs::write(repo.join("dir"), b"the merge's file\n").expect("a file where the directory stood");
    git(repo.as_path(), &["add", "--", "dir"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
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

/// What:
///  A repository whose merge changes exactly one path,
///  `f.txt`,
///  with the merge commit ready and nothing of it in the checkout.
/// Why:
///  With one changed path,
/// every entry matching is what an independent commit of that content leaves,
/// so the
///  control needs a merge that touches one path only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(...); write('f.txt'); write('h.txt'); commit(); /* feature changes f.txt */ const merge = mergeTreeCommit(...);
/// ```
fn one_path_merge_world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("f.txt"), b"f.txt base\n").expect("file");
    std::fs::write(repo.join("h.txt"), b"h.txt base\n").expect("file");
    git(repo.as_path(), &["add", "--", "f.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
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

/// What:
///  A repository whose branch holds a symbolic link and whose merge replaces it with a regular file,
///  with the merge commit
///  ready and nothing of it in the checkout.
/// Why:
///  A link's identity is the text of its target and never the bytes of the file it points at,
/// so a control needs a
///  path that is a link on the old side and a file on the merged side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(...); write('target.txt', 'payload\n'); symlink('target.txt', 'link'); commit(); /* feature: rm link, write file 'link' */ const merge = mergeTreeCommit(...);
/// ```
fn symlink_to_file_world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("target.txt"), b"payload\n").expect("the link's target");
    std::fs::write(repo.join("h.txt"), b"h.txt base\n").expect("file");
    std::os::unix::fs::symlink("target.txt", repo.join("link")).expect("the link");
    git(repo.as_path(), &["add", "--", "target.txt", "h.txt", "link"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::remove_file(repo.join("link")).expect("the link goes");
    std::fs::write(repo.join("link"), b"link as a file\n").expect("a file where the link stood");
    git(repo.as_path(), &["add", "--", "link"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
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

/// What:
///  The record of a merge that was saved on `side_ref` because the index was locked,
///  and the side ref itself.
/// Why:
///  A merge that met a live holder leaves that state,
/// and the controls about a saved merge start there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMergeRecord({ ...intent, sideRef }); await git(['update-ref', '--no-deref', sideRef, newOid]);
/// ```
fn deferred_record_of(world: &World, id: &str, side_ref: &str) -> PathBuf {
    git(world.repo.as_path(), &["update-ref", "--no-deref", side_ref, world.new_oid.as_str()]);
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner(id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: Some(String::from(side_ref)),
        carried: None,
    };
    return write_merge_intent(world.git_dir.as_path(), &intent).expect("the record of a saved merge");
}

/// Everything about the staged entries and the worktree that another session would notice.
fn snapshot(world: &World) -> (String, String) {
    return (
        git_text(world.repo.as_path(), &["ls-files", "--stage"]),
        git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]),
    );
}

/// What:
///  What recovery did for the record at one path,
///  or nothing when it settled no record there.
/// Why:
///  A worktree can hold more than one record,
/// and recovery reads them in the directory's order,
/// so a control
///  names the record it asks about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theActionFor(outcomes, record): MergeRecoveryAction | undefined; // outcomes.find((outcome) => outcome.record === record)?.action
/// ```
fn the_action_for(outcomes: &[MergeRecoveryOutcome], record: &Path) -> Option<MergeRecoveryAction> {
    for outcome in outcomes {
        if outcome.record.as_path() == record {
            return Some(outcome.action);
        }
    }
    return None;
}

/// A merge killed before it touched anything is discarded,
/// and the checkout is as it was.
#[test]
fn a_merge_that_never_began_is_discarded() {
    let world: World = world("merge-recovery-never-began");
    std::fs::write(world.repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "u.txt"]);
    let before = snapshot(&world);
    let record: PathBuf = record_of(&world, dead_owner("11111111-0000-4000-8000-000000000001"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(
        outcomes,
        vec![MergeRecoveryOutcome {
            record: record.clone(),
            action: MergeRecoveryAction::Discarded,
            target_ref: Some(String::from("refs/heads/main")),
            new_oid: world.new_oid.clone(),
            side_ref: None,
        }]
    );
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
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::OwnerActive);
    assert!(record.exists());
    remove(world.parent.as_path());
}

/// A dead owner's record,
/// a malformed record and no record are told apart without a lock or a process.
#[test]
fn only_dead_or_unreadable_records_need_recovery() {
    let world: World = world("merge-recovery-scan");
    assert!(!merge_intents_to_recover(world.git_dir.as_path()).expect("no directory"));
    let record: PathBuf = record_of(&world, dead_owner("33333333-0000-4000-8000-000000000003"), None);
    assert!(merge_intents_to_recover(world.git_dir.as_path()).expect("dead owner"));
    std::fs::write(record.as_path(), b"{").expect("damage");
    assert!(merge_intents_to_recover(world.git_dir.as_path()).expect("malformed"));
    let error = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect_err("fails closed");
    assert!(error.0.contains("malformed") && error.0.contains(record.to_string_lossy().as_ref()), "{error}");
    assert!(record.exists(), "a record that cannot be read is kept");
    remove(world.parent.as_path());
}

/// A branch that already names the merge needs only the record removed.
/// The same record that names a side ref needs only that too:
/// a record a power loss brought back after the removal was not flushed is
///  this state,
/// and the next command must settle it as quietly as the command that ended the merge.
#[test]
fn a_landed_merge_only_loses_its_record() {
    let landed: World = world("merge-recovery-landed");
    git(landed.repo.as_path(), &["merge", "--ff-only", "--quiet", landed.new_oid.as_str()]);
    let record: PathBuf = record_of(&landed, dead_owner("44444444-0000-4000-8000-000000000004"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&landed), landed.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded);
    assert!(!record.exists());
    assert_eq!(git_text(landed.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(landed.parent.as_path());

    let saved: World = world("merge-recovery-landed-sided");
    git(saved.repo.as_path(), &["merge", "--ff-only", "--quiet", saved.new_oid.as_str()]);
    let side_ref: &str = "refs/cli-git/kept/2";
    let side_record: PathBuf = deferred_record_of(&saved, "44444444-0000-4000-8000-000000000044", side_ref);
    let saved_outcomes = recover_interrupted_merges(&place(&saved), saved.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(the_action_for(&saved_outcomes, side_record.as_path()), Some(MergeRecoveryAction::AlreadyLanded));
    assert!(!side_record.exists(), "the record is gone");
    assert_eq!(
        git_text(saved.repo.as_path(), &["for-each-ref", "--format=%(refname)", side_ref]),
        "",
        "the side ref of a landed merge is removed with its record"
    );
    assert_eq!(git_text(saved.repo.as_path(), &["rev-parse", "main"]), saved.new_oid, "the branch still names the merge");
    assert_eq!(git_text(saved.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(saved.parent.as_path());
}

/// A branch that moved to something else before the merge touched the checkout:
/// nothing to put back,
/// the record goes,
/// and
/// the merge can be computed again on the moved head.
#[test]
fn a_branch_that_moved_before_the_checkout_changed_only_loses_its_record() {
    let world: World = world("merge-recovery-moved");
    std::fs::write(world.repo.join("h.txt"), b"h.txt again\n").expect("file");
    git(world.repo.as_path(), &["commit", "--quiet", "--all", "--message=elsewhere"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let record: PathBuf = record_of(&world, dead_owner("55555555-0000-4000-8000-000000000005"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::BranchMoved);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(world.parent.as_path());
}

/// The branch moved after Git installed the merge's paths (the fast-forward lost the race for the branch between its
/// checkout and its ref update):
/// the paths go back to what the moved head holds,
/// so the checkout matches the head,
/// another
/// session's staged entry is untouched,
/// and nothing of the merge is left half applied.
#[test]
fn a_branch_that_moved_after_the_install_puts_the_merged_paths_back() {
    let world: World = world("merge-recovery-moved-after-install");
    std::fs::write(world.repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "u.txt"]);
    let staged_entry: String = git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]);
    // The winner: a commit on top of the old head that adds a file the merge does not touch, checked out.
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    // The loser: Git's two-way update from the old head to the merge, whose ref update then failed.
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    assert!(world.repo.join("dir/new.txt").exists(), "the merge was installed");
    let record: PathBuf = record_of(&world, dead_owner("77777777-0000-4000-8000-000000000007"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::BranchMoved);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the winner's");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt", "only the other session's work differs from the head");
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]), staged_entry);
    assert!(!world.repo.join("dir").exists() && world.repo.join("gone.txt").exists(), "the merge's changes are put back");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("file"), b"f.txt base\n");
    assert!(world.repo.join("z.txt").exists(), "the winner's file stays");
    remove(world.parent.as_path());
}

/// Git updated the worktree and the index and died before it moved the branch:
/// recovery moves the branch,
/// and
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
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt");
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]), staged_entry);
    assert!(!world.repo.join("gone.txt").exists());
    assert_eq!(std::fs::read(world.repo.join("dir/new.txt")).expect("added file"), b"new\n");
    remove(world.parent.as_path());
}

/// Git wrote some files and died before it wrote the index:
/// recovery finishes the files and the index.
#[test]
fn a_merge_killed_between_the_worktree_and_the_index_is_completed() {
    let world: World = world("merge-recovery-before-index");
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(world.repo.join("gone.txt")).expect("the file Git removed");
    let record: PathBuf = record_of(&world, dead_owner("77777777-0000-4000-8000-000000000007"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]), "");
    assert_eq!(std::fs::read(world.repo.join("dir/new.txt")).expect("added file"), b"new\n");
    remove(world.parent.as_path());
}

/// A path that holds neither its old nor its merged content belongs to someone else:
/// nothing is changed.
#[test]
fn a_foreign_edit_stops_the_completion() {
    let world: World = world("merge-recovery-foreign");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    std::fs::write(world.repo.join("f.txt"), b"someone else's edit\n").expect("foreign edit");
    let record: PathBuf = record_of(&world, dead_owner("88888888-0000-4000-8000-000000000008"), Some("refs/heads/main"));
    let error = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect_err("fails closed");
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
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
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
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
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
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(world.parent.as_path());
}

/// The merge's own process settles a failure the same way,
/// whatever its owner.
#[test]
fn a_failed_merge_settles_without_a_liveness_check() {
    let world: World = world("merge-recovery-settle");
    let live: TransactionOwner =
        current_transaction_owner("bbbbbbbb-0000-4000-8000-00000000000b", "2026-10-29T00:00:00.000Z").expect("owner");
    let record: PathBuf = record_of(&world, live.clone(), Some("refs/heads/main"));
    let intent: MergeIntent = parse_merge_intent(std::fs::read(record.as_path()).expect("read").as_slice()).expect("intent");
    assert_eq!(intent_path(world.git_dir.as_path(), live.transaction_id.as_str()), record);
    let action = settle_failed_merge(&place(&world), record.as_path(), &intent, &mut Vec::new()).expect("settled");
    assert_eq!(action, MergeRecoveryAction::Discarded);
    assert!(!record.exists());
    remove(world.parent.as_path());
}

/// What:
///  The starts a recovery makes while it completes a merge that wrote only some of its files.
/// Why:
///   The owner's rule is that no Git start of the wrapper gets `--quiet` or `-q`.
/// The recovery's `checkout-index` is the one
///  that carried it (Git 2.56.0 `builtin/checkout-index.c` gates only the message on it,
/// and the exit status is 1 for a
///  path that is not in the index with or without it);
/// the control looks at the start that checks out the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInterruptedMerges(place(recordingGit)); expect(startsOf('checkout-index')).toHaveLength(1); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_recovery_checks_files_out_without_quiet() {
    let world: World = world("merge-recovery-no-quiet");
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(world.repo.join("gone.txt")).expect("the file Git removed");
    let _record: PathBuf = record_of(&world, dead_owner("88888888-0000-4000-8000-000000000008"), Some("refs/heads/main"));
    let (mut recording, log) = crate::recording_git::recording_context(world.parent.as_path());
    // The checkout and the index update hand their paths on standard input, which a request makes in the common directory.
    recording.common_dir = Some(world.git_dir.clone());
    let recording_place: RecoveryPlace =
        RecoveryPlace { git: recording, cwd: world.repo.clone(), common_dir: world.git_dir.clone(), git_dir: world.git_dir.clone() };
    let outcomes = recover_interrupted_merges(&recording_place, world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    let checkouts: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.contains("checkout-index"); }).collect();
    assert_eq!(checkouts.len(), 1, "one start checks the files out: {checkouts:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(world.parent.as_path());
}

/// A record whose owner is in another PID namespace and older than the patience is neither alive nor dead (owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)"):
/// recovery stops with an error that names the record,
/// the owner,
/// the namespace,
/// the age and
/// the `rm`,
/// and changes nothing:
/// the record stays,
/// the branch stays,
/// the staged entries and the worktree stay.
#[test]
fn an_old_record_of_another_namespace_is_refused_and_named_and_nothing_changes() {
    let world: World = world("merge-recovery-foreign-expired");
    let mut owner: TransactionOwner = dead_owner("11111111-0000-4000-8000-000000000042");
    owner.owner_pid_namespace = Some(String::from("pid:[4026531999]"));
    owner.owner_pid = 31337;
    let record: PathBuf = record_of(&world, owner, Some("refs/heads/main"));
    std::fs::File::options().write(true).open(record.as_path()).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age the record");
    let before = snapshot(&world);
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect_err("an expired foreign record is refused");
    let text: String = refused.0;
    assert!(text.contains(record.to_string_lossy().as_ref()), "the record file is named: {text}");
    assert!(text.contains("31337") && text.contains("pid:[4026531999]"), "the owner and its namespace are named: {text}");
    assert!(text.contains("minutes old") && text.contains("another container") && text.contains("rm -- "), "age, the live-owner possibility and the rm: {text}");
    assert!(text.contains("left running"), "the condition covers Git processes the wrapper left running, not only a monochromatic-git-policy-cli command: {text}");
    assert!(record.exists(), "the record is untouched");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(snapshot(&world), before);
    assert!(merge_intents_to_recover(world.git_dir.as_path()).is_err(), "the check that decides whether recovery is needed refuses as well");
    remove(world.parent.as_path());
}

/// Records `[dead A, expired-foreign B]`:
/// recovery refuses before it settles A (sol-1 round 3,
/// P1 C:
/// the refusal said nothing was
/// changed after the first record had been settled).
#[test]
fn an_old_record_of_another_namespace_after_a_dead_one_stops_recovery_before_the_dead_one_is_settled() {
    let world: World = world("merge-recovery-foreign-after-dead");
    let dead: PathBuf = record_of(&world, dead_owner("00000000-0000-4000-8000-000000000001"), Some("refs/heads/main"));
    let mut owner: TransactionOwner = dead_owner("ffffffff-0000-4000-8000-000000000002");
    owner.owner_pid_namespace = Some(String::from("pid:[4026531999]"));
    owner.owner_pid = 31337;
    let foreign: PathBuf = record_of(&world, owner, Some("refs/heads/main"));
    std::fs::File::options().write(true).open(foreign.as_path()).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age the record");
    assert!(dead.file_name() < foreign.file_name(), "the dead owner's record is read first");
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect_err("an expired foreign record is refused");
    assert!(refused.0.contains("31337"), "{}", refused.0);
    assert!(dead.exists() && foreign.exists(), "nothing was settled before the refusal");
    remove(world.parent.as_path());
}

/// What:
///  A merge that recovery finishes tells the person so,
///  naming the ref and the commit,
///  and a merge it discards tells them nothing.
/// Why:
///  A command run only to look at the worktree must not move the branch onto a merge commit in silence
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 1);
/// the notice is the one record of that change outside the debug log,
/// and it goes to standard error alone,
/// because scripts read the command's standard output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = []; await recoverInterruptedMerges(place, gitDir, notices); expect(notices).toEqual([line]); expect(stdout).toBe('');
/// ```
#[test]
fn a_completed_merge_is_reported_with_the_ref_and_the_commit() {
    let world: World = world("merge-recovery-reported");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let _record: PathBuf = record_of(&world, dead_owner("dddddddd-0000-4000-8000-00000000000d"), Some("refs/heads/main"));
    let mut notices: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut notices).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    let said: String = String::from_utf8(notices).expect("the notices are UTF-8");
    assert_eq!(said.lines().count(), 1, "one line for the one merge that changed the branch: {said}");
    assert!(said.contains("refs/heads/main"), "the ref is named: {said}");
    assert!(said.contains(world.new_oid.as_str()), "the merge commit is named: {said}");
    assert!(said.starts_with("monochromatic-git-policy-cli: "), "the line is the wrapper's: {said}");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    remove(world.parent.as_path());
}

/// A record that is only discarded,
/// a live owner's record and a branch that already names the merge change nothing of the branch,
/// so they say nothing.
#[test]
fn a_merge_that_changed_nothing_is_not_reported() {
    let discarded: World = world("merge-recovery-unreported");
    std::fs::write(discarded.repo.join("f.txt"), b"an edit made while the merge was dead\n").expect("edit");
    let _record: PathBuf = record_of(&discarded, dead_owner("eeeeeeee-0000-4000-8000-00000000000e"), Some("refs/heads/main"));
    let mut notices: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&discarded), discarded.git_dir.as_path(), &mut notices).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Discarded);
    assert_eq!(String::from_utf8_lossy(&notices), "", "a discarded merge was never in the checkout");
    remove(discarded.parent.as_path());

    let landed: World = world("merge-recovery-unreported-landed");
    git(landed.repo.as_path(), &["merge", "--ff-only", "--quiet", landed.new_oid.as_str()]);
    let _landed_record: PathBuf = record_of(&landed, dead_owner("ffffffff-0000-4000-8000-00000000000f"), Some("refs/heads/main"));
    let mut landed_notices: Vec<u8> = Vec::new();
    let landed_outcomes = recover_interrupted_merges(&place(&landed), landed.git_dir.as_path(), &mut landed_notices).expect("recovery");
    assert_eq!(landed_outcomes[0].action, MergeRecoveryAction::AlreadyLanded);
    assert_eq!(String::from_utf8_lossy(&landed_notices), "", "a branch that already names the merge did not move");
    remove(landed.parent.as_path());
}

/// What:
///  A writer that lets another program's `index.lock` go the first time the wait says what it waits for,
///  and keeps everything the wait wrote.
/// Why:
///  The control must not sleep until a lock is released:
/// the notice of the wait is the instant the holder is known to be
///  waited for (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReleasingSink { write(bytes) { this.said += bytes; if (exists(lock)) rm(lock); return bytes.length; } }
/// ```
struct ReleasingSink<'a> {
    /// The lock the other program holds.
    lock: &'a Path,
    /// Everything the wait wrote.
    said: Vec<u8>,
}

impl std::io::Write for ReleasingSink<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.said.extend_from_slice(bytes);
        // A wait that has said what it waits for is waiting; the other program lets go at that instant.
        if self.lock.exists() {
            std::fs::remove_file(self.lock)?;
        }
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A recovery waits for another program's `index.lock`,
///  says what it waits for,
///  and finishes the merge when the lock is free.
/// Why:
///  `git update-index` dies on a held lock,
/// so one held lock used to end every command in the worktree with
///  status 2 until it was gone (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sink = new ReleasingSink(lock); await recoverInterruptedMerges(place, dir, sink); expect(outcome.action).toBe('completed');
/// ```
#[test]
fn recovery_waits_for_another_programs_index_lock() {
    let world: World = world("merge-recovery-waits-for-lock");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let _record: PathBuf = record_of(&world, dead_owner("12121212-0000-4000-8000-000000000012"), Some("refs/heads/main"));
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("another program holds the index");
    let mut sink: ReleasingSink<'_> = ReleasingSink { lock: lock.as_path(), said: Vec::new() };
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut sink).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed, "the merge is finished once the lock is free");
    let said: String = String::from_utf8(sink.said).expect("the notices are UTF-8");
    assert!(said.contains(lock.to_string_lossy().as_ref()), "the lock is named: {said}");
    assert!(said.contains("waits for the index lock"), "it says what it waits for: {said}");
    assert!(!lock.exists(), "the lock was the other program's and nothing removed it on the wrapper's side");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    remove(world.parent.as_path());
}

/// What:
///  A wait that ends with the index still held by a program that lives writes nothing,
/// a free or released lock lets the merge go on,
///  and a lock a gone process left is removed and then lets it go on.
/// Why:
///  The decision is the whole of recovery's behaviour on a held lock,
/// and every half of it is checked:
///  a recovery that wrote the index after giving up would put Git's own sentence in place of the record,
/// while one that
///  treated a dead Git's lock as a live holder's left every command in the worktree exiting 2
///  (finding 4 of the merge-and-pull audit of 2026-11-09).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(mayInstall(place, { kind: 'not-held' })).toBe(true);
/// expect(mayInstall(place, { kind: 'gave-up' })).toBe(false);
/// expect(mayInstall(place, { kind: 'owner-gone' })).toBe(true); expect(exists(lock)).toBe(false);
/// ```
#[test]
fn only_a_lock_that_is_free_or_left_by_a_gone_process_lets_the_merge_be_written() {
    let second: std::time::Duration = std::time::Duration::from_secs(1);
    let world: World = world("merge-recovery-may-install");
    assert!(may_install(world.git_dir.as_path(), IndexWait::NotHeld));
    assert!(may_install(world.git_dir.as_path(), IndexWait::Released(second)));
    assert!(!may_install(world.git_dir.as_path(), IndexWait::GaveUp(second)));

    // A lock whose PID file names a process that no longer runs is nobody's to release, so it is removed.
    let dead: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(dead.as_path(), b"").expect("a killed Git left its empty lock");
    std::fs::write(crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()), b"pid 4294967\n")
        .expect("the PID file of a process that is gone");
    assert!(may_install(world.git_dir.as_path(), IndexWait::OwnerGone(second)), "a dead Git's lock is cleared and the merge goes on");
    assert!(!dead.exists(), "the lock of a Git nothing runs was removed");

    // A lock a program that lives holds is not the recovery's to remove.
    std::fs::write(dead.as_path(), b"").expect("a program holds the index");
    std::fs::write(crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()), format!("pid {}\n", std::process::id()))
        .expect("the PID file names this process, which runs");
    assert!(!may_install(world.git_dir.as_path(), IndexWait::OwnerGone(second)), "a live holder's lock stays");
    assert!(dead.exists(), "the live holder's lock is not removed");
    remove(world.parent.as_path());
}

/// What:
///  A held index stops a merge that is already installed and leaves a merge that installed nothing for the next
///  command.
/// Why:
///  The control proves both ends of the rule:
/// a merge whose files and entries are installed must not be read
///  as another session's staged work,
/// while one that installed nothing is left for the next command.
///  The holder lives here,
/// because a lock a gone process left is cleared and the merge finished
///  (`recovery_clears_the_index_lock_of_a_git_that_was_killed_and_finishes_the_merge`),
/// and the ceiling is shortened so the
///  control ends in a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shortCeiling(ms(100)); await expect(settleFailedMerge(installed)).rejects; await expect(settleFailedMerge(saved)).resolves.toBe('discarded');
/// ```
#[test]
fn a_held_index_defers_a_merge_that_installed_nothing_and_stops_an_installed_one() {
    // A merge that was already installed is not deferred: a command that went on would read it as staged work, so it stops.
    let installed: World = world("merge-recovery-dead-holder-lock-installed");
    git(installed.repo.as_path(), &["read-tree", "-m", "-u", installed.old_oid.as_str(), installed.new_oid.as_str()]);
    let installed_record: PathBuf = record_of(&installed, dead_owner("13131313-0000-4000-8000-000000000013"), Some("refs/heads/main"));
    let installed_intent: MergeIntent =
        parse_merge_intent(std::fs::read(installed_record.as_path()).expect("read").as_slice()).expect("intent");
    std::fs::write(installed.git_dir.join("index.lock"), b"").expect("a Git left the index lock");
    // The holder is this process, which runs, so nothing may remove the lock and the wait ends at its ceiling.
    std::fs::write(
        crate::recovery_files::lock_pid_path(installed.git_dir.join("index").as_path()),
        format!("pid {}\n", std::process::id()),
    )
    .expect("the PID file of a process that runs");
    let mut installed_said: Vec<u8> = Vec::new();
    let settled = crate::index_holder_wait::RECOVERY_LOCK_CEILING_FOR_THIS_THREAD.with(|shortened| {
        shortened.set(Some(std::time::Duration::from_millis(100)));
        return settle_failed_merge(&place(&installed), installed_record.as_path(), &installed_intent, &mut installed_said);
    });
    let refused: MergeRefusal = settled.expect_err("an installed merge behind a held index stops the command");
    assert!(refused.began, "it reports that the merge is installed");
    assert!(installed_record.exists(), "the record stays");
    remove(installed.parent.as_path());

    // A merge that was saved and installed nothing is deferred: the next command finishes it.
    let world: World = world("merge-recovery-dead-holder-lock");
    let before = snapshot(&world);
    let record: PathBuf = record_of(&world, dead_owner("13131313-0000-4000-8000-000000000014"), Some("refs/heads/main"));
    let intent: MergeIntent = parse_merge_intent(std::fs::read(record.as_path()).expect("read").as_slice()).expect("intent");
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("a Git left the index lock");
    std::fs::write(crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()), b"pid 4294967\n").expect("the PID file of a process that is gone");
    let mut said: Vec<u8> = Vec::new();
    let action = settle_failed_merge(&place(&world), record.as_path(), &intent, &mut said).expect("settled");
    assert_eq!(action, MergeRecoveryAction::Discarded, "nothing of it was installed, so the record only goes");
    assert!(!record.exists(), "the record goes");
    assert!(lock.exists(), "the lock is not the recovery's to remove");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid, "the branch did not move");
    assert_eq!(snapshot(&world), before, "the index and the worktree are exactly as they were");
    remove(world.parent.as_path());
}

/// What:
///  A lock a Git that no longer runs left over a merge it had partly installed is cleared on the same proof the
///  merge's own path acts on,
///  and the command finishes the merge instead of exiting 2.
/// Why:
///  An out-of-memory kill of the cgroup,
///  or `kill -9` of the process group,
///  leaves the merge's record,
///  its installed checkout and the dead Git's `index.lock`:
/// `git status`,
///  `git log`,
/// `git rev-parse HEAD`,
/// `git merge` and `git commit` all exited 2 until the person deleted the lock by hand,
///  where real Git answers `git status` with 0 in the same state
///  (finding 4 of the merge-and-pull audit).
/// The merge's own path already removes such a lock (`merge_isolation_apply.rs`),
/// and recovery is the same
///  decision from the same evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeLock(index, { pid: gonePid }); await expect(recoverInterruptedMerges(place, dir)).resolves.toMatchObject([{ action: 'completed' }]); expect(exists(lock)).toBe(false);
/// ```
#[test]
fn recovery_clears_the_index_lock_of_a_git_that_was_killed_and_finishes_the_merge() {
    let world: World = world("merge-recovery-dead-git-lock");
    // Git had updated the checkout and had not moved the branch when it was killed with the lock.
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = record_of(&world, dead_owner("15151515-0000-4000-8000-000000000015"), Some("refs/heads/main"));
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("a killed Git left its empty lock");
    std::fs::write(crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()), b"pid 4294967\n")
        .expect("the PID file of a process that is gone");
    let mut said: Vec<u8> = Vec::new();
    let outcomes =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the merge is finished, not refused");
    assert_eq!(the_action_for(&outcomes, record.as_path()), Some(MergeRecoveryAction::Completed));
    assert!(!lock.exists(), "the lock of a Git nothing runs was removed");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid, "the branch moved onto the merge");
    remove(world.parent.as_path());
}

/// What:
///  A merge saved on its side ref,
///  with nothing of it in the checkout and a live program holding `index.lock`,
///  is left for the next command at once,
///  which says where the merge is and what it waits for.
/// Why:
///  The merge itself saves its result and exits 0 (`MERGE_LOCK_CEILING`,
///  one minute);
/// after that every command,
/// `git rev-parse HEAD` included,
/// waited the whole ceiling again for as long as
///  the lock stood,
/// where real Git answers each of them in a few milliseconds
///  (finding 3 of the merge-and-pull audit of 2026-11-09).
/// Nothing of the merge is in the checkout,
///  so the next command has nothing to wait for:
/// the merge is on its record and its side ref,
/// and a later
///  command finishes it once the lock is free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = now(); const outcomes = await recoverInterruptedMerges(place, dir, said);
/// expect(now() - began).toBeLessThan(seconds(2)); expect(outcomes[0].action).toBe('index-held');
/// ```
#[test]
fn a_saved_merge_behind_a_live_lock_is_left_for_the_next_command_at_once() {
    let world: World = world("merge-recovery-saved-live-lock");
    let side_ref: &str = "refs/cli-git/kept/1";
    let record: PathBuf = deferred_record_of(&world, "16161616-0000-4000-8000-000000000016", side_ref);
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("another program holds the index");
    // The holder is this process, which runs: nothing may remove the lock, and waiting gains nothing.
    std::fs::write(crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()), format!("pid {}\n", std::process::id()))
        .expect("the PID file of a process that runs");
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let began: std::time::Instant = std::time::Instant::now();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    let took: std::time::Duration = began.elapsed();
    assert_eq!(the_action_for(&outcomes, record.as_path()), Some(MergeRecoveryAction::IndexHeld));
    assert!(took < std::time::Duration::from_secs(2), "the command does not wait for a lock that stands over a merge nothing of which is installed: {took:?}");
    assert!(record.exists(), "the record stays for the command that finishes the merge");
    assert_eq!(snapshot(&world), before, "nothing was written");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains(side_ref), "the line says where the merge is saved: {text}");
    assert!(text.contains(lock.to_string_lossy().as_ref()), "the line names what it waits for: {text}");
    assert!(text.contains("goes on"), "the line says this command goes on: {text}");
    remove(world.parent.as_path());
}

/// What:
///  Judge every changed path of the world's merge the way recovery does,
///  and return the changes with their judgements.
/// Why:
///  A control about a file that changes after it was judged needs the judgement and the write separated,
/// and that
///  separation is the very instant the finding is about;
/// no sleep and no second process is needed,
/// because the judgement is
///  the value the control hands on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judge(world) { return mergeChanges(place, root, intent).map((change) => ({ change, judged: { standing: standing(change, stateOf(change)), identity: identityOf(change) } })); }
/// ```
fn judged_changes(world: &World) -> (Vec<Change>, Vec<Judged>) {
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner("14141414-0000-4000-8000-000000000014"),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: None,
        carried: None,
    };
    let place: RecoveryPlace = place(world);
    let changes: Vec<Change> = merge_changes(&place, world.repo.as_path(), &intent).expect("the merge's changes");
    let records_file_modes: bool = crate::merge_isolation_changes::the_repository_records_file_modes(&place.git, world.repo.as_path());
    let mut judgements: Vec<Judged> = Vec::with_capacity(changes.len());
    for change in &changes {
        // The identity first, as recovery reads it, then the content: exactly the order of `settle_failed_merge`.
        let identity = file_identity(worktree_path(world.repo.as_path(), change.path.as_slice()).as_path());
        let state: WorktreeState = worktree_state(&place.git, world.repo.as_path(), change.path.as_slice()).expect("the state");
        let content: Option<String> = match &state {
            WorktreeState::Content { object, .. } => Some(object.clone()),
            WorktreeState::Missing | WorktreeState::Other => None,
        };
        judgements.push(Judged { standing: standing(change, &state, records_file_modes), identity, content });
    }
    return (changes, judgements);
}

/// The identifier a control's own take names itself by,
/// which no record of a control's fixture claims unless the control writes one.
const A_CONTROLS_MERGE: &str = "17171717-0000-4000-8000-000000000017";

/// The names in `repository` that are the private names a take leaves.
fn aside_names(repository: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let listing = std::fs::read_dir(repository).expect("the worktree root");
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(".cli-git-recovery-") {
            found.push(name);
        }
    }
    return found;
}

/// What:
///  A file saved after recovery judged it is never replaced by the merge.
/// Why:
///  The judgement and the write of a merged file were separate steps,
/// so an editor that saved in between lost the
///  edit with no notice (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 4).
/// The take checks
///  what it took against the file system identity the judgement read,
/// so the edit stays and the merge stops,
/// naming
///  the path and keeping the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); write('f.txt', edit); await expect(installEntries(place, root, changes, judged, sink)).rejects.toMatch(/f.txt/); expect(read('f.txt')).toEqual(edit);
/// ```
#[test]
fn a_file_saved_after_the_judgement_is_not_replaced() {
    let world: World = world("merge-recovery-edit-after-judging");
    // Git had not written this path yet: the checkout holds the old content, so the merge writes it.
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let before = snapshot(&world);
    // The person's editor saves one of the merge's paths after the judgement.
    std::fs::write(world.repo.join("f.txt"), b"an edit saved after the judgement\n").expect("the edit");
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said)
        .expect_err("a file that changed after the judgement is not the merge's to replace");
    assert!(refused.error.0.contains("f.txt"), "the path is named: {}", refused.error.0);
    assert!(refused.error.0.contains("changed after this command judged them"), "the reason is stated: {}", refused.error.0);
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("the edit"), b"an edit saved after the judgement\n", "the edit is kept");
    let after = snapshot(&world);
    assert_eq!(after.0, before.0, "the index is exactly as it was");
    assert_eq!(after.1, " M f.txt", "the checkout is exactly as it was, and the merge is still unfinished");
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing was left at a private name");
    remove(world.parent.as_path());
}

/// What:
///  A file that appeared where the merge adds one is refused,
///  put back,
///  and never replaced.
/// Why:
///  The judged state there was `Missing`,
/// so any file is someone else's;
/// without the take the merge would have written over it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm('dir/new.txt'); const (changes, judged) = judge(world); write('dir/new.txt', mine); await expect(installEntries(...)).rejects.toMatch(/dir\/new.txt/); expect(read('dir/new.txt')).toEqual(mine);
/// ```
#[test]
fn a_file_created_after_the_judgement_is_not_replaced() {
    let world: World = world("merge-recovery-create-after-judging");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    // The merge adds `dir/new.txt`; it is gone when the paths are judged, so its judged state is `Missing`.
    std::fs::remove_file(world.repo.join("dir/new.txt")).expect("the merge's file goes");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    std::fs::write(world.repo.join("dir/new.txt"), b"a file of my own\n").expect("the person's file");
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said)
        .expect_err("a file that appeared where the merge adds one is not the merge's to replace");
    assert!(refused.error.0.contains("dir/new.txt"), "the path is named: {}", refused.error.0);
    assert_eq!(std::fs::read(world.repo.join("dir/new.txt")).expect("the person's file"), b"a file of my own\n", "the file is put back");
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing was left at a private name");
    remove(world.parent.as_path());
}

/// What:
///  The name a killed take leaves a file under,
///  built the way recovery builds it,
///  so a control can make that state without the race.
/// Why:
///  The control reproduces the instant after the rename of a take,
/// which no seam of the recovery reaches otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stranded = `${dirname}/.cli-git-recovery-${basename}-${process.pid}-0-1`;
/// ```
fn stranded_name(location: &Path, transaction_id: &str) -> PathBuf {
    return crate::merge_isolation_take_aside_names::aside_name(location, transaction_id);
}

/// What:
///  A file a killed take set aside is put back by the next command when its path is free,
///  and the command says so.
/// Why:
///  A kill between the rename of a take and the write that follows leaves the file under a private name and its
///  path missing,
/// so recovery judged the path as someone else's and stopped every command,
/// while the person's file sat
///  where nothing looks
/// (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 4's take,
/// round
///  two).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename('f.txt', stranded); await recoverInterruptedMerges(place, dir, sink); expect(sink).toContain('put back'); expect(exists(stranded)).toBe(false);
/// ```
#[test]
fn a_file_a_killed_take_set_aside_is_put_back_and_reported() {
    let world: World = world("merge-recovery-stranded-take");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let location: PathBuf = world.repo.join("f.txt");
    let stranded: PathBuf = stranded_name(location.as_path(), "15151515-0000-4000-8000-000000000015");
    let content: Vec<u8> = std::fs::read(location.as_path()).expect("the file Git wrote");
    std::fs::rename(location.as_path(), stranded.as_path()).expect("a killed take");
    let record: PathBuf = record_of(&world, dead_owner("15151515-0000-4000-8000-000000000015"), Some("refs/heads/main"));
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed, "the merge is finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("put back") && text.contains("f.txt"), "the putting back is said: {text}");
    assert_eq!(std::fs::read(location.as_path()).expect("the file is back"), content, "the bytes are the ones that were set aside");
    assert!(!stranded.exists(), "the private name is gone");
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    remove(world.parent.as_path());
}

/// What:
///  A file a killed take set aside is kept beside a path that now holds something else,
///  and the private name is said.
/// Why:
///  Nothing of the person's may be removed,
/// so both files stay and the person is told where the second one is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename('f.txt', stranded); write('f.txt', mine); const error = await recover(...); expect(error).toMatch(/f.txt/); expect(sink).toContain(stranded); expect(exists(stranded)).toBe(true);
/// ```
#[test]
fn a_file_a_killed_take_set_aside_is_kept_beside_a_path_that_holds_something_else() {
    let world: World = world("merge-recovery-stranded-take-both");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let location: PathBuf = world.repo.join("f.txt");
    let stranded: PathBuf = stranded_name(location.as_path(), "16161616-0000-4000-8000-000000000016");
    std::fs::rename(location.as_path(), stranded.as_path()).expect("a killed take");
    std::fs::write(location.as_path(), b"my own file\n").expect("the person's file");
    let _record: PathBuf = record_of(&world, dead_owner("16161616-0000-4000-8000-000000000016"), Some("refs/heads/main"));
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("the path holds someone else's content");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("f.txt") && text.contains(stranded.to_string_lossy().as_ref()), "the private name is said: {text}");
    assert!(refused.0.contains("f.txt"), "{}", refused.0);
    assert_eq!(std::fs::read(location.as_path()).expect("the person's file"), b"my own file\n", "nothing was removed");
    assert!(stranded.exists(), "the file that was set aside is kept");
    remove(world.parent.as_path());
}

/// What:
///  A private name no record claims is never put back on a guess,
///  and a copy whose bytes Git holds is removed and said so,
///  while one a record claims is put back.
/// Why:
///  A merge that ended keeps no record,
/// so a file it left beside a path belongs to no checkout state at all;
/// putting it back when a later
///  merge found the path free brought back a file the person had deleted,
/// and the later merge carried that older version
///  forward
///  (a cross-vendor review of the second round,
/// its third finding).
///  A copy the merge's own commit holds the bytes of is the second copy of them,
/// so it
///  goes and the line names the commit,
/// and the path the person emptied stays empty.
/// The claimed file is the control:
/// the same step still puts back what a live record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// record(deadOwner('2020...')); rename('f.txt', asideOf('f.txt', '2020...')); rename('gone.txt', asideOf('gone.txt', '2121...')); await putBackStrandedTakes(...); expect(exists(asideOf('gone.txt'))).toBe(false); expect(exists('gone.txt')).toBe(false); expect(sink).toContain('removed');
/// ```
#[test]
fn a_private_name_no_record_claims_is_never_put_back_on_a_guess() {
    let world: World = world("merge-recovery-aside-without-record");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    // One merge is still unfinished, and its own take waits beside `f.txt`.
    let record: PathBuf = record_of(&world, dead_owner("20202020-0000-4000-8000-000000000020"), Some("refs/heads/main"));
    let claimed_location: PathBuf = world.repo.join("f.txt");
    let claimed: PathBuf = stranded_name(claimed_location.as_path(), "20202020-0000-4000-8000-000000000020");
    std::fs::rename(claimed_location.as_path(), claimed.as_path()).expect("a take of the record's merge");
    // A merge that ended left this one, and no record names it any more.
    let ended_location: PathBuf = world.repo.join("gone.txt");
    let ended: PathBuf = stranded_name(ended_location.as_path(), "21212121-0000-4000-8000-000000000021");
    std::fs::rename(ended_location.as_path(), ended.as_path()).expect("a take of a merge that ended");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        &mut said,
    ))
    .expect("the runtime of the controls");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(!claimed.exists(), "a file a record claims loses its private name");
    assert!(claimed_location.exists(), "the claimed path holds its file again");
    assert!(!ended.exists(), "a copy whose bytes the merge's commit holds is removed");
    assert!(!ended_location.exists(), "the path a person emptied stays empty");
    assert!(text.contains(ended.to_string_lossy().as_ref()), "the copy that went is named: {text}");
    assert!(text.contains("git restore"), "the line says how to reach the bytes: {text}");
    assert!(record.exists(), "the record is not the putting back's to remove");
    remove(world.parent.as_path());
}

/// What:
///  A path the merge deletes that someone fills after it was judged is not deleted.
/// Why:
///  The take removes the judged file by its rename,
/// so the later removal has nothing to do with the merge and would
///  delete whatever appeared there
/// (a cross-vendor review's finding on the round-one take).
/// The judged state here is `Missing`
///  because the path is free when the judgements are made,
/// which is the seam for a file that appears later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); write('gone.txt', mine); await installEntries(place, root, changes, judged, sink); expect(read('gone.txt')).toEqual(mine);
/// ```
#[test]
fn a_file_that_appears_where_the_merge_deletes_is_not_deleted() {
    let world: World = world("merge-recovery-delete-race");
    // `gone.txt` is a path the merge deletes; it is free when the judgements are made.
    std::fs::remove_file(world.repo.join("gone.txt")).expect("the judged state is `Missing`");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    std::fs::write(world.repo.join("gone.txt"), b"a file made after the judgement\n").expect("the person's file");
    let mut said: Vec<u8> = Vec::new();
    let installed = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said);
    assert!(installed.is_ok(), "{installed:?}");
    assert_eq!(std::fs::read(world.repo.join("gone.txt")).expect("the person's file"), b"a file made after the judgement\n", "a file that appeared is never deleted");
    remove(world.parent.as_path());
}

/// What:
///  A file written after it was taken aside is kept rather than dropped with the private name,
///  and the private name is said.
/// Why:
///  A program that already had the file open writes into the inode the take renamed,
/// after the check,
/// so
///  dropping the aside would lose that edit when the writer closes
/// (a cross-vendor review's finding on the round-one take).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const take = takeAside(file); write(aside, edit); const left = finishAsides([take], sink); expect(exists(aside)).toBe(true); expect(sink).toContain(aside);
/// ```
#[test]
fn a_file_written_after_it_was_taken_is_kept_and_named() {
    let world: World = world("merge-recovery-written-after-take");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    let expected: String = git_text(world.repo.as_path(), &["hash-object", "--", "f.txt"]).trim().to_owned();
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    // The program that had the file open writes its edit into the inode the take holds.
    std::fs::write(aside.as_path(), b"an edit written after the check\n").expect("the writer's edit");
    let mut said: Vec<u8> = Vec::new();
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected: expected.clone(),
    }];
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said).expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("f.txt") && text.contains(aside.to_string_lossy().as_ref()), "the private name is said: {text}");
    assert!(aside.exists(), "the file written after the check is kept");
    assert_eq!(std::fs::read(aside.as_path()).expect("the writer's edit"), b"an edit written after the check\n", "the edit is not lost");
    remove(world.parent.as_path());
}

/// What:
///  One change the merge does not have,
///  for a path this control made,
///  with the object it should end as.
/// Why:
///  The seam a take that fails needs is a path whose directory refuses the rename,
/// and the merge's own paths are all in
///  the worktree root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const extra = { path: Buffer.from('sealed/zeta.txt'), oldMode: '100644', newMode: '100644', oldOid: old, newOid: next };
/// ```
fn extra_change(path: &str, old_mode: &str, new_mode: &str, old_oid: &str, new_oid: &str) -> Change {
    return Change {
        path: path.as_bytes().to_vec(),
        old_mode: String::from(old_mode),
        new_mode: String::from(new_mode),
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
    };
}

/// What:
///  A take that fails after another file was taken puts that file back.
/// Why:
///  The take pass returned on the error of one path and left every earlier one under its private name,
/// where the
///  next command refuses the path and nothing looks
/// (a cross-vendor review's finding on the round-one take).
/// The failing take is a path in a directory that refuses the rename,
/// which this control skips where it runs as
///  root and permissions do not apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); changes.push(extra('sealed/zeta.txt')); chmod('sealed', 0o555); await expect(installEntries(...)).rejects; expect(read('f.txt')).toEqual(old); expect(asides()).toEqual([]);
/// ```
#[cfg(unix)]
#[test]
fn a_take_that_fails_puts_the_files_it_already_took_back() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipped: a directory that refuses writes does not stop root, so this control proves nothing here");
        return;
    }
    let world: World = world("merge-recovery-take-error");
    let (mut changes, mut judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let old: Vec<u8> = std::fs::read(world.repo.join("f.txt")).expect("the old content");
    // A path of this control's own, in a directory that refuses the rename into it; it is judged last, so the merge's own
    //  files are taken before it.
    std::fs::create_dir(world.repo.join("sealed")).expect("the sealed directory");
    std::fs::write(world.repo.join("sealed/zeta.txt"), b"old zeta\n").expect("the file");
    let identity: Option<crate::merge_isolation_changes::FileIdentity> = file_identity(world.repo.join("sealed/zeta.txt").as_path());
    let old_oid: String = git_text(world.repo.as_path(), &["hash-object", "--", "sealed/zeta.txt"]).trim().to_owned();
    changes.push(extra_change("sealed/zeta.txt", "100644", "100644", old_oid.as_str(), old_oid.as_str()));
    judgements.push(Judged { standing: Standing::Old, identity, content: Some(old_oid.clone()) });
    std::fs::set_permissions(world.repo.join("sealed"), std::fs::Permissions::from_mode(0o555)).expect("a directory that refuses writes");
    let mut said: Vec<u8> = Vec::new();
    let failed = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said)
        .expect_err("the take of a path in a read-only directory fails");
    let _writable = std::fs::set_permissions(world.repo.join("sealed"), std::fs::Permissions::from_mode(0o755));
    assert!(!failed.error.0.is_empty(), "the failure is named");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("the file"), old, "the file taken before the failure is put back");
    assert_eq!(std::fs::read(world.repo.join("sealed/zeta.txt")).expect("the file"), b"old zeta\n", "the file that failed is where it was");
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing is left at a private name");
    remove(world.parent.as_path());
}

/// What:
///  A write of the index that Git refuses puts back every file that was taken aside.
/// Why:
///  The index write and the checkout both come after the take,
/// and their error arms deleted the originals instead of
///  restoring them,
/// leaving a file missing that the next command then refuses
///  (a cross-vendor review's finding on the round-one take).
/// The index write is made to fail by an object id no repository holds,
/// which needs no permission to arrange.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); changes.push(extra('zeta.txt', old, '1234...')); await expect(installEntries(...)).rejects; expect(read('f.txt')).toEqual(old); expect(asides()).toEqual([]);
/// ```
#[test]
fn a_write_that_fails_puts_the_files_it_already_took_back() {
    let world: World = world("merge-recovery-write-error");
    let (mut changes, mut judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    std::fs::write(world.repo.join("zeta.txt"), b"old zeta\n").expect("the file");
    let identity: Option<crate::merge_isolation_changes::FileIdentity> = file_identity(world.repo.join("zeta.txt").as_path());
    let old_oid: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", "zeta.txt"]).trim().to_owned();
    // An object no repository holds: the index write and the checkout both refuse it.
    let absent: String = String::from("1234567890123456789012345678901234567890");
    // The failing path is written first, so the checkout stops before it writes any file the merge changes.
    changes.insert(0, extra_change("zeta.txt", "100644", "100644", old_oid.as_str(), absent.as_str()));
    judgements.insert(0, Judged { standing: Standing::Old, identity, content: Some(old_oid.clone()) });
    let mut said: Vec<u8> = Vec::new();
    let failed = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said)
        .expect_err("an object the repository does not hold stops the write");
    assert!(!failed.error.0.is_empty(), "the failure is named");
    // The path whose write Git refused holds its old content again; the files Git did write hold the merge's.
    assert_eq!(std::fs::read(world.repo.join("zeta.txt")).expect("the file Git did not write"), b"old zeta\n", "the file is put back where it was");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("the file Git wrote"), b"f.txt feature\n", "what Git wrote stays written");
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing is left at a private name");
    remove(world.parent.as_path());
}

/// What:
///  A file whose permissions changed after it was judged is not replaced.
/// Why:
///  The identity the take compares holds the device,
/// the inode,
/// the size and the modification time,
/// none of which a
///  `chmod` changes,
/// so an executable bit the person set was lost without a word
///  (a cross-vendor review's finding on the round-one take).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); chmod('f.txt', 0o755); await expect(installEntries(...)).rejects.toMatch(/f.txt/); expect(mode('f.txt')).toEqual(0o755);
/// ```
#[cfg(unix)]
#[test]
fn a_file_whose_mode_changed_after_the_judgement_is_not_replaced() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = world("merge-recovery-mode-after-judging");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let location: PathBuf = world.repo.join("f.txt");
    std::fs::set_permissions(location.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the person's executable bit");
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said)
        .expect_err("a path whose mode changed after the judgement is not the merge's to replace");
    assert!(refused.error.0.contains("f.txt"), "{}", refused.error.0);
    let mode: u32 = std::fs::metadata(location.as_path()).expect("the file").permissions().mode() & 0o777;
    assert_eq!(mode, 0o755, "the person's executable bit is kept");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose branch advanced is kept and named,
///  and the command that finds it goes on.
/// Why:
///  The holder of the index commits an unrelated file,
/// so the branch is no longer the commit the merge was computed on;
///  the merge cannot land as it was saved,
/// and the command that reported it saved had already exited 0
///  (`SPEC.md:2187`).
/// Settling it as an ordinary moved-branch record dropped the record and the side ref,
/// so the
///  acknowledged merge vanished and its commit became collectable
/// (a cross-vendor review's finding on the second round).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(world, 'refs/cli-git/kept/7'); commitUnrelated(); const outcomes = await recover(...); expect(sink).toContain('refs/cli-git/kept/7'); expect(await revParse(side)).toEqual(mergeOid); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_saved_merge_whose_branch_moved_is_kept_and_named() {
    let world: World = world("merge-recovery-saved-branch-moved");
    let record: PathBuf = deferred_record_of(&world, "17171717-0000-4000-8000-000000000017", "refs/cli-git/kept/7");
    // The holder of the index commits an unrelated file: the branch is no longer the merge's first parent.
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("refs/cli-git/kept/7"), "where the merge is kept is said: {text}");
    assert!(text.contains(world.new_oid.as_str()), "the merge commit is named: {text}");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/7"]), world.new_oid, "the saved merge is kept");
    assert!(!record.exists(), "the record is gone, so nothing stops the next command");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the holder's");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::SavedBranchMoved);
    remove(world.parent.as_path());
}

/// What:
///  A staged edit of a path the saved merge replaces stops the finish,
///  and the staged content is still in the index.
/// Why:
///  The judgement reads the worktree,
/// where the branch's content stands,
/// so a version staged only in the index was
///  overwritten with the merge's without a warning or a copy
/// (a cross-vendor review's finding on the second round).
/// The staged version has no other name,
/// so the finish must stop and name the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', C); git add f.txt; write('f.txt', A); await expect(recover(...)).rejects.toMatch(/f.txt/); expect(indexOf('f.txt')).toEqual(C);
/// ```
#[test]
fn a_staged_edit_of_a_merge_path_stops_the_finish_and_is_kept() {
    let world: World = world("merge-recovery-staged-only");
    // Git installed the merge and died before the branch moved; another program then staged its own version of `f.txt`
    //  over the merge's entry, and the worktree was left as the merge wrote it.
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let _record: PathBuf = record_of(&world, dead_owner("18181818-0000-4000-8000-000000000018"), Some("refs/heads/main"));
    std::fs::write(world.repo.join("f.txt"), b"f.txt staged by someone else\n").expect("the staged version");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    // The worktree goes back to what the merge wrote there, so only the index holds the person's version.
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the worktree as the merge left it");
    let staged_entry: String = git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "f.txt"]);
    assert!(staged_entry.contains("f.txt"), "the index holds the staged entry under test: {staged_entry}");
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("a staged edit of a merge path stops the finish");
    assert!(refused.0.contains("f.txt"), "the path is named: {}", refused.0);
    assert!(refused.0.contains("index"), "it says the index holds it: {}", refused.0);
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "f.txt"]), staged_entry, "the staged version is still in the index");
    assert_eq!(snapshot(&world), before, "nothing of the merge was written");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid, "the branch did not move");
    remove(world.parent.as_path());
}

/// What:
///  A directory the merge replaces with a file does not stop the finish,
///  because the directory holds the branch's own files that the merge deletes.
/// Why:
///  `worktree_state` reads a directory as neither the old nor the new content,
/// so the saved merge refused at every
///  later command,
/// and restoring the path from the branch recreated the same directory
/// (a cross-vendor review's finding on the
///  second round).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFile(); await recover(...); expect(read('dir')).toEqual(fileContent); expect(exists('dir/child.txt')).toBe(false);
/// ```
#[test]
fn a_saved_merge_that_replaces_a_directory_with_a_file_finishes() {
    let world: World = directory_to_file_world("merge-recovery-directory-to-file");
    let _record: PathBuf = deferred_record_of(&world, "19191919-0000-4000-8000-000000000019", "refs/cli-git/kept/9");
    assert!(world.repo.join("dir/child.txt").is_file() && !world.repo.join("dir").is_file(), "the old tree holds a directory");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the merge is finished, not refused");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert_eq!(std::fs::read(world.repo.join("dir")).expect("the file the merge adds"), b"the merge's file\n", "the path is the merge's file");
    assert!(!world.repo.join("dir/child.txt").exists(), "the path the merge deletes is gone");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    remove(world.parent.as_path());
}

/// What:
///  A file whose name fills the file system's limit is taken aside and finished,
///  because the private name does not carry it whole.
/// Why:
///  The private name embedded the whole file name and then added a prefix,
/// a PID,
/// a count and a clock
///  reading,
/// so a valid file name made a name the file system refuses (`ENAMETOOLONG`),
/// and the retained record then
///  stopped every command (a cross-vendor review's finding on the second round).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const long = 'l'.repeat(240) + '.txt'; await installEntries(...); expect(read(long)).toEqual(mergeContent);
/// ```
#[test]
fn a_file_whose_name_is_at_the_file_systems_limit_is_taken_and_finished() {
    let world: World = world("merge-recovery-long-name");
    let (mut changes, mut judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let long: String = format!("{}.txt", "l".repeat(240));
    std::fs::write(world.repo.join(long.as_str()), b"old long\n").expect("a valid long file name");
    let identity: Option<crate::merge_isolation_changes::FileIdentity> = file_identity(world.repo.join(long.as_str()).as_path());
    let old_oid: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", long.as_str()]).trim().to_owned();
    let next_oid: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--stdin"]).trim().to_owned();
    changes.push(extra_change(long.as_str(), "100644", "100644", old_oid.as_str(), next_oid.as_str()));
    judgements.push(Judged { standing: Standing::Old, identity, content: Some(old_oid.clone()) });
    let mut said: Vec<u8> = Vec::new();
    let installed = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said);
    assert!(installed.is_ok(), "a valid file name is taken and finished: {installed:?} {}", String::from_utf8_lossy(&said));
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing is left at a private name");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose finish is refused says so and lets the command go on.
/// Why:
///  A record that cannot be finished must not stop unrelated commands
/// (the shape behind two cross-vendor review
///  findings):
/// nothing of a saved merge is in the checkout,
/// so a command that only reads it has nothing to
///  be stopped for,
/// and the line says where the merge is and what blocked it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deferredRecord(); stage('f.txt', C); const outcomes = await recover(...); expect(outcomes[0].action).toBe('deferred-refused'); expect(indexOf('f.txt')).toEqual(C);
/// ```
#[test]
fn a_saved_merge_that_cannot_be_finished_does_not_stop_the_command() {
    let world: World = world("merge-recovery-saved-refused");
    let record: PathBuf = deferred_record_of(&world, "20202020-0000-4000-8000-000000000020", "refs/cli-git/kept/20");
    std::fs::write(world.repo.join("f.txt"), b"f.txt staged by someone else\n").expect("the staged version");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    std::fs::write(world.repo.join("f.txt"), b"f.txt base\n").expect("the worktree back");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::DeferredRefused);
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("refs/cli-git/kept/20") && text.contains("f.txt"), "where it is and what blocked it: {text}");
    assert!(record.exists(), "the merge is still to be finished once the path is free");
    assert!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "f.txt"]).contains("f.txt"), "the staged version is kept");
    remove(world.parent.as_path());
}

/// What:
///  A begun merge whose finish is refused still stops the command.
/// Why:
///  The checkout may hold the merge's files,
/// so a command must not read it as another session's staged work;
/// only a
///  saved merge,
/// with nothing of it in the checkout,
/// goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// begunRecord(); write('f.txt', mine); await expect(recover(...)).rejects.toMatch(/f.txt/);
/// ```
#[test]
fn a_begun_merge_whose_finish_is_refused_still_stops_the_command() {
    let world: World = world("merge-recovery-begun-refused");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let _record: PathBuf = record_of(&world, dead_owner("21212121-0000-4000-8000-000000000021"), Some("refs/heads/main"));
    std::fs::write(world.repo.join("f.txt"), b"my own edit\n").expect("the person's edit");
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("a begun merge stops the command");
    assert!(refused.0.contains("f.txt"), "{}", refused.0);
    remove(world.parent.as_path());
}

/// What:
///  A file a process has open is never removed with its private name,
///  and a file nothing has open is.
/// Why:
///  A writer that opened the file before the take writes into the inode the take renamed,
/// so removing the
///  private name deletes the only name that edit has;
/// a check just before the unlink cannot close that,
/// but a
///  scan for a process that has the file open can prove the file has no writer left,
/// which is the proof the commit
///  service's lock removal acts on (a cross-vendor review's finding on the second round).
/// The limit is the same as that removal's:
/// a process in another
///  user namespace,
/// or one that cannot be read,
/// is not seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holderOf(aside); finishAsides([take], sink); expect(exists(aside)).toBe(true); expect(sink).toContain(aside); release(held); finishAsides([take], sink); expect(exists(aside)).toBe(false);
/// ```
#[test]
fn a_file_a_process_holds_open_is_kept_and_a_file_nothing_holds_is_removed() {
    let world: World = world("merge-recovery-held-open-aside");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    let expected: String = git_text(world.repo.as_path(), &["hash-object", "--", "f.txt"]).trim().to_owned();
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected: expected.clone(),
    }];
    // A writer that had the file open before the take holds the very inode the take renamed; the scan leaves this process out,
    //  so the holder is a child of it, as an editor would be.
    let kept_open: std::fs::File = std::fs::File::open(aside.as_path()).expect("the writer's descriptor to hand on");
    let mut holder: std::process::Command = std::process::Command::new("sleep");
    holder.arg("120").stdin(std::process::Stdio::from(kept_open));
    let mut writer: std::process::Child = holder.spawn().expect("a writer holding the file open");
    drop(holder);
    let mut said: Vec<u8> = Vec::new();
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said).expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "a file a process holds open is never removed");
    assert!(text.contains(aside.to_string_lossy().as_ref()), "the private name is said: {text}");
    // With the writer gone, nothing proves a write can still arrive, and the private name goes.
    writer.kill().expect("the writer ends");
    writer.wait().expect("reaped");
    let mut afterwards: Vec<u8> = Vec::new();
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut afterwards).expect("the asides are finished");
    assert!(!aside.exists(), "a file nothing holds open is removed once its content was checked");
    remove(world.parent.as_path());
}

/// What:
///  A staged edit of a path the merge changes stops the settling of a branch that moved,
///  and the staged version stays in the index.
/// Why:
///  The straight completion path refuses a foreign staged entry,
/// and the moved-branch settling had no such check,
/// so
///  the person's staged version of a path the merge changes was overwritten with the moved head's and the record
///  removed
/// (a cross-vendor review of the third round).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); commitUnrelated(); stage('f.txt', C); await expect(recover(...)).rejects.toMatch(/f.txt/);
/// ```
#[test]
fn a_staged_edit_of_a_merge_path_stops_the_moved_branch_settling() {
    let world: World = world("merge-recovery-moved-staged");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    // A commit on top of the old head that leaves `f.txt` alone, checked out: the branch moved.
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    // Another program stages its own version of `f.txt` and puts the worktree back to the merge's content.
    std::fs::write(world.repo.join("f.txt"), b"f.txt staged by someone else\n").expect("the staged version");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the worktree as the merge left it");
    let staged_entry: String = git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "f.txt"]);
    let _record: PathBuf = record_of(&world, dead_owner("22222222-0000-4000-8000-000000000022"), Some("refs/heads/main"));
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("a staged edit of a merge path stops the settling");
    assert!(refused.0.contains("f.txt"), "the path is named: {}", refused.0);
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "f.txt"]), staged_entry, "the staged version is kept");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the moved one");
    remove(world.parent.as_path());
}

/// What:
///  A directory that the branch's own files explain does not stop the settling of a branch that moved.
/// Why:
///  The moved-branch settling judged paths with its own loop,
/// so the directory-to-file shape refused it for good,
/// as the
///  straight path did before its own rule
/// (a cross-vendor review of the third round,
/// and the brief's item 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); await readTree(repo, old, merged, { updateWorktree: false }); commitUnrelated(); expect((await recover(...))[0].action).toBe('branch-moved');
/// ```
#[test]
fn a_directory_that_explains_itself_does_not_stop_the_moved_branch_settling() {
    let world: World = directory_to_file_world("merge-recovery-moved-directory");
    // The merge's index is installed and the worktree still holds the branch's directory.
    git(world.repo.as_path(), &["read-tree", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    // A commit on top of the old head leaves the directory alone: the branch moved.
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let _record: PathBuf = record_of(&world, dead_owner("23232323-0000-4000-8000-000000000023"), Some("refs/heads/main"));
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the settling goes on");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::BranchMoved);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the moved one");
    assert!(world.repo.join("dir/child.txt").is_file(), "the branch's directory is still the checkout's");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose install stops after the index was written is reported as begun,
///  and the command stops.
/// Why:
///  The catch for a saved merge accepted every error and said nothing of the merge was in the checkout,
/// although the
///  index had been written and a file had been moved aside
/// (a cross-vendor review of the third round,
/// finding 5).
/// A partly installed merge read by a command as another session's work is exactly what the stop exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); deferredRecord(); const held = holdOpen('dir/child.txt'); await expect(recover(...)).rejects.toMatch(/partly installed/); release(held);
/// ```
#[test]
fn a_saved_merge_that_is_partly_installed_stops_the_command() {
    let world: World = directory_to_file_world("merge-recovery-partly-installed");
    let _record: PathBuf = deferred_record_of(&world, "24242424-0000-4000-8000-000000000024", "refs/cli-git/kept/24");
    // A process that can be read holds the file the merge deletes open, so its private name is kept and the directory the
    //  merge replaces cannot be emptied: the checkout then fails after the index was written.
    let kept_open: std::fs::File = std::fs::File::open(world.repo.join("dir/child.txt")).expect("the file to keep open");
    let mut holder: std::process::Command = std::process::Command::new("sleep");
    holder.arg("120").stdin(std::process::Stdio::from(kept_open));
    let mut writer: std::process::Child = holder.spawn().expect("a process holding the file open");
    drop(holder);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("a partly installed merge stops the command");
    writer.kill().expect("the process ends");
    writer.wait().expect("reaped");
    assert!(refused.0.contains("partly installed"), "it says the merge is partly installed: {}", refused.0);
    assert!(!world.repo.join("dir/child.txt").exists(), "the file was moved aside before the failure");
    remove(world.parent.as_path());
}

/// What:
///  A side ref another command repointed is not deleted as if it still held this merge.
/// Why:
///  The deletion named only the ref,
/// so a ref that had come to hold another merge's commit was deleted with it,
/// and that
///  merge's only name went with it
/// (a cross-vendor review of the third round,
/// finding 6).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(world, 'refs/cli-git/kept/25'); await git(['update-ref', '--no-deref', 'refs/cli-git/kept/25', other]); await recover(...); expect(await revParse('refs/cli-git/kept/25')).toEqual(other);
/// ```
#[test]
fn a_side_ref_another_command_repointed_is_not_deleted() {
    let world: World = world("merge-recovery-side-ref-repointed");
    let _record: PathBuf = deferred_record_of(&world, "25252525-0000-4000-8000-000000000025", "refs/cli-git/kept/25");
    // Another command puts its own commit on that ref, the only name it has.
    let other: String = git_text(world.repo.as_path(), &["commit-tree", "HEAD^{tree}", "-m", "another merge"]);
    git(world.repo.as_path(), &["update-ref", "--no-deref", "refs/cli-git/kept/25", other.as_str()]);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the merge is finished");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/25"]), other, "the other command's ref is kept");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid, "this merge landed");
    remove(world.parent.as_path());
}

/// What:
///  A merge saved for one branch is never installed into another branch's checkout.
/// Why:
///  The settlement resolved the recorded ref,
/// which still held the same commit,
/// and installed the merge into whatever the
///  worktree had checked out,
/// moving the recorded branch from a checkout that was not its own
///  (a cross-vendor review of the third round,
/// finding 7).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(world, 'refs/cli-git/kept/26'); await git(['branch', 'other']); await git(['switch', 'other']); const outcomes = await recover(...); expect(await symbolicRef()).toEqual('refs/heads/other'); expect(await revParse('main')).toEqual(oldOid); expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_saved_merge_for_another_branch_is_not_installed_into_the_checked_out_one() {
    let world: World = world("merge-recovery-other-branch");
    let record: PathBuf = deferred_record_of(&world, "26262626-0000-4000-8000-000000000026", "refs/cli-git/kept/26");
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    git(world.repo.as_path(), &["switch", "--quiet", "other"]);
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("refs/heads/main"), "the branch the merge is for is named: {text}");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::DeferredRefused);
    assert_eq!(git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/other", "the checkout stays on its own branch");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid, "the merge's branch did not move");
    assert_eq!(snapshot(&world), before, "nothing of the merge is in that checkout");
    assert!(record.exists(), "the record is kept for when its branch is checked out");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/26"]), world.new_oid, "the side ref is kept");
    remove(world.parent.as_path());
}

/// What:
///  A file whose bytes were replaced with the same length,
///  and whose modification time was put back,
///  is not taken for the file that was judged.
/// Why:
///  The identity a take compares is the device,
/// the inode,
/// the size,
/// the mode and the modification time,
/// and a program that
///  writes the same length and restores the time leaves every one of them as it was,
/// so the merge replaced an
///  edit that was never hashed or copied
/// (a cross-vendor review of the third round,
/// the second of its timing findings).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); writeSameLength('f.txt', edit); restoreMtime('f.txt', judged.identity.modified); await expect(installEntries(...)).rejects.toMatch(/f.txt/); expect(read('f.txt')).toEqual(edit);
/// ```
#[test]
fn a_file_whose_bytes_changed_with_its_length_and_time_kept_is_not_replaced() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    let world: World = world("merge-recovery-same-length-edit");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let location: PathBuf = world.repo.join("f.txt");
    let judged: &Judged = &judgements[changes.iter().position(|change| { return change.path == b"f.txt"; }).expect("the path")];
    let kept_time = judged.identity.expect("the identity the judgement read");
    let modified: SystemTime = UNIX_EPOCH + Duration::new(u64::try_from(kept_time.modified_seconds).expect("a time"), u32::try_from(kept_time.modified_nanos).expect("nanoseconds"));
    // The same length as "f.txt base\n", written over the same inode, and the modification time put back.
    std::fs::write(location.as_path(), b"f.txt EDIT\n").expect("the person's edit");
    std::fs::File::options().write(true).open(location.as_path()).expect("open").set_modified(modified).expect("the time put back");
    assert_eq!(file_identity(location.as_path()), Some(kept_time), "the identity is the one that was judged");
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said)
        .expect_err("bytes that changed are not the bytes that were judged");
    assert!(refused.error.0.contains("f.txt"), "the path is named: {}", refused.error.0);
    assert_eq!(std::fs::read(location.as_path()).expect("the file"), b"f.txt EDIT\n", "the person's bytes are kept");
    remove(world.parent.as_path());
}

/// What:
///  A taken file whose bytes were replaced,
///  with its length and modification time kept,
///  is kept rather than removed with its private name.
/// Why:
///  A program that had the file open keeps its descriptor across the rename,
/// and a write through it lands in the file
///  that waits aside;
/// the identity cannot see that,
/// but the bytes can,
/// and a file whose bytes are not the ones that
///  were judged is kept and named
/// (a cross-vendor review of the third round,
/// the first and second of its timing findings).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const take = takeAside(file); writeSameLength(take.aside, edit); restoreMtime(take.aside, take.taken.modified); const left = finishAsides([take], sink); expect(exists(take.aside)).toBe(true); expect(sink).toContain(take.aside);
/// ```
#[test]
fn a_taken_file_whose_bytes_changed_is_kept_and_named() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    let world: World = world("merge-recovery-same-length-aside");
    let location: PathBuf = world.repo.join("f.txt");
    let expected: String = git_text(world.repo.as_path(), &["hash-object", "--", "f.txt"]).trim().to_owned();
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    // A write through the descriptor a program kept: the same length, and the time put back.
    std::fs::write(aside.as_path(), b"f.txt EDIT\n").expect("the writer's edit");
    let modified: SystemTime = UNIX_EPOCH + Duration::new(u64::try_from(taken.modified_seconds).expect("a time"), u32::try_from(taken.modified_nanos).expect("nanoseconds"));
    std::fs::File::options().write(true).open(aside.as_path()).expect("open").set_modified(modified).expect("the time put back");
    assert_eq!(file_identity(aside.as_path()), Some(taken), "the identity is the one that was judged");
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected,
    }];
    let mut said: Vec<u8> = Vec::new();
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said).expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "a file whose bytes changed is kept");
    assert!(text.contains(aside.to_string_lossy().as_ref()), "the private name is said: {text}");
    assert_eq!(std::fs::read(aside.as_path()).expect("the writer's edit"), b"f.txt EDIT\n", "the edit is not lost");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose index and files are installed stops the command when a file is foreign,
///  instead of letting the command read the merge as ordinary staged work.
/// Why:
///  The foreign-content refusal was wrapped as one that wrote nothing,
/// so the saved merge's catch let the command
///  go on although the index already held the merge beside the old branch
/// (a cross-vendor review of the fourth round,
/// the brief's item 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged, { updateWorktree: true }); const record = deferredRecord(); write('f.txt', foreign); await expect(recover(...)).rejects.toMatch(/f.txt/);
/// ```
#[test]
fn a_saved_merge_whose_installed_state_meets_a_foreign_file_stops_the_command() {
    let world: World = world("merge-recovery-installed-foreign");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = deferred_record_of(&world, "27272727-0000-4000-8000-000000000027", "refs/cli-git/kept/27");
    // An editor saves something else into a path the merge changes.
    std::fs::write(world.repo.join("f.txt"), b"an editor's own save\n").expect("the editor's file");
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("a begun merge stops the command");
    assert!(refused.0.contains("f.txt"), "the path is named: {}", refused.0);
    assert!(record.exists(), "the record is kept");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid, "the branch did not move");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge that is partly installed keeps its record when the branch advances,
///  and the command is told.
/// Why:
///  The branch-moved branch dropped the record because a side ref existed,
/// without looking at what was installed,
/// so a
///  file under its private name and the merge's staged entry were left with nothing that would ever recover them
///  (a cross-vendor review of the fourth round,
/// the brief's item 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); deferredRecord(); commitUnrelated(); await expect(recover(...)).rejects.toMatch(/partly installed/); expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_saved_merge_that_is_partly_installed_and_whose_branch_advanced_keeps_its_record() {
    let world: World = world("merge-recovery-advanced-partly-installed");
    // The merge's index is installed and its files are not: the command that installed them was killed here.
    git(world.repo.as_path(), &["read-tree", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = deferred_record_of(&world, "28282828-0000-4000-8000-000000000028", "refs/cli-git/kept/28");
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let staged_entry: String = git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "f.txt"]);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("a partly installed merge stops the command");
    assert!(refused.0.contains("partly installed"), "it says the merge is partly installed: {}", refused.0);
    assert!(refused.0.contains(record.to_string_lossy().as_ref()), "the record is named: {}", refused.0);
    assert!(record.exists(), "the record is kept, so the install can still be settled");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the moved one");
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "f.txt"]), staged_entry, "the installed entry is kept");
    remove(world.parent.as_path());
}

/// What:
///  A take whose content read fails after the rename puts the file back and reports nothing installed.
/// Why:
///  The rename had already happened when the read failed,
/// so returning an error without restoring the file left it under
///  its private name while recovery said nothing of the merge was in the checkout
///  (a cross-vendor review of the fourth round,
/// the brief's item 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitattributes('f.txt filter=broken'); const (changes, judged) = judge(world); gitConfig('filter.broken.clean', "exit 1"); gitConfig('filter.broken.required', 'true'); await expect(installEntries(...)).rejects; expect(read('f.txt')).toEqual(old); expect(asides()).toEqual([]);
/// ```
#[test]
fn a_take_whose_read_fails_after_the_rename_puts_the_file_back() {
    let world: World = world("merge-recovery-take-read-fails");
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=broken\n").expect("the attribute file");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let old: Vec<u8> = std::fs::read(world.repo.join("f.txt")).expect("the old content");
    // From here the clean filter is required and fails, so the read after the rename cannot run.
    git(world.repo.as_path(), &["config", "filter.broken.clean", "false"]);
    git(world.repo.as_path(), &["config", "filter.broken.required", "true"]);
    let mut said: Vec<u8> = Vec::new();
    let failed = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said)
        .expect_err("a read that fails after the rename stops the merge");
    assert!(!failed.began, "nothing of the merge was installed");
    assert!(!failed.error.0.is_empty(), "the failure is named");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("the file is back"), old, "the file the take moved is put back");
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing is left at a private name");
    remove(world.parent.as_path());
}

/// What:
///  A taken file whose appended bytes a clean filter drops is kept rather than removed.
/// Why:
///  `may_remove` compared only the filtered hash,
/// so a write a clean filter hides left that hash as it was,
/// and the
///  file that held the person's bytes was deleted with its private name
/// (a cross-vendor review of the fourth round,
/// the brief's item 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanFilter('strip', 'sed /^NOTE:/d'); const take = takeAside(file); append(take.aside, 'NOTE: locally saved data\n'); finishAsides([take], sink); expect(exists(take.aside)).toBe(true);
/// ```
#[test]
fn a_taken_file_whose_appended_bytes_a_filter_drops_is_kept_and_named() {
    let world: World = world("merge-recovery-filtered-append");
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=strip\n").expect("the attribute file");
    git(world.repo.as_path(), &["config", "filter.strip.clean", "sed /^NOTE:/d"]);
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let judged: &Judged = &judgements[changes.iter().position(|change| { return change.path == b"f.txt"; }).expect("the path")];
    let expected: String = judged.content.clone().expect("the object the judgement read");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    // A writer that kept its descriptor appends a line the clean filter drops.
    use std::io::Write;
    let mut file: std::fs::File = std::fs::File::options().append(true).open(aside.as_path()).expect("the writer's descriptor");
    file.write_all(b"NOTE: locally saved data\n").expect("the appended line");
    drop(file);
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected,
    }];
    let mut said: Vec<u8> = Vec::new();
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said).expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "a file whose bytes changed behind a filter is kept");
    assert!(text.contains(aside.to_string_lossy().as_ref()), "the private name is said: {text}");
    assert!(std::fs::read(aside.as_path()).expect("the aside").ends_with(b"NOTE: locally saved data\n"), "the appended bytes are kept");
    remove(world.parent.as_path());
}

/// What:
///  A write that arrives after the holder scan is seen by the last look,
///  so the file that holds it is kept.
/// Why:
///  The identity was read before the scan,
/// so a program that kept its descriptor open could append a line the clean filter drops
///  after that read and close it,
/// and the scan then found no holder while the filtered bytes still matched
///  (a cross-vendor review of the fifth round,
/// its first finding).
/// The seam is the point a control reaches after the scan and before the last looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const take = takeAside(file); hooks.set('aside-scanned', () => append(take.aside, 'NOTE: kept\n')); finishAsides([take], sink); expect(exists(take.aside)).toBe(true);
/// ```
#[test]
fn a_write_that_arrives_after_the_holder_scan_is_seen_by_the_last_look() {
    let world: World = world("merge-recovery-write-after-scan");
    // A clean filter that drops the line the writer appends, so only the identity can see the change.
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=strip\n").expect("the attribute file");
    git(world.repo.as_path(), &["config", "filter.strip.clean", "sed /^NOTE:/d"]);
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let judged: &Judged = &judgements[changes.iter().position(|change| { return change.path == b"f.txt"; }).expect("the path")];
    let expected: String = judged.content.clone().expect("the object the judgement read");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected,
    }];
    // The program that kept its descriptor writes at the instant the scan has just finished.
    let written: PathBuf = aside.clone();
    crate::service_fault::hooks::set(
        "aside-scanned",
        std::rc::Rc::new(move || {
            use std::io::Write;
            let mut file: std::fs::File = std::fs::File::options().append(true).open(written.as_path()).expect("the writer's descriptor");
            file.write_all(b"NOTE: locally saved data\n").expect("the appended line");
        }),
    );
    let mut said: Vec<u8> = Vec::new();
    let outcome = finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said);
    crate::service_fault::hooks::clear();
    outcome.expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "a file written after the scan is kept");
    assert!(text.contains(aside.to_string_lossy().as_ref()), "the private name is said: {text}");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose index was installed stops the command when the worktree has another branch checked out.
/// Why:
///  The branch-mismatch refusal was wrapped as one that wrote nothing,
/// so a saved merge whose index entries were already
///  installed was reported as having nothing in the checkout and the command went on
///  (a cross-vendor review of the fifth round,
/// its fourth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); const record = deferredRecord(); await git(['branch', 'other']); await git(['switch', 'other']); await expect(recover(...)).rejects.toMatch(/checked out/);
/// ```
#[test]
fn a_saved_merge_whose_index_was_installed_and_whose_checkout_changed_branch_stops_the_command() {
    let world: World = world("merge-recovery-installed-other-branch");
    git(world.repo.as_path(), &["read-tree", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = deferred_record_of(&world, "29292929-0000-4000-8000-000000000029", "refs/cli-git/kept/29");
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    git(world.repo.as_path(), &["switch", "--quiet", "other"]);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("an installed merge stops the command");
    assert!(refused.0.contains("refs/heads/main"), "the branch the merge is for is named: {}", refused.0);
    assert!(record.exists(), "the record is kept");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid, "the merge's branch did not move");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose index was installed stops the command when a file cannot be read.
/// Why:
///  The worktree read failure was wrapped as one that wrote nothing,
/// so a merge whose index entries were installed was
///  reported as not installed and the command went on
///  (a cross-vendor review of the fifth round,
/// its fifth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); deferredRecord(); gitConfig('filter.broken.required', 'true'); await expect(recover(...)).rejects;
/// ```
#[test]
fn a_saved_merge_whose_index_was_installed_and_a_file_cannot_be_read_stops_the_command() {
    let world: World = world("merge-recovery-installed-unreadable");
    git(world.repo.as_path(), &["read-tree", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = deferred_record_of(&world, "30303030-0000-4000-8000-000000000030", "refs/cli-git/kept/30");
    // A clean filter the path requires now fails, so the worktree cannot be read.
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=broken\n").expect("the attribute file");
    git(world.repo.as_path(), &["config", "filter.broken.clean", "false"]);
    git(world.repo.as_path(), &["config", "filter.broken.required", "true"]);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("an installed merge stops the command");
    assert!(!refused.0.is_empty(), "the failure is named");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A retry whose take fails reports the index entries an earlier attempt installed,
///  so the command stops.
/// Why:
///  The take failure computed `began` from its own rollback alone,
/// so a merge whose index was installed by an
///  earlier attempt was reported as not installed and the command went on
///  (a cross-vendor review of the fifth round,
/// its sixth finding).
/// The failing take is a path in a directory that refuses the rename,
/// which the control skips where it runs as root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); await readTree(repo, old, merged); deferredRecord(); chmod('dir', 0o555); await expect(recover(...)).rejects; expect(exists(record)).toBe(true);
/// ```
#[cfg(unix)]
#[test]
fn a_retry_after_an_earlier_install_that_fails_reports_the_install() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipped: a directory that refuses writes does not stop root, so this control proves nothing here");
        return;
    }
    let world: World = directory_to_file_world("merge-recovery-retry-installed");
    // An earlier attempt installed the merge's index entries and died before it wrote the files.
    git(world.repo.as_path(), &["read-tree", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = deferred_record_of(&world, "31313131-0000-4000-8000-000000000031", "refs/cli-git/kept/31");
    // The take of the file the merge deletes cannot rename into the directory that holds it.
    std::fs::set_permissions(world.repo.join("dir"), std::fs::Permissions::from_mode(0o555)).expect("a directory that refuses writes");
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("the retry's failure stops the command");
    let _writable = std::fs::set_permissions(world.repo.join("dir"), std::fs::Permissions::from_mode(0o755));
    assert!(!refused.0.is_empty(), "the failure is named");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A recovery whose ref lookup cannot start still says the merge is partly installed.
/// Why:
///  The branch and `HEAD` reads were wrapped as refusals that wrote nothing,
/// so a saved merge whose index an earlier
///  attempt had installed was reported as having nothing in the checkout when the lookup's child could not
///  start,
/// and the requested command went on against the half-applied merge
///  (a cross-vendor review of the sixth round,
/// its second finding).
/// The seam is a Git that removes itself while the last file is read,
/// so the lookup that follows cannot start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); deferredRecord(); const place = withGit(vanishingShim); await expect(recover(place)).rejects; expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_ref_lookup_that_cannot_start_reports_the_install_already_found() {
    let world: World = world("merge-recovery-ref-lookup-fails");
    // An earlier attempt installed the merge's index entries and died before moving the branch.
    git(world.repo.as_path(), &["read-tree", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = deferred_record_of(&world, "32323232-0000-4000-8000-000000000032", "refs/cli-git/kept/32");
    // A Git that removes itself while the last judged file is read: the ref lookup that follows cannot start.
    let shim: PathBuf = world.parent.join("vanishing-git");
    crate::test_support::executable(
        shim.as_path(),
        format!(
            "#!/bin/sh\nfor argument in \"$@\"; do\n  case \"$argument\" in\n    --path=gone.txt) rm --force -- '{shim}' ;;\n  esac\ndone\nexec {real} \"$@\"\n",
            shim = shim.display(),
            real = crate::test_support::REAL_GIT
        )
        .as_bytes(),
    );
    // Named options make the in-process readers decline, so every step here is a child and the last one can be missing.
    let mut context: crate::transaction_git::GitContext = git_context();
    context.real_git = shim.clone();
    context.global_prefix = vec![
        std::ffi::OsString::from("--git-dir"),
        world.git_dir.as_os_str().to_owned(),
        std::ffi::OsString::from("--work-tree"),
        world.repo.as_os_str().to_owned(),
    ];
    let place: RecoveryPlace = RecoveryPlace {
        git: context,
        cwd: world.repo.clone(),
        common_dir: world.git_dir.clone(),
        git_dir: world.git_dir.clone(),
    };
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place, world.git_dir.as_path(), &mut said).expect_err("the refusal must report the install");
    assert!(!shim.exists(), "the seam ran");
    assert!(!refused.0.is_empty(), "the failure is named");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A worktree that switches branch in the instant before the branch move keeps the record,
///  and the branch is put back.
/// Why:
///  The branch write verifies the branch and not the worktree's `HEAD`,
/// so a switch that lands after the last look moved `main`
///  while `other` owned the checkout,
/// and recovery then removed the only record of it
///  (a cross-vendor review of the sixth round,
/// its first finding).
/// The seam is the instant between the last look and the write,
/// which no other test can reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(); await git(['branch', 'other']); hooks.set('before-the-branch-write', () => git(['symbolic-ref', 'HEAD', 'refs/heads/other'])); await expect(recover(...)).rejects; expect(await revParse('main')).toEqual(oldOid); expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_worktree_that_switches_before_the_branch_write_keeps_the_record_and_the_branch() {
    let world: World = world("merge-recovery-switch-before-write");
    let record: PathBuf = deferred_record_of(&world, "34343434-0000-4000-8000-000000000034", "refs/cli-git/kept/34");
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    let repository: PathBuf = world.repo.clone();
    crate::service_fault::hooks::set(
        "before-the-branch-write",
        std::rc::Rc::new(move || {
            git(repository.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
        }),
    );
    let mut said: Vec<u8> = Vec::new();
    let attempted = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said);
    crate::service_fault::hooks::clear();
    let refused = attempted.expect_err("a switched worktree stops the command");
    assert!(refused.0.contains("refs/heads/other"), "the branch the worktree switched to is named: {}", refused.0);
    assert!(
        refused.0.contains("index entries and files are still in this worktree"),
        "it says the merge's entries and files stay: {}",
        refused.0
    );
    assert!(
        !refused.0.contains("nothing of this merge stays staged"),
        "it does not claim the checkout is clean: {}",
        refused.0
    );
    assert!(
        refused.0.contains("git restore --no-enforce-worktree --staged --worktree"),
        "it names the way to undo the staged content, as the wrapper runs it: {}",
        refused.0
    );
    assert!(record.exists(), "the record is kept");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid, "the branch was put back");
    assert_eq!(git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/other", "the worktree is where the other command put it");
    remove(world.parent.as_path());
}

/// What:
///  A merge whose index entries an earlier attempt installed stops the command when the index stays held,
///  instead of letting it read the checkout.
/// Why:
///  The install returned `Installed::IndexHeld` whether or not the merge was already installed,
/// so the startup gate let a
///  read-only `status` or `diff --cached` observe the unfinished merge as ordinary staged work
///  (a cross-vendor review of the seventh round,
/// its fourth finding).
/// The lock is one the removal proof refuses,
/// so the wait ends at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); const record = deferredRecord(); write('.git/index.lock', content); write('.git/index~pid.lock', 'pid 1\n'); await expect(recover(...)).rejects; expect(exists(record)).toBe(true);
/// ```
#[test]
fn an_installed_merge_whose_index_stays_held_stops_the_command() {
    let world: World = world("merge-recovery-installed-index-held");
    // An earlier attempt installed the merge's index entries and died.
    git(world.repo.as_path(), &["read-tree", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = deferred_record_of(&world, "35353535-0000-4000-8000-000000000035", "refs/cli-git/kept/35");
    // A lock the removal proof does not cover, in the shape a person's editor-phase commit leaves.
    std::fs::write(world.git_dir.join("index.lock"), b"an index written into the lock").expect("a lock with content");
    std::fs::write(world.git_dir.join("index~pid.lock"), b"pid 4294967\n").expect("the PID file of a process that is gone");
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("an installed merge behind a held index stops the command");
    assert!(!refused.0.is_empty(), "the failure is named");
    assert!(refused.0.contains("index.lock"), "the lock is named: {}", refused.0);
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A directory a killed recovery left empty is removed by the retry,
///  so the merge that puts a file there finishes.
/// Why:
///  The kill sits between finishing the child's private name and removing the emptied directory,
/// and the retry judged the
///  missing child as already at its end state,
/// so it never scheduled the removal and `checkout-index` refused the
///  existing directory on every attempt
///  (a cross-vendor review of the seventh round,
/// its fifth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); await readTree(repo, old, merged); rm('dir/child.txt'); deferredRecord(); const outcomes = await recover(...); expect(read('dir')).toEqual(mergeFile);
/// ```
#[test]
fn a_directory_left_empty_by_a_killed_recovery_is_removed_on_the_retry() {
    let world: World = directory_to_file_world("merge-recovery-empty-directory-retry");
    // An earlier attempt installed the merge's index, finished the child's private name and died before removing the directory.
    git(world.repo.as_path(), &["read-tree", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    std::fs::remove_file(world.repo.join("dir/child.txt")).expect("the child's file goes, as a finished take leaves it");
    assert!(world.repo.join("dir").is_dir(), "the emptied directory is still there");
    assert_eq!(std::fs::read_dir(world.repo.join("dir")).expect("the directory").count(), 0, "and it is empty");
    let _record: PathBuf = deferred_record_of(&world, "36363636-0000-4000-8000-000000000036", "refs/cli-git/kept/36");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the retry finishes the merge");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert_eq!(std::fs::read(world.repo.join("dir")).expect("the merge's file"), b"the merge's file\n");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    remove(world.parent.as_path());
}

/// What:
///  An independent commit of the same content does not make a saved merge look installed,
///  so the command goes on.
/// Why:
///  One index entry or file matching the merge's side was taken as evidence that the merge began,
/// so a lock holder who
///  independently committed the same content for one path made every later command stop on a saved merge that had
///  installed nothing
///  (a cross-vendor review of the seventh round,
/// its sixth finding).
/// The merge changes two other paths that hold nothing of it,
/// so only every path matching is an install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(); write('f.txt', mergeContent); git add f.txt; commit('independent'); const outcomes = await recover(...); expect(outcomes[0].action).toBe('saved-branch-moved'); expect(exists(record)).toBe(false);
/// ```
#[test]
fn an_unrelated_commit_of_the_same_content_does_not_make_a_saved_merge_began() {
    let world: World = world("merge-recovery-same-content-elsewhere");
    let record: PathBuf = deferred_record_of(&world, "37373737-0000-4000-8000-000000000037", "refs/cli-git/kept/37");
    // The holder of the index independently commits the content this merge would give `f.txt`, and nothing else of it.
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the same content");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=independent", "--", "f.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::SavedBranchMoved, "nothing of the merge was installed");
    assert!(!record.exists(), "the record only goes");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the independent commit");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/37"]), world.new_oid, "the saved merge is kept");
    remove(world.parent.as_path());
}

/// What:
///  A file that carries bytes a clean filter drops is not replaced by the merge,
///  and its bytes stay where they are.
/// Why:
///  Recovery judged such a file by its filtered hash,
/// which equals the branch's object,
/// so the note the person saved was
///  accepted as the branch's own content and replaced by the merge,
/// and the aside that held its only copy was
///  removed
///  (a cross-vendor review of the eighth round,
/// its second finding).
/// The aside exists to keep the person's bytes,
/// so it compares them raw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanFilter('strip', 'sed /^NOTE:/d'); write('f.txt', 'f.txt base\nNOTE: mine\n'); const (changes, judged) = judge(world); await expect(installEntries(...)).rejects.toMatch(/f.txt/); expect(read('f.txt')).toEqual('f.txt base\nNOTE: mine\n');
/// ```
#[test]
fn a_note_a_filter_hides_is_not_replaced_and_stays_where_it_is() {
    let world: World = world("merge-recovery-filtered-note-kept");
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=strip\n").expect("the attribute file");
    git(world.repo.as_path(), &["config", "filter.strip.clean", "sed /^NOTE:/d"]);
    // The person's note is in the file before recovery judges it, and the filter hides it.
    let note: &[u8] = b"f.txt base\nNOTE: locally saved data\n";
    std::fs::write(world.repo.join("f.txt"), note).expect("the person's note");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let judged: &Judged = &judgements[changes.iter().position(|change| { return change.path == b"f.txt"; }).expect("the path")];
    assert_eq!(judged.standing, Standing::Old, "the filtered content is the branch's, which is why the note is invisible to it");
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(&place(&world), world.repo.as_path(), changes.as_slice(), judgements.as_slice(), false, A_CONTROLS_MERGE, &mut said)
        .expect_err("a file whose raw bytes are not the branch's is not the merge's to replace");
    assert!(refused.error.0.contains("f.txt"), "the path is named: {}", refused.error.0);
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("the file"), note, "the note stays where it is");
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing is left at a private name");
    remove(world.parent.as_path());
}

/// What:
///  A merge that is installed and then touched by another command still counts as installed,
///  so the command stops.
/// Why:
///  The installed state was read as every path holding the merge,
/// so one path another command edited and staged made a
///  half-installed merge look uninstalled,
/// the refusal was deferred,
/// and a later commit could record the other half
///  as ordinary staged work with no merge parent
///  (a cross-vendor review of the eighth round,
/// its third finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged, {updateWorktree: true}); deferredRecord(); write('f.txt', edit); git add f.txt; await expect(recover(...)).rejects.toMatch(/f.txt|partly installed/);
/// ```
#[test]
fn an_installed_merge_another_command_touched_still_stops_the_command() {
    let world: World = world("merge-recovery-installed-then-touched");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = deferred_record_of(&world, "38383838-0000-4000-8000-000000000038", "refs/cli-git/kept/38");
    // Another command edits and stages `f.txt`; `dir/new.txt` and `gone.txt` still hold the merge.
    std::fs::write(world.repo.join("f.txt"), b"another command's edit\n").expect("the edit");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect_err("an installed merge stops the command");
    assert!(refused.0.contains("f.txt") || refused.0.contains("partly installed"), "the state is named: {}", refused.0);
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A one-path merge whose content the branch's holder committed independently is not read as installed,
///  so the command goes on instead of being refused for ever.
/// Why:
///  With one changed path,
/// every entry and every file matching the merge is exactly what that independent
///  commit leaves,
/// so the branch being unchanged is what tells an install from it
///  (a cross-vendor review of the eighth round,
/// its fourth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = onePathMergeWorld(); const record = deferredRecord(); write('f.txt', mergeContent); git add f.txt; commit('independent'); const outcomes = await recover(...); expect(outcomes[0].action).toBe('saved-branch-moved');
/// ```
#[test]
fn an_independent_commit_of_a_one_path_merge_does_not_block_the_command() {
    let world: World = one_path_merge_world("merge-recovery-one-path-independent");
    let record: PathBuf = deferred_record_of(&world, "39393939-0000-4000-8000-000000000039", "refs/cli-git/kept/39");
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the merge's content, committed independently");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=independent", "--", "f.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::SavedBranchMoved, "nothing of the merge was installed");
    assert!(!record.exists(), "the record only goes");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the independent commit");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/39"]), world.new_oid, "the saved merge is kept");
    remove(world.parent.as_path());
}

/// What:
///  A directory that holds a file the merge does not delete stops the merge before anything is written.
/// Why:
///  The directory rule read a directory that had a tracked deletion beneath it as the branch's own,
/// without looking at
///  what else was in it,
/// so an untracked file there was met after the merge's index entries were written and its child was
///  taken,
/// and every later command failed trying to replace the same nonempty directory
///  (a cross-vendor review of the eighth round,
/// its fifth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); write('dir/notes.txt', mine); const record = deferredRecord(); const before = snapshot(world); await expect(recover(...)).rejects.toMatch(/dir/); expect(snapshot(world)).toEqual(before);
/// ```
#[test]
fn a_directory_holding_an_untracked_file_refuses_before_anything_is_written() {
    let world: World = directory_to_file_world("merge-recovery-directory-extra-file");
    std::fs::write(world.repo.join("dir/notes.txt"), b"my own notes\n").expect("the person's untracked file");
    let record: PathBuf = deferred_record_of(&world, "40404040-0000-4000-8000-000000000040", "refs/cli-git/kept/40");
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("a saved merge that wrote nothing lets the command go on");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::DeferredRefused, "it is refused, not installed");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("dir"), "the path is named: {text}");
    assert_eq!(snapshot(&world), before, "nothing was written and nothing was taken");
    assert_eq!(std::fs::read(world.repo.join("dir/notes.txt")).expect("the person's file"), b"my own notes\n");
    assert!(world.repo.join("dir/child.txt").is_file(), "the child is where it was");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A merge that already landed is settled without reading the worktree,
///  so a file that cannot be read does not keep its leftover record alive.
/// Why:
///  The worktree was read before the branch was resolved,
/// so a merge whose branch already named the merge commit
///  and whose changed path had become unreadable stopped every command,
/// and nothing could clear the record
///  because the failure came before the check that would have removed it
///  (a cross-vendor review of the eighth round,
/// its seventh finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged, {updateWorktree: true}); await updateRef('main', merged, old); const record = recordOf(deadOwner); gitConfig('filter.broken.required', 'true'); const outcomes = await recover(...); expect(outcomes[0].action).toBe('already-landed'); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_completed_merge_is_settled_without_reading_the_worktree() {
    let world: World = world("merge-recovery-landed-unreadable");
    // The merge landed: the checkout, the index and the branch all hold it, and only the record was left behind.
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    git(world.repo.as_path(), &["update-ref", "refs/heads/main", world.new_oid.as_str(), world.old_oid.as_str()]);
    let record: PathBuf = record_of(&world, dead_owner("41414141-0000-4000-8000-000000000041"), Some("refs/heads/main"));
    // A clean filter the path requires now fails, so any read of the worktree would stop the command.
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=broken\n").expect("the attribute file");
    git(world.repo.as_path(), &["config", "filter.broken.clean", "false"]);
    git(world.repo.as_path(), &["config", "filter.broken.required", "true"]);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("a landed merge needs no worktree read");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded);
    assert!(!record.exists(), "the leftover record goes");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge that replaces an existing symbolic link finishes.
/// Why:
///  The take hashed the path it moved aside with `git hash-object`,
/// which follows a link,
/// so the bytes it compared
///  were the link's target's and the link's own object never matched:
/// the link was put back,
/// the finish was refused,
///  and every retry met the same mismatch,
/// so a saved merge the command had already reported as saved never landed
///  (a cross-vendor review of the ninth round,
/// its sixth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = symlinkToFileWorld(); const record = deferredRecord(); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed'); expect(read('link')).toEqual("link as a file\n"); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_saved_merge_that_replaces_a_symlink_finishes() {
    let world: World = symlink_to_file_world("merge-recovery-symlink-replaced");
    let record: PathBuf = deferred_record_of(&world, "51515151-0000-4000-8000-000000000051", "refs/cli-git/kept/51");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed, "the merge lands: {text}");
    assert!(!record.exists(), "the record only goes once the merge landed");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid, "the branch names the merge");
    assert_eq!(std::fs::read(world.repo.join("link")).expect("the merge's file"), b"link as a file\n", "the merge's file is in place of the link");
    assert_eq!(
        std::fs::read(world.repo.join("target.txt")).expect("the link's target"),
        b"payload\n",
        "the file the link pointed at is untouched"
    );
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing is left at a private name");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]), "");
    remove(world.parent.as_path());
}

/// What:
///  A saved merge that replaces a directory holding only nested tracked files it deletes finishes.
/// Why:
///  The directory was judged by its immediate children compared with the tree diff,
/// and that diff names the
///  deleted file `dir/sub/child.txt` and never `dir/sub`,
/// so a directory of the branch's own files was read as someone
///  else's and the saved merge could never finish
///  (a cross-vendor review of the ninth round,
/// its eighth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = nestedDirectoryToFileWorld(); const record = deferredRecord(); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed'); expect(read('dir')).toEqual("the merge's file\n"); expect(exists('dir/sub')).toBe(false);
/// ```
#[test]
fn a_saved_merge_that_replaces_a_nested_directory_finishes() {
    let world: World = nested_directory_to_file_world("merge-recovery-nested-directory");
    let record: PathBuf = deferred_record_of(&world, "52525252-0000-4000-8000-000000000052", "refs/cli-git/kept/52");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed, "the merge lands: {text}");
    assert!(!record.exists(), "the record only goes once the merge landed");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid, "the branch names the merge");
    assert_eq!(
        std::fs::read(world.repo.join("dir")).expect("the merge's file"),
        b"the merge's file\n",
        "the merge's file stands where the branch's directory stood"
    );
    assert!(!world.repo.join("dir/sub").exists(), "the emptied directories of the branch's files are pruned");
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing is left at a private name");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]), "");
    remove(world.parent.as_path());
}

/// What:
///  An emptied directory the merge's own pruning never reaches stops the finish before anything is written.
/// Why:
///  Judging a directory through its whole subtree must not accept a directory with nothing in it:
/// the pruning walks up
///  from a deleted path,
/// so an emptied directory with no such path below it keeps the checkout from holding the
///  merge's file,
/// and the merge would be half installed before the failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = nestedDirectoryToFileWorld(); mkdir('dir/notes'); const record = deferredRecord(); const before = snapshot(world); const outcomes = await recover(...); expect(outcomes[0].action).toBe('deferred-refused'); expect(snapshot(world)).toEqual(before);
/// ```
#[test]
fn an_emptied_directory_the_merge_does_not_reach_refuses_before_anything_is_written() {
    let world: World = nested_directory_to_file_world("merge-recovery-nested-directory-emptied");
    std::fs::create_dir(world.repo.join("dir/notes")).expect("the person's empty directory");
    let record: PathBuf = deferred_record_of(&world, "53535353-0000-4000-8000-000000000053", "refs/cli-git/kept/53");
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("a saved merge that wrote nothing lets the command go on");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::DeferredRefused, "it is refused, not installed");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("dir"), "the path is named: {text}");
    assert_eq!(snapshot(&world), before, "nothing was written and nothing was taken");
    assert!(world.repo.join("dir/sub/child.txt").is_file(), "the branch's nested file is where it was");
    assert!(world.repo.join("dir/notes").is_dir(), "the person's directory is where it was");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A checkout a killed merge wrote one file of,
/// whose branch another command then advanced,
///  is set back to the moved
///  head,
///  and the notice that says the files were put back is true.
/// Why:
///  An install was read as every path holding the merge,
/// so a checkout holding the merge's file for one path and the
///  branch's content for the others looked untouched:
/// the record was removed without putting anything back,
/// the
///  notice claimed the files were put back and the checkout matched the branch,
/// and the person's `git status` showed the
///  abandoned merge's file as their own edit
///  (a cross-vendor review of the ninth round,
/// its fourth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = world(); write('f.txt', mergeContent); git commit -m elsewhere -- h.txt; const record = deferredRecord(); const outcomes = await recover(...); expect(outcomes[0].action).toBe('branch-moved'); expect(read('f.txt')).toEqual(base); expect(status()).toEqual('');
/// ```
#[test]
fn a_partly_written_checkout_whose_branch_moved_puts_the_written_paths_back() {
    let world: World = world("merge-recovery-moved-after-partial-write");
    // Git wrote the merge's `f.txt` and was killed before the other paths or the index.
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    // Another command advances the branch by a commit that leaves both paths as the old side has them.
    std::fs::write(world.repo.join("h.txt"), b"h.txt elsewhere\n").expect("file");
    git(world.repo.as_path(), &["commit", "--quiet", "--message=elsewhere", "--", "h.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let record: PathBuf = record_of(&world, dead_owner("54545454-0000-4000-8000-000000000054"), Some("refs/heads/main"));
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::BranchMoved, "the moved branch settles the merge: {text}");
    assert!(!record.exists(), "the record only goes once the checkout is the moved head's");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the other command's");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("file"), b"f.txt base\n", "the merge's file is put back");
    assert!(!world.repo.join("dir").exists(), "nothing the merge adds is left");
    assert_eq!(std::fs::read(world.repo.join("gone.txt")).expect("file"), b"gone.txt base\n", "nothing the merge deletes is left deleted");
    assert_eq!(aside_names(world.repo.as_path()), Vec::<String>::new(), "nothing is left at a private name");
    assert_eq!(
        git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]),
        "",
        "the checkout matches the branch, as the notice says"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge that was saved on a side ref,
/// and whose first record names no side ref,
///  does not stop a command on another
///  branch when nothing of the merge is in the checkout.
/// Why:
///  The branch check refused the record without a side ref,
/// and only a saved record's refusal was let past,
/// so the first
///  record of a merge that installed nothing stopped every command in a worktree another command had switched
/// to a branch naming the same commit,
/// and no later command could clear it
///  (a cross-vendor review of the ninth round,
/// its fifth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const original = recordOf(deadOwner, { sideRef: undefined }); const saved = deferredRecord(); git switch -c other; const outcomes = await recover(...); expect(outcomes.find(o => o.record === original).action).toBe('discarded'); expect(exists(original)).toBe(false); expect(exists(saved)).toBe(true);
/// ```
#[test]
fn a_merge_that_installed_nothing_does_not_stop_a_command_on_another_branch() {
    let world: World = world("merge-recovery-other-branch");
    // The merge met a lock it could not clear: its first record names no side ref and a second one names the kept commit.
    let original: PathBuf = record_of(&world, dead_owner("56565656-0000-4000-8000-000000000056"), Some("refs/heads/main"));
    let saved: PathBuf = deferred_record_of(&world, "57575757-0000-4000-8000-000000000057", "refs/cli-git/kept/57");
    let before = snapshot(&world);
    // Another command switches the worktree to a branch that names the same commit and installs nothing.
    git(world.repo.as_path(), &["switch", "--quiet", "--create", "other"]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "other"]), world.old_oid, "the other branch names the old commit");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("nothing of the merge is in the checkout, so the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        the_action_for(outcomes.as_slice(), original.as_path()),
        Some(MergeRecoveryAction::Discarded),
        "the record that names no side ref only goes: {text}"
    );
    assert!(!original.exists(), "the record nothing can finish is gone");
    assert!(saved.exists(), "the saved merge's record is kept for the branch it is for");
    assert_eq!(
        the_action_for(outcomes.as_slice(), saved.as_path()),
        Some(MergeRecoveryAction::DeferredRefused),
        "the saved merge is named and the command goes on: {text}"
    );
    assert!(text.contains("refs/heads/other"), "the branch the worktree has is named: {text}");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid, "the merged branch did not move");
    assert_eq!(git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/other", "the worktree keeps the other command's branch");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/57"]), world.new_oid, "the saved merge is kept");
    assert_eq!(snapshot(&world), before, "nothing of the checkout changed");
    remove(world.parent.as_path());
}

/// What:
///  An executable bit a person set before recovery judged the file keeps the saved merge from replacing it.
/// Why:
///  The judgement compared content alone,
/// so a file whose bytes were the branch's and whose mode was the person's was
///  read as untouched,
/// the take's identity check passed because it read the mode the person had already set,
/// and
///  the merge's file replaced it with its own mode:
/// the bit was erased without a word
///  (a cross-vendor review of the ninth round,
/// its seventh finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = world(); const record = deferredRecord(); chmod('f.txt', 0o755); const outcomes = await recover(...); expect(outcomes[0].action).toBe('deferred-refused'); expect(mode('f.txt')).toEqual(0o755); expect(read('f.txt')).toEqual(base);
/// ```
#[cfg(unix)]
#[test]
fn a_file_whose_mode_changed_before_the_judgement_is_not_replaced() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = world("merge-recovery-mode-before-judging");
    let record: PathBuf = deferred_record_of(&world, "58585858-0000-4000-8000-000000000058", "refs/cli-git/kept/58");
    // The person makes the file executable while the merge waits for the index lock.
    let location: PathBuf = world.repo.join("f.txt");
    std::fs::set_permissions(location.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the person's executable bit");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("a saved merge that wrote nothing lets the command go on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::DeferredRefused, "the merge is refused, not installed: {text}");
    assert!(text.contains("f.txt"), "the path is named: {text}");
    assert_eq!(std::fs::read(location.as_path()).expect("the file"), b"f.txt base\n", "the person's file is where it was");
    let mode: u32 = std::fs::metadata(location.as_path()).expect("the file").permissions().mode() & 0o777;
    assert_eq!(mode, 0o755, "the person's executable bit is kept");
    assert!(record.exists(), "the record is kept");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), " M f.txt", "the checkout is exactly as the person left it");
    remove(world.parent.as_path());
}

/// What:
///  A mode difference in a repository that ignores file modes does not stop the merge.
/// Why:
///  `core.fileMode` is false where the filesystem cannot keep the executable bit,
/// and Git then reads no mode
///  difference at all;
/// a judgement that refused one there would stop every recovery of a merge that touches an
///  executable path on such a filesystem,
/// which is the class of failure this route exists to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = world(); const record = deferredRecord(); gitConfig('core.filemode', 'false'); chmod('f.txt', 0o755); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed'); expect(read('f.txt')).toEqual(mergeContent);
/// ```
#[cfg(unix)]
#[test]
fn a_mode_a_repository_ignores_does_not_stop_the_merge() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = world("merge-recovery-mode-ignored");
    let record: PathBuf = deferred_record_of(&world, "59595959-0000-4000-8000-000000000059", "refs/cli-git/kept/59");
    git(world.repo.as_path(), &["config", "core.filemode", "false"]);
    std::fs::set_permissions(world.repo.join("f.txt").as_path(), std::fs::Permissions::from_mode(0o755)).expect("a mode the repository ignores");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed, "the merge lands, as Git would land it: {text}");
    assert!(!record.exists(), "the record only goes once the merge landed");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("the merge's file"), b"f.txt feature\n", "the merge's content is in place");
    remove(world.parent.as_path());
}

/// What:
///  The awaited settle decides what the synchronous one decides,
/// and the synchronous one is refused from inside the runtime.
/// Why:
///   A task of the runtime is the caller the awaited face exists for,
/// so the two faces must agree on the action,
///  the notices,
/// the checkout and the record,
/// and the refusal of the synchronous face is the reason the awaited one is not optional.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = settleFailedMerge(place, record, intent); const after = await runOnRuntime(() => settleFailedMergeAwaiting(place, record, intent)); expect(after).toEqual(before);
/// ```
#[test]
fn the_awaited_settle_answers_what_the_synchronous_one_answers() {
    // One world per face, built the same way, with Git having written part of the merge.
    let synchronous: World = world("merge-recovery-two-faces-synchronous");
    std::fs::write(synchronous.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(synchronous.repo.join("gone.txt")).expect("the file Git removed");
    let synchronous_record: PathBuf =
        record_of(&synchronous, dead_owner("97979797-0000-4000-8000-000000000097"), Some("refs/heads/main"));
    let synchronous_intent: MergeIntent = parse_merge_intent(
        std::fs::read(synchronous_record.as_path()).expect("read").as_slice(),
    )
    .expect("intent");
    let mut synchronous_said: Vec<u8> = Vec::new();
    let synchronous_action: MergeRecoveryAction = settle_failed_merge(
        &place(&synchronous),
        synchronous_record.as_path(),
        &synchronous_intent,
        &mut synchronous_said,
    )
    .expect("settled");
    let synchronous_head: String = git_text(synchronous.repo.as_path(), &["rev-parse", "main"]);
    let synchronous_status: String = git_text(synchronous.repo.as_path(), &["status", "--porcelain=v1"]);

    let awaited: World = world("merge-recovery-two-faces-awaited");
    std::fs::write(awaited.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(awaited.repo.join("gone.txt")).expect("the file Git removed");
    let awaited_record: PathBuf = record_of(&awaited, dead_owner("97979797-0000-4000-8000-000000000098"), Some("refs/heads/main"));
    let awaited_intent: MergeIntent =
        parse_merge_intent(std::fs::read(awaited_record.as_path()).expect("read").as_slice()).expect("intent");
    let mut awaited_said: Vec<u8> = Vec::new();
    let awaited_action: MergeRecoveryAction = crate::lazy_runtime::run_on_runtime(async {
        return settle_failed_merge_awaiting(&place(&awaited), awaited_record.as_path(), &awaited_intent, &mut awaited_said).await;
    })
    .expect("the runtime runs the settle")
    .expect("settled");

    assert_eq!(awaited_action, synchronous_action, "the two faces decide the same");
    assert_eq!(awaited_action, MergeRecoveryAction::Completed, "the merge the files show is completed");
    assert_eq!(
        String::from_utf8(awaited_said).expect("UTF-8"),
        String::from_utf8(synchronous_said).expect("UTF-8"),
        "the two faces say the same"
    );
    assert_eq!(
        git_text(synchronous.repo.as_path(), &["rev-parse", "main"]),
        synchronous_intent.new_oid,
        "the synchronous face moves the branch to the merge"
    );
    assert_eq!(
        git_text(awaited.repo.as_path(), &["rev-parse", "main"]),
        awaited_intent.new_oid,
        "the awaited face moves the branch to its own merge"
    );
    // Each fixture names its own merge, and the two need not be the same commit.
    let _ = synchronous_head;
    assert_eq!(git_text(awaited.repo.as_path(), &["status", "--porcelain=v1"]), synchronous_status, "the same checkout");
    assert!(!awaited_record.exists(), "the record goes once the merge landed");
    remove(synchronous.parent.as_path());
    remove(awaited.parent.as_path());

    // The positive control: the synchronous face registers its step's pipes by entering the runtime,
    //  which a task cannot do, so a task that reached it would settle nothing.
    let refused_world: World = world("merge-recovery-two-faces-refused");
    let refused_record: PathBuf =
        record_of(&refused_world, dead_owner("97979797-0000-4000-8000-000000000099"), Some("refs/heads/main"));
    let refused_intent: MergeIntent =
        parse_merge_intent(std::fs::read(refused_record.as_path()).expect("read").as_slice()).expect("intent");
    let refused: MergeRefusal = crate::lazy_runtime::run_on_runtime(async {
        let mut said: Vec<u8> = Vec::new();
        return settle_failed_merge(&place(&refused_world), refused_record.as_path(), &refused_intent, &mut said);
    })
    .expect("the runtime runs the call")
    .expect_err("a task cannot settle through the synchronous face");
    assert!(
        refused.error.0.contains("was asked for from inside itself"),
        "the refusal names what it is: {}",
        refused.error.0
    );
    assert!(refused_record.exists(), "nothing of the record was touched");
    remove(refused_world.parent.as_path());
}

/// A recovery place whose real Git is `shim`.
fn place_with_git(world: &World, shim: PathBuf) -> RecoveryPlace {
    let mut context: crate::transaction_git::GitContext = git_context();
    context.real_git = shim;
    return RecoveryPlace {
        git: context,
        cwd: world.repo.clone(),
        common_dir: world.git_dir.clone(),
        git_dir: world.git_dir.clone(),
    };
}

/// What:
///  A branch move that succeeded but could not be put back is reported as that,
///  and never as a branch put back where it was.
/// Why:
///  The undo's failure only reached the debug log,
/// and the refusal that followed said the branch was restored,
///  so a person read a successful restoration while `main` still named the merge,
/// and the next command removed the record
/// (`SPEC.md`,
/// "Every wait has a bound",
/// and the owner's "never discard user data";
/// a cross-vendor review's second finding).
/// The shim leaves a concurrent holder's own ref lock behind after the branch move,
/// which no other test can place in the instant before the undo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(); hooks.set('before-the-branch-write', () => git(['symbolic-ref', 'HEAD', 'refs/heads/other'])); const refused = await recover({ realGit: lockingShim }).catch(e => e); expect(revParse('main')).toBe(newOid); expect(refused.message).not.toContain('was put back where it was');
/// ```
#[cfg(unix)]
#[test]
fn a_branch_that_could_not_be_put_back_is_never_reported_as_put_back() {
    let world: World = world("merge-recovery-undo-fails");
    let record: PathBuf = deferred_record_of(&world, "43434343-0000-4000-8000-000000000043", "refs/cli-git/kept/43");
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    let lock: PathBuf = world.git_dir.join("refs/heads/main.lock");
    let shim: PathBuf = world.parent.join("locking-git");
    crate::test_support::executable(
        shim.as_path(),
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"update-ref\" ] && [ \"$2\" = \"-m\" ]; then\n  {real} \"$@\"\n  status=$?\n  if [ \"$status\" -eq 0 ]; then\n    : > '{lock}'\n  fi\n  exit \"$status\"\nfi\nexec {real} \"$@\"\n",
            real = crate::test_support::REAL_GIT,
            lock = lock.display()
        )
        .as_bytes(),
    );
    let place: RecoveryPlace = place_with_git(&world, shim);
    let repository: PathBuf = world.repo.clone();
    crate::service_fault::hooks::set(
        "before-the-branch-write",
        std::rc::Rc::new(move || {
            git(repository.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
        }),
    );
    let mut said: Vec<u8> = Vec::new();
    let attempted = recover_interrupted_merges(&place, world.git_dir.as_path(), &mut said);
    crate::service_fault::hooks::clear();
    let refused = attempted.expect_err("a branch that could not be put back stops the command");
    assert!(
        !refused.0.contains("The branch was put back where it was"),
        "it never claims the branch was put back: {}",
        refused.0
    );
    assert!(refused.0.contains("refs/heads/main"), "the branch that still names the merge is named: {}", refused.0);
    assert!(refused.0.contains("still names the merge"), "it says where the branch is: {}", refused.0);
    assert!(refused.0.contains("refs/heads/other"), "the branch the worktree switched to is named: {}", refused.0);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid, "the branch still names the merge");
    assert!(lock.is_file(), "the other command still holds the ref");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A merge that landed in a checkout another branch now owns keeps its record and is reported,
///  instead of losing the record that names the merge's installed entries and files.
/// Why:
///  The branch resolved to the merge commit took the record and the side ref away before asking which branch
/// owned the checkout,
///  so a worktree switched after the install lost the only record of what the merge left staged in it
/// (`SPEC.md`,
/// "Every wait has a bound",
/// and the owner's "never discard user data";
/// a cross-vendor review's second finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readTree(repo, old, merged, {updateWorktree: true}); updateRef('main', merged, old); git(['symbolic-ref', 'HEAD', 'refs/heads/other']); const record = recordOf(deadOwner); await expect(recover(...)).rejects; expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_landed_merge_in_another_branchs_worktree_keeps_its_record() {
    let world: World = world("merge-recovery-landed-elsewhere");
    // The merge landed and its index entries and files are in the checkout, which another command then switched.
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    git(world.repo.as_path(), &["update-ref", "refs/heads/main", world.new_oid.as_str(), world.old_oid.as_str()]);
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
    let record: PathBuf = record_of(&world, dead_owner("45454545-0000-4000-8000-000000000045"), Some("refs/heads/main"));
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("a landed merge in another branch's checkout stops the command");
    assert!(refused.0.contains("refs/heads/other"), "the branch the worktree has is named: {}", refused.0);
    assert!(refused.0.contains("refs/heads/main"), "the branch that names the merge is named: {}", refused.0);
    assert!(
        refused.0.contains("index entries and files"),
        "it says what stays in this worktree: {}",
        refused.0
    );
    assert!(record.exists(), "the record that names the installed merge is kept");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid, "the branch still names the merge");
    remove(world.parent.as_path());
}

/// What:
///  A merge another command landed between this command's read of the branch and its write is not a failure,
///  and the record is not said to be kept after that command removed it.
/// Why:
///  Nothing claims a dead record,
/// so two commands recover the same one,
///  and the second command's compare-and-swap fails on a branch that already names the merge:
/// it reported a partly installed merge and a kept record that was gone,
///  and whether a command failed turned on the schedule
/// (a cross-vendor review's third finding).
/// The seam is the instant before this command's branch write,
/// which no other test can reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(); hooks.set('before-the-branch-write', () => { git(['update-ref', 'main', new, old]); rm(record) }); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed');
/// ```
#[test]
fn a_merge_another_command_landed_before_the_write_is_not_a_failure() {
    let world: World = world("merge-recovery-landed-in-the-window");
    let record: PathBuf = deferred_record_of(&world, "46464646-0000-4000-8000-000000000046", "refs/cli-git/kept/46");
    let repository: PathBuf = world.repo.clone();
    let removed: PathBuf = record.clone();
    let old_oid: String = world.old_oid.clone();
    let new_oid: String = world.new_oid.clone();
    crate::service_fault::hooks::set(
        "before-the-branch-write",
        std::rc::Rc::new(move || {
            git(repository.as_path(), &["update-ref", "refs/heads/main", new_oid.as_str(), old_oid.as_str()]);
            std::fs::remove_file(removed.as_path()).expect("the other command removes its record");
        }),
    );
    let mut said: Vec<u8> = Vec::new();
    let attempted = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said);
    crate::service_fault::hooks::clear();
    let outcomes = attempted.expect("a merge that landed must not fail the command that finds it landed");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid, "the branch names the merge");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(!text.contains("was kept"), "nothing claims a record that is gone: {text}");
    remove(world.parent.as_path());
}

/// What:
///  A private name an earlier release of the wrapper wrote carries no merge key.
/// Why:
///  The name is the only thing a command has to go on,
/// and the earlier form is exactly this one:
///  the key of the path's name,
///  the process,
///  the count and the moment,
/// with no key of a merge between
///  them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${dirname}/.cli-git-recovery-${nameKey(basename)}-${pid}-0-1`
/// ```
fn an_earlier_releases_name(location: &Path) -> PathBuf {
    let found: Option<std::ffi::OsString> = location.file_name().map(std::ffi::OsStr::to_os_string);
    let name: String = found.unwrap_or_default().to_string_lossy().into_owned();
    return location.with_file_name(format!(
        ".cli-git-recovery-{}-{}-0-1",
        crate::merge_isolation_take_aside_names::name_key(name.as_str()),
        std::process::id()
    ));
}

/// The bytes the file the merge holds at one path has,
/// from the tree of a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(['show', `${oid}:${path}`])
/// ```
fn the_bytes_of_a_commit(world: &World, oid: &str, path: &str) -> Vec<u8> {
    let text: String = git_text(world.repo.as_path(), &["show", format!("{oid}:{path}").as_str()]);
    return format!("{text}\n").into_bytes();
}

/// What:
///  A copy an earlier release left of the file the merge holds at a free path is put back,
///  and the merge goes on from the checkout it expected.
/// Why:
///  The name an earlier release wrote carries no merge key,
/// so no record could say whether an unfinished merge took
///  it;
///  the bytes can,
/// and a copy of the branch's own file or the merge's own file is
///  the file the merge judged there.
/// Without this,
/// a merge interrupted before this release was never finished:
///  the path stayed empty,
/// the copy stayed beside it,
/// and every later merge read the emptiness as the
///  person's own deletion (`merge-carry-review-round-2-findings-1`,
/// its second finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// record(deadOwner(...)); rename('f.txt', earlierName('f.txt')); await putBackStrandedTakes(...); expect(exists('f.txt')).toBe(true); expect(sink).toContain('put back');
/// ```
#[test]
fn a_copy_an_earlier_release_left_of_the_merges_own_file_is_put_back() {
    let world: World = world("merge-recovery-earlier-release-copy");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let _record: PathBuf = record_of(&world, dead_owner("23232323-0000-4000-8000-000000000023"), Some("refs/heads/main"));
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    // The copy holds the branch's own file, which is what a take of an interrupted install moved aside.
    std::fs::write(aside.as_path(), the_bytes_of_a_commit(&world, world.old_oid.as_str(), "f.txt")).expect("the copy");
    std::fs::remove_file(location.as_path()).expect("the path a killed take emptied");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        &mut said,
    ))
    .expect("the runtime of the controls");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(location.exists(), "the path holds the file the merge judged again");
    assert_eq!(
        std::fs::read(location.as_path()).expect("the file is back"),
        the_bytes_of_a_commit(&world, world.old_oid.as_str(), "f.txt"),
        "its bytes are the ones it had"
    );
    assert!(!aside.exists(), "the copy is not a copy any more");
    assert!(text.contains("put back"), "the putting back is said: {text}");
    remove(world.parent.as_path());
}

/// What:
///  A copy an earlier release left of bytes nothing else holds is never put back on a guess,
///  and it is recorded so a later command ends it.
/// Why:
///  The bytes are the person's newest,
/// and nothing else holds them:
///  not a commit,
/// and not the capture the merge's record names.
/// Putting them back would
///  overwrite whatever the person put at the path afterwards,
/// so the copy stays,
/// the line names
///  it,
/// and the record makes the next command look at it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename('f.txt', earlierName('f.txt')); write(aside, 'the person\'s own newer bytes\n'); await putBackStrandedTakes(...); expect(exists(location)).toBe(false); expect(exists(aside)).toBe(true); expect(records(gitDir)).toHaveLength(1);
/// ```
#[test]
fn a_copy_an_earlier_release_left_of_newer_bytes_stays_and_is_recorded() {
    let world: World = world("merge-recovery-earlier-release-newer");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let _record: PathBuf = record_of(&world, dead_owner("24242424-0000-4000-8000-000000000024"), Some("refs/heads/main"));
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    std::fs::write(aside.as_path(), b"the person's own newer bytes\n").expect("the copy");
    std::fs::remove_file(location.as_path()).expect("the path a killed take emptied");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        &mut said,
    ))
    .expect("the runtime of the controls");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(!location.exists(), "the path the person emptied stays empty");
    assert!(aside.exists(), "the bytes nothing else holds are kept");
    assert_eq!(std::fs::read(aside.as_path()).expect("the copy"), b"the person's own newer bytes\n", "and they are whole");
    assert!(text.contains(aside.to_string_lossy().as_ref()), "the copy is named: {text}");
    assert!(text.contains("mv --"), "the line says how to put it back: {text}");
    assert!(
        crate::merge_isolation_kept_copies::record_of_a_control(world.git_dir.as_path(), aside.as_path()).exists(),
        "the copy is recorded, so a later command decides it again"
    );
    remove(world.parent.as_path());
}

/// What:
///  A copy whose bytes the person's file holds again is removed,
///  and the line says the file holds them.
/// Why:
///  Two files holding one run of bytes are one too many,
/// and the copy beside the path is the one to go:
///  the person's file is where they look,
/// and a copy that stayed would be a name nothing explains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', 'mine\n'); rename('f.txt', aside); write('f.txt', 'mine\n'); await putBackStrandedTakes(...); expect(exists(aside)).toBe(false); expect(sink).toContain('holds those same bytes again');
/// ```
#[test]
fn a_copy_the_persons_file_holds_again_is_removed() {
    let world: World = world("merge-recovery-copy-the-file-holds");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    std::fs::write(aside.as_path(), b"mine\n").expect("the copy");
    std::fs::write(location.as_path(), b"mine\n").expect("the person's file holds the same bytes");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        &mut said,
    ))
    .expect("the runtime of the controls");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(!aside.exists(), "the second copy goes");
    assert_eq!(std::fs::read(location.as_path()).expect("the person's file"), b"mine\n", "their file is untouched");
    assert!(text.contains("holds those same bytes again"), "the line says why the copy went: {text}");
    remove(world.parent.as_path());
}

/// What:
///  A copy a merge kept is decided by a command that has no merge record at all,
///  and it ends the moment the person's file holds its bytes.
/// Why:
///  A copy is left beside a path one merge touched,
/// and a later merge that touches another path says nothing
///  about it;
/// without this the copy is a file no command ever ends
///  (`merge-carry-review-round-2-findings-1`,
/// its first finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await putBackStrandedTakes(...); expect(records).toHaveLength(1); const first = await recover(noRecord); expect(sink).toContain(aside); write('f.txt', copyBytes); const second = await recover(noRecord); expect(exists(aside)).toBe(false); expect(sink).toContain('removed');
/// ```
#[test]
fn a_kept_copy_is_decided_once_more_and_ends_when_the_file_holds_its_bytes() {
    let world: World = world("merge-recovery-kept-copy-end");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    let kept: &[u8] = b"the person's own newer bytes\n";
    std::fs::write(aside.as_path(), kept).expect("the copy");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        &mut said,
    ))
    .expect("the runtime of the controls");
    let record: PathBuf = crate::merge_isolation_kept_copies::record_of_a_control(world.git_dir.as_path(), aside.as_path());
    assert!(record.exists(), "the copy is recorded");
    // A later command with no merge record in this worktree: the file the merge held is gone, so nothing judges the copy's
    //  bytes again, and the copy still ends.
    let mut later: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut later).expect("recovery");
    assert!(outcomes.is_empty(), "no merge record is left to settle");
    let text: String = String::from_utf8(later).expect("the notices are UTF-8");
    assert!(text.contains(aside.to_string_lossy().as_ref()), "the copy is named again: {text}");
    assert_eq!(std::fs::read(location.as_path()).expect("the world's file"), b"f.txt base\n", "nothing was put back on a guess");
    // The person copies their bytes into the file themselves, which is the end the line named.
    std::fs::write(location.as_path(), kept).expect("the person's file holds the bytes again");
    let mut last: Vec<u8> = Vec::new();
    let _outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut last).expect("recovery");
    let ending: String = String::from_utf8(last).expect("the notices are UTF-8");
    assert!(!aside.exists(), "the copy goes once the file holds its bytes");
    assert!(!record.exists(), "and its record goes with it");
    assert!(ending.contains("removed"), "the line says the copy went: {ending}");
    assert_eq!(std::fs::read(location.as_path()).expect("the person's file"), kept, "their file is untouched");
    remove(world.parent.as_path());
}

/// Write the record of a merge of the world whose paths held the person's own work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMergeIntent(gitDir, { ...record(id), carried: capture });
/// ```
fn record_with_a_capture(world: &World, id: &str, capture: &crate::merge_isolation_carry_parts::Capture) -> PathBuf {
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner(id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: None,
        carried: Some(capture.clone()),
    };
    return write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
}

/// What:
///  A merge the world's own steps left half carried,
///  with the person's own edit in `f.txt` captured and the merge already in the branch.
/// Why:
///  The state a kill between Git's update and the carry leaves is the one thing these controls need,
/// and every
///  step of it is a step the merge makes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', `${BRANCH}\n${PERSON_LINE}\n`); const capture = keepCapture(captureWork([f.txt])); record(capture); clear(); readTree(old, new); updateRef(main, new);
/// ```
fn half_carried(world: &World, id: &str) -> crate::merge_isolation_carry_parts::Capture {
    let person_lines: &str = "f.txt base\nthe person's line\n";
    std::fs::write(world.repo.join("f.txt"), person_lines).expect("the person's edit");
    let context = crate::test_support::git_context();
    let capture: crate::merge_isolation_carry_parts::Capture = crate::merge_isolation_carry::keep_capture(
        &context,
        world.repo.as_path(),
        &crate::merge_isolation_carry::capture_work(&context, world.repo.as_path(), &[b"f.txt".to_vec()]).expect("the capture"),
    )
    .expect("kept");
    let _record: PathBuf = record_with_a_capture(world, id, &capture);
    // The person's path is put back to the branch, which is what the merge's own clear does before Git's update.
    git(world.repo.as_path(), &["checkout", "--quiet", "--", "f.txt"]);
    git(world.repo.as_path(), &["read-tree", "-u", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    git(world.repo.as_path(), &["update-ref", "refs/heads/main", world.new_oid.as_str()]);
    return capture;
}

/// What:
///  A save that holds exactly the merge's bytes and arrives after a kill stands,
///  because the reading of what Git's update left travels beside the record.
/// Why:
///  The recovery of a killed merge had no reading of its own,
/// so a person's save whose bytes accept every line the
///  merge brought was read as the merge's own file and replaced with their older layer,
/// and the bytes of their newer
///  decision existed nowhere (`merge-carry-review-round-2-findings-1`,
/// its third finding).
/// The reading the command that moved the checkout made is
///  what tells the two apart,
/// so a recovery reads it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = halfCarried(world); writeLandedRecord(gitDir, id, landedOf(root, ['f.txt'])); write('f.txt', mergeVersion); await recover(...); expect(read('f.txt')).toEqual(mergeVersion); expect(sink).toContain('changed');
/// ```
#[test]
fn a_save_of_the_merges_own_bytes_after_a_kill_stands() {
    let world: World = world("merge-recovery-landed-reading");
    let id: &str = "26262626-0000-4000-8000-000000000026";
    let _capture: crate::merge_isolation_carry_parts::Capture = half_carried(&world, id);
    let merged: Vec<u8> = the_bytes_of_a_commit(&world, world.new_oid.as_str(), "f.txt");
    // What the command that moved the checkout read, written where the recovery of a kill finds it.
    let landed: Vec<crate::merge_isolation_carry_landed::LandedFile> =
        crate::merge_isolation_carry_landed::landed_of(world.repo.as_path(), &[b"f.txt".to_vec()]);
    crate::merge_isolation_carry_landed::write_landed_record(world.git_dir.as_path(), id, landed.as_slice());
    // The person's editor saves its buffer, which holds the merge's version and no longer their older line.
    std::fs::write(world.repo.join("f.txt"), merged.as_slice()).expect("the save");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded, "the merge is the branch's");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("f.txt"), merged, "the newer save stands");
    assert!(text.contains("f.txt"), "the line names the path it left alone: {text}");
    assert!(text.contains("changed"), "the line says the path changed after the capture: {text}");
    assert!(!text.contains("git merge refs/heads/"), "the line never calls a repository ref the merge the person ran: {text}");
    remove(world.parent.as_path());
}

/// What:
///  The control of the line above:
///  without the reading,
///  the same save is replaced by the captured layer.
/// Why:
///  A control that only passes after a change proves nothing unless the same steps go the other way before it,
///  and this is the state every recovery of a killed merge was in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = halfCarried(world); write('f.txt', mergeVersion); await recover(...); expect(read('f.txt')).toEqual(`${mergeVersion}${PERSON_LINE}\n`);
/// ```
#[test]
fn without_the_landed_reading_a_save_of_the_merges_own_bytes_is_carried_over() {
    let world: World = world("merge-recovery-no-landed-reading");
    let id: &str = "27272727-0000-4000-8000-000000000027";
    let _capture: crate::merge_isolation_carry_parts::Capture = half_carried(&world, id);
    let merged: Vec<u8> = the_bytes_of_a_commit(&world, world.new_oid.as_str(), "f.txt");
    std::fs::write(world.repo.join("f.txt"), merged.as_slice()).expect("the save");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded, "the merge is the branch's");
    let after: Vec<u8> = std::fs::read(world.repo.join("f.txt")).expect("f.txt");
    assert_ne!(after, merged, "without a reading the save is read as the merge's own file, which is the defect");
    assert!(String::from_utf8_lossy(after.as_slice()).contains("the person's line"), "the captured layer is written over it");
    remove(world.parent.as_path());
}
