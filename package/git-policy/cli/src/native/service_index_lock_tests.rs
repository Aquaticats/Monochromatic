//! Controls for the service's hold on the index lock: exclusive creation, a wait that ends when the holder lets go, a limit that
//!  ends the wait with nothing changed, release by rename, and the removal of a stale lock that is the service's own.

use super::*;
use crate::test_support::{fixture, remove};
use std::sync::Mutex;

fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(&git_dir).expect("directory");
    let index: PathBuf = git_dir.join("index");
    std::fs::write(&index, "old index").expect("index");
    return (root, index);
}

/// The lock is created exclusively with the service's marker, a second attempt fails, and the release moves the new index over the
///  real one and leaves no lock.
#[test]
fn the_lock_is_exclusive_and_its_release_installs_the_new_index() {
    let (root, index) = index_in("index-lock-release");
    let lock = acquire(&index, 7, Duration::from_secs(1), &|_| {}).expect("taken");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), "cli-git-service 7\n");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    lock.release_with(&built).expect("released");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "new index");
    assert!(!lock_path_of(&index).exists() && !built.exists());
    remove(root.as_path());
}

/// A lock that goes out of scope without a release is removed, and the index is as it was.
#[test]
fn a_lock_that_is_not_released_leaves_the_index_as_it_was() {
    let (root, index) = index_in("index-lock-drop");
    drop(acquire(&index, 1, Duration::from_secs(1), &|_| {}).expect("taken"));
    assert!(!lock_path_of(&index).exists());
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "old index");
    remove(root.as_path());
}

/// A foreign holder that lets go while the service waits is followed at once by the service's own lock, long before the
///  repetition of the notice, and the wait announced who it waited for.
#[test]
fn a_wait_ends_when_the_foreign_holder_lets_go() {
    let (root, index) = index_in("index-lock-wait");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    let foreign: PathBuf = lock_path_of(&index);
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(400));
        std::fs::remove_file(foreign).expect("the foreign holder lets go");
    });
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let began: Instant = Instant::now();
    let lock = acquire(&index, 2, Duration::from_secs(30), &|note| { heard.lock().expect("lock").push(note); }).expect("taken after the wait");
    let waited: Duration = began.elapsed();
    release.join().expect("thread");
    assert!(waited >= Duration::from_millis(300) && waited < NOTICE_EVERY, "{waited:?}");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), "cli-git-service 2\n");
    drop(lock);
    remove(root.as_path());
}

/// A holder that outlives the limit ends the wait with the plain failure, and nothing was changed: the foreign lock and the
///  index stand, and the clients are told who held it.
#[test]
fn a_holder_that_outlives_the_limit_ends_the_wait_with_nothing_changed() {
    let (root, index) = index_in("index-lock-limit");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let began: Instant = Instant::now();
    let failure = acquire(&index, 3, Duration::from_millis(500), &|note| { heard.lock().expect("lock").push(note); }).expect_err("refused");
    assert_eq!(failure.code, FailureCode::IndexInUse);
    assert!(began.elapsed() >= Duration::from_millis(500) && began.elapsed() < Duration::from_secs(10), "{:?}", began.elapsed());
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("foreign lock stands"), "foreign");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "old index");
    assert!(heard.lock().expect("lock").iter().any(|note| { return matches!(note, Announce::Holder(_)); }), "the holder was announced");
    remove(root.as_path());
}

/// A lock with the service's marker is a dead instance's (one service runs, and it holds no lock between landings): the next
///  landing removes it and takes its own at once, instead of waiting out the limit for a holder that is gone. A foreign lock is
///  still waited for.
#[test]
fn a_dead_service_instances_lock_does_not_block_the_next_landing() {
    let (root, index) = index_in("index-lock-dead-instance");
    std::fs::write(lock_path_of(&index), "cli-git-service 99\n").expect("a killed service's lock");
    let began: Instant = Instant::now();
    let lock = acquire(&index, 100, Duration::from_secs(30), &|_| {}).expect("taken at once");
    assert!(began.elapsed() < Duration::from_secs(5), "{:?}", began.elapsed());
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), "cli-git-service 100\n");
    drop(lock);
    std::fs::write(lock_path_of(&index), "cli-git-service 7\n").expect("again");
    assert!(try_acquire(&index, 101).is_some(), "an attempt without a wait removes it too");
    remove(root.as_path());
}

/// A stale lock is removed only when it carries the service's marker for that batch.
#[test]
fn a_stale_lock_is_removed_only_when_it_is_the_services_own() {
    let (root, index) = index_in("index-lock-stale");
    std::fs::write(lock_path_of(&index), "cli-git-service 5\n").expect("ours");
    remove_stale(&index, 6);
    assert!(lock_path_of(&index).exists(), "another batch's marker is left");
    remove_stale(&index, 5);
    assert!(!lock_path_of(&index).exists(), "its own marker is removed");
    std::fs::write(lock_path_of(&index), "DIRCbinary").expect("foreign");
    remove_stale(&index, 5);
    assert!(lock_path_of(&index).exists(), "a foreign lock is never touched");
    remove(root.as_path());
}
