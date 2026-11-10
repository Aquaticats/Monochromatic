//! Controls for the proof that a held push's commit has reached the branch's remote,
//! against real repositories and bare remotes.
//! What the proof says decides whether a command keeps saying a commit is held,
//! so each answer is checked where it is made:
//! on the local repository alone,
//! with the remote-tracking ref Git's own push updates.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove};
use std::path::{Path, PathBuf};

/// A bare remote,
/// the repository that tracks it,
/// and the context the questions are asked through.
struct World {
    /// The fixture root,
    /// removed when the control ends.
    root: PathBuf,
    /// The bare remote named `origin`.
    remote: PathBuf,
    /// The repository under test,
    /// with `main` pushed and tracking `origin/main`.
    repo: PathBuf,
}

fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let remote: PathBuf = root.join("remote.git");
    git(&root, &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(&repo, &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a.txt");
    git(&repo, &["add", "a.txt"]);
    git(&repo, &["commit", "--quiet", "--message=base"]);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    return World { root, remote, repo };
}

/// The proof asked of the world's repository.
fn proof<'a>(context: &'a crate::transaction_git::GitContext, world: &'a World) -> GitRemoteRecord<'a> {
    return GitRemoteRecord { context, cwd: world.repo.as_path() };
}

/// The commit `HEAD` names.
fn head(repo: &Path) -> String {
    return git_text(repo, &["rev-parse", "HEAD"]);
}

/// What:
///  A commit the branch's push carried is in the remote-tracking ref the push updates,
/// so the proof answers "there".
/// Why:
///   This is the state a held record is settled in:
/// the branch moved,
/// its push delivered the held commit with it,
/// and the
///  record would otherwise say the opposite of `git status`.
#[test]
fn a_commit_a_later_push_carried_is_on_the_remote() {
    let world: World = world("proof-carried");
    let context: crate::transaction_git::GitContext = git_context();
    let held: String = head(world.repo.as_path());
    std::fs::write(world.repo.join("b.txt"), "later\n").expect("b.txt");
    git(&world.repo, &["add", "b.txt"]);
    git(&world.repo, &["commit", "--quiet", "--message=later"]);
    git(&world.repo, &["push", "--quiet", "origin", "main"]);
    assert_eq!(proof(&context, &world).on_the_remote("refs/heads/main", held.as_str()), OnTheRemote::There);
    // Git's own status agrees: nothing of the branch is ahead of the remote.
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--branch"]), "## main...origin/main");
    remove(world.root.as_path());
}

/// What:
///  A commit no push carried is not in the remote-tracking ref,
/// so the proof answers "not there".
/// Why:
///   This is the state the held line is true in,
/// and the record must not be settled.
#[test]
fn a_commit_no_push_carried_is_not_on_the_remote() {
    let world: World = world("proof-not-carried");
    let context: crate::transaction_git::GitContext = git_context();
    std::fs::write(world.repo.join("b.txt"), "local\n").expect("b.txt");
    git(&world.repo, &["add", "b.txt"]);
    git(&world.repo, &["commit", "--quiet", "--message=local"]);
    let held: String = head(world.repo.as_path());
    assert_eq!(proof(&context, &world).on_the_remote("refs/heads/main", held.as_str()), OnTheRemote::NotThere);
    remove(world.root.as_path());
}

/// What:
///  A branch with no upstream leaves nothing to ask,
/// so the proof answers "nothing is known" and no record is settled.
/// Why:
///   The question is Git's own answer about the branch's upstream ref;
/// a branch that tracks nothing has no such ref,
/// and a
///  guess from the settings would settle a record on a ref nobody pushed to.
#[test]
fn a_branch_with_no_upstream_is_unknown() {
    let world: World = world("proof-no-upstream");
    let context: crate::transaction_git::GitContext = git_context();
    git(&world.repo, &["branch", "--quiet", "solo"]);
    assert_eq!(proof(&context, &world).on_the_remote("refs/heads/solo", head(world.repo.as_path()).as_str()), OnTheRemote::Unknown);
    // A ref outside `refs/heads` has no upstream either.
    assert_eq!(proof(&context, &world).on_the_remote("refs/tags/main", head(world.repo.as_path()).as_str()), OnTheRemote::Unknown);
    remove(world.root.as_path());
}

/// What:
///  A commit that does not exist in this repository leaves the question unanswered,
/// never "not there".
/// Why:
///   A `merge-base` that fails for a missing object is not the answer "the remote lacks it",
/// and settling
///  or not settling on it would both be a guess.
#[test]
fn a_commit_this_repository_does_not_have_is_unknown() {
    let world: World = world("proof-missing-object");
    let context: crate::transaction_git::GitContext = git_context();
    assert_eq!(proof(&context, &world).on_the_remote("refs/heads/main", "0123456789abcdef0123456789abcdef01234567"), OnTheRemote::Unknown);
    remove(world.root.as_path());
}

/// What:
///  A remote-tracking ref that lags the remote answers "not there",
/// which is the limit the proof is worded with.
/// Why:
///   A push whose reference update was refused still delivered the commits,
/// and the wrapper asks no remote,
/// so
///  the local record is all there is;
/// a lagging one leaves the held line standing,
/// which costs a line and
///  loses nothing.
#[test]
fn a_remote_tracking_ref_that_lags_the_remote_is_the_proof_s_limit() {
    let world: World = world("proof-lagging");
    let context: crate::transaction_git::GitContext = git_context();
    std::fs::write(world.repo.join("b.txt"), "later\n").expect("b.txt");
    git(&world.repo, &["add", "b.txt"]);
    git(&world.repo, &["commit", "--quiet", "--message=later"]);
    let carried: String = head(world.repo.as_path());
    git(&world.repo, &["push", "--quiet", "origin", "main"]);
    // A record that never got the new tip, as a refused reference update leaves it.
    let stale: String = git_text(&world.remote, &["rev-parse", "refs/heads/main~1"]);
    git(&world.repo, &["update-ref", "refs/remotes/origin/main", stale.as_str()]);
    assert_eq!(proof(&context, &world).on_the_remote("refs/heads/main", carried.as_str()), OnTheRemote::NotThere);
    remove(world.root.as_path());
}
