//! What:
//!  The collection of the repository scratch directory:
//!  that it happens beside the caller,
//!  that it takes what a killed command left,
//!  and that it keeps what a live command may still read.
//! Why:
//!  A collection the landing loop waited for would put back on every commit's path exactly the work this took off it;
//!  a collection that silently did nothing would leave the trees of killed commands to pile up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // start(common); expect(await waitUntilGone(stale)).toBe(true);
//! ```

/// The collection under control,
/// and the point a control holds one repository's collection at.
use super::{collection_point, start};
use crate::command_test_support::{fixture, remove};
use crate::service_fault::hooks::{HoldOnAnyThread, hold_on_any_thread};
use crate::private_storage::{PRIVATE_DIRECTORY_MODE, protect_path};
use crate::scratch_location::{REPOSITORY_SCRATCH_NAME, SCRATCH_STALE_AFTER};
use crate::service_scratch_trees_of_queued_requests::{note, release};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The longest a control waits for the collection to end.
const PATIENCE: Duration = Duration::from_secs(30);

/// Whether `path` is gone within the patience.
fn wait_until_gone(path: &Path) -> bool {
    let began: Instant = Instant::now();
    while path.exists() {
        if began.elapsed() >= PATIENCE {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    return true;
}

/// A repository scratch directory with one stale entry a killed command left and one fresh entry.
fn scratch_with(stale: &str, fresh: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture("repository-scratch-sweep");
    let directory: PathBuf = root.join(REPOSITORY_SCRATCH_NAME);
    std::fs::create_dir_all(&directory).expect("the repository scratch directory");
    protect_path(directory.as_path(), PRIVATE_DIRECTORY_MODE).expect("private");
    let stale_path: PathBuf = directory.join(stale);
    std::fs::create_dir_all(stale_path.join("shadow")).expect("a tree a killed command left");
    std::fs::write(stale_path.join("shadow/HEAD"), b"ref: refs/heads/main\n").expect("a file in the tree");
    std::fs::File::open(&stale_path)
        .expect("open")
        .set_modified(std::time::SystemTime::now() - SCRATCH_STALE_AFTER * 2)
        .expect("an abandoned tree's age");
    std::fs::write(directory.join(fresh), b"x").expect("a fresh entry");
    return (root, directory);
}

/// A second repository scratch directory,
/// of a fixture of its own,
/// holding one entry a killed command left and nothing else.
fn another_scratch_with(name: &str, stale: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let directory: PathBuf = root.join(REPOSITORY_SCRATCH_NAME);
    std::fs::create_dir_all(directory.as_path()).expect("the repository scratch directory");
    protect_path(directory.as_path(), PRIVATE_DIRECTORY_MODE).expect("private");
    let stale_path: PathBuf = directory.join(stale);
    std::fs::write(stale_path.as_path(), b"x").expect("an entry a killed command left");
    std::fs::File::open(stale_path.as_path())
        .expect("open")
        .set_modified(std::time::SystemTime::now() - SCRATCH_STALE_AFTER * 2)
        .expect("an abandoned entry's age");
    return (root, directory);
}

/// A collection started beside the caller takes a tree a killed command left,
/// and the caller does not wait for it.
#[test]
fn the_collection_takes_what_a_killed_command_left_beside_the_caller() {
    let (root, directory) = scratch_with("monochromatic-git-policy-cli-service-0a1b2c3d", "cli-git-input-fresh");
    let began: Instant = Instant::now();
    start(root.as_path());
    assert!(began.elapsed() < Duration::from_secs(1), "the caller returned after {:?}", began.elapsed());
    let abandoned: PathBuf = directory.join("monochromatic-git-policy-cli-service-0a1b2c3d");
    assert!(wait_until_gone(abandoned.as_path()), "an abandoned tree of a killed command goes");
    assert!(directory.join("cli-git-input-fresh").exists(), "an entry that is not old enough stays");
    remove(root.as_path());
}

/// A common directory whose scratch directory was never made is no failure and no line.
#[test]
fn a_repository_without_a_scratch_directory_ends_quietly() {
    let root: PathBuf = fixture("repository-scratch-sweep-absent");
    start(root.as_path());
    assert!(!root.join(REPOSITORY_SCRATCH_NAME).exists(), "a collection makes nothing");
    remove(root.as_path());
}

/// The identity the control's request is queued under.
const A_QUEUED_REQUEST: &str = "repository-scratch-sweep-queued-request";

/// What:
///  A collection takes the tree of a command that is gone,
///  and keeps the tree a request this service still holds lands on,
///  with the change that request captured in it.
/// Why:
///   A client killed while its request is queued renews nothing,
///  so a landing that waited longer than `SCRATCH_TREE_ABANDONED_AFTER` before it takes the request
///  finds a tree the age rule would remove,
///  and the landing reads the private index and the conclusion copies in exactly that tree
///  (`service_scratch_trees_of_queued_requests.rs`).
///  The tree goes back to the age rule when the service is done with the request,
///  so the entry a killed command left is still collected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// note(request, queued); start(common); expect(shadowOf(queued)).toBeThere(); release(request); expect(await waitUntilGone(queued)).toBe(true);
/// ```
#[test]
fn the_collection_keeps_the_tree_a_request_the_service_still_holds() {
    let (root, directory) = scratch_with("monochromatic-git-policy-cli-service-0a1b2c3e", "cli-git-input-fresh");
    let queued: PathBuf = directory.join("monochromatic-git-policy-cli-service-0a1b2c3e");
    // A second tree of the same age whose command is gone, so one collection decides both and the control can tell that it ran.
    let orphaned: PathBuf = directory.join("monochromatic-git-policy-cli-service-0a1b2c3f");
    std::fs::create_dir_all(orphaned.as_path()).expect("a tree a killed command left");
    std::fs::write(orphaned.join("commit.index"), b"an index nothing reads").expect("a file in the tree");
    std::fs::File::open(orphaned.as_path()).expect("open").set_modified(std::time::SystemTime::now() - SCRATCH_STALE_AFTER * 2).expect("an abandoned tree's age");
    note(A_QUEUED_REQUEST, queued.as_path());
    start(root.as_path());
    assert!(wait_until_gone(orphaned.as_path()), "the tree of a command that is gone goes");
    assert_eq!(
        std::fs::read(queued.join("shadow/HEAD")).expect("the tree a queued request lands on is still there"),
        b"ref: refs/heads/main\n",
        "the tree keeps the change the request captured"
    );
    // The collection is over once the tree beside it is gone; asking again until the entry goes keeps the control from racing it.
    release(A_QUEUED_REQUEST);
    let began: Instant = Instant::now();
    while queued.exists() {
        assert!(began.elapsed() < PATIENCE, "a tree no request holds any more goes by the age rule");
        start(root.as_path());
        std::thread::sleep(Duration::from_millis(50));
    }
    remove(root.as_path());
}

/// What:
///  A collection of one repository's scratch directory does not stop the collection of another repository's.
/// Why:
///   The development service serves several common Git directories at once (`instance` is `None`,
///  `service_server.rs`),
///  and two linked worktrees of one repository are two services over one scratch directory each.
///  A landing of repository B must collect B while repository A is collecting A,
///  because nothing retries B's collection after A's ends:
///  B's stale entries then wait for a landing that may never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hold(collectionPoint(a)); start(a); reached; start(b); expect(await waitUntilGone(staleOf(b))).toBe(true); letGo;
/// ```
#[test]
fn a_collection_of_one_repository_does_not_stop_the_collection_of_another() {
    let (root_a, directory_a) = scratch_with("monochromatic-git-policy-cli-service-0a1b2c4d", "cli-git-input-fresh");
    let (root_b, directory_b) = another_scratch_with("repository-scratch-sweep-second", "cli-git-input-stale");
    let stale_a: PathBuf = directory_a.join("monochromatic-git-policy-cli-service-0a1b2c4d");
    let stale_b: PathBuf = directory_b.join("cli-git-input-stale");
    // Repository A's collection is held at its beginning, so "while A collects" is a state and not a duration.
    let (reached, reached_wait) = tokio::sync::oneshot::channel::<()>();
    let (let_go, held) = std::sync::mpsc::channel::<()>();
    hold_on_any_thread(HoldOnAnyThread { point: collection_point(root_a.as_path()), reached, let_go: held });
    start(root_a.as_path());
    reached_wait.blocking_recv().expect("A's collection reached the point this control holds");
    // B's landing asks for its own collection while A's is still running.
    start(root_b.as_path());
    assert!(wait_until_gone(stale_b.as_path()), "repository B's landing collects B while repository A collects A");
    // A's own collection ends when the control lets it go, and takes A's entry with it.
    let _let_go = let_go.send(());
    assert!(wait_until_gone(stale_a.as_path()), "A's collection ends and takes its own entry");
    remove(root_a.as_path());
    remove(root_b.as_path());
}
