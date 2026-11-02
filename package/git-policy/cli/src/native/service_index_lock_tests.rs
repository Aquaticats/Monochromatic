//! Controls for the service's hold on the index lock: exclusive creation with the marker in place, a wait that ends when the
//!  holder lets go and is not woken by anything else, a limit that ends the wait with nothing changed, release by rename, and
//!  the removal of a stale lock only when it is the service's own and its holder is gone.

use super::*;
use crate::process_identity::{ProcessIdentityError, current_birth_identity, process_birth_identity};
use crate::service_index_placement::rename_noreplace;
use crate::test_support::{fixture, remove};
use std::time::Instant;
use std::cell::Cell;
use std::sync::Mutex;

fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(&git_dir).expect("directory");
    let index: PathBuf = git_dir.join("index");
    std::fs::write(&index, "old index").expect("index");
    return (root, index);
}

/// The marker this process writes for batch `seq`.
fn own_marker(seq: u64) -> String {
    return marker_text(seq, &current_birth_identity).expect("this process has an identity");
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
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(7));
    assert!(parse_marker(own_marker(7).as_str()).is_some_and(|marked| { return marked.holder.is_some(); }), "the marker names its holder");
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
    let live_staged: PathBuf = staged_path_for(&lock_path, 40, "token");
    std::fs::write(&live_staged, own_marker(40)).expect("a live writer's staging file");
    let lock = acquire(&index, 42, Duration::from_secs(1), &|_| {}).expect("taken");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(42));
    assert!(!fixed.exists() && !dead_staged.exists(), "{:?}", leftovers(&index));
    assert!(live_staged.exists(), "a live writer's staging file stays");
    std::fs::remove_file(&live_staged).expect("cleanup");
    drop(lock);
    let held = try_acquire(&index, 43).expect("an attempt without a wait takes it too");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(43));
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
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(51));
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
    let attempts_made: Cell<usize> = Cell::new(0);
    let counting = |from: &Path, to: &Path| -> std::io::Result<()> {
        attempts_made.set(attempts_made.get() + 1);
        return rename_noreplace(from, to);
    };
    let platform: Platform = Platform { rename: &counting, ..SYSTEM };
    let began: Instant = Instant::now();
    let lock = acquire_on(&platform, &index, 2, Duration::from_secs(30), &|note| { heard.lock().expect("lock").push(note); }).expect("taken after the wait");
    let waited: Duration = began.elapsed();
    let attempts: usize = attempts_made.get();
    release.join().expect("thread");
    assert!(waited >= Duration::from_millis(300) && waited < NOTICE_EVERY, "{waited:?}");
    // A wait woken by its own files tries thousands of times in 400 ms; one woken only by the lock tries a few times.
    assert!(attempts < 20, "{attempts} attempts in {waited:?}");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(2));
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
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(100));
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
    std::fs::write(lock_path_of(&index), own_marker(5)).expect("a live holder's");
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

//region Fixtures that stand in for environments no fixture can be made to have.

/// A refusal with `code`, as the kernel gives it.
fn refusal(code: i32) -> std::io::Result<()> {
    return Err(std::io::Error::from_raw_os_error(code));
}

/// The calls of a filesystem that has neither a rename without replacing nor hard links (vfat before Linux 4.x).
fn bare_filesystem<'a>(rename: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>, link: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>) -> Platform<'a> {
    return Platform { rename, link, ..SYSTEM };
}

/// The staging name the release before this one used: the process and the batch, with nothing that makes it new.
fn staged_by_the_last_release(lock: &Path, pid: u32, seq: u64) -> PathBuf {
    return PathBuf::from(format!("{}{STAGED_INFIX}-{pid}-{seq}", lock.display()));
}

//endregion

/// Where neither a rename without replacing nor a hard link works, no lock is created at all: creating one and writing the
///  marker after would leave an empty lock for as long as a kill between the two lasts, which no restart may take over.
///  The attempt fails with the code the clients are answered with, and nothing stays in the directory.
#[test]
fn a_filesystem_that_can_place_no_lock_atomically_gets_none() {
    let (root, index) = index_in("index-lock-bare");
    let rename = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(libc::EINVAL); };
    let link = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(libc::EPERM); };
    let platform: Platform = bare_filesystem(&rename, &link);
    let failure = acquire_on(&platform, &index, 20, Duration::from_secs(1), &|_| {}).expect_err("refused");
    assert_eq!(failure.code, FailureCode::Unavailable);
    assert!(!lock_path_of(&index).exists(), "no lock, so none can be left empty");
    assert!(try_acquire_on(&platform, &index, 21).is_none());
    assert!(!lock_path_of(&index).exists());
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// Where only the rename is refused as unsupported, the hard link places the lock, marked, and its staging name goes.
#[test]
fn a_filesystem_with_links_but_no_exclusive_rename_gets_the_lock_by_link() {
    let (root, index) = index_in("index-lock-link");
    let rename = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(libc::EINVAL); };
    let platform: Platform = Platform { rename: &rename, ..SYSTEM };
    let lock = acquire_on(&platform, &index, 22, Duration::from_secs(1), &|_| {}).expect("taken by link");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(22));
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    drop(lock);
    assert!(!lock_path_of(&index).exists());
    remove(root.as_path());
}

/// A process that cannot read its own birth identity writes no marker and takes no lock: a marker without an identity reads
///  as a dead instance's, so any other process could take the lock from a holder that runs.
#[test]
fn a_process_without_an_identity_takes_no_lock() {
    let (root, index) = index_in("index-lock-no-identity");
    let missing = || -> Result<String, ProcessIdentityError> { return Err(ProcessIdentityError::CurrentUnavailable); };
    let platform: Platform = Platform { identity: &missing, ..SYSTEM };
    assert!(marker_text(23, &missing).is_err(), "no marker is written without an identity");
    let failure = acquire_on(&platform, &index, 23, Duration::from_secs(1), &|_| {}).expect_err("refused");
    assert_eq!(failure.code, FailureCode::Unavailable);
    assert!(try_acquire_on(&platform, &index, 24).is_none());
    assert!(!lock_path_of(&index).exists());
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// A dead holder's lock, with the hard-link alias it left under the name this process would use for the same batch in the
///  last release (the process number came round again), is taken over: the alias is never opened, so it cannot empty the
///  lock, and the batch is not refused or left holding an empty lock.
#[test]
fn a_recycled_process_number_does_not_reopen_a_dead_holders_alias() {
    let (root, index) = index_in("index-lock-recycled");
    let lock_path: PathBuf = lock_path_of(&index);
    let dead: i64 = dead_pid();
    std::fs::write(&lock_path, marker_of_holder(60, dead, "gone")).expect("a killed holder's lock");
    let alias: PathBuf = staged_by_the_last_release(&lock_path, std::process::id(), 60);
    std::fs::hard_link(&lock_path, &alias).expect("the alias a kill left");
    let lock = acquire(&index, 60, Duration::from_secs(2), &|_| {}).expect("taken over, not refused");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(60));
    assert_eq!(std::fs::read_to_string(&alias).expect("the alias is not ours to empty"), marker_of_holder(60, dead, "gone"));
    drop(lock);
    let held = try_acquire(&index, 60).expect("an attempt without a wait takes over as well");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(60));
    drop(held);
    remove(root.as_path());
}

/// Two attempts of one process for one batch use two names, and a name that exists is an error, not a file to empty.
#[test]
fn staging_files_are_created_new_under_names_no_attempt_repeats() {
    let (root, index) = index_in("index-lock-new-names");
    let lock_path: PathBuf = lock_path_of(&index);
    let first: PathBuf = stage_marker(&lock_path, 61, "one").expect("first");
    let second: PathBuf = stage_marker(&lock_path, 61, "two").expect("second");
    assert_ne!(first, second);
    assert_eq!(std::fs::read_to_string(&first).expect("first"), "one");
    let taken = std::fs::OpenOptions::new().write(true).create_new(true).open(&first);
    assert!(taken.is_err_and(|error| { return error.kind() == std::io::ErrorKind::AlreadyExists; }), "creation never truncates");
    std::fs::remove_file(&first).expect("cleanup");
    std::fs::remove_file(&second).expect("cleanup");
    remove(root.as_path());
}

/// A staging file is judged by the process its name carries, not by its content: one that is still empty or half written
///  belongs to a writer that runs and stays; an empty one of a process that is gone, and one an earlier release named
///  without a process, go; a name that names no process is not this service's to remove.
#[test]
fn staging_files_are_judged_by_the_writer_in_their_name_and_never_by_their_content() {
    let (root, index) = index_in("index-lock-staging-by-name");
    let lock_path: PathBuf = lock_path_of(&index);
    let live_empty: PathBuf = staged_path_for(&lock_path, 62, "writing");
    std::fs::write(&live_empty, "").expect("a live writer between creating its file and writing it");
    let live_half: PathBuf = staged_by_the_last_release(&lock_path, std::process::id(), 63);
    std::fs::write(&live_half, "cli-git-ser").expect("a live writer part way through");
    let dead_empty: PathBuf = staged_by_the_last_release(&lock_path, u32::try_from(dead_pid()).expect("pid"), 64);
    std::fs::write(&dead_empty, "").expect("a dead writer's empty file");
    let unnamed: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}-not-a-process", lock_path.display()));
    std::fs::write(&unnamed, "").expect("a file that names no process");
    let lock = acquire(&index, 65, Duration::from_secs(1), &|_| {}).expect("taken");
    assert!(live_empty.exists() && live_half.exists(), "a running writer's file stays whatever it holds: {:?}", leftovers(&index));
    assert!(!dead_empty.exists(), "a dead writer's file goes: {:?}", leftovers(&index));
    assert!(unnamed.exists(), "a name that names no process is left alone");
    drop(lock);
    remove(root.as_path());
}

/// A stale lock that cannot be removed ends the attempt at once with the plain failure, naming nothing to wait for: waiting
///  longer would not remove it. Before, the failed removal skipped the deadline and the attempt repeated without end.
#[test]
fn a_stale_lock_that_cannot_be_removed_ends_the_wait_instead_of_repeating_it() {
    let (root, index) = index_in("index-lock-unremovable");
    let dead: i64 = dead_pid();
    std::fs::write(lock_path_of(&index), marker_of_holder(66, dead, "gone")).expect("a killed holder's lock");
    let calls: Cell<usize> = Cell::new(0);
    let unlink = |_: &Path| -> std::io::Result<()> {
        calls.set(calls.get() + 1);
        assert!(calls.get() < 50, "the attempt repeats without end");
        return refusal(libc::EPERM);
    };
    let platform: Platform = Platform { unlink: &unlink, ..SYSTEM };
    let began: Instant = Instant::now();
    let failure = acquire_on(&platform, &index, 67, Duration::from_secs(30), &|_| {}).expect_err("refused");
    assert_eq!(failure.code, FailureCode::Unavailable);
    assert!(began.elapsed() < Duration::from_secs(5), "{:?}", began.elapsed());
    assert_eq!(calls.get(), 1, "removal is tried once");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("stands"), marker_of_holder(66, dead, "gone"));
    calls.set(0);
    assert!(try_acquire_on(&platform, &index, 68).is_none());
    assert_eq!(calls.get(), 1);
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// A lock that is gone by the time it is removed is not a failure: the attempt goes on and takes it.
#[test]
fn a_stale_lock_that_vanishes_before_its_removal_is_taken_at_once() {
    let (root, index) = index_in("index-lock-vanished");
    std::fs::write(lock_path_of(&index), marker_of_holder(69, dead_pid(), "gone")).expect("a killed holder's lock");
    let vanished = |path: &Path| -> std::io::Result<()> {
        std::fs::remove_file(path)?;
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    };
    let platform: Platform = Platform { unlink: &vanished, ..SYSTEM };
    let lock = acquire_on(&platform, &index, 70, Duration::from_secs(5), &|_| {}).expect("taken");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(70));
    drop(lock);
    remove(root.as_path());
}

/// The number of attempts is that of one acquisition: another acquisition running at the same time, on its own thread and
///  its own index, adds nothing to it.
#[test]
fn attempts_are_counted_per_acquisition() {
    let (root, index) = index_in("index-lock-attempts");
    let (other_root, other_index) = index_in("index-lock-attempts-other");
    let busy = std::thread::spawn(move || {
        for seq in 0..40 {
            drop(acquire(&other_index, 1000 + seq, Duration::from_secs(1), &|_| {}).expect("taken"));
        }
    });
    let attempts_made: Cell<usize> = Cell::new(0);
    let counting = |from: &Path, to: &Path| -> std::io::Result<()> {
        attempts_made.set(attempts_made.get() + 1);
        return rename_noreplace(from, to);
    };
    let platform: Platform = Platform { rename: &counting, ..SYSTEM };
    for seq in 0..40 {
        drop(acquire_on(&platform, &index, seq, Duration::from_secs(1), &|_| {}).expect("taken"));
    }
    busy.join().expect("thread");
    assert_eq!(attempts_made.get(), 40, "one attempt for each of the acquisitions that met no holder");
    remove(root.as_path());
    remove(other_root.as_path());
}

/// A lock that was removed and replaced by another program's is not released or removed by the holder that placed the
///  first: the index is not installed, the other lock stands, and the failure names the lock. Two takers of one lock thus
///  never both finish.
#[test]
fn a_lock_another_program_replaced_is_neither_released_nor_removed() {
    let (root, index) = index_in("index-lock-replaced");
    let lock_path: PathBuf = lock_path_of(&index);
    let lock = acquire(&index, 71, Duration::from_secs(1), &|_| {}).expect("taken");
    std::fs::remove_file(&lock_path).expect("a person deletes the lock");
    std::fs::write(&lock_path, marker_of_holder(72, 1, "another service")).expect("another program takes it");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    let error = lock.release_with(&built).expect_err("refused");
    assert!(error.to_string().contains(lock_path.to_string_lossy().as_ref()), "{error}");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "old index", "the index was not installed");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("the other lock stands"), marker_of_holder(72, 1, "another service"));
    assert!(built.exists());
    remove(root.as_path());
}

/// A holder that goes out of scope without a release leaves a lock that is no longer its own where it is, and removes its own.
#[test]
fn a_lock_that_is_not_the_holders_own_is_left_when_the_holder_goes_out_of_scope() {
    let (root, index) = index_in("index-lock-replaced-drop");
    let lock_path: PathBuf = lock_path_of(&index);
    let lock = acquire(&index, 73, Duration::from_secs(1), &|_| {}).expect("taken");
    std::fs::write(&lock_path, "foreign").expect("replaced");
    drop(lock);
    assert_eq!(std::fs::read_to_string(&lock_path).expect("stands"), "foreign");
    std::fs::remove_file(&lock_path).expect("cleanup");
    let gone = acquire(&index, 74, Duration::from_secs(1), &|_| {}).expect("taken");
    std::fs::remove_file(&lock_path).expect("removed by someone");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    gone.release_with(&built).expect_err("a lock that is gone is not released");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "old index");
    remove(root.as_path());
}

/// The events that end a watch (an unmount, the kernel dropping the watch) wake the wait and say the watch is over, though
///  they name no file; a removal of another file does neither.
#[test]
fn events_that_end_a_watch_wake_the_wait_and_are_told_apart() {
    let event = |mask: u32, name: &str| -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&1_i32.to_ne_bytes());
        bytes.extend_from_slice(&mask.to_ne_bytes());
        bytes.extend_from_slice(&0_u32.to_ne_bytes());
        let mut padded: Vec<u8> = name.as_bytes().to_vec();
        padded.resize(name.len().div_ceil(16) * 16, 0);
        bytes.extend_from_slice(&u32::try_from(padded.len()).expect("small").to_ne_bytes());
        bytes.extend_from_slice(padded.as_slice());
        return bytes;
    };
    for mask in [libc::IN_UNMOUNT, libc::IN_IGNORED] {
        let ended: Vec<u8> = event(mask, "");
        assert!(events_name(ended.as_slice(), b"index.lock"), "{mask:#x} wakes the wait");
        assert!(events_end_watch(ended.as_slice()), "{mask:#x} ends the watch");
    }
    let other: Vec<u8> = event(libc::IN_DELETE, "index.lock.other");
    assert!(!events_end_watch(other.as_slice()));
    assert!(!events_end_watch(event(libc::IN_Q_OVERFLOW, "").as_slice()), "an overflow wakes the wait but leaves the watch");
    assert!(events_name(event(libc::IN_Q_OVERFLOW, "").as_slice(), b"index.lock"));
    let lock_then_end: Vec<u8> = [event(libc::IN_DELETE, "index.lock"), event(libc::IN_IGNORED, "")].concat();
    assert!(events_end_watch(lock_then_end.as_slice()), "the end is found after another event");
}

/// A watch whose directory is removed is over: the wait returns at once, not at its timer, and says so.
#[test]
fn a_wait_on_a_watch_whose_directory_is_removed_returns_at_once_and_reports_it() {
    let root: PathBuf = fixture("index-lock-watch-ended");
    let directory: PathBuf = root.join("watched");
    std::fs::create_dir(&directory).expect("directory");
    let watch: Watch = Watch::on(&directory).expect("watch");
    assert!(watch.wait(Duration::from_millis(50), std::ffi::OsStr::new("index.lock")), "a quiet directory leaves the watch standing");
    std::fs::remove_dir(&directory).expect("removed");
    let began: Instant = Instant::now();
    let standing: bool = watch.wait(Duration::from_secs(30), std::ffi::OsStr::new("index.lock"));
    assert!(!standing, "the watch is over");
    assert!(began.elapsed() < Duration::from_secs(5), "{:?}", began.elapsed());
    remove(root.as_path());
}
