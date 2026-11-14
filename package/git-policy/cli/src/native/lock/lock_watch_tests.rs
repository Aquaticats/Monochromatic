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

use crate::lock_bindings::{self, Binding};
use crate::lock_watch::{A_FINISHED_CHILD_FORGETS, Event, Tracker, Watched, Watch, wait_for_a_watched_child, watch_before_starting};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

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
    let watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
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

/// What:
///  A directory shaped like a Git directory,
///  whose records folder the wrapper writes,
///  remembered as the one this process's Git runs are watched in.
/// Why:
///   The run under test takes its roots from that memory,
///  and its records are written under the common directory it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember({ commonDir: dir, roots: [dir] }); const common = dir;
/// ```
fn a_watched_git_directory(name: &str) -> PathBuf {
    let common: PathBuf = fixture(name);
    std::fs::create_dir_all(lock_bindings::folder_of(common.as_path())).expect("the record folder");
    crate::lock_watch::remember(Some(Watched { common_dir: common.clone(), roots: vec![common.clone()] }));
    return common;
}

/// What:
///  The device and inode of the file at `path`,
///  for a control that tells one lock instance from the next at one path.
/// Why:
///   A lock renamed over its target and taken again is a different file at the same path,
///  and that is what the record of the second take has to describe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = statSync(path); [file.dev, file.ino]
/// ```
fn file_of(path: &Path) -> Option<(u64, u64)> {
    let seen = crate::index_lock_evidence::read_lock_metadata(path).ok()??;
    return Some((seen.device, seen.inode));
}

/// What:
///  A child that takes `index.lock`, writes the PID file naming itself,
///  holds it for a moment,
///  and then does what `tail` says.
/// Why:
///   The run under test writes a record from the kernel's own events for a real child,
///  so the control states the child's whole behaviour and its bounds.
///  The program waits for its own end where the control kills it,
///  so the control never races a program that exits by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn('sh', ['-c', `cd ${dir}; : > index.lock; printf 'pid %s\n' $$ > index~pid.lock; sleep 0.2; ${tail}`])
/// ```
fn a_child_that_takes_the_lock(directory: &Path, tail: &str) -> Child {
    let script: String = format!(
        "cd {directory} && : > index.lock && printf 'pid %s\\n' $$ > index~pid.lock && sleep 0.2 && {tail}",
        directory = directory.display()
    );
    return Command::new("sh")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("a child that takes the lock");
}

/// What:
///  Wait until `condition` holds, bounded.
/// Why:
///   The kernel queues the events and the run writes the record when it reads them,
///  so the control asks until the state it needs is there instead of sleeping a guessed time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await until(() => bindingOf(common, lock) !== undefined, 20_000)
/// ```
fn until(condition: impl Fn() -> bool) -> bool {
    let began: Instant = Instant::now();
    while began.elapsed() < Duration::from_secs(20) {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    return false;
}

/// What:
///  The identity of the lock standing at `path` that has a record,
///  waiting for one that is not `not_this` when the same path is two files one after the other.
/// Why:
///  A control that waits for a record asks the run's own state instead of sleeping a guessed time,
///  and this run takes one path twice,
///  so the wait names which of the two files it is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await until(() => bindingOf(common, lock) && stat(lock).ino !== notThis) // then [dev, ino]
/// ```
fn recorded_lock_at(common: &Path, path: &Path, not_this: Option<(u64, u64)>) -> Option<(u64, u64)> {
    let common_dir: PathBuf = common.to_path_buf();
    let lock_path: PathBuf = path.to_path_buf();
    let recorded: bool = until(move || {
        let standing: Option<(u64, u64)> = file_of(lock_path.as_path());
        return standing.is_some() && standing != not_this && lock_bindings::of(common_dir.as_path(), lock_path.as_path()).is_some();
    });
    if !recorded {
        return None;
    }
    return file_of(path);
}

/// What:
///  A command that finished its own lock takes away its own record only,
///  though another command's Git took the same path and wrote its own record in that instant.
/// Why:
///  This is the interleaving a review of `f8f5cba090` reproduced (finding 1):
///  the child's lock is renamed away,
///  the command decides that its record is litter,
///  another command's Git takes the freed name and writes the record of a lock that still stands,
///  and the first command's removal must not take that record with it.
///  A lock whose record is destroyed loses the one proof this volume leaves,
///  so a Git killed while holding it leaves a lock the next command cannot clear,
///  and the commit behind it fails.
///  The removal acts on the name the first command worked out from the file it pinned,
///  which cannot be the name of a file that took the path afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runChild(lockThenRemoveIt); at(AFinishedChildForgets, () => { takeTheFreedName(); writeItsRecord(); });
/// expect(bindingOf(common, lock)?.pid).toBe(theConcurrentCommandPid);
/// ```
#[test]
fn a_finished_child_takes_away_only_the_record_of_its_own_lock() {
    let common: PathBuf = a_watched_git_directory("watch-forget-own-record");
    let lock: PathBuf = common.join("index.lock");
    let concurrent_holder: (i64, String) = (4242, String::from("linux:4242#a-boot#4026531836#a-machine"));
    let planted: std::rc::Rc<std::cell::Cell<bool>> = std::rc::Rc::new(std::cell::Cell::new(false));
    let hook_common: PathBuf = common.clone();
    let hook_lock: PathBuf = lock.clone();
    let planted_by_the_hook: std::rc::Rc<std::cell::Cell<bool>> = planted.clone();
    let held: (i64, String) = concurrent_holder.clone();
    crate::service_fault::hooks::set(
        A_FINISHED_CHILD_FORGETS,
        std::rc::Rc::new(move || {
            // The concurrent command's Git takes the freed name and its command writes the record of that lock.
            std::fs::write(hook_lock.as_path(), b"another command's unfinished index\n").expect("the concurrent lock");
            assert!(lock_bindings::write(hook_common.as_path(), hook_lock.as_path(), &held).is_some(), "the concurrent command pins its own lock");
            planted_by_the_hook.set(true);
        }),
    );
    let started: Option<(Watch, Watched)> = watch_before_starting();
    let mut child: Child = a_child_that_takes_the_lock(common.as_path(), "rm -f index.lock index~pid.lock");
    crate::lock_holder_records::note_child_started(child.id());
    let waited = wait_for_a_watched_child(&mut child, started);
    crate::service_fault::hooks::clear();
    assert!(waited.is_ok(), "the child's run ends: {waited:?}");
    assert!(planted.get(), "the control reached the instant a concurrent command takes the freed name");
    let found: Option<Binding> = lock_bindings::of(common.as_path(), lock.as_path());
    let kept: bool = matches!(&found, Some(bound) if bound.pid == concurrent_holder.0);
    let records: usize = std::fs::read_dir(lock_bindings::folder_of(common.as_path()).as_path())
        .map(|entries| { return entries.flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".hold"); }).count(); })
        .unwrap_or(0);
    remove(common.as_path());
    assert!(kept, "the finished child took away only its own record, and the concurrent lock's record stays: {found:?} (records left: {records})");
}

/// What:
///  A child that takes the same lock path a second time is recorded again,
///  so a Git killed while holding the second lock leaves a lock whose proof is there.
/// Why:
///  One Git can take a lock, rename it over its target, and take the path again.
///  The second lock is a different file at the same path,
///  and the record of the first describes a file that is now the target,
///  so a run that remembered the path alone skipped the second take and left the standing lock with no record
///  (the review of `f8f5cba090` names it as a state the change already had).
///  A lock with no record is kept by every command on this volume,
///  and the commit behind it fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ch = runChild(take, remove, takeAgain); await until(() => bindingOf(common, lock) && stat(lock).ino !== first);
/// expect(bindingOf(common, lock)).toBeDefined()
/// ```
#[test]
fn a_child_that_takes_the_same_lock_again_is_recorded_again() {
    let common: PathBuf = a_watched_git_directory("watch-take-again");
    let lock: PathBuf = common.join("index.lock");
    let started: Option<(Watch, Watched)> = watch_before_starting();
    let mut child: Child = a_child_that_takes_the_lock(
        common.as_path(),
        "rm -f index.lock index~pid.lock; sleep 0.05; : > index.lock; printf 'pid %s\\n' $$ > index~pid.lock; sleep 30",
    );
    crate::lock_holder_records::note_child_started(child.id());
    let child_pid: u32 = child.id();
    let watched: std::thread::JoinHandle<std::io::Result<std::process::ExitStatus>> = std::thread::spawn(move || {
        return wait_for_a_watched_child(&mut child, started);
    });
    let first: Option<(u64, u64)> = recorded_lock_at(common.as_path(), lock.as_path(), None);
    assert!(first.is_some(), "the child's first lock is recorded");
    let second: Option<(u64, u64)> = recorded_lock_at(common.as_path(), lock.as_path(), first);
    assert!(second.is_some(), "the child's second lock at the same path is recorded too (it is a different file: {first:?} then {:?})", file_of(lock.as_path()));
    assert_ne!(first, second, "the two holds of the path are two files");
    // The child is killed while it holds the second lock, as a killed command's Git is.
    // SAFETY: `kill` takes a number and a signal and touches no memory; the number is this child's own.
    unsafe { libc::kill(child_pid as libc::pid_t, libc::SIGKILL) };
    let joined = watched.join();
    assert!(matches!(joined, Ok(Ok(_))), "the watched run collects the killed child: {joined:?}");
    let found: Option<Binding> = lock_bindings::of(common.as_path(), lock.as_path());
    remove(common.as_path());
    assert!(found.is_some(), "the killed child's standing lock keeps the record of its own take: {found:?}");
}

/// What:
///  The most records one lock path may hold while one Git run keeps taking and releasing that path:
///  the take that stands at the path now, and the take whose lock the run has not yet noticed is gone.
/// Why:
///   A Git of many takes, such as a `git rebase` or `git am` of many commits, takes `index.lock` once
///  per pick,
///  and each take is a full copy of the index.
///  A record is a hard link to the take's file, so every record still held keeps one whole index copy
///  alive on disk:
///  a run of many takes that held one record per take would hold every intermediate copy until the
///  run ended,
///  and on a long rebase of a large repository that can exhaust the disk,
///  which is a commit that fails
///  (a review of `1ee73d2728`, finding F1,
/// which measured the pins of one rebase growing from 10 to 46 and 240 KB to 2.64 MB mid-run).
///  The bound is two and does not depend on the run's length:
///  the take at the path now is kept, and a take that has been renamed away is taken away as the
///  run's next look notices its lock is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const peak = sampleWhile(childTakesTheLockTwelveTimes()); expect(peak).toBeLessThanOrEqual(2);
/// ```
#[test]
fn a_run_of_many_takes_of_one_path_holds_no_more_than_the_bound() {
    const BOUND: usize = 2;
    let common: PathBuf = a_watched_git_directory("watch-many-takes");
    let folder: PathBuf = lock_bindings::folder_of(common.as_path());
    let started: Option<(Watch, Watched)> = watch_before_starting();
    let takes: String = format!(
        "i=0; while [ $i -lt 12 ]; do : > index.lock; printf 'pid %s\\n' $$ > index~pid.lock; mv index.lock index; rm -f index~pid.lock; i=$((i+1)); sleep 0.04; done; sleep 30"
    );
    let mut child: Child = a_child_that_takes_the_lock(common.as_path(), takes.as_str());
    crate::lock_holder_records::note_child_started(child.id());
    let child_pid: u32 = child.id();
    let common_for_run: PathBuf = common.clone();
    let watched: std::thread::JoinHandle<std::io::Result<std::process::ExitStatus>> = std::thread::spawn(move || {
        return wait_for_a_watched_child(&mut child, started);
    });
    let held = |where_records_are: &Path| -> usize {
        return std::fs::read_dir(where_records_are)
            .map(|entries| { return entries.flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".hold"); }).count(); })
            .unwrap_or(0);
    };
    // Sample the whole run, bounded, and remember the most this path ever held at one moment.
    let mut peak: usize = 0;
    let mut saw_takes: bool = false;
    let began: Instant = Instant::now();
    // The twelve takes take about half a second, so this samples the whole of them and then a while with none.
    while began.elapsed() < Duration::from_secs(4) {
        let now: usize = held(folder.as_path());
        peak = peak.max(now);
        saw_takes = saw_takes || now > 0;
        std::thread::sleep(Duration::from_millis(2));
    }
    // SAFETY: `kill` takes a number and a signal and touches no memory; the number is this child's own.
    unsafe { libc::kill(child_pid as libc::pid_t, libc::SIGKILL) };
    let joined = watched.join();
    let _unused: PathBuf = common_for_run;
    remove(common.as_path());
    assert!(matches!(joined, Ok(Ok(_))), "the watched run collects the killed child: {joined:?}");
    assert!(saw_takes, "the run took the lock at least once");
    assert!(
        peak <= BOUND,
        "a run of many takes of one path holds no more than {BOUND} records at any moment, and this one held {peak}"
    );
}
