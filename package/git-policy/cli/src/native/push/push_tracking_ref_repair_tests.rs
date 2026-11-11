//! Controls for the tracking reference a successful push leaves behind,
//! against real repositories with a bare remote.
//! What:
//!  Git delivers a push and can then be refused the update of the branch's remote-tracking reference,
//!  which leaves it naming an older commit while the remote already holds the pushed one.
//! Why:
//!  Every later question about the branch's remote is answered from that reference,
//! so a stale one says a published commit is held.

use super::*;
use crate::auto_push_single_flight::PushGit;
use crate::test_support::{REAL_GIT, fixture, git, git_text, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The reference every push of `main` updates.
const TRACKING: &str = "refs/remotes/origin/main";

/// The Git the repair runs,
/// with the environment the service gives it.
fn push_git(root: &Path) -> PushGit {
    let mut environment: Vec<(OsString, OsString)> = [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ]
    .iter()
    .map(|(name, value)| {
        return (OsString::from(name), OsString::from(value));
    })
    .collect();
    environment.extend(crate::test_support::scratch_environment());
    return PushGit { real_git: PathBuf::from(REAL_GIT), cwd: root.to_path_buf(), environment, lfs: false };
}

/// A repository whose `main` was pushed to its bare remote,
/// with the commit the reference named before the second one.
struct World {
    /// The fixture root,
    /// removed when the control ends.
    root: PathBuf,
    /// The repository with its bare remote.
    repo: PathBuf,
    /// The commit `main` named before the second one.
    before: String,
    /// The commit the push delivered,
    /// and which the remote holds.
    delivered: String,
}

/// The world:
/// `main` pushed once,
/// a second commit made and pushed,
/// and the tracking reference made to name the first commit,
/// which is the state Git leaves behind when another Git holds the reference's lock and it exits `0` all the same.
fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a.txt");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let before: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("b.txt"), "later\n").expect("b.txt");
    git(repo.as_path(), &["add", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=later"]);
    git(repo.as_path(), &["push", "--quiet", "origin", "main"]);
    let delivered: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-ref", TRACKING, before.as_str()]);
    return World { root, repo, before, delivered };
}

/// What the reference names now.
fn tracking(repo: &Path) -> String {
    return git_text(repo, &["rev-parse", TRACKING]);
}

/// What:
///  A reference that names an older commit than the one the push delivered is made to name the delivered commit.
/// Why:
///   This is the write Git refused,
/// and the wrapper's own push knows exactly what it sent.
#[test]
fn a_tracking_reference_behind_the_delivered_commit_is_brought_up_to_date() {
    let world: World = world("tracking-ref-repair-behind");
    assert_eq!(tracking(world.repo.as_path()), world.before, "the reference is behind before the repair");
    assert!(bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()), "the repair names the delivered commit");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference names what the remote holds");
    // The row Git writes for a push is the row this write leaves, so a reader of the reflog cannot tell the two apart.
    let reflog: String = git_text(world.repo.as_path(), &["reflog", "show", "--format=%gs", TRACKING]);
    assert!(reflog.contains(UPDATE_BY_PUSH), "the reflog names the update as a push's: {reflog}");
    remove(world.root.as_path());
}

/// What:
///  A reference that already holds the delivered commit is left alone,
///  even while its lock stands.
/// Why:
///   An update there is the write Git itself made,
/// and a needless write that meets a held lock would answer that the reference is behind when it is not.
#[test]
fn a_tracking_reference_that_already_holds_the_commit_is_left_alone() {
    let world: World = world("tracking-ref-repair-already-there");
    git(world.repo.as_path(), &["update-ref", TRACKING, world.delivered.as_str()]);
    let lock: PathBuf = world.repo.join(".git").join(format!("{TRACKING}.lock"));
    std::fs::write(lock.as_path(), "").expect("a held lock");
    assert!(bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()), "nothing had to be written");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference is as it was");
    remove(world.root.as_path());
}

/// What:
///  A reference whose lock still stands is reported,
///  and it is written once the lock goes.
/// Why:
///   The push delivered either way,
/// so a repair that cannot write is a line and never a failed push,
/// and a later look must find the reference writable again.
#[test]
fn a_tracking_reference_whose_lock_stands_is_reported_and_then_written() {
    let world: World = world("tracking-ref-repair-locked");
    let lock: PathBuf = world.repo.join(".git").join(format!("{TRACKING}.lock"));
    std::fs::write(lock.as_path(), "").expect("a held lock");
    assert!(!bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()), "the lock refused the write");
    assert_eq!(tracking(world.repo.as_path()), world.before, "the reference is where the refused push left it");
    std::fs::remove_file(lock.as_path()).expect("the lock goes");
    assert!(bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()), "the reference is written once the lock is gone");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference names what the remote holds");
    remove(world.root.as_path());
}
