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

/// The collection under control.
use super::start;
use crate::command_test_support::{fixture, remove};
use crate::private_storage::{PRIVATE_DIRECTORY_MODE, protect_path};
use crate::scratch_location::{REPOSITORY_SCRATCH_NAME, SCRATCH_STALE_AFTER};
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
