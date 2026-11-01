//! Controls for the exclusive creation of the real index lock,
//!  and for the landing lock that is created in place after the filesystem refuses the hard link.

use super::*;
use crate::test_support::{fixture, remove};

/// Creating a lock file reports a created file, a file that already exists, and any other failure as three
///  different answers:
///  only an existing file means another owner holds the lock.
#[test]
#[cfg(unix)]
fn creating_a_lock_file_tells_created_held_and_failed_apart() {
    let root: PathBuf = fixture("real-lock-exclusive");
    let path: PathBuf = root.join("index.lock");
    assert_eq!(create_exclusive(path.as_path()), Ok(true), "a free path is created");
    assert!(path.exists());
    assert_eq!(create_exclusive(path.as_path()), Ok(false), "an existing file is a holder, not a failure");
    let failure: String = create_exclusive(root.join("missing").join("index.lock").as_path())
        .expect_err("a path with no directory cannot be created");
    assert!(failure.contains("No such file"), "{failure}");
    remove(root.as_path());
}

/// A directory on a volume of its own, removed when the test ends however it ends.
#[cfg(target_os = "linux")]
struct SharedMemoryDirectory(PathBuf);

/// Removal is best effort: a directory already gone is not a failure of the test.
#[cfg(target_os = "linux")]
impl Drop for SharedMemoryDirectory {
    /// Remove the directory and what the test left in it.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.0.as_path());
    }
}

/// When the transaction directory lies on another volume than the index,
///  the link of the journaled candidate fails,
///  and the lock created in place is still journaled in `index-lock-<n>.json` and announced through the PID file.
/// `/dev/shm` is another mount than the fixture root on purpose:
///  a link cannot cross mounts,
///  and no other layout makes the link fail while the creation in place succeeds.
#[test]
#[cfg(target_os = "linux")]
fn a_landing_lock_created_in_place_is_journaled_and_announced() {
    use std::os::unix::fs::MetadataExt;
    let (root, index): (PathBuf, PathBuf) = {
        let root: PathBuf = fixture("real-lock-in-place");
        let index: PathBuf = root.join("index");
        (root, index)
    };
    let shared: SharedMemoryDirectory = SharedMemoryDirectory(
        PathBuf::from("/dev/shm").join(format!("git-policy-cli-real-lock-in-place-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(shared.0.as_path());
    std::fs::create_dir(shared.0.as_path()).expect("transaction directory on its own mount");
    let index_device: u64 = std::fs::metadata(root.as_path()).expect("fixture").dev();
    let directory_device: u64 = std::fs::metadata(shared.0.as_path()).expect("transaction directory").dev();
    assert_ne!(index_device, directory_device, "the layout under test needs the two directories on different mounts");
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), shared.0.as_path(), 2, 100, "landed nothing")
        .expect("a free index is lockable");
    let expected: String = encode_index_lock(&IndexLockRecord { attempt: 2, lock: lock.identity().clone() });
    let record: Vec<u8> = std::fs::read(shared.0.join("index-lock-2.json")).expect("the in-place lock is journaled");
    assert_eq!(String::from_utf8(record).expect("UTF-8 record"), expected);
    assert_eq!(
        std::fs::read_to_string(real_pid_path(index.as_path())).expect("the in-place lock is announced"),
        format!("pid {}\n", std::process::id())
    );
    assert!(lock.assert_identity().is_ok(), "the lock in place is the file the identity names");
    assert!(
        !shared.0.join("index-lock-1.candidate").exists() && !shared.0.join("index-lock-2.candidate").exists(),
        "the candidate does not outlive the refused link"
    );
    lock.release_after_landing().expect("release");
    assert!(!real_lock_path(index.as_path()).exists());
    assert!(!real_pid_path(index.as_path()).exists());
    remove(root.as_path());
}
