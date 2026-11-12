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
use crate::service_fault::hooks;
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

/// Every reference the repository has.
fn references(repo: &Path) -> String {
    return git_text(repo, &["for-each-ref", "--format=%(refname)"]);
}

/// The lock of the tracking reference,
/// as a Git that was killed leaves one,
/// with the directory that holds it made when the reference is not there.
fn the_lock_of_the_reference(repo: &Path) -> PathBuf {
    let lock: PathBuf = repo.join(".git").join(format!("{TRACKING}.lock"));
    std::fs::create_dir_all(lock.parent().expect("the lock's directory")).expect("the lock's directory");
    return lock;
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
    assert_eq!(bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()), None, "the repair names the delivered commit");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference names what the remote holds");
    // The row Git writes for a push is the row this write leaves, so a reader of the reflog cannot tell the two apart.
    let reflog: String = git_text(world.repo.as_path(), &["reflog", "show", "--format=%gs", TRACKING]);
    assert!(reflog.contains(UPDATE_BY_PUSH), "the reflog names the update as a push's: {reflog}");
    remove(world.root.as_path());
}

/// What:
///  A reference that is not there is created,
///  as Git creates it on a push whose fetch refspec maps the branch,
///  and nothing else under that name is made.
/// Why:
///   A push whose reference update another Git's lock refused can leave no reference at all,
///  where the branch's upstream was read before the reference went;
/// every later question about the branch's remote then answers nothing,
///  and Git itself creates the reference on the next push it can update.
#[test]
fn a_tracking_reference_that_is_absent_is_created() {
    let world: World = world("tracking-ref-repair-absent");
    git(world.repo.as_path(), &["update-ref", "-d", TRACKING]);
    let git: PushGit = push_git(world.repo.as_path());
    assert_eq!(resolve(&git, "origin/main"), Some(String::from(TRACKING)), "the name of the reference is known while the reference is not there");
    assert_eq!(bring_up_to_date(&git, TRACKING, world.delivered.as_str()), None, "the absent reference is created");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the created reference names what the remote holds");
    let reflog: String = git_text(world.repo.as_path(), &["reflog", "show", "--format=%gs", TRACKING]);
    assert!(reflog.contains(UPDATE_BY_PUSH), "the created reference has a push's reflog row: {reflog}");
    assert_eq!(references(world.repo.as_path()).lines().filter(|name| { return name.contains("origin"); }).collect::<Vec<&str>>(), vec![TRACKING], "no other reference was made");
    remove(world.root.as_path());
}

/// What:
///  The awaited repair creates an absent reference as the synchronous one does.
/// Why:
///   The service's push runs on the runtime,
/// so the face it uses must do what the command's face does,
///  including the creation Git itself makes.
#[test]
fn the_awaited_repair_creates_a_reference_that_is_absent() {
    let world: World = world("tracking-ref-repair-absent-awaiting");
    git(world.repo.as_path(), &["update-ref", "-d", TRACKING]);
    let git: PushGit = push_git(world.repo.as_path());
    let runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    assert_eq!(
        runtime.block_on(bring_up_to_date_awaiting(&git, TRACKING, world.delivered.as_str())),
        None,
        "the awaited face creates the absent reference"
    );
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the created reference names what the remote holds");
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
    let lock: PathBuf = the_lock_of_the_reference(world.repo.as_path());
    std::fs::write(lock.as_path(), "").expect("a held lock");
    assert_eq!(bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()), None, "nothing had to be written");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference is as it was");
    remove(world.root.as_path());
}

/// What:
///  A reference whose lock still stands is reported as a lock,
///  and it is written once the lock goes.
/// Why:
///   The push delivered either way,
/// so a repair that cannot write is a line and never a failed push,
/// and a later look must find the reference writable again.
#[test]
fn a_tracking_reference_whose_lock_stands_is_reported_and_then_written() {
    let world: World = world("tracking-ref-repair-locked");
    let lock: PathBuf = the_lock_of_the_reference(world.repo.as_path());
    std::fs::write(lock.as_path(), "").expect("a held lock");
    assert_eq!(
        bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()),
        Some(Impediment::Locked),
        "the lock refused the write, and the answer names the lock"
    );
    assert_eq!(tracking(world.repo.as_path()), world.before, "the reference is where the refused push left it");
    std::fs::remove_file(lock.as_path()).expect("the lock goes");
    assert_eq!(
        bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()),
        None,
        "the reference is written once the lock is gone"
    );
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference names what the remote holds");
    remove(world.root.as_path());
}

/// What:
///  A reference whose lock stands is a reference that is not there at all,
///  and the creation meets the same lock.
/// Why:
///   The absent reference is created by a compare-and-swap,
/// which takes the same lock another Git holds,
/// so the answer must name the lock rather than report a reference that was made.
#[test]
fn an_absent_reference_whose_lock_stands_is_reported_as_a_lock() {
    let world: World = world("tracking-ref-repair-absent-locked");
    git(world.repo.as_path(), &["update-ref", "-d", TRACKING]);
    let lock: PathBuf = the_lock_of_the_reference(world.repo.as_path());
    std::fs::write(lock.as_path(), "").expect("a held lock");
    assert_eq!(
        bring_up_to_date(&push_git(world.repo.as_path()), TRACKING, world.delivered.as_str()),
        Some(Impediment::Locked),
        "the absent reference could not be created while another Git held its lock"
    );
    assert!(!references(world.repo.as_path()).lines().any(|name| { return name == TRACKING; }), "no reference was made");
    remove(world.root.as_path());
}

/// What:
///  Every refusal of this repair says what stopped it,
///  and only the lock is called a lock.
/// Why:
///   A person who reads "another Git holds its lock" when no lock stands is told to wait for something that will not
///  end,
/// and the other states have their own cause and their own next step.
#[test]
fn a_repair_that_could_not_write_says_what_stopped_it() {
    let delivered: String = "b".repeat(40);
    let locked: String = still_behind(TRACKING, delivered.as_str(), Impediment::Locked);
    assert!(locked.contains("another Git held the lock"), "{locked}");
    assert!(locked.contains(TRACKING), "{locked}");
    let other: String = still_behind(TRACKING, delivered.as_str(), Impediment::AnotherWriter);
    assert!(other.contains("another process wrote"), "{other}");
    assert!(!other.contains("lock"), "a value another writer keeps is no lock: {other}");
    let unreadable: String = still_behind(TRACKING, delivered.as_str(), Impediment::Unreadable);
    assert!(unreadable.contains("could not read"), "{unreadable}");
    assert!(!unreadable.contains("lock"), "a reference Git cannot read is no lock: {unreadable}");
    let not_tracking: String = still_behind("origin/main", delivered.as_str(), Impediment::NotATrackingReference);
    assert!(not_tracking.contains("is not a remote-tracking reference"), "{not_tracking}");
    let not_a_commit: String = still_behind(TRACKING, "refs/heads/main", Impediment::NotACommit);
    assert!(not_a_commit.contains("is not a commit name"), "{not_a_commit}");
    for said in [&locked, &other, &unreadable, &not_tracking, &not_a_commit] {
        assert!(said.starts_with("the push delivered"), "{said}");
    }
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
    let wrote: Option<Impediment> = runtime.block_on(bring_up_to_date_awaiting(&git, TRACKING, world.delivered.as_str()));
    assert_eq!(wrote, None, "the awaited repair writes the reference");
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
    assert_eq!(bring_up_to_date(&git, TRACKING, world.delivered.as_str()), None, "the full reference is written");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the full reference names what the remote holds");
    let about_origin: Vec<String> =
        git_text(world.repo.as_path(), &["for-each-ref", "--format=%(refname)"]).lines().filter(|name| { return name.contains("origin"); }).map(str::to_owned).collect();
    assert_eq!(about_origin, vec![String::from(TRACKING)], "no other reference took the short name");
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
    assert_eq!(
        bring_up_to_date(&git, TRACKING, world.delivered.as_str()),
        None,
        "a reference that holds a commit above the delivered one needs no write"
    );
    assert_eq!(tracking(world.repo.as_path()), newer, "the synchronous repair leaves the newer commit another push wrote");
    remove(world.root.as_path());
}

/// What:
///  A reference another writer moves at the instant this repair reads it is left where that writer put it.
/// Why:
///   The repair reads the reference once and asks its ancestry question about that value,
/// so a value that arrives between the look and the write is the value it decides about.
/// A repair that decided about the reference and then read it again would write the delivered commit over the newer one,
/// which is the rewind this control stops the code in the middle of:
/// the hook is another push or a fetch,
/// and it runs at the instant before the read.
#[test]
fn a_repair_leaves_a_reference_another_writer_moved_before_its_read() {
    let world: World = world("tracking-ref-repair-moved-before-the-read");
    let newer: String = another_push(&world);
    // The reference is put back where the refused push left it, so the move the hook makes is the only one that arrives.
    git(world.repo.as_path(), &["update-ref", TRACKING, world.before.as_str()]);
    let (hook_repo, hook_value): (PathBuf, String) = (world.repo.clone(), newer.clone());
    hooks::set(
        TRACKING_REF_BEFORE_THE_READ,
        std::rc::Rc::new(move || {
            // As a fetch or another push does: the reference is advanced between the reader's look and its write.
            git(hook_repo.as_path(), &["update-ref", TRACKING, hook_value.as_str()]);
        }),
    );
    let git: PushGit = push_git(world.repo.as_path());
    let _answer: Option<Impediment> = bring_up_to_date(&git, TRACKING, world.delivered.as_str());
    hooks::clear();
    assert_eq!(tracking(world.repo.as_path()), newer, "the reference keeps the commit another writer put there rather than moving back to an older one");
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
    assert_eq!(bring_up_to_date(&git, TRACKING, world.delivered.as_str()), None, "the synchronous face reads the newer commit as already there");
    assert_eq!(tracking(world.repo.as_path()), newer, "the synchronous face leaves the newer commit");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    assert_eq!(
        runtime.block_on(bring_up_to_date_awaiting(&git, TRACKING, world.delivered.as_str())),
        None,
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
    assert_eq!(
        bring_up_to_date(&git, TRACKING, "refs/heads/main"),
        Some(Impediment::NotACommit),
        "the branch is no statement of what the push delivered"
    );
    assert_eq!(tracking(world.repo.as_path()), world.before, "the reference still names what the remote holds");
    assert_ne!(tracking(world.repo.as_path()), local_only, "the reference never names the commit the remote never received");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    assert_eq!(
        runtime.block_on(bring_up_to_date_awaiting(&git, TRACKING, "refs/heads/main")),
        Some(Impediment::NotACommit),
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
    assert_eq!(resolve(&git, "refs/heads/origin/main"), None, "a local branch is never resolved to a reference the repair writes");
    assert_eq!(
        bring_up_to_date(&git, "refs/heads/origin/main", world.delivered.as_str()),
        Some(Impediment::NotATrackingReference),
        "the repair writes a remote-tracking reference and no other"
    );
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/heads/origin/main"]), world.before, "the person's branch is where they left it");
    let names: String = references(world.repo.as_path());
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
    assert_eq!(bring_up_to_date(&git, TRACKING, world.delivered.as_str()), None, "the tracking reference is brought up to date");
    assert_eq!(tracking(world.repo.as_path()), world.delivered, "the reference names what the remote holds");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/heads/origin/main"]),
        world.before,
        "the person's branch of the same short name keeps its commit"
    );
    remove(world.root.as_path());
}

/// What:
///  A name that is not a remote-tracking reference is answered by nothing and written by nothing.
/// Why:
///   The repair writes the branch's remote-tracking reference and no other,
/// so a name outside `refs/remotes/` must leave the repository exactly as it is
///  rather than create a reference nobody named.
#[test]
fn a_name_that_is_no_tracking_reference_is_not_written() {
    let world: World = world("tracking-ref-repair-not-a-ref");
    let git: PushGit = push_git(world.repo.as_path());
    assert_eq!(resolve(&git, "refs/heads/main"), None, "a local branch is not a remote-tracking reference");
    assert_eq!(resolve(&git, "refs/tags/v1"), None, "a tag is not a remote-tracking reference");
    assert_eq!(
        bring_up_to_date(&git, "no-such-reference", world.delivered.as_str()),
        Some(Impediment::NotATrackingReference),
        "nothing is written under a name that is no reference of this repair's"
    );
    assert_eq!(tracking(world.repo.as_path()), world.before, "the reference is as it was");
    let names: String = references(world.repo.as_path());
    assert!(!names.contains("no-such-reference"), "no reference was made under the name: {names}");
    remove(world.root.as_path());
}
