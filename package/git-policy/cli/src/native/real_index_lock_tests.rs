//! Controls for the real index lock against real files:
//!  exclusive creation,
//!  identity and PID
//! recording,
//!  refusal to release a replaced lock,
//!  and the budget for a dead foreign owner.

use super::*;
use crate::private_storage::take_file_syncs;
use crate::test_support::{fixture, remove};

/// The index path inside a fresh fixture directory.
fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let index: PathBuf = root.join("index");
    return (root, index);
}

/// A free index is locked:
///  the lock exists with the recorded identity,
///  the PID file names this process,
/// and release removes both.
#[test]
#[cfg(unix)]
fn a_free_index_is_locked_and_released() {
    use std::os::unix::fs::MetadataExt;
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-free");
    let lock: RealIndexLock = match acquire_real_index_lock(index.as_path(), 100, "did nothing") {
        Ok(found) => found,
        Err(failure) => panic!("a free index is lockable: {failure:?}"),
    };
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(real_lock_path(index.as_path())).expect("the lock exists");
    assert_eq!(lock.identity().file.device, metadata.dev().to_string());
    assert_eq!(lock.identity().file.inode, metadata.ino().to_string());
    assert!(!lock.identity().fs_id.is_empty());
    let pid: String = std::fs::read_to_string(real_pid_path(index.as_path())).expect("PID file");
    assert_eq!(pid, format!("pid {}\n", std::process::id()));
    lock.release().expect("release");
    assert!(!real_lock_path(index.as_path()).exists());
    assert!(!real_pid_path(index.as_path()).exists());
    remove(root.as_path());
}

/// A lock replaced by another file is refused on release:
///  the replacement stays where it is.
#[test]
#[cfg(unix)]
fn a_replaced_lock_is_never_released() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-replaced");
    let lock: RealIndexLock = match acquire_real_index_lock(index.as_path(), 100, "did nothing") {
        Ok(found) => found,
        Err(failure) => panic!("a free index is lockable: {failure:?}"),
    };
    let path: PathBuf = real_lock_path(index.as_path());
    std::fs::remove_file(&path).expect("remove the created lock");
    std::fs::write(&path, b"another process's lock").expect("replacement");
    let refusal: String = lock.release().expect_err("a replaced lock must not be released");
    assert!(refusal.contains("identity changed"), "{refusal}");
    assert_eq!(std::fs::read(&path).expect("replacement kept"), b"another process's lock");
    remove(root.as_path());
}

/// A dead foreign owner exhausts the budget and the lock is left in place.
#[test]
#[cfg(unix)]
fn a_dead_foreign_owner_exhausts_the_budget() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-dead");
    std::fs::write(real_lock_path(index.as_path()), b"").expect("foreign lock");
    std::fs::write(real_pid_path(index.as_path()), "pid 2147483646\n").expect("dead PID file");
    let failure: CoordinationFailure = match acquire_real_index_lock(index.as_path(), 50, "landed nothing") {
        Ok(_) => panic!("a dead owner must not be overtaken"),
        Err(found) => found,
    };
    assert!(matches!(failure, CoordinationFailure::UnprovenOwner(_)));
    assert!(real_lock_path(index.as_path()).exists());
    remove(root.as_path());
}

/// A recorded identity that differs in only the device or only the inode is refused,
///  and the lock's own identity is accepted.
#[test]
#[cfg(unix)]
fn a_lock_with_one_mismatched_identity_field_is_refused() {
    use std::os::unix::fs::MetadataExt;
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-fields");
    let lock: RealIndexLock = match acquire_real_index_lock(index.as_path(), 100, "did nothing") {
        Ok(found) => found,
        Err(failure) => panic!("a free index is lockable: {failure:?}"),
    };
    let path: PathBuf = real_lock_path(index.as_path());
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(&path).expect("the lock is stat-ed");
    let fs_id: String = lock.identity().fs_id.clone();
    let wrong_device: LockIdentity = LockIdentity {
        file: FileIdentity { device: (metadata.dev() + 1).to_string(), inode: metadata.ino().to_string() },
        fs_id: fs_id.clone(),
    };
    let wrong_inode: LockIdentity = LockIdentity {
        file: FileIdentity { device: metadata.dev().to_string(), inode: (metadata.ino() + 1).to_string() },
        fs_id,
    };
    assert!(same_file_identity(path.as_path(), &wrong_device).is_err(), "a wrong device is refused");
    assert!(same_file_identity(path.as_path(), &wrong_inode).is_err(), "a wrong inode is refused");
    assert!(same_file_identity(path.as_path(), lock.identity()).is_ok(), "the own identity is accepted");
    lock.release().expect("the real lock releases");
    remove(root.as_path());
}

/// A transaction directory beside the index of a fresh fixture.
fn transaction_in(root: &Path) -> PathBuf {
    let directory: PathBuf = root.join("transaction");
    std::fs::create_dir(&directory).expect("transaction directory");
    return directory;
}

/// Take a landing lock on a thread and fail instead of hanging when the acquisition never ends,
///  so a defect that makes it wait for ever is a failure and not a timeout.
fn acquire_bounded(index: &Path, directory: &Path) -> Result<RealIndexLock, CoordinationFailure> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let owned_index: PathBuf = index.to_path_buf();
    let owned_directory: PathBuf = directory.to_path_buf();
    std::thread::spawn(move || {
        let _ = sender.send(acquire_landing_index_lock(owned_index.as_path(), owned_directory.as_path(), 1, 100, "landed nothing"));
    });
    return receiver
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("the acquisition ended instead of waiting for ever");
}

/// A landing lock journals its identity in `index-lock-<n>.json`, with the incumbent's bytes,
///  and announces this process through the PID file.
#[test]
#[cfg(unix)]
fn a_landing_lock_journals_its_identity_and_announces_the_process() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-journal");
    let directory: PathBuf = transaction_in(root.as_path());
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 3, 100, "landed nothing")
        .expect("a free index is lockable");
    let bytes: Vec<u8> = std::fs::read(directory.join("index-lock-3.json")).expect("the record exists");
    let expected: String = encode_index_lock(&IndexLockRecord { attempt: 3, lock: lock.identity().clone() });
    assert_eq!(String::from_utf8(bytes).expect("UTF-8 record"), expected);
    assert_eq!(
        std::fs::read_to_string(real_pid_path(index.as_path())).expect("PID file"),
        format!("pid {}\n", std::process::id())
    );
    lock.release_after_landing().expect("release");
    remove(root.as_path());
}

/// The identity record keeps its file sync (a torn one sticks every guarded command), and the PID file has none.
#[test]
#[cfg(unix)]
fn a_landing_lock_syncs_its_record_file_and_not_its_pid_file() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-sync");
    let directory: PathBuf = transaction_in(root.as_path());
    take_file_syncs();
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 3, 100, "landed nothing")
        .expect("a free index is lockable");
    assert_eq!(take_file_syncs(), vec![directory.join("index-lock-3.json")]);
    lock.release_after_landing().expect("release");
    remove(root.as_path());
}

/// The journal is written before the PID file: a journal that cannot be written leaves neither the lock
///  nor a PID file, and the failure is a stop.
#[test]
#[cfg(unix)]
fn a_failed_journal_write_leaves_no_lock_and_no_pid_file() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-journal-fails");
    let directory: PathBuf = transaction_in(root.as_path());
    std::fs::write(directory.join("index-lock-1.json"), b"an earlier record").expect("blocking record");
    let failure: CoordinationFailure =
        match acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing") {
            Ok(_) => panic!("the record cannot be written twice"),
            Err(found) => found,
        };
    assert!(matches!(&failure, CoordinationFailure::Stop(text) if text.contains("index-lock-1.json")), "{failure:?}");
    assert!(!real_lock_path(index.as_path()).exists());
    assert!(!real_pid_path(index.as_path()).exists());
    assert_eq!(std::fs::read(directory.join("index-lock-1.json")).expect("kept"), b"an earlier record");
    remove(root.as_path());
}

/// A PID file that already exists is left alone, as the incumbent leaves it, and the lock is still held.
#[test]
#[cfg(unix)]
fn an_existing_pid_file_is_left_alone() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-pid-exists");
    let directory: PathBuf = transaction_in(root.as_path());
    std::fs::write(real_pid_path(index.as_path()), "pid 2147483646\n").expect("foreign PID file");
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing")
        .expect("the lock is free");
    assert_eq!(std::fs::read_to_string(real_pid_path(index.as_path())).expect("PID file"), "pid 2147483646\n");
    assert!(real_lock_path(index.as_path()).exists());
    lock.release_after_landing().expect("release");
    assert!(real_pid_path(index.as_path()).exists(), "a PID file that names another process stays");
    assert!(!real_lock_path(index.as_path()).exists());
    remove(root.as_path());
}

/// A dead foreign owner exhausts the budget before anything is journaled.
#[test]
#[cfg(unix)]
fn a_landing_lock_behind_a_dead_owner_writes_no_record() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-landing-dead");
    let directory: PathBuf = transaction_in(root.as_path());
    std::fs::write(real_lock_path(index.as_path()), b"").expect("foreign lock");
    std::fs::write(real_pid_path(index.as_path()), "pid 2147483646\n").expect("dead PID file");
    let failure: CoordinationFailure =
        match acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 50, "landed nothing") {
            Ok(_) => panic!("a dead owner must not be overtaken"),
            Err(found) => found,
        };
    assert!(matches!(failure, CoordinationFailure::UnprovenOwner(_)));
    assert!(!directory.join("index-lock-1.json").exists());
    assert!(real_lock_path(index.as_path()).exists());
    remove(root.as_path());
}

/// An install writes the post-index through the held lock, so the new index is the lock file itself,
///  then removes the lock and this process's PID file.
#[test]
#[cfg(unix)]
fn an_install_replaces_the_index_with_the_lock_file_and_releases_everything() {
    use std::os::unix::fs::MetadataExt;
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-install");
    let directory: PathBuf = transaction_in(root.as_path());
    std::fs::write(index.as_path(), b"old index").expect("old index");
    let post: PathBuf = directory.join("post-1.index");
    std::fs::write(post.as_path(), b"post index").expect("post index");
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing")
        .expect("a free index is lockable");
    let lock_inode: u64 = std::fs::symlink_metadata(real_lock_path(index.as_path())).expect("lock").ino();
    lock.install(post.as_path()).expect("install");
    assert_eq!(std::fs::read(index.as_path()).expect("installed index"), b"post index");
    assert_eq!(std::fs::symlink_metadata(index.as_path()).expect("index").ino(), lock_inode);
    assert!(!real_lock_path(index.as_path()).exists());
    assert!(!real_pid_path(index.as_path()).exists());
    remove(root.as_path());
}

/// An install through a lock another process replaced refuses, changes nothing, and leaves the replacement.
#[test]
#[cfg(unix)]
fn an_install_through_a_replaced_lock_is_refused() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-install-replaced");
    let directory: PathBuf = transaction_in(root.as_path());
    std::fs::write(index.as_path(), b"old index").expect("old index");
    let post: PathBuf = directory.join("post-1.index");
    std::fs::write(post.as_path(), b"post index").expect("post index");
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing")
        .expect("a free index is lockable");
    let path: PathBuf = real_lock_path(index.as_path());
    std::fs::remove_file(&path).expect("remove the created lock");
    std::fs::write(&path, b"another process's lock").expect("replacement");
    let refusal: RecoveryError = lock.install(post.as_path()).expect_err("a replaced lock must not be written");
    assert!(refusal.0.contains("identity changed"), "{refusal:?}");
    assert_eq!(std::fs::read(index.as_path()).expect("old index kept"), b"old index");
    assert_eq!(std::fs::read(&path).expect("replacement kept"), b"another process's lock");
    remove(root.as_path());
}

/// Releasing after a landing removes the lock and the PID file.
#[test]
#[cfg(unix)]
fn releasing_after_a_landing_removes_the_lock_and_the_pid_file() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-release-after");
    let directory: PathBuf = transaction_in(root.as_path());
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing")
        .expect("a free index is lockable");
    lock.release_after_landing().expect("release");
    assert!(!real_lock_path(index.as_path()).exists());
    assert!(!real_pid_path(index.as_path()).exists());
    remove(root.as_path());
}

/// A replaced lock stays where it is when the landing releases, and this process's PID file still goes,
///  as the incumbent's disposal warns and carries on.
#[test]
#[cfg(unix)]
fn releasing_after_a_landing_leaves_a_replaced_lock() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-release-replaced");
    let directory: PathBuf = transaction_in(root.as_path());
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing")
        .expect("a free index is lockable");
    let path: PathBuf = real_lock_path(index.as_path());
    std::fs::remove_file(&path).expect("remove the created lock");
    std::fs::write(&path, b"another process's lock").expect("replacement");
    lock.release_after_landing().expect("a replaced lock only warns");
    assert_eq!(std::fs::read(&path).expect("replacement kept"), b"another process's lock");
    assert!(!real_pid_path(index.as_path()).exists());
    remove(root.as_path());
}

/// A lock that vanished before the release only warns, and the PID file goes.
#[test]
#[cfg(unix)]
fn releasing_after_a_landing_tolerates_a_vanished_lock() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-release-vanished");
    let directory: PathBuf = transaction_in(root.as_path());
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing")
        .expect("a free index is lockable");
    std::fs::remove_file(real_lock_path(index.as_path())).expect("remove the created lock");
    lock.release_after_landing().expect("an absent lock only warns");
    assert!(!real_pid_path(index.as_path()).exists());
    remove(root.as_path());
}

/// A release finds the PID file already gone: it is not an error.
#[test]
#[cfg(unix)]
fn releasing_after_a_landing_tolerates_a_vanished_pid_file() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-release-no-pid");
    let directory: PathBuf = transaction_in(root.as_path());
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing")
        .expect("a free index is lockable");
    std::fs::remove_file(real_pid_path(index.as_path())).expect("remove the PID file");
    lock.release_after_landing().expect("an absent PID file is not an error");
    assert!(!real_lock_path(index.as_path()).exists());
    remove(root.as_path());
}

/// A PID path that cannot be read as a file is an error of the release, and the lock is still removed first.
#[test]
#[cfg(unix)]
fn releasing_after_a_landing_reports_an_unreadable_pid_file() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-release-bad-pid");
    let directory: PathBuf = transaction_in(root.as_path());
    let lock: RealIndexLock = acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing")
        .expect("a free index is lockable");
    std::fs::remove_file(real_pid_path(index.as_path())).expect("remove the PID file");
    std::fs::create_dir(real_pid_path(index.as_path())).expect("a directory where the PID file was");
    let failure: String = lock.release_after_landing().expect_err("a directory cannot be read as the PID file");
    assert!(!failure.is_empty());
    assert!(!real_lock_path(index.as_path()).exists());
    remove(root.as_path());
}

/// A PID file that cannot be created for a reason other than existing fails the acquisition and removes the lock.
/// The index name is as long as a file name may be with the lock suffix, so the PID name is one too long.
#[test]
#[cfg(unix)]
fn a_pid_file_that_cannot_be_created_fails_the_acquisition() {
    let root: PathBuf = crate::test_support::fixture("real-lock-pid-too-long");
    let directory: PathBuf = transaction_in(root.as_path());
    let index: PathBuf = root.join("i".repeat(250));
    let failure: CoordinationFailure =
        match acquire_landing_index_lock(index.as_path(), directory.as_path(), 1, 100, "landed nothing") {
            Ok(_) => panic!("the PID file name is too long"),
            Err(found) => found,
        };
    assert!(matches!(&failure, CoordinationFailure::Stop(text) if text.contains("~pid.lock")), "{failure:?}");
    assert!(!real_lock_path(index.as_path()).exists(), "the lock is removed again");
    remove(root.as_path());
}

/// A free index is locked at once: the acquisition never waits when nobody holds the lock.
#[test]
#[cfg(unix)]
fn a_free_index_is_locked_without_waiting() {
    let (root, index): (PathBuf, PathBuf) = index_in("real-lock-no-wait");
    let directory: PathBuf = transaction_in(root.as_path());
    let lock: RealIndexLock = acquire_bounded(index.as_path(), directory.as_path()).expect("a free index is lockable");
    assert!(real_lock_path(index.as_path()).exists());
    lock.release_after_landing().expect("release");
    remove(root.as_path());
}

/// A lock that cannot be created for a reason other than being held (here, no directory to hold it)
///  is a stop at once, never a wait for a holder that does not exist.
#[test]
#[cfg(unix)]
fn a_lock_that_cannot_be_created_is_a_stop_and_not_a_wait() {
    let root: PathBuf = crate::test_support::fixture("real-lock-no-directory");
    let directory: PathBuf = transaction_in(root.as_path());
    let index: PathBuf = root.join("missing").join("index");
    let failure: CoordinationFailure = match acquire_bounded(index.as_path(), directory.as_path()) {
        Ok(_) => panic!("there is no directory for the lock"),
        Err(found) => found,
    };
    assert!(matches!(failure, CoordinationFailure::Stop(_)), "{failure:?}");
    remove(root.as_path());
}
