//! What:
//!  The removal of a released scratch directory:
//!  that it happens beside the caller,
//!  that it happens at all,
//!  and that it names nothing it was not given.
//! Why:
//!  A removal waited for in the landing loop would make every queued commit wait for the device,
//!  which is what taking it off the person's path was for;
//!  a removal that silently did nothing would leave the trees to pile up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // start(tree); expect(await waitUntilGone(tree)).toBe(true);
//! ```

/// The removal under control.
use super::start;
use crate::command_test_support::{fixture, remove};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The longest a control waits for the removal to end.
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

/// A released scratch directory goes,
/// with what it held,
/// and the caller does not wait for it.
#[test]
fn a_released_scratch_directory_is_removed_beside_the_caller() {
    let root: PathBuf = fixture("release-removal");
    let tree: PathBuf = root.join("monochromatic-git-policy-cli-service-0a1b2c3d");
    std::fs::create_dir_all(tree.join("shadow/objects")).expect("the tree");
    std::fs::write(tree.join("commit.index"), b"index").expect("a file in the tree");
    let began: Instant = Instant::now();
    start(tree.clone());
    assert!(began.elapsed() < Duration::from_secs(1), "the caller returned after {:?}", began.elapsed());
    assert!(wait_until_gone(tree.as_path()), "the tree is still there");
    assert!(root.exists(), "only the tree goes");
    remove(root.as_path());
}

/// A directory that is already gone is no failure and no line.
#[test]
fn a_directory_that_is_already_gone_ends_quietly() {
    let root: PathBuf = fixture("release-removal-gone");
    std::fs::create_dir_all(root.as_path()).expect("the fixture");
    start(root.join("absent"));
    assert!(!root.join("absent").exists());
    remove(root.as_path());
}
