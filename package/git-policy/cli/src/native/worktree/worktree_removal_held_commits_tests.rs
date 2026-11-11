//! Controls for keeping the commit a reference lock holds in a linked worktree,
//! before a command that may remove that worktree:
//! the commit reaches a reference of the common Git directory,
//! the branch does not move,
//! the person is told,
//! real Git's own maintenance keeps the commit after the removal,
//! and a command that cannot make any such reference stops instead of removing the last holder.

use super::keep_held_commits;
use crate::held_commit_record::{Held, write};
use crate::service_land_git_steps_and_import_stream::keep_on_a_side_ref_only_if_free;
use crate::test_support::{fixture, git, git_text, remove};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// The branch the linked worktree has checked out.
const BRANCH: &str = "feature";

/// The file the held commit changes.
const EDITED: &str = "edited.txt";

/// The subject of the held commit.
const SUBJECT: &str = "the held edit";

/// The batch number of the held commit.
const SEQ: u64 = 7;

/// The reference of the common Git directory a held commit of batch `seq` is kept on.
fn the_kept_name(seq: u64) -> String {
    return crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq);
}

/// The reference beside it,
/// written when the batch's own kept reference cannot be written.
fn the_rescue_name(seq: u64) -> String {
    return crate::service_scratch_ref_of_a_dropped_batch::rescue_ref(seq);
}

/// A repository,
/// a linked worktree of its own branch with the file the held commit changed,
/// and the commit the branch does not name:
/// the state a commit that met a reference lock leaves in that worktree.
fn a_linked_worktree_with_a_held_commit(label: &str) -> (PathBuf, PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(label);
    let repo: PathBuf = root.join("repo");
    let linked: PathBuf = root.join("linked");
    std::fs::create_dir(repo.as_path()).expect("the repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(repo.as_path(), &["add", "--", "base.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked_text: String = linked.display().to_string();
    git(repo.as_path(), &["worktree", "add", "--quiet", linked_text.as_str(), "-b", BRANCH]);
    std::fs::write(linked.join(EDITED), b"the edit\n").expect("the edit");
    git(linked.as_path(), &["add", "--", EDITED]);
    git(linked.as_path(), &["commit", "--quiet", "--message", SUBJECT]);
    let held_commit: String = git_text(linked.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    // The reference lock refused the branch write, so the branch still names the commit before it, and no reflog names the
    //  commit either: it was never on the branch, so the reference of the worktree is the only thing holding it.
    git(linked.as_path(), &["reset", "--hard", "--quiet", "HEAD~1"]);
    git(repo.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
    git(linked.as_path(), &["update-ref", "--no-deref", format!("refs/worktree/cli-git-service/{SEQ}").as_str(), held_commit.as_str()]);
    let git_dir: PathBuf = repo.join(".git");
    let worktree_git_dir: PathBuf = git_dir.join("worktrees/linked");
    let held: Held = Held {
        seq: SEQ,
        journal: root.join("journal"),
        repository: linked.clone(),
        common_dir: git_dir.clone(),
        git_dir: worktree_git_dir.clone(),
        reference: format!("refs/heads/{BRANCH}"),
        old: git_text(linked.as_path(), &["rev-parse", "HEAD"]).trim().to_string(),
        new: held_commit.clone(),
        commits: vec![held_commit.clone()],
        index: worktree_git_dir.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![git_dir.join(format!("refs/heads/{BRANCH}.lock"))],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    write(worktree_git_dir.as_path(), &held).expect("the record of the hold");
    return (root, repo, linked, held_commit);
}

/// The Git the checks of this crate run real Git through.
fn context(git_dir: &Path) -> GitContext {
    return GitContext { real_git: PathBuf::from("/usr/bin/git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(git_dir.to_path_buf()) };
}

/// Plant the lock file an update of `name` refused, as a killed or a concurrent Git leaves it.
fn plant_lock(common_dir: &Path, name: &str) {
    let lock: PathBuf = common_dir.join(format!("{name}.lock"));
    std::fs::create_dir_all(lock.parent().expect("the directory of the lock")).expect("the lock's directory");
    std::fs::write(lock.as_path(), b"").expect("the lock file");
}

/// What:
///  The commit a reference lock holds in a linked worktree is kept on a reference of the common Git directory,
///  the person is told the branch does not move,
///  and the commit survives the two things the removal deletes:
///  the worktree's own Git directory and its per-worktree reference.
/// Why:
///  The record of the hold lives in that Git directory and the commit is held by that reference,
///  so a removal takes both,
///  no command settles that batch afterwards,
///  and real Git's maintenance then removes a commit the wrapper said it kept
///  (a review of `aa7f601c6d`,
///  finding 2).
#[test]
fn a_held_commit_of_a_removed_linked_worktree_survives_on_the_kept_reference() {
    let (root, repo, linked, held_commit) = a_linked_worktree_with_a_held_commit("removal-held-commits");
    let git_dir: PathBuf = repo.join(".git");
    // The state the wrapper's own hold leaves: the worktree's reference is the only one that names the commit, and
    //  per-worktree references are listed only from the worktree they belong to.
    assert_eq!(
        git_text(linked.as_path(), &["for-each-ref", "--contains", held_commit.as_str(), "--format=%(refname)"]).trim(),
        format!("refs/worktree/cli-git-service/{SEQ}"),
        "nothing but the worktree's own reference names the held commit"
    );
    let kept = keep_held_commits(git_dir.as_path(), &context(git_dir.as_path()));
    assert!(kept.refused.is_empty(), "every held commit of a whole repository is kept: {:?}", kept.refused);
    assert_eq!(kept.said.len(), 1, "the person is told about the one held commit: {:?}", kept.said);
    let kept_name: String = the_kept_name(SEQ);
    assert!(kept.said[0].contains(kept_name.as_str()), "the line names where the commit is kept: {}", kept.said[0]);
    assert!(kept.said[0].contains(BRANCH), "the line names the branch: {}", kept.said[0]);
    assert!(kept.said[0].contains("stays where it is"), "the line says the branch does not move: {}", kept.said[0]);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", kept_name.as_str()]).trim(), held_commit, "the kept reference names the held commit");
    // A second command keeps nothing more: the batch's own commit is already there.
    assert_eq!(keep_held_commits(git_dir.as_path(), &context(git_dir.as_path())).said, Vec::<String>::new(), "nothing is written twice");
    // Git removes the worktree, with the record of the hold and its per-worktree reference inside it, and its own maintenance
    //  then removes what no reference names.
    let linked_text: String = linked.display().to_string();
    git(repo.as_path(), &["worktree", "remove", "--force", linked_text.as_str()]);
    assert!(!git_dir.join("worktrees/linked").exists(), "the removal took the worktree's own Git directory");
    assert_eq!(
        git_text(repo.as_path(), &["for-each-ref", "--contains", held_commit.as_str(), "--format=%(refname)"]).trim(),
        kept_name,
        "the commit is held by the reference the removal left alone"
    );
    git(repo.as_path(), &["gc", "--prune=now"]);
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-t", held_commit.as_str()]), "commit", "the commit is not lost with the worktree");
    let branch_reference: String = format!("refs/heads/{BRANCH}");
    assert_ne!(git_text(repo.as_path(), &["rev-parse", branch_reference.as_str()]).trim(), held_commit, "the branch still names the commit before the held one, as a lock keeps it");
    remove(root.as_path());
}

/// What:
///  A kept reference that already names another commit is left where it is,
///  and the person is told so.
/// Why:
///  The kept references are where commits that did not land are kept,
///  so replacing one would forget the commit it holds,
///  which is the owner's rule "never discard user data" again.
#[test]
fn a_kept_reference_that_names_another_commit_is_left_and_said() {
    let (root, repo, _linked, held_commit) = a_linked_worktree_with_a_held_commit("removal-held-commits-clash");
    let git_dir: PathBuf = repo.join(".git");
    let elsewhere: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let kept_name: String = the_kept_name(SEQ);
    git(repo.as_path(), &["update-ref", kept_name.as_str(), elsewhere.as_str()]);
    let kept = keep_held_commits(git_dir.as_path(), &context(git_dir.as_path()));
    assert!(kept.refused.is_empty(), "a name already taken is not a reason to stop: {:?}", kept.refused);
    assert_eq!(kept.said.len(), 1, "the person is told nothing was written: {:?}", kept.said);
    assert!(kept.said[0].contains("already names"), "the line says what stands in the way: {}", kept.said[0]);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", kept_name.as_str()]).trim(), elsewhere, "the commit an earlier batch was kept under stays named");
    assert_ne!(elsewhere, held_commit, "the control keeps two different commits");
    remove(root.as_path());
}

/// What:
///  A linked worktree whose checkout a person deleted, which is the state `git worktree prune` exists for,
///  still has its held commit kept before Git takes the registration and the per-worktree reference.
/// Why:
///  The question and the write ran Git with the deleted checkout as its working directory,
///  which Git cannot start in,
///  so the wrapper warned and handed the prune to Git,
///  which took the record, the per-worktree reference and then the commit itself
///  (a review of `ee545033ad`,
///  finding 1).
#[test]
fn a_held_commit_of_a_deleted_checkout_survives_the_prune() {
    let (root, repo, linked, held_commit) = a_linked_worktree_with_a_held_commit("removal-held-commits-checkout-gone");
    let git_dir: PathBuf = repo.join(".git");
    std::fs::remove_dir_all(linked.as_path()).expect("the checkout a person deleted");
    let kept = keep_held_commits(git_dir.as_path(), &context(git_dir.as_path()));
    assert!(kept.refused.is_empty(), "the common Git directory answers for a checkout that is gone: {:?}", kept.refused);
    assert_eq!(kept.said.len(), 1, "the person is told the commit is kept: {:?}", kept.said);
    let kept_name: String = the_kept_name(SEQ);
    assert!(kept.said[0].contains(kept_name.as_str()), "the line names where the commit is kept: {}", kept.said[0]);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", kept_name.as_str()]).trim(), held_commit, "the kept reference names the held commit");
    // What the removal does next: the registration and the per-worktree reference go, and Git's own maintenance follows.
    git(repo.as_path(), &["worktree", "prune", "--verbose"]);
    assert!(!git_dir.join("worktrees/linked").exists(), "the prune took the registration of the deleted checkout");
    git(repo.as_path(), &["gc", "--prune=now"]);
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-t", held_commit.as_str()]), "commit", "the commit is not lost with the registration");
    assert!(
        git_text(repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/worktree/"]).trim().is_empty(),
        "the per-worktree reference went with the registration, so the kept reference is the only holder"
    );
    remove(root.as_path());
}

/// What:
///  A lock standing on the batch's own kept reference keeps the commit on the reference beside it,
///  which a removal leaves alone, so the command still finishes and the commit survives it.
/// Why:
///  The write of the batch's kept reference was reported as failed and the removal ran anyway,
///  which deleted the per-worktree reference and then the commit
///  (a review of `ee545033ad`,
///  finding 2).
#[test]
fn a_held_commit_is_kept_beside_a_locked_kept_reference_and_survives_the_removal() {
    let (root, repo, linked, held_commit) = a_linked_worktree_with_a_held_commit("removal-held-commits-kept-locked");
    let git_dir: PathBuf = repo.join(".git");
    let kept_name: String = the_kept_name(SEQ);
    let rescue_name: String = the_rescue_name(SEQ);
    plant_lock(git_dir.as_path(), kept_name.as_str());
    let kept = keep_held_commits(git_dir.as_path(), &context(git_dir.as_path()));
    assert!(kept.refused.is_empty(), "a name beside the locked one keeps the commit: {:?}", kept.refused);
    assert_eq!(kept.said.len(), 1, "the person is told where the commit is kept: {:?}", kept.said);
    assert!(kept.said[0].contains(rescue_name.as_str()), "the line names the reference the commit is kept at: {}", kept.said[0]);
    assert!(kept.said[0].contains(kept_name.as_str()), "the line names the reference that could not be written: {}", kept.said[0]);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", rescue_name.as_str()]).trim(), held_commit, "the reference beside the locked one names the held commit");
    assert!(!git_dir.join(kept_name.as_str()).exists(), "the locked reference itself is not written");
    let linked_text: String = linked.display().to_string();
    git(repo.as_path(), &["worktree", "remove", "--force", linked_text.as_str()]);
    git(repo.as_path(), &["gc", "--prune=now"]);
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-t", held_commit.as_str()]), "commit", "the commit is not lost with the worktree");
    assert_eq!(
        git_text(repo.as_path(), &["for-each-ref", "--contains", held_commit.as_str(), "--format=%(refname)"]).trim(),
        rescue_name,
        "the reference beside the locked one is the one holder left"
    );
    remove(root.as_path());
}

/// What:
///  When neither the batch's kept reference nor the one beside it can be written,
///  the answer stops the command and says what is still held and where,
///  instead of letting Git delete the last references that name the commit.
/// Why:
///  The wrapper knew the write had failed and carried on into Git's removal anyway,
///  which deleted the commit
///  (a review of `ee545033ad`,
///  finding 2).
#[test]
fn a_removal_stops_when_no_reference_can_hold_the_held_commit() {
    let (root, repo, linked, held_commit) = a_linked_worktree_with_a_held_commit("removal-held-commits-nothing-keeps");
    let git_dir: PathBuf = repo.join(".git");
    let kept_name: String = the_kept_name(SEQ);
    let rescue_name: String = the_rescue_name(SEQ);
    plant_lock(git_dir.as_path(), kept_name.as_str());
    plant_lock(git_dir.as_path(), rescue_name.as_str());
    let kept = keep_held_commits(git_dir.as_path(), &context(git_dir.as_path()));
    assert!(kept.said.is_empty(), "nothing was kept, so no line claims it was: {:?}", kept.said);
    assert_eq!(kept.refused.len(), 2, "one paragraph for the commit and one for what the person does next: {:?}", kept.refused);
    assert!(kept.refused[0].contains(held_commit.as_str()), "the paragraph names the commit: {}", kept.refused[0]);
    assert!(kept.refused[0].contains(BRANCH), "the paragraph names the branch: {}", kept.refused[0]);
    assert!(kept.refused[0].contains(kept_name.as_str()), "the paragraph names the reference tried first: {}", kept.refused[0]);
    assert!(kept.refused[0].contains(rescue_name.as_str()), "the paragraph names the reference tried beside it: {}", kept.refused[0]);
    assert!(kept.refused[0].contains("could not be kept"), "the paragraph says nothing holds the commit: {}", kept.refused[0]);
    assert!(kept.refused[1].contains("nothing was removed"), "the closing paragraph says nothing changed: {}", kept.refused[1]);
    assert!(kept.refused[1].contains("run the same command again"), "the closing paragraph says what to do: {}", kept.refused[1]);
    assert!(kept.refused[1].contains(format!("{kept_name}.lock").as_str()), "the closing paragraph names the lock that stands there: {}", kept.refused[1]);
    assert!(kept.refused[1].contains("rm -- "), "the closing paragraph gives the command that removes a lock nothing holds: {}", kept.refused[1]);
    assert!(!git_dir.join(kept_name.as_str()).exists() && !git_dir.join(rescue_name.as_str()).exists(), "neither name was written");
    assert!(linked.is_dir(), "nothing removed the worktree the answer is about");
    assert_eq!(
        git_text(linked.as_path(), &["for-each-ref", "--contains", held_commit.as_str(), "--format=%(refname)"]).trim(),
        format!("refs/worktree/cli-git-service/{SEQ}"),
        "the worktree's own reference still names the commit the answer kept"
    );
    remove(root.as_path());
}

/// What:
///  A side-reference write leaves a name another command already wrote where it is,
///  and says what Git answered.
/// Why:
///  The removal keeps a held commit on a reference of the common Git directory,
///  and replacing one would forget the commit that reference was made for
///  (the owner's rule "never discard user data").
#[test]
fn a_side_reference_another_command_wrote_is_never_replaced() {
    let (root, repo, _linked, held_commit) = a_linked_worktree_with_a_held_commit("removal-held-commits-create-only");
    let git_dir: PathBuf = repo.join(".git");
    let elsewhere: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let name: String = the_kept_name(SEQ);
    git(repo.as_path(), &["update-ref", name.as_str(), elsewhere.as_str()]);
    let refused: Option<String> = keep_on_a_side_ref_only_if_free(&context(git_dir.as_path()), git_dir.as_path(), name.as_str(), held_commit.as_str());
    assert!(refused.is_some(), "a name that is taken is not written");
    assert!(refused.as_deref().unwrap_or_default().contains("cannot lock ref"), "Git's own words say why: {refused:?}");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", name.as_str()]).trim(), elsewhere, "the commit the name already held stays named");
    assert_ne!(elsewhere, held_commit, "the control keeps two different commits");
    remove(root.as_path());
}
