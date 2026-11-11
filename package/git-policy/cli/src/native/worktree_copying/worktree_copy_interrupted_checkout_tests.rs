//! What:
//!  The checkout an interrupted `git worktree add` never finished:
//!  finished when everything standing there came from the source,
//!  and left alone with its reason when anything else does.
//! Why:
//!  The pass replaces tracked files with Git's own checkout,
//! so a control that only showed the happy path would leave the promise
//!  "nothing of the person's is ever replaced" untested.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const end = await finishInterruptedCheckout({ commonDir, destination, source, context });
//! expect(end.kind).toBe('finished');
//! ```

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::os::unix::fs::MetadataExt as _;

/// What:
///  A repository with one tracked file,
/// and the worktree Git registered for it without a checkout,
/// which is the state an add killed inside its own checkout leaves.
/// Why:
///   The controls need the two objects the pass acts on,
/// and Git's own `--no-checkout` is the only way to get a registration without a checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(root, 'repo'); write(repo, 'a.txt', 'first\n'); git(repo, ['add', 'a.txt']); git(repo, ['commit', '-m', 'one']); git(repo, ['worktree', 'add', '--no-checkout', '-b', 'one', '../one']);
/// ```
fn interrupted_checkout(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"first\n").expect("tracked file");
    std::fs::create_dir_all(repo.join("deep")).expect("tracked directory");
    std::fs::write(repo.join("deep/b.txt"), b"second\n").expect("tracked file in a directory");
    git(repo.as_path(), &["add", "a.txt", "deep/b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["worktree", "add", "--no-checkout", "-b", "one", "../one"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("one")).expect("canonical destination");
    return (root, repo, destination);
}

/// The administrative directory Git made for a destination.
fn admin_of(repo: &Path, destination: &Path) -> PathBuf {
    return bind_registration(repo.join(".git").as_path(), destination).expect("bound").binding.admin;
}

/// The checkout a kill left incomplete is finished,
/// and the worktree then holds the tracked files and an index.
#[test]
fn the_checkout_a_kill_left_incomplete_is_finished() {
    let (root, repo, destination) = interrupted_checkout("checkout-incomplete");
    // The interrupted add had already made one of the source's files and no index.
    std::fs::copy(repo.join("a.txt"), destination.join("a.txt")).expect("the file the kill left");
    assert!(!add_is_finished(destination.as_path(), admin_of(repo.as_path(), destination.as_path()).as_path()), "the control starts from a checkout that is not whole");
    let end: CheckoutFinish = finish_interrupted_checkout(repo.join(".git").as_path(), destination.as_path(), repo.as_path(), &git_context(), true, &CopyCeiling::unbounded()).expect("the pass answers");
    assert!(matches!(end, CheckoutFinish::Finished(_)), "the checkout is finished: {end:?}");
    assert_eq!(std::fs::read(destination.join("a.txt")).expect("the tracked file"), b"first\n");
    assert_eq!(std::fs::read(destination.join("deep/b.txt")).expect("the tracked file below"), b"second\n", "every tracked file Git's checkout writes arrives");
    assert!(add_is_finished(destination.as_path(), admin_of(repo.as_path(), destination.as_path()).as_path()), "and the worktree holds an index now");
    remove(root.as_path());
}

/// A file the source does not hold at that name is the person's,
/// so nothing is written and the reason names it.
#[test]
fn a_file_the_source_does_not_hold_stops_the_checkout() {
    let (root, repo, destination) = interrupted_checkout("checkout-foreign-file");
    std::fs::write(destination.join("mine.txt"), b"a person's file").expect("the person's file");
    let end: CheckoutFinish = finish_interrupted_checkout(repo.join(".git").as_path(), destination.as_path(), repo.as_path(), &git_context(), true, &CopyCeiling::unbounded()).expect("the pass answers");
    match end {
        CheckoutFinish::NotProvable(reason) => assert!(reason.contains("mine.txt"), "the reason names the entry: {reason}"),
        other => panic!("a file the source does not hold stops the finish: {other:?}"),
    }
    assert_eq!(std::fs::read(destination.join("mine.txt")).expect("the file stays"), b"a person's file");
    assert!(!destination.join("a.txt").exists(), "and nothing else was written");
    remove(root.as_path());
}

/// A file at a tracked name that does not hold the source's bytes is the person's,
/// so it is not replaced by Git's checkout.
#[test]
fn a_file_of_other_bytes_stops_the_checkout() {
    let (root, repo, destination) = interrupted_checkout("checkout-other-bytes");
    std::fs::write(destination.join("a.txt"), b"what this person wrote").expect("the person's bytes");
    let end: CheckoutFinish = finish_interrupted_checkout(repo.join(".git").as_path(), destination.as_path(), repo.as_path(), &git_context(), true, &CopyCeiling::unbounded()).expect("the pass answers");
    assert!(matches!(end, CheckoutFinish::NotProvable(_)), "a file of other bytes stops the finish: {end:?}");
    assert_eq!(std::fs::read(destination.join("a.txt")).expect("the bytes stay"), b"what this person wrote");
    remove(root.as_path());
}

/// The lock a killed index install leaves is taken away when nothing holds it,
/// and Git's own checkout then completes the worktree.
#[test]
fn the_lock_a_killed_index_install_left_is_taken_away() {
    let (root, repo, destination) = interrupted_checkout("checkout-stale-lock");
    std::fs::copy(repo.join("a.txt"), destination.join("a.txt")).expect("the file the kill left");
    let admin: PathBuf = admin_of(repo.as_path(), destination.as_path());
    std::fs::write(admin.join("index.lock"), b"half an index").expect("the lock a kill left");
    assert!(!add_is_finished(destination.as_path(), admin.as_path()), "the lock alone keeps the checkout from being whole");
    let end: CheckoutFinish = finish_interrupted_checkout(repo.join(".git").as_path(), destination.as_path(), repo.as_path(), &git_context(), true, &CopyCeiling::unbounded()).expect("the pass answers");
    assert!(matches!(end, CheckoutFinish::Finished(_)), "the checkout is finished: {end:?}");
    assert!(!admin.join("index.lock").exists(), "the lock a dead add left is gone");
    assert!(admin.join("index").exists(), "and the index Git's own checkout wrote stands");
    remove(root.as_path());
}

/// A lock a running process holds open is that process's,
/// so it stays and the reason says so.
#[test]
fn a_lock_a_process_holds_open_stops_the_checkout() {
    let (root, repo, destination) = interrupted_checkout("checkout-held-lock");
    std::fs::copy(repo.join("a.txt"), destination.join("a.txt")).expect("the file the kill left");
    let admin: PathBuf = admin_of(repo.as_path(), destination.as_path());
    let lock: PathBuf = admin.join("index.lock");
    std::fs::write(lock.as_path(), b"a live writer's lock").expect("the lock");
    // A child that keeps the lock open, and says so once it has it.
    let ready: PathBuf = root.join("held");
    let mut holder: std::process::Child = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("exec 9>>{}; : > {}; sleep 60", lock.display(), ready.display()))
        .spawn()
        .expect("the holder starts");
    let opened: bool = (0..400).any(|_| {
        let there: bool = ready.exists();
        if !there {
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        return there;
    });
    assert!(opened, "the holder opened the lock");
    let end: Result<CheckoutFinish, WorktreeCopyError> = finish_interrupted_checkout(repo.join(".git").as_path(), destination.as_path(), repo.as_path(), &git_context(), true, &CopyCeiling::unbounded());
    let _ = holder.kill();
    let _ = holder.wait();
    let failure: WorktreeCopyError = end.expect_err("a lock a live process holds stops the finish");
    assert!(failure.0.contains("may hold it open"), "the reason says what stands in the way: {}", failure.0);
    assert!(lock.exists(), "the live writer's lock stays");
    assert!(!admin.join("index").exists(), "and nothing was written into the worktree");
    remove(root.as_path());
}

/// A destination that is not a registration of the repository is not this pass's to finish.
#[test]
fn a_directory_that_is_not_a_registration_is_not_finished() {
    let root: PathBuf = fixture("checkout-not-registered");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(elsewhere.as_path()).expect("a directory nothing registered");
    let end: CheckoutFinish = finish_interrupted_checkout(repo.join(".git").as_path(), std::fs::canonicalize(elsewhere.as_path()).expect("canonical").as_path(), repo.as_path(), &git_context(), true, &CopyCeiling::unbounded()).expect("the pass answers");
    assert!(matches!(end, CheckoutFinish::NotProvable(_)), "a directory Git did not register is not finished: {end:?}");
    assert_eq!(std::fs::read_dir(elsewhere.as_path()).expect("listed").count(), 0, "and nothing is written into it");
    remove(root.as_path());
}

/// The mode of a directory the walk reads is no proof and no bar:
/// a destination holding an empty directory the source does not hold is still finished,
/// because Git's own checkout leaves that directory exactly as it is.
#[test]
fn an_empty_directory_the_source_does_not_hold_does_not_stop_the_checkout() {
    let (root, repo, destination) = interrupted_checkout("checkout-empty-directory");
    std::fs::create_dir_all(destination.join("empty")).expect("an empty directory");
    let end: CheckoutFinish = finish_interrupted_checkout(repo.join(".git").as_path(), destination.as_path(), repo.as_path(), &git_context(), true, &CopyCeiling::unbounded()).expect("the pass answers");
    assert!(matches!(end, CheckoutFinish::Finished(_)), "an empty directory loses nothing and stops nothing: {end:?}");
    assert!(destination.join("empty").is_dir(), "the directory stays");
    assert_eq!(std::fs::read(destination.join("a.txt")).expect("the tracked file"), b"first\n");
    assert_eq!(std::fs::metadata(destination.join("a.txt")).expect("metadata").mode() & 0o777, 0o644, "and the checkout gives the file its own mode");
    remove(root.as_path());
}
