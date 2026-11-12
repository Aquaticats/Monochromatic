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
///  One more commit on the branch, pushed,
///  so the tracking reference names it,
///  and that commit.
/// Why:
///   A push or a fetch that lands between a push and its repair is the state the repair must not undo,
/// and the commit it leaves is what the controls read afterwards.
fn another_push(world: &World) -> String {
    std::fs::write(world.repo.join("c.txt"), "newer\n").expect("c.txt");
    git(world.repo.as_path(), &["add", "c.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=newer"]);
    git(world.repo.as_path(), &["push", "--quiet", "origin", "main"]);
    return another_push_commit(world);
}

/// What:
///  One more commit on the branch that is not pushed,
///  and that commit.
/// Why:
///   A commit that only the worktree has is what a repair reading the branch's name instead of the delivered commit would
///  publish.
fn a_commit_the_remote_never_received(world: &World) -> String {
    std::fs::write(world.repo.join("c.txt"), "local only\n").expect("c.txt");
    git(world.repo.as_path(), &["add", "c.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=local-only"]);
    return another_push_commit(world);
}

/// The commit `main` names now, which the last push delivered.
fn another_push_commit(world: &World) -> String {
    return git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
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

/// What:
///  A reference this program cannot write is named,
///  and the words say what becomes of it.
/// Why:
///   Silently leaving a reference behind is the state a later command cannot explain;
/// the sentence names the reference,
/// the commit the push
///  delivered and the one thing that writes the reference next.
#[test]
fn a_reference_that_could_not_be_written_is_named_with_what_writes_it() {
    let said: String = still_behind(TRACKING, "b".repeat(40).as_str());
    assert!(said.contains("could not update the tracking reference refs/remotes/origin/main"), "{said}");
    assert!(said.contains("until a push or a fetch writes it"), "{said}");
}

/// What:
///  The awaited repair writes the reference as the synchronous one does.
/// Why:
///   The service's push runs on the runtime,
/// where a synchronous Git start is refused,
/// so the face it uses is the awaited one and it must reach the same reference.
#[test]
fn the_awaited_repair_writes_the_reference_the_synchronous_one_does() {
    let world: World = world("tracking-ref-repair-awaiting");
    let git: PushGit = push_git(world.repo.as_path());
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    let resolved: Option<String> = runtime.block_on(resolve_awaiting(&git, "origin/main"));
    assert_eq!(resolved, Some(String::from(TRACKING)), "the awaited face resolves the same shorthand");
    let wrote: bool = runtime.block_on(bring_up_to_date_awaiting(&git, TRACKING, world.delivered.as_str()));
    assert!(wrote, "the awaited repair writes the reference");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference names what the remote holds");
    remove(world.root.as_path());
}

/// What:
///  A reference named the way a branch's upstream names it (`origin/main`) is written by its full name.
/// Why:
///   `git update-ref origin/main <commit>` is accepted and changes nothing
/// (measured with Git 2.56.0 on 2026-11-11),
/// so a repair that wrote the name it was given would report success while the reference stayed behind.
#[test]
fn a_reference_named_short_is_written_by_its_full_name() {
    let world: World = world("tracking-ref-repair-short-name");
    let git: PushGit = push_git(world.repo.as_path());
    assert_eq!(resolve(&git, "origin/main"), Some(String::from(TRACKING)), "the shorthand a branch's upstream is answered with is resolved to its ref");
    assert!(bring_up_to_date(&git, TRACKING, world.delivered.as_str()), "the full reference is written");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the full reference names what the remote holds");
    let about_origin: Vec<String> = git_text(world.repo.as_path(), &["for-each-ref", "--format=%(refname)"])
        .lines()
        .filter(|name| { return name.contains("origin"); })
        .map(str::to_owned)
        .collect();
    assert_eq!(about_origin, vec![String::from("refs/remotes/origin/main")], "no other reference took the short name");
    remove(world.root.as_path());
}

/// What:
///  A repair whose reference already names a newer commit than the one its push delivered leaves that commit where it is.
/// Why:
///   Another push or a fetch can advance the reference before this push's repair runs,
/// and a write there would name an older commit than the remote holds:
/// a published commit then reads as unpublished,
/// which is the stale held notice this repair exists to remove.
#[test]
fn a_repair_leaves_a_reference_that_already_names_a_newer_commit() {
    let world: World = world("tracking-ref-repair-newer-synchronous");
    let newer: String = another_push(&world);
    let git: PushGit = push_git(world.repo.as_path());
    assert!(bring_up_to_date(&git, TRACKING, world.delivered.as_str()), "a reference that holds a commit above the delivered one needs no write");
    assert_eq!(tracking(world.repo.as_path()), newer, "the synchronous repair leaves the newer commit another push wrote");
    remove(world.root.as_path());
}

/// What:
///  The awaited repair leaves a newer commit where the synchronous one does.
/// Why:
///   The two faces are one rule, and a reference rewound by the face the service uses names an older commit than the remote
///  holds.
#[test]
fn the_awaited_repair_leaves_a_reference_that_already_names_a_newer_commit() {
    let world: World = world("tracking-ref-repair-newer-awaited");
    let newer: String = another_push(&world);
    let git: PushGit = push_git(world.repo.as_path());
    assert!(bring_up_to_date(&git, TRACKING, world.delivered.as_str()), "the synchronous face reads the newer commit as already there");
    assert_eq!(tracking(world.repo.as_path()), newer, "the synchronous face leaves the newer commit");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    assert!(
        runtime.block_on(bring_up_to_date_awaiting(&git, TRACKING, world.delivered.as_str())),
        "the awaited face reads the same newer commit as already there"
    );
    assert_eq!(tracking(world.repo.as_path()), newer, "the awaited face leaves the newer commit the synchronous one leaves");
    remove(world.root.as_path());
}

/// What:
///  A repair asked to write a name instead of a commit writes nothing.
/// Why:
///   `git update-ref` resolves the value it is given at write time,
/// so a name that stands for the branch would publish a commit that landed after the push and never reached the remote,
/// and a later look would settle that commit's held record as a push that was made.
#[test]
fn a_repair_asked_for_a_name_instead_of_a_commit_writes_nothing() {
    let world: World = world("tracking-ref-repair-name-is-no-commit");
    // The branch is rewritten to a commit the remote never received, so a repair that resolved the name would publish it.
    git(world.repo.as_path(), &["reset", "--hard", world.before.as_str()]);
    let local_only: String = a_commit_the_remote_never_received(&world);
    assert_ne!(local_only, world.delivered, "the commit the branch names is not the one the remote holds");
    let git: PushGit = push_git(world.repo.as_path());
    assert!(!bring_up_to_date(&git, TRACKING, "refs/heads/main"), "the branch is no statement of what the push delivered");
    assert_eq!(tracking(world.repo.as_path()), world.before, "the reference still names what the remote holds");
    assert_ne!(tracking(world.repo.as_path()), local_only, "the reference never names the commit the remote never received");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    assert!(
        !runtime.block_on(bring_up_to_date_awaiting(&git, TRACKING, "refs/heads/main")),
        "the awaited face refuses a name as the synchronous one does"
    );
    assert_eq!(tracking(world.repo.as_path()), world.before, "the awaited face writes nothing either");
    remove(world.root.as_path());
}

/// What:
///  A local branch named the way the branch's upstream is abbreviated is never the reference the repair writes.
/// Why:
///   `git rev-parse` answers the short name `origin/main` from `refs/heads` as readily as from `refs/remotes`,
/// so a repair that took its first answer would move a person's own branch to the delivered commit
/// while the remote-tracking reference stayed where it was.
#[test]
fn a_local_branch_named_like_the_upstream_is_never_the_reference_written() {
    let world: World = world("tracking-ref-repair-local-branch-of-the-same-name");
    git(world.repo.as_path(), &["branch", "origin/main", world.before.as_str()]);
    git(world.repo.as_path(), &["update-ref", "-d", TRACKING]);
    let git: PushGit = push_git(world.repo.as_path());
    assert_eq!(resolve(&git, "origin/main"), None, "with the tracking reference gone this name is no reference of the repair's");
    assert_eq!(resolve(&git, "refs/heads/origin/main"), None, "a local branch is never resolved to a reference the repair writes");
    assert!(
        !bring_up_to_date(&git, "refs/heads/origin/main", world.delivered.as_str()),
        "the repair writes a remote-tracking reference and no other"
    );
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/heads/origin/main"]), world.before, "the person's branch is where they left it");
    let names: String = git_text(world.repo.as_path(), &["for-each-ref", "--format=%(refname)"]);
    assert!(!names.lines().any(|name| { return name == TRACKING; }), "no tracking reference was written under the name: {names}");
    remove(world.root.as_path());
}

/// What:
///  A tracking reference that is behind, beside a local branch of the same short name, is the one repaired.
/// Why:
///   This is the state the repair exists for,
/// and `git rev-parse` reads the ambiguous short name as neither reference,
/// so the derived remote-tracking name is what makes both the repair and the person's branch keep their commits.
#[test]
fn a_tracking_reference_beside_a_local_branch_of_the_same_name_is_the_one_repaired() {
    let world: World = world("tracking-ref-repair-beside-a-local-branch");
    git(world.repo.as_path(), &["branch", "origin/main", world.before.as_str()]);
    let git: PushGit = push_git(world.repo.as_path());
    assert_eq!(resolve(&git, "origin/main"), Some(String::from(TRACKING)), "the derived name is the tracking reference Git holds");
    assert!(bring_up_to_date(&git, TRACKING, world.delivered.as_str()), "the tracking reference is brought up to date");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference names what the remote holds");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/heads/origin/main"]),
        world.before,
        "the person's branch of the same short name keeps its commit"
    );
    remove(world.root.as_path());
}

/// What:
///  A name that is no reference writes nothing.
/// Why:
///   The write names what the caller asked for,
/// so a name this repository does not have must leave the repository exactly as it is
///  rather than create a reference nobody named.
#[test]
fn a_name_that_is_no_reference_is_not_written() {
    let world: World = world("tracking-ref-repair-not-a-ref");
    let git: PushGit = push_git(world.repo.as_path());
    assert_eq!(resolve(&git, "no-such-reference"), None, "a name this repository does not have is no reference");
    assert!(!bring_up_to_date(&git, "no-such-reference", world.delivered.as_str()), "nothing is written under it");
    assert_eq!(tracking(world.repo.as_path()), world.before, "the reference is as it was");
    let names: String = git_text(world.repo.as_path(), &["for-each-ref", "--format=%(refname)"]);
    assert!(!names.contains("no-such-reference"), "no reference was made under the name: {names}");
    remove(world.root.as_path());
}
