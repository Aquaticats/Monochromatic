//! What:
//!  Controls of the events that justify a record about a lock a command's Git took:
//!  which creations prove whose PID file a lock's is,
//!  and which ones leave the lock to the wait it has today.
//! Why:
//!  The events are the whole proof,
//!  and the ways they can be read wrongly are finite:
//!  a PID file that was already there,
//!  a PID file written before the lock,
//!  a lock removed and made again under the same name,
//!  a PID file that names another process,
//!  and a queue the kernel dropped.
//!  Each has a control here,
//!  and one control reads a real watch so the parser and the kernel's own events are exercised too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tracker = new Tracker(); tracker.feed(events); expect(tracker.locksTaken(child)).toEqual([lock]);
//! ```

use crate::lock_watch::{Event, Tracker, Watch};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// What:
///  One creation event for `name` in `directory`.
/// Why:
///   The tracker reads only the masks and the names the kernel reports,
///  so a control states them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// created(dir, 'index.lock') // { mask: IN_CREATE, name }
/// ```
fn created(directory: &Path, name: &str) -> Event {
    return Event { directory: directory.to_path_buf(), mask: libc::IN_CREATE, name: std::ffi::OsString::from(name) };
}

/// What:
///  One removal event for `name` in `directory`.
/// Why:
///   A lock that was taken away is no longer the file its name was given to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removed(dir, 'index.lock') // { mask: IN_DELETE, name }
/// ```
fn removed(directory: &Path, name: &str) -> Event {
    return Event { directory: directory.to_path_buf(), mask: libc::IN_DELETE, name: std::ffi::OsString::from(name) };
}

/// What:
///  A directory with a lock and the PID file Git writes beside it,
///  the PID file naming `pid`.
/// Why:
///   The tracker reads the PID file's number itself,
///  so a control makes the file the child would have left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { dir, lock } = withLockAndPidFile('take', child)
/// ```
fn with_lock_and_pid_file(name: &str, pid: i64) -> (PathBuf, PathBuf) {
    let directory: PathBuf = fixture(name);
    std::fs::create_dir_all(directory.as_path()).expect("the fixture directory");
    let lock: PathBuf = directory.join("index.lock");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock");
    std::fs::write(directory.join("index~pid.lock"), format!("pid {pid}\n").as_bytes()).expect("the PID file");
    return (directory, lock);
}

/// What:
///  A PID file created after its lock,
///  naming the child,
///  is that child's own lock.
/// Why:
///  This is the state a kill leaves when the command watched its Git take the lock,
///  and the record that follows from it is the one the change exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tracker.feed([created(lock), created(pidFile)]); locksTaken(child) // [lock]
/// ```
#[test]
fn a_pid_file_created_after_its_lock_is_that_locks_holder() {
    let (directory, lock) = with_lock_and_pid_file("watch-taken", 42);
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(&[created(directory.as_path(), "index.lock"), created(directory.as_path(), "index~pid.lock")]);
    assert_eq!(tracker.locks_taken(42), vec![lock.clone()], "the lock the child's own PID file was written for");
    remove(directory.as_path());
}

/// What:
///  A PID file that stood before the watch began
///  proves nothing about the lock beside it,
///  and a lock a PID file was written before is not that PID file's.
/// Why:
///  Git creates the lock and then the PID file,
///  and a PID file it finds in the way is left alone (`lockfile.c:121-123`,
/// Git 2.56.0),
///  so the state this keeps is exactly the state a later Git's lock beside a dead holder's PID file has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tracker.feed([created(lock)]); locksTaken(child) // []
/// tracker.feed([created(pidFile), created(lock)]); locksTaken(child) // []
/// ```
#[test]
fn a_pid_file_that_was_not_written_after_the_lock_names_nobody() {
    let (directory, _lock) = with_lock_and_pid_file("watch-stale-pid-file", 42);
    let mut already_there: Tracker = Tracker::new();
    already_there.feed(&[created(directory.as_path(), "index.lock")]);
    assert!(already_there.locks_taken(42).is_empty(), "a PID file the watch never saw created is not this child's");
    let mut before_the_lock: Tracker = Tracker::new();
    before_the_lock.feed(&[created(directory.as_path(), "index~pid.lock"), created(directory.as_path(), "index.lock")]);
    assert!(before_the_lock.locks_taken(42).is_empty(), "a PID file written before the lock is an earlier holder's");
    remove(directory.as_path());
}

/// What:
///  A lock removed and made again under the same name
///  is not the lock the PID file beside it was written for.
/// Why:
///  This is a person clearing a dead holder's lock and a later Git taking the name:
///  the PID file stays,
///  and the events say the name was given to a second file after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// feed([created(lock), deleted(lock), created(lock)]); locksTaken(child) // []
/// ```
#[test]
fn a_lock_made_again_after_its_pid_file_is_not_that_pid_files_lock() {
    let (directory, _lock) = with_lock_and_pid_file("watch-lock-remade", 42);
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(&[
        created(directory.as_path(), "index.lock"),
        created(directory.as_path(), "index~pid.lock"),
        removed(directory.as_path(), "index.lock"),
        created(directory.as_path(), "index.lock"),
    ]);
    assert!(tracker.locks_taken(42).is_empty(), "the PID file names a number, and the lock standing is a later file's");
    remove(directory.as_path());
}

/// What:
///  A PID file naming another process is not the child's,
///  and a lock removed after its PID file was written is not standing.
/// Why:
///  Another Git of the same repository writes its own number,
///  and a child that renamed its lock over its target took the name away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withLockAndPidFile(99); locksTaken(42) // []
/// ```
#[test]
fn a_pid_file_of_another_process_names_no_lock_of_this_child() {
    let (directory, _lock) = with_lock_and_pid_file("watch-another-process", 99);
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(&[created(directory.as_path(), "index.lock"), created(directory.as_path(), "index~pid.lock")]);
    assert!(tracker.locks_taken(42).is_empty(), "a PID file that names another process is that process's");
    remove(directory.as_path());
    let (gone_directory, _gone) = with_lock_and_pid_file("watch-lock-renamed", 42);
    let mut renamed: Tracker = Tracker::new();
    renamed.feed(&[
        created(gone_directory.as_path(), "index.lock"),
        created(gone_directory.as_path(), "index~pid.lock"),
        removed(gone_directory.as_path(), "index.lock"),
    ]);
    assert!(renamed.locks_taken(42).is_empty(), "a lock the child renamed over its target left no lock to clear");
    remove(gone_directory.as_path());
}

/// What:
///  A queue the kernel dropped proves nothing,
///  and neither does a watch that ended.
/// Why:
///  An event that was lost may be the one that showed the lock was replaced,
///  so every candidate of that run is unprovable and the lock keeps the wait it has today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// feed([created(lock), created(pidFile), { mask: IN_Q_OVERFLOW }]); locksTaken(child) // []
/// ```
#[test]
fn an_event_queue_that_overflowed_proves_nothing() {
    let (directory, _lock) = with_lock_and_pid_file("watch-overflow", 42);
    for (name, mask) in [("overflow", libc::IN_Q_OVERFLOW), ("ignored", libc::IN_IGNORED), ("unmounted", libc::IN_UNMOUNT)] {
        let mut tracker: Tracker = Tracker::new();
        tracker.feed(&[created(directory.as_path(), "index.lock"), created(directory.as_path(), "index~pid.lock"), Event { directory: directory.clone(), mask, name: std::ffi::OsString::from(name) }]);
        assert!(tracker.locks_taken(42).is_empty(), "{name}: a run whose events are incomplete proves nothing");
    }
    remove(directory.as_path());
}

/// What:
///  A watch on a real directory reports the two creations,
///  in the order the kernel saw them,
///  and the tracker reads them as the lock the child took.
/// Why:
///  The controls above state the events;
/// this one reads the kernel's own,
///  so the parser, the watch descriptor's directory and the order are exercised as they run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = Watch.over([dir]); writeFileSync(lock); writeFileSync(pidFile); expect(tracker.locksTaken(me)).toEqual([lock]);
/// ```
#[test]
fn a_watch_on_a_real_directory_reports_the_lock_a_child_took() {
    let (directory, lock) = with_lock_and_pid_file("watch-real", i64::from(std::process::id()));
    std::fs::remove_file(lock.as_path()).expect("the control makes the lock while the watch runs");
    std::fs::remove_file(directory.join("index~pid.lock")).expect("the control makes the PID file while the watch runs");
    let watch: Watch = Watch::over(&[directory.clone()]).expect("a watch on a real directory");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock");
    std::fs::write(directory.join("index~pid.lock"), format!("pid {}\n", std::process::id()).as_bytes()).expect("the PID file");
    let mut tracker: Tracker = Tracker::new();
    // The events are queued by the kernel, so they are read here without waiting for them to arrive.
    for _ in 0..50 {
        tracker.feed(watch.read_events().as_slice());
        if !tracker.locks_taken(i64::from(std::process::id())).is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(tracker.locks_taken(i64::from(std::process::id())), vec![lock], "the kernel reported the lock and the PID file written for it");
    drop(watch);
    remove(directory.as_path());
}
