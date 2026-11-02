//! Controls for the service's hold on the index lock: exclusive creation with the marker in place, a wait that ends when the
//!  holder lets go and is not woken by anything else, a limit that ends the wait with nothing changed, release by rename, and
//!  the removal of a stale lock only when it is the service's own and its holder is gone.

use super::*;
use crate::test_support::{fixture, remove};
use std::sync::Mutex;
use std::sync::atomic::Ordering;

fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(&git_dir).expect("directory");
    let index: PathBuf = git_dir.join("index");
    std::fs::write(&index, "old index").expect("index");
    return (root, index);
}

/// The number of a process that has exited and been reaped, so nothing runs under it.
fn dead_pid() -> i64 {
    let mut child = std::process::Command::new("true").spawn().expect("a short process");
    let pid: i64 = i64::from(child.id());
    child.wait().expect("reaped");
    return pid;
}

/// The marker text of a service holder `pid` born as `birth`.
fn marker_of_holder(seq: u64, pid: i64, birth: &str) -> String {
    return format!("{LOCK_MARKER}{seq} {pid} {birth}\n");
}

/// The names in the index's directory other than the index and its lock.
fn leftovers(index: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(index.parent().expect("directory"))
        .expect("listing")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name != "index" && name != "index.lock"; })
        .collect();
    names.sort();
    return names;
}

/// The lock is created exclusively with this process's marker, a second attempt fails while it is held (its holder runs),
///  and the release moves the new index over the real one and leaves no lock and no staging file.
#[test]
fn the_lock_is_exclusive_and_its_release_installs_the_new_index() {
    let (root, index) = index_in("index-lock-release");
    let lock = acquire(&index, 7, Duration::from_secs(1), &|_| {}).expect("taken");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), marker_text(7));
    assert!(parse_marker(marker_text(7).as_str()).is_some_and(|marked| { return marked.holder.is_some(); }), "the marker names its holder");
    assert!(try_acquire(&index, 70).is_none(), "a live holder's lock is not taken over");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    lock.release_with(&built).expect("released");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "new index");
    assert!(!lock_path_of(&index).exists() && !built.exists());
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// The release installs the built file itself by one rename (the index is the built file's inode afterwards), so the marked
///  lock stands until the new index is in place; copying the index over the lock first is what left a lock without its marker.
#[test]
fn the_release_renames_the_built_index_into_place_before_the_lock_goes() {
    use std::os::unix::fs::MetadataExt;
    let (root, index) = index_in("index-lock-rename");
    let lock = acquire(&index, 8, Duration::from_secs(1), &|_| {}).expect("taken");
    let built: PathBuf = index.with_file_name("index.cli-git-8");
    std::fs::write(&built, "new index").expect("built");
    let built_inode: u64 = std::fs::metadata(&built).expect("built").ino();
    lock.release_with(&built).expect("released");
    assert_eq!(std::fs::metadata(&index).expect("index").ino(), built_inode, "installed by rename, not by a copy");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "new index");
    assert!(!lock_path_of(&index).exists() && !built.exists());
    remove(root.as_path());
}

/// Staging files a killed attempt left are removed once the lock is taken: the fixed name an earlier release used and one
///  whose writer is gone; a live writer's staging file stays. The new lock carries its own marker.
#[test]
fn staging_files_of_killed_attempts_are_removed_and_a_live_writers_stays() {
    let (root, index) = index_in("index-lock-staged");
    let lock_path: PathBuf = lock_path_of(&index);
    let fixed: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}", lock_path.display()));
    std::fs::write(&fixed, "cli-git-service 41\n").expect("an earlier release's staging file");
    let dead: i64 = dead_pid();
    let dead_staged: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}-{dead}-41", lock_path.display()));
    std::fs::write(&dead_staged, marker_of_holder(41, dead, "gone")).expect("a killed writer's staging file");
    let live_staged: PathBuf = staged_path_for(&lock_path, 40);
    std::fs::write(&live_staged, marker_text(40)).expect("a live writer's staging file");
    let lock = acquire(&index, 42, Duration::from_secs(1), &|_| {}).expect("taken");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), marker_text(42));
    assert!(!fixed.exists() && !dead_staged.exists(), "{:?}", leftovers(&index));
    assert!(live_staged.exists(), "a live writer's staging file stays");
    std::fs::remove_file(&live_staged).expect("cleanup");
    drop(lock);
    let held = try_acquire(&index, 43).expect("an attempt without a wait takes it too");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), marker_text(43));
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    drop(held);
    remove(root.as_path());
}

/// A dead holder's lock with a hard-linked alias (a kill after the link, before the alias was removed) is taken over without
///  opening the alias, so the alias never empties the lock; the alias goes with the dead holder.
#[test]
fn a_dead_holders_lock_with_a_linked_alias_is_taken_over_without_emptying_it() {
    let (root, index) = index_in("index-lock-alias");
    let lock_path: PathBuf = lock_path_of(&index);
    let dead: i64 = dead_pid();
    std::fs::write(&lock_path, marker_of_holder(50, dead, "gone")).expect("a killed holder's lock");
    let alias: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}-{dead}-50", lock_path.display()));
    std::fs::hard_link(&lock_path, &alias).expect("the alias a kill left");
    let lock = acquire(&index, 51, Duration::from_secs(5), &|_| {}).expect("taken at once");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), marker_text(51));
    assert!(!alias.exists(), "{:?}", leftovers(&index));
    drop(lock);
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
///  repetition of the notice; the wait makes a handful of attempts, not one per event it causes itself.
#[test]
fn a_wait_ends_when_the_foreign_holder_lets_go_and_does_not_wake_itself() {
    let (root, index) = index_in("index-lock-wait");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    let foreign: PathBuf = lock_path_of(&index);
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(400));
        std::fs::remove_file(foreign).expect("the foreign holder lets go");
    });
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let attempts_before: usize = PLACE_ATTEMPTS.load(Ordering::SeqCst);
    let began: Instant = Instant::now();
    let lock = acquire(&index, 2, Duration::from_secs(30), &|note| { heard.lock().expect("lock").push(note); }).expect("taken after the wait");
    let waited: Duration = began.elapsed();
    let attempts: usize = PLACE_ATTEMPTS.load(Ordering::SeqCst) - attempts_before;
    release.join().expect("thread");
    assert!(waited >= Duration::from_millis(300) && waited < NOTICE_EVERY, "{waited:?}");
    // A wait woken by its own files tries thousands of times in 400 ms; one woken only by the lock tries a few times.
    assert!(attempts < 20, "{attempts} attempts in {waited:?}");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), marker_text(2));
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

/// A lock with the service's marker whose holder is gone is a dead instance's: the next landing removes it and takes its own at
///  once, whether the marker names a dead holder or is one an earlier release wrote without a holder.
#[test]
fn a_dead_service_instances_lock_does_not_block_the_next_landing() {
    let (root, index) = index_in("index-lock-dead-instance");
    std::fs::write(lock_path_of(&index), "cli-git-service 99\n").expect("an earlier release's lock");
    let began: Instant = Instant::now();
    let lock = acquire(&index, 100, Duration::from_secs(30), &|_| {}).expect("taken at once");
    assert!(began.elapsed() < Duration::from_secs(5), "{:?}", began.elapsed());
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), marker_text(100));
    drop(lock);
    std::fs::write(lock_path_of(&index), marker_of_holder(7, dead_pid(), "gone")).expect("a killed holder's lock");
    assert!(try_acquire(&index, 101).is_some(), "an attempt without a wait removes it too");
    remove(root.as_path());
}

/// A child process that is killed and reaped when the control ends, failed or not, so a failing control leaks nothing.
struct Holder(std::process::Child);

impl Drop for Holder {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _reaped = self.0.wait();
    }
}

/// A service lock whose holder still runs is waited for like a foreign one and never taken over; once that holder is gone, the
///  lock is taken at once.
#[test]
fn a_live_service_holders_lock_is_never_taken_over() {
    let (root, index) = index_in("index-lock-live-holder");
    let holder: Holder = Holder(std::process::Command::new("sleep").arg("30").spawn().expect("a live holder"));
    let pid: i64 = i64::from(holder.0.id());
    let birth: String = process_birth_identity(pid).expect("identity").expect("running");
    std::fs::write(lock_path_of(&index), marker_of_holder(9, pid, birth.as_str())).expect("a live holder's lock");
    let failure = acquire(&index, 10, Duration::from_millis(300), &|_| {}).expect_err("waited for");
    assert_eq!(failure.code, FailureCode::IndexInUse);
    assert!(try_acquire(&index, 10).is_none());
    remove_stale(&index, 9);
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("stands"), marker_of_holder(9, pid, birth.as_str()));
    drop(holder);
    let began: Instant = Instant::now();
    let lock = acquire(&index, 11, Duration::from_secs(30), &|_| {}).expect("taken once the holder is gone");
    assert!(began.elapsed() < Duration::from_secs(5), "{:?}", began.elapsed());
    drop(lock);
    remove(root.as_path());
}

/// A stale lock is removed only when it carries the service's marker for that batch and its holder is gone.
#[test]
fn a_stale_lock_is_removed_only_when_it_is_the_services_own() {
    let (root, index) = index_in("index-lock-stale");
    std::fs::write(lock_path_of(&index), "cli-git-service 5\n").expect("ours");
    remove_stale(&index, 6);
    assert!(lock_path_of(&index).exists(), "another batch's marker is left");
    remove_stale(&index, 5);
    assert!(!lock_path_of(&index).exists(), "its own marker is removed");
    std::fs::write(lock_path_of(&index), marker_text(5)).expect("a live holder's");
    remove_stale(&index, 5);
    assert!(lock_path_of(&index).exists(), "a live holder's lock is never touched");
    std::fs::write(lock_path_of(&index), "DIRCbinary").expect("foreign");
    remove_stale(&index, 5);
    assert!(lock_path_of(&index).exists(), "a foreign lock is never touched");
    remove(root.as_path());
}

/// The marker reads back as written, an earlier release's marker reads as one without a holder, and anything else is not a
///  service marker.
#[test]
fn markers_read_back_and_others_are_not_the_services() {
    assert_eq!(parse_marker("cli-git-service 12 345 linux:boot:678\n"), Some(Marked { seq: 12, holder: Some((345, String::from("linux:boot:678"))) }));
    assert_eq!(parse_marker("cli-git-service 12\n"), Some(Marked { seq: 12, holder: None }));
    assert_eq!(parse_marker("cli-git-service 12 345 a birth with spaces\n").and_then(|marked| { return marked.holder; }), Some((345, String::from("a birth with spaces"))));
    for other in ["", "DIRC", "cli-git-service \n", "cli-git-service x\n", "cli-git-service 12 345\n", "cli-git-service 12 pid birth\n", "Cli-git-service 12\n"] {
        assert_eq!(parse_marker(other), None, "{other:?}");
    }
}

/// The wait's event filter wakes for the lock's own removal and for an overflow, never for another file in the directory.
#[test]
fn the_wait_wakes_only_for_the_lock_itself() {
    let event = |mask: u32, name: &str| -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&1_i32.to_ne_bytes());
        bytes.extend_from_slice(&mask.to_ne_bytes());
        bytes.extend_from_slice(&0_u32.to_ne_bytes());
        let mut padded: Vec<u8> = name.as_bytes().to_vec();
        padded.resize(name.len().div_ceil(16).max(1) * 16, 0);
        bytes.extend_from_slice(&u32::try_from(padded.len()).expect("small").to_ne_bytes());
        bytes.extend_from_slice(padded.as_slice());
        return bytes;
    };
    let staging: Vec<u8> = event(libc::IN_DELETE, "index.lock.cli-git-marker-1-2");
    let lock: Vec<u8> = event(libc::IN_DELETE, "index.lock");
    let both: Vec<u8> = [staging.clone(), lock.clone()].concat();
    assert!(!events_name(staging.as_slice(), b"index.lock"), "a staging file does not wake the wait");
    assert!(events_name(lock.as_slice(), b"index.lock"));
    assert!(events_name(both.as_slice(), b"index.lock"), "the lock's event is found after another");
    assert!(!events_name(event(libc::IN_DELETE, "index.lock.other").as_slice(), b"index.lock"), "a name that only starts like it");
    assert!(events_name(event(libc::IN_Q_OVERFLOW, "").as_slice(), b"index.lock"), "an overflow may have hidden it");
    assert!(!events_name(&[], b"index.lock"));
}
