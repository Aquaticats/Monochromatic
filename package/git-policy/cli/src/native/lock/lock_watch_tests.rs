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
use crate::lock_watch::{A_FINISHED_CHILD_FORGETS, BEFORE_A_TAKE_IS_PINNED, Event, Tracker, Watched, Watch, wait_for_a_watched_child, watch_before_starting, with_made_up_events};
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
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
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
            // The concurrent command's Git takes the freed name, writes its PID file, and its command writes the record
            //  of that lock.
            std::fs::write(hook_lock.as_path(), b"another command's unfinished index\n").expect("the concurrent lock");
            std::fs::write(crate::stale_ref_lock_walk::pid_file_beside(hook_lock.as_path()).as_path(), format!("pid {}\n", held.0).as_bytes())
                .expect("the concurrent Git's PID file");
            assert!(
                lock_bindings::write(hook_common.as_path(), hook_lock.as_path(), &held, folder_named(hook_common.as_path())).is_some(),
                "the concurrent command pins its own lock"
            );
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

/// What:
///  A take that finished in the gap between the look that reported it and the pin that records it
///  leaves no record naming the Git that took it,
///  though another Git took the same path in that gap.
/// Why:
///  The tracker reads the PID file at one instant and the record's pin takes the file's identity at
///  another,
///  and a Git that finishes its take and another that takes the path between the two leave a record
///  that pins the new file and names the Git that no longer holds it.
///  That record shadows the right one for a reader that stops at the first record pinning the inode,
///  and on this volume a dead Git's lock then stands unproven and the next commit fails
///  (a review of `5327cbbfa4`, finding 1).
///  The record must therefore name a Git that held the file it pins:
///  the writer looks at the PID file beside that file once more after it has pinned, and takes its own
///  pin away when that file names another process.
///  The control forces the gap at the code's own fault point `BEFORE_A_TAKE_IS_PINNED`,
///  where it acts as the finishing Git and as the Git that takes the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// at(BeforeATakeIsPinned, () => { rename(lock, index); writeFileSync(lock); writeFileSync(pidFile, other) });
/// expect(write(common, lock, [child, identity])).toBeUndefined();
/// ```
#[test]
fn a_take_that_finished_before_it_was_pinned_is_not_recorded_for_that_git() {
    let common: PathBuf = a_watched_git_directory("watch-take-finished-before-pinning");
    let lock: PathBuf = common.join("index.lock");
    let pid_file: PathBuf = common.join("index~pid.lock");
    // The other Git's number: a process that has ended, so nothing in this fixture claims to run.
    let (other_pid, _other_identity) = a_dead_holder_for_the_gap();
    let hook_common: PathBuf = common.clone();
    crate::service_fault::hooks::set(
        BEFORE_A_TAKE_IS_PINNED,
        std::rc::Rc::new(move || {
            // The Git whose take was reported finishes it, and another Git takes the path in the gap.
            std::fs::rename(hook_common.join("index.lock").as_path(), hook_common.join("index").as_path()).expect("the finished take is renamed away");
            std::fs::write(hook_common.join("index.lock").as_path(), b"another Git's take\n").expect("the other take");
            std::fs::write(hook_common.join("index~pid.lock").as_path(), format!("pid {other_pid}\n").as_bytes()).expect("the other Git's PID file");
        }),
    );
    let started: Option<(Watch, Watched)> = watch_before_starting();
    let mut child: Child = a_child_that_takes_the_lock(common.as_path(), "sleep 2");
    crate::lock_holder_records::note_child_started(child.id());
    let child_pid: i64 = i64::from(child.id());
    let child_identity: String = crate::lock_holder_records::identity_of_process(child_pid).expect("the child's identity while it waits");
    let waited = wait_for_a_watched_child(&mut child, started);
    crate::service_fault::hooks::clear();
    assert!(waited.is_ok(), "the run ends: {waited:?}");
    let _unused: PathBuf = pid_file;
    let named: Option<Binding> = lock_bindings::of(common.as_path(), lock.as_path());
    let names_the_child: bool = matches!(&named, Some(bound) if bound.pid == child_pid && bound.identity == child_identity);
    remove(common.as_path());
    assert!(
        !names_the_child,
        "no record names the Git whose take finished before it was pinned: {named:?}"
    );
}

/// What:
///  A process that ran and ended,
///  for the gap a control writes another Git's number into.
/// Why:
///   The other Git's PID file must name a process this machine does not have,
///  so nothing in the fixture is mistaken for a live holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [pid, identity] = deadHolder();
/// ```
fn a_dead_holder_for_the_gap() -> (i64, String) {
    let mut child: Child = Command::new("sleep").arg("600").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("a helper starts");
    let pid: i64 = i64::from(child.id());
    let identity: String = crate::lock_holder_records::identity_of_process(pid).expect("the helper's identity while it waits");
    let _killed = child.kill();
    child.wait().expect("the helper ends");
    return (pid, identity);
}

/// What:
///  A lock inside a folder below a Git directory is reported by the watch,
///  as one at the root of it is.
/// Why:
///  A watch on a folder does not cover its subfolders,
///  and Git takes reference locks in `refs/heads`, `refs/tags`, the reftable's folder and others:
///  a merge or a rebase updates a branch through such a lock,
///  so a Git killed while it holds one leaves a lock no record describes,
///  and the retried command stops at it with Git's own refusal
///  (a review of `fee5557595`, finding 1).
///  The watch opens a watch on every folder below its roots,
///  so the creation of the lock and of the PID file beside it are reported like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = Watch.over([gitDir]); writePair(gitDir/refs/heads/topic.lock);
/// expect(tracker.locksTaken(process.pid)).toContain('refs/heads/topic.lock')
/// ```
#[test]
fn a_lock_in_a_nested_folder_is_reported() {
    let directory: PathBuf = fixture("watch-nested-folder");
    std::fs::create_dir_all(directory.join("refs/heads")).expect("the reference folders");
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
    let root_lock: PathBuf = directory.join("index.lock");
    let nested_lock: PathBuf = directory.join("refs/heads/topic.lock");
    let pid: i64 = i64::from(std::process::id());
    for lock in [root_lock.as_path(), nested_lock.as_path()] {
        std::fs::write(lock, b"a half-written copy\n").expect("the lock");
        std::fs::write(pid_file_of(lock).as_path(), format!("pid {pid}\n").as_bytes()).expect("the PID file");
    }
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(queued(&mut watch).as_slice());
    let found: Vec<PathBuf> = tracker.locks_taken(pid);
    drop(watch);
    remove(directory.as_path());
    assert!(found.contains(&root_lock), "the positive control: a lock at the root of the Git directory is reported");
    assert!(found.contains(&nested_lock), "a lock inside refs/heads is reported too: {found:?}");
}

/// What:
///  A lock inside a folder made after the watch began is reported,
///  as a folder that was there when the watch began is.
/// Why:
///  Git makes a folder for a branch whose name has a slash in it
///  (`refs/heads/feature/x`), and a reftable repository has no `reftable` folder until the first write,
///  so a watch that only knew the folders of its first look would miss those locks.
///  The watch opens a watch on a folder the kernel reports created, before the lock inside it is taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = Watch.over([gitDir]); mkdir(gitDir/refs/heads/feature); writePair(…/feature/x.lock);
/// expect(tracker.locksTaken(process.pid)).toContain('refs/heads/feature/x.lock')
/// ```
#[test]
fn a_lock_in_a_folder_made_after_the_watch_began_is_reported() {
    let directory: PathBuf = fixture("watch-new-folder");
    std::fs::create_dir_all(directory.join("refs/heads")).expect("the reference folders");
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
    let mut tracker: Tracker = Tracker::new();
    let pid: i64 = i64::from(std::process::id());
    // The folder is made while the watch runs, and the watch is given a moment to open a watch on it.
    std::fs::create_dir_all(directory.join("refs/heads/feature")).expect("the new folder");
    tracker.feed(queued(&mut watch).as_slice());
    let lock: PathBuf = directory.join("refs/heads/feature/x.lock");
    std::fs::write(lock.as_path(), b"a half-written copy\n").expect("the lock");
    std::fs::write(pid_file_of(lock.as_path()).as_path(), format!("pid {pid}\n").as_bytes()).expect("the PID file");
    tracker.feed(queued(&mut watch).as_slice());
    let found: Vec<PathBuf> = tracker.locks_taken(pid);
    drop(watch);
    remove(directory.as_path());
    assert!(found.contains(&lock), "a lock inside a folder made after the watch began is reported: {found:?}");
}

/// What:
///  An event that says the kernel dropped events makes the run's record say nothing,
///  though the lock and its PID file were seen created.
/// Why:
///  The tracker already treats a lost queue as unprovable
///  (`Tracker::feed`, the `IN_Q_OVERFLOW` mask),
///  but the reader dropped that event before the tracker could see it:
///  the kernel reports an overflow with a watch descriptor of `-1` (`inotify(7)`),
///  and the reader skipped every event whose descriptor matched no watched folder.
///  With events lost, a replacement lock another live command took can be taken for the first
///  child's and recorded under that child,
///  and later cleared while the other command works with no descriptor open
///  (a review of `fee5557595`, finding 2).
///  So an event that says events were lost is delivered, whatever its descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withMadeUpEvents([[-1, IN_Q_OVERFLOW, '']], () => { feed(realTake); expect(locksTaken(child)).toEqual([]) })
/// ```
#[test]
fn a_lost_queue_makes_the_run_record_nothing() {
    let (directory, lock) = with_lock_and_pid_file("watch-lost-queue", i64::from(std::process::id()));
    std::fs::remove_file(lock.as_path()).expect("the control makes the lock while the watch runs");
    std::fs::remove_file(directory.join("index~pid.lock")).expect("the control makes the PID file while the watch runs");
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock");
    std::fs::write(directory.join("index~pid.lock"), format!("pid {}\n", std::process::id()).as_bytes()).expect("the PID file");
    let mut tracker: Tracker = Tracker::new();
    let seen: Vec<Event> = with_made_up_events(vec![(-1, libc::IN_Q_OVERFLOW, std::ffi::OsString::from(""))], || { return queued(&mut watch); });
    tracker.feed(seen.as_slice());
    let taken: Vec<PathBuf> = tracker.locks_taken(i64::from(std::process::id()));
    drop(watch);
    remove(directory.as_path());
    assert!(seen.iter().any(|event| { return event.mask & libc::IN_Q_OVERFLOW != 0; }), "the reader delivers the event that says events were lost");
    assert!(taken.is_empty(), "a run whose queue was dropped records nothing: {taken:?}");
}

/// What:
///  Everything the watch holds for a moment, read until nothing more arrives.
/// Why:
///  The kernel queues an event a moment after the call that made it,
///  so a control reads a few times before it concludes that nothing else is coming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const events = []; for (let i = 0; i < 20; i++) events.push(...watch.readEvents()); sleep(2)
/// ```
fn queued(watch: &mut Watch) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    for _attempt in 0..20 {
        events.extend(watch.read_events());
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    return events;
}

/// The PID file Git writes beside `lock`.
fn pid_file_of(lock: &Path) -> PathBuf {
    return crate::stale_ref_lock_walk::pid_file_beside(lock);
}

/// What:
///  A helper program that takes a lock in the directory its last argument names,
///  writes the PID file beside it,
///  and kills itself,
///  so a runner is judged by what a kill of its Git leaves.
/// Why:
///   The bounded and the asynchronous runners are judged by the binding their child's lock gets,
///  and a child that exits by itself takes its lock away.
///  The program is a script of its own rather than `sh -c`, because a runner builds its Git command
///  with `-c` settings in front of the arguments, which `sh -c` would read as its own command string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fakeGit = writeScript(`for a in "$@"; do last="$a"; done; cd "$last"; : > index.lock; printf 'pid %s\n' $$ > index~pid.lock; kill -9 $$`)
/// ```
fn a_fake_git_that_takes_a_lock_and_is_killed(directory: &Path, name: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt as _;
    let program: PathBuf = directory.join(name);
    std::fs::write(
        program.as_path(),
        b"#!/bin/sh\nfor argument in \"$@\"; do last=\"$argument\"; done\ncd \"$last\" || exit 1\n: > index.lock\nprintf 'pid %s\\n' \"$$\" > index~pid.lock\nkill -9 \"$$\"\n",
    )
    .expect("the fake Git");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the fake Git runs");
    return program;
}

/// What:
///  The arguments one call of a fake Git takes:
///  the program reads the directory to work in from its last argument.
/// Why:
///   A runner's command carries settings in front of the arguments,
///  and the program ignores them all but the last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [directory]
/// ```
fn pointed_at(directory: &Path) -> Vec<std::ffi::OsString> {
    return vec![directory.as_os_str().to_owned()];
}

/// What:
///  A lock in a folder this watch no longer has is never bound to this process's Git,
///  though the events of the folder that moved away arrive under the path the watch remembers.
/// Why:
///  This is the gravest state any review of this work has reported:
///  a watched folder is renamed away and made again,
///  the events of the folder that moved arrive under the remembered pathname,
///  and the tracker joins them with the new folder's PID file,
///  so another Git's lock is bound to this process's Git.
///  The bound Git is killed, a later command finds the record, and the lock of a writer that still
///  runs is deleted: two writers then proceed under one name, which loses work.
///  What a watch knows of a folder is the folder it opened the watch on,
///  and a path is something this module remembers,
///  so a binding is written only while the folder at that path is still that one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watch(refs/heads); rename(refs/heads, refs/heads-moved); mkdir(refs/heads); writePair(refs/heads/index.lock, watchedPid);
/// writePair(refs/heads-moved/index.lock, watchedPid); await until(bindingsOf(refs/heads/index.lock).length)
/// kill(watchedGit); expect(bindingsOf(refs/heads/index.lock)).toEqual([]); expect(clearOne(...)).toBeUndefined()
/// ```
#[test]
fn a_lock_in_a_folder_this_watch_lost_is_never_bound_to_this_git() {
    let common: PathBuf = a_watched_git_directory("watch-moved-folder");
    let heads: PathBuf = common.join("refs/heads");
    std::fs::create_dir_all(heads.as_path()).expect("the reference folder");
    // The watched Git waits for its own end, so its identity can be read while it runs and the control's kill is the only
    //  end it has.
    let mut watched_git: Child = Command::new("sleep").arg("30").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("the watched Git");
    crate::lock_holder_records::note_child_started(watched_git.id());
    let watched_pid: i64 = i64::from(watched_git.id());
    let (watch, about): (Watch, Watched) = watch_before_starting().expect("the watch the plan remembered");
    let binder = crate::lock_watch::bind_while_the_caller_waits(Some((watch, about)), watched_git.id()).expect("the production binder");
    // Another Git holds a lock in the folder that takes this path afterwards, with no descriptor open on it.
    let mut replacement_writer: Child = Command::new("sleep").arg("30").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("the replacement writer");
    // The watched folder is renamed away and the path is made again.
    let moved: PathBuf = common.join("refs/heads-moved");
    std::fs::rename(heads.as_path(), moved.as_path()).expect("the watched folder moves away");
    std::fs::create_dir(heads.as_path()).expect("a folder at the same path again");
    // The new folder's lock is another Git's, and the PID file beside it names the watched Git, as one it left would.
    let replacement_lock: PathBuf = heads.join("index.lock");
    std::fs::write(pid_file_of(replacement_lock.as_path()).as_path(), format!("pid {watched_pid}\n")).expect("the PID file the new folder already had");
    std::fs::write(replacement_lock.as_path(), b"another Git's lock\n").expect("another Git's lock");
    // The lock and the PID file the watched Git's own Git took, in the folder that moved away: the kernel reports their
    //  creations under the path the watch remembers, which is the new folder's path.
    let moved_lock: PathBuf = moved.join("index.lock");
    std::fs::write(moved_lock.as_path(), b"the watched Git's lock\n").expect("the watched Git's lock");
    std::fs::write(pid_file_of(moved_lock.as_path()).as_path(), format!("pid {watched_pid}\n")).expect("its PID file");
    // The watcher is given a bounded moment to read those events and write whatever it makes of them: before the fix it
    //  writes one at once, and after it, it never does.
    let they_were_read: bool = bound_within(Duration::from_secs(2), common.as_path(), replacement_lock.as_path());
    let bound: Vec<Binding> = lock_bindings::bindings_of(common.as_path(), replacement_lock.as_path());
    let _killed = watched_git.kill();
    let _reaped = watched_git.wait();
    binder.finish();
    // The watched Git is gone and a child record names it; only a record can licence this removal, which is the state the
    //  review reproduced.
    let clearance = crate::abandoned_lock_proving::clear_one(replacement_lock.as_path(), common.as_path());
    let replacement_writer_alive: bool = crate::lock_holder_records::identity_of_process(i64::from(replacement_writer.id())).is_some();
    let _writer_killed = replacement_writer.kill();
    let _writer_reaped = replacement_writer.wait();
    let stands: bool = replacement_lock.exists();
    remove(common.as_path());
    assert!(replacement_writer_alive, "the replacement writer ran through the judgement, as a live writer does");
    let _read = they_were_read;
    assert!(bound.is_empty(), "no record may name a lock in a folder this watch lost: {bound:?}");
    assert!(clearance.is_none(), "nothing may license the removal of a lock this run never saw created: {clearance:?}");
    assert!(stands, "the other writer's lock must still stand");
}

/// What:
///  A drain that reaches its bound says this watch may have missed something,
///  and a run whose watch may have missed something records nothing.
/// Why:
///  The drain reads until the kernel has nothing more,
///  and a program that makes names at least as fast as they are read keeps it reading for good:
///  the forwarded command then waits after its Git has ended,
///  and the events it collects grow without end.
///  So one drain reads at most its bound and reports what a lost queue reports,
///  which leaves the lock to the wait and the report it always had and never to a wrong record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watch(dir); writePair(dir/index.lock, me); for (i < bound + 100) write(dir/made-i, '');
/// const events = watch.readEvents(); expect(events.length).toBeLessThanOrEqual(bound); expect(tracker.locksTaken(me)).toEqual([])
/// ```
#[test]
fn a_drain_that_reaches_its_bound_records_nothing() {
    let directory: PathBuf = fixture("watch-drain-bound");
    std::fs::create_dir_all(directory.as_path()).expect("the fixture directory");
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
    let pid: i64 = i64::from(std::process::id());
    let lock: PathBuf = directory.join("index.lock");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock");
    std::fs::write(pid_file_of(lock.as_path()).as_path(), format!("pid {pid}\n")).expect("the PID file");
    let made: usize = crate::lock_watch::MOST_DRAINED_EVENTS + 100;
    for index in 0..made {
        std::fs::write(directory.join(format!("made-{index}")), b"").expect("a name in the watched folder");
    }
    let events: Vec<Event> = watch.read_events();
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(events.as_slice());
    let taken: Vec<PathBuf> = tracker.locks_taken(pid);
    drop(watch);
    remove(directory.as_path());
    let read: usize = events.iter().filter(|event| { return event.mask & libc::IN_Q_OVERFLOW == 0; }).count();
    assert!(read <= crate::lock_watch::MOST_DRAINED_EVENTS, "one drain reads at most its bound, and this one read {read} events");
    assert!(
        events.iter().any(|event| { return event.mask & libc::IN_Q_OVERFLOW != 0; }),
        "a drain that stops at its bound says the watch may have missed something"
    );
    assert!(taken.is_empty(), "a watch that may have missed something records nothing: {taken:?}");
}

/// What:
///  A Git started by the wrapper's bounded runner binds the lock it took,
///  as one started by the watched runner does.
/// Why:
///  The wrapper's own ladder starts Git with a ceiling (`run_real_git_until`),
///  and a checkout held by a filter is one of those:
///  its Git takes `index.lock`, the ceiling kills it, and the lock stands.
///  A start that opened no watch leaves the lock with no record,
///  so the next command cannot prove it and keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember({commonDir, roots}); runRealGitUntil(fakeGit, [dir], 30s); expect(bindingsOf(dir/index.lock)).not.toEqual([])
/// ```
#[test]
fn a_git_the_bounded_runner_started_binds_the_lock_it_took() {
    let common: PathBuf = a_watched_git_directory("watch-bounded-start");
    let directory: PathBuf = common.join("bounded");
    std::fs::create_dir(directory.as_path()).expect("the bounded step's directory");
    let fake_git: PathBuf = a_fake_git_that_takes_a_lock_and_is_killed(common.as_path(), "fake-git-bounded");
    let outcome = crate::forwarding::run_real_git_until(
        fake_git.as_path(),
        pointed_at(directory.as_path()).as_slice(),
        &[],
        Duration::from_secs(30),
        crate::hooks_off::StartOwner::Wrapper,
    )
    .expect("the bounded runner starts and collects its Git");
    let lock: PathBuf = directory.join("index.lock");
    let bound: Vec<Binding> = lock_bindings::bindings_of(common.as_path(), lock.as_path());
    assert!(
        matches!(outcome, crate::forwarding::ChildOutcome::Signaled(9)),
        "the control's own Git must die by the kill the control sends: {outcome:?}"
    );
    assert!(!bound.is_empty(), "a Git the bounded runner started must bind the lock it took");
    // The child record is the other half of the clearance, and this control plants the one a start of this kind writes.
    let recorded: &Binding = bound.first().expect("the record the assertion above demands");
    plant_a_child_record(common.as_path(), recorded.pid, recorded.identity.as_str());
    let clearance = crate::abandoned_lock_proving::clear_one(lock.as_path(), common.as_path());
    let stands: bool = lock.exists();
    remove(common.as_path());
    assert!(
        matches!(clearance, Some(ref found) if found.evidence == crate::abandoned_lock_proving::Evidence::Binding),
        "the pass clears it by that record: {clearance:?}"
    );
    assert!(!stands, "the lock it took is gone after the pass");
}

/// What:
///  A Git started by the runner that awaits on the runtime binds the lock it took,
///  as one started by the synchronous runner does.
/// Why:
///  The transaction's asynchronous face and the merge recovery's `update-ref` start Git there,
///  and a runner that opened no watch leaves a reference lock with no record:
///  on a file system that gives the lock and its PID file one birth stamp, the PID file proves
///  nothing, so the next command keeps the lock and fails on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember({commonDir, roots}); await onTheRuntime(runAwaiting(fakeGit, [dir])); expect(bindingsOf(dir/index.lock)).not.toEqual([])
/// ```
#[test]
fn a_git_the_awaiting_runner_started_binds_the_lock_it_took() {
    let common: PathBuf = a_watched_git_directory("watch-awaiting-start");
    let directory: PathBuf = common.join("awaiting");
    std::fs::create_dir(directory.as_path()).expect("the awaiting step's directory");
    let fake_git: PathBuf = a_fake_git_that_takes_a_lock_and_is_killed(common.as_path(), "fake-git-awaiting");
    let mut command: Command = Command::new(fake_git.as_path());
    command.args(pointed_at(directory.as_path())).stdin(Stdio::null());
    let ran = crate::lazy_runtime::run_on_runtime(async {
        return crate::child_watch_output_watched_on_the_runtime::output_watched_within_for_a_stop_awaiting(
            &mut command,
            "a Git the awaiting runner started",
            "a Git the awaiting runner started",
            crate::child_watch::WatchTerms::for_class(crate::child_class::ChildClass::Object),
            false,
            crate::child_watch_step_bounds_and_errors::StepBound::Unbounded,
            false,
        )
        .await;
    })
    .expect("the runtime runs the step");
    let lock: PathBuf = directory.join("index.lock");
    let bound: Vec<Binding> = lock_bindings::bindings_of(common.as_path(), lock.as_path());
    remove(common.as_path());
    assert!(ran.is_ok(), "the step's own Git ends as the control's kill ends it: {ran:?}");
    assert!(!bound.is_empty(), "a Git the awaiting runner started must bind the lock it took");
}

/// What:
///  A reference in a namespace Git accepts named `objects` or `logs` is watched,
///  as one in any other namespace is.
/// Why:
///  Git accepts `refs/heads/objects/topic` and `refs/heads/logs/topic`,
///  and their locks stand in folders those names give.
///  A walk that skipped a name at every depth skipped exactly those folders:
///  the skip is right for the object store and the reflog folders of a Git directory,
///  which hold no lock of the worktree's state,
///  and wrong for a reference namespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watch(gitDir); writePair(refs/heads/objects/topic.lock); writePair(refs/heads/logs/topic.lock);
/// expect(tracker.locksTaken(me)).toContain(each)
/// ```
#[test]
fn a_reference_in_a_namespace_named_objects_is_reported() {
    let directory: PathBuf = fixture("watch-objects-namespace");
    for namespace in ["ordinary", "objects", "logs"] {
        std::fs::create_dir_all(directory.join("refs/heads").join(namespace)).expect("the reference namespace");
    }
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
    let pid: i64 = i64::from(std::process::id());
    let mut locks: Vec<PathBuf> = Vec::new();
    for namespace in ["ordinary", "objects", "logs"] {
        let lock: PathBuf = directory.join("refs/heads").join(namespace).join("topic.lock");
        std::fs::write(lock.as_path(), b"a reference update\n").expect("the reference lock");
        std::fs::write(pid_file_of(lock.as_path()).as_path(), format!("pid {pid}\n")).expect("the reference PID file");
        locks.push(lock);
    }
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(queued(&mut watch).as_slice());
    let found: Vec<PathBuf> = tracker.locks_taken(pid);
    drop(watch);
    remove(directory.as_path());
    for lock in locks {
        assert!(found.contains(&lock), "a reference lock under {} must be reported: {found:?}", lock.display());
    }
}

/// What:
///  Whether a record for `lock` is written within `within`,
///  asked until it is or the bound comes.
/// Why:
///   The watcher reads the kernel's queue a moment after the names are made,
///  and a control that means to see the record a defect writes waits for it instead of guessing a sleep,
///  while a control of the fixed code waits out the same bound once and then says it was never written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await untilWithin(2_000, () => bindingsOf(common, lock).length > 0)
/// ```
fn bound_within(within: Duration, common: &Path, lock: &Path) -> bool {
    let began: Instant = Instant::now();
    while began.elapsed() < within {
        if !lock_bindings::bindings_of(common, lock).is_empty() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    return false;
}

/// What:
///  Write the record a start of a Git writes about the child it started,
///  for a control whose child is gone before the pass meets its lock.
/// Why:
///   The pass clears a lock on two facts of this repository:
///  the record pinning the lock,
///  and the record of the child's start,
///  which says the number and the birth are a process this boot had and no longer has.
///  A runner writes the second as it starts its Git;
///  a control whose child died for the control's own reason writes it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(holderFolder, 'a-child-record'), recordText([['child', pid, identity]]))
/// ```
fn plant_a_child_record(common: &Path, pid: i64, identity: &str) {
    let folder: PathBuf = crate::lock_holder_records::folder_of(common);
    std::fs::create_dir_all(folder.as_path()).expect("the record folder");
    std::fs::write(folder.join("a-child-record"), crate::lock_holder_records::record_text(&[("child", pid, identity.to_owned())])).expect("the child record");
}

/// What:
///  The device and inode the kernel reports for one folder,
///  which is how a record names the folder a lock was taken in.
/// Why:
///  A record is written only while the folder standing at the lock's path is the folder the watch opened there,
///  and a control that plants one states that folder itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = folderNamed(dir) // [statSync(dir).dev, statSync(dir).ino]
/// ```
fn folder_named(path: &Path) -> (u64, u64) {
    let seen = crate::index_lock_evidence::read_lock_metadata(path).ok().flatten().expect("the folder's own identity");
    return (seen.device, seen.inode);
}

/// What:
///  A lock in a reference namespace is reported by the watch,
///  though the folder above it holds a reference named `HEAD`
///  and the namespace is named `objects`, `logs`, `worktrees` or `modules`.
/// Why:
///  The watch decided what a Git directory is by a file the folder holds,
///  so `refs/heads/marker/HEAD` made `marker` look like one and its `objects` child was never
///  watched:
///  a killed update's lock there got no record, and the next command kept it
///  (a review of `ca936e0b93`,
/// finding 2).
///  The kind of a folder is now a lookup in the Git directories the wrapper was told,
///  and nothing under `refs/` is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = Watch.over([gitDir]); write(refs/heads/marker/HEAD); writePair(…/objects/topic.lock);
/// expect(tracker.locksTaken(process.pid)).toContain('…/objects/topic.lock')
/// ```
#[test]
fn a_reference_namespace_beside_a_reference_named_head_is_watched() {
    let directory: PathBuf = fixture("watch-head-named-reference");
    let marker: PathBuf = directory.join("refs/heads/marker");
    std::fs::create_dir_all(marker.as_path()).expect("the folder a reference named HEAD lives in");
    std::fs::write(marker.join("HEAD"), b"").expect("a reference named HEAD");
    // The namespaces and their references stand before the watch opens, as they do in a repository a command meets.
    let mut locks: Vec<PathBuf> = Vec::new();
    for name in ["objects", "logs", "worktrees", "modules"] {
        let namespace: PathBuf = marker.join(name);
        std::fs::create_dir_all(namespace.as_path()).expect("the reference namespace");
        std::fs::write(namespace.join("topic"), b"").expect("a reference in the namespace");
        locks.push(namespace.join("topic.lock"));
    }
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
    let pid: i64 = i64::from(std::process::id());
    // The locks are written while the watch runs, as a killed update writes them.
    for lock in locks.iter() {
        std::fs::write(lock.as_path(), b"a killed update's lock\n").expect("the reference lock");
        std::fs::write(pid_file_of(lock.as_path()).as_path(), format!("pid {pid}\n")).expect("the reference PID file");
    }
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(queued(&mut watch).as_slice());
    let found: Vec<PathBuf> = tracker.locks_taken(pid);
    drop(watch);
    remove(directory.as_path());
    for lock in locks {
        assert!(found.contains(&lock), "a reference lock under {} must be reported: {found:?}", lock.display());
    }
}

/// What:
///  A lock made in a folder the watch learns of only afterwards is known to stand,
///  and is never bound to the child:
///  the reading that finds it cannot say when it was made or whose PID file it inherits.
/// Why:
///  Git makes `refs/heads/new-namespace/` and takes its first lock before the watcher has drained the
///  parent folder's event,
///  so the watch on the new folder is added after both creations generated no event
///  (a review of `6c2272132b`,
/// finding 3).
///  The watch now adds the watch and reads the folder,
///  and what it finds is reported as a name that stands with no creation seen:
///  the command knows the lock is there,
///  and no record may name a Git for it,
///  because the PID file beside a found lock may be one an earlier holder left and this one inherited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// buildFolderAndLockBeforeTheDrain(); const events = watch.readEvents(); tracker.feed(events);
/// expect(tracker.wasFoundByReading(lock)).toBe(true); expect(tracker.locksTaken(pid)).toEqual([])
/// ```
#[test]
fn a_lock_made_with_its_new_folder_is_known_and_never_bound() {
    let directory: PathBuf = fixture("watch-lock-with-its-folder");
    std::fs::create_dir_all(directory.join("refs/heads")).expect("the existing reference folder");
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
    let pid: i64 = i64::from(std::process::id());
    let lock: PathBuf = directory.join("refs/heads/new-namespace/topic.lock");
    std::fs::create_dir(lock.parent().expect("the new namespace")).expect("the new namespace");
    std::fs::write(lock.as_path(), b"the killed update's unfinished reference\n").expect("the lock in the new namespace");
    std::fs::write(pid_file_of(lock.as_path()).as_path(), format!("pid {pid}\n")).expect("its PID file");
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(queued(&mut watch).as_slice());
    let known: bool = tracker.was_found_by_reading(lock.as_path());
    let taken: Vec<PathBuf> = tracker.locks_taken(pid);
    drop(watch);
    remove(directory.as_path());
    assert!(known, "the watch must report the first lock of a folder it learns about");
    assert!(taken.is_empty(), "a lock found by reading is never bound to a child: {taken:?}");
}

/// What:
///  A lock that stands in a folder when the watch opens on it is known to stand,
///  and is never bound to the child,
///  for the same reason as a folder learned of later.
/// Why:
///  The walk of the tree adds a watch to each folder and only then reads it,
///  so a lock made in a folder before the walk reached it is seen by the reading and by no event
///  (a review of `6c2272132b`,
/// finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePair(refs/heads/standing/topic.lock); const watch = Watch.over([dir]); const tracker = new Tracker();
/// tracker.feed(watch.readEvents()); expect(tracker.wasFoundByReading(lock)).toBe(true); expect(tracker.locksTaken(pid)).toEqual([])
/// ```
#[test]
fn a_lock_that_stands_when_the_watch_opens_is_known_and_never_bound() {
    let directory: PathBuf = fixture("watch-lock-standing-at-open");
    let pid: i64 = i64::from(std::process::id());
    let lock: PathBuf = directory.join("refs/heads/standing/topic.lock");
    std::fs::create_dir_all(lock.parent().expect("the reference folder")).expect("the reference folder");
    std::fs::write(lock.as_path(), b"a reference update an earlier command left\n").expect("the lock");
    std::fs::write(pid_file_of(lock.as_path()).as_path(), format!("pid {pid}\n")).expect("its PID file");
    let mut watch: Watch = Watch::over(std::slice::from_ref(&directory)).expect("a watch on a real directory");
    let mut tracker: Tracker = Tracker::new();
    tracker.feed(queued(&mut watch).as_slice());
    let known: bool = tracker.was_found_by_reading(lock.as_path());
    let taken: Vec<PathBuf> = tracker.locks_taken(pid);
    drop(watch);
    remove(directory.as_path());
    assert!(known, "a lock standing in a folder the watch opens on must be known");
    assert!(taken.is_empty(), "a lock found by reading is never bound to a child: {taken:?}");
}

/// What:
///  A script that takes a lock in the directory its last argument names,
///  writes the PID file beside it only when `write_pid` says so,
///  and kills itself.
/// Why:
///  The two states a kill can leave are the ones the review of `0fe1e01e9f` separates:
///  a lock that stands with its PID file, and a lock whose Git was killed between creating it and
///  writing the PID file (finding 1), whose only evidence is what the watcher saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fakeGit = writeScript(`for a in "$@"; do last="$a"; done; cd "$last"; : > index.lock; ${writePid ? "printf 'pid %s\\n' $$ > index~pid.lock" : ""}; sleep 0.05; kill -9 $$`)
/// ```
fn a_fake_git_that_takes_a_lock_and_dies(directory: &Path, name: &str, write_pid: bool) -> PathBuf {
    use std::os::unix::fs::PermissionsExt as _;
    let program: PathBuf = directory.join(name);
    let pid_line: &str = if write_pid { "printf 'pid %s\\n' \"$$\" > index~pid.lock\n" } else { "" };
    let script: String = format!("#!/bin/sh\nfor argument in \"$@\"; do last=\"$argument\"; done\ncd \"$last\" || exit 1\n: > index.lock\n{pid_line}sleep 0.05\nkill -9 \"$$\"\n");
    std::fs::write(program.as_path(), script).expect("the fake Git");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the fake Git runs");
    return program;
}

/// A child-start report that records the child, as a caller of the watched runner does.
struct RecordTheChild;

impl crate::forwarding::ChildStarted for RecordTheChild {
    fn started(&mut self, pid: u32) {
        crate::lock_holder_records::note_child_started(pid);
    }
}

/// What:
///  A lock whose Git was killed between creating it and writing its PID file is bound by the sight of it,
///  and the next command clears it and a commit lands.
/// Why:
///  The watched Git creates `index.lock` and is killed before `index~pid.lock` is written;
///  the binder required a PID file, so no record was written,
///  and the next command had neither a record nor a PID file to prove the holder gone:
///  the lock stayed for good and every commit after it failed
///  (a review of `0fe1e01e9f`,
/// finding 1).
///  What the wrapper saw is the evidence, not what Git wrote beside the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = runWatched(fakeGitTakingOnlyALock()); await until(bindingsOf(indexLock).length);
/// expect(clearOne(indexLock, {births: 'coarse'})).toBeDefined(); expect(git(['commit', '-m', 'next']).status).toBe(0)
/// ```
#[test]
fn a_lock_its_git_left_without_a_pid_file_is_bound_by_the_sight_and_cleared() {
    let directory: PathBuf = crate::test_support::fixture("watch-lock-without-a-pid-file");
    let repository: PathBuf = crate::test_support::repository(directory.as_path(), "repo");
    let common: PathBuf = repository.join(".git");
    std::fs::write(repository.join("staged.txt"), "a change the next commit lands\n").expect("a staged change");
    let _added = crate::test_support::git(repository.as_path(), &["add", "staged.txt"]);
    crate::lock_holder_records::remember_repository(common.as_path());
    crate::lock_holder_records::record_this_process(common.as_path());
    crate::lock_watch::remember(Some(Watched { common_dir: common.clone(), roots: vec![common.clone()] }));
    let fake_git: PathBuf = a_fake_git_that_takes_a_lock_and_dies(directory.as_path(), "fake-lock-before-pid", false);
    let mut started = RecordTheChild;
    let outcome = crate::forwarding::run_real_git_naming(fake_git.as_path(), &[common.as_os_str().to_owned()], &[], &mut started, crate::hooks_off::StartOwner::Wrapper)
        .expect("the watched runner collects its Git");
    crate::lock_watch::remember(None);
    let lock: PathBuf = common.join("index.lock");
    let bindings: Vec<Binding> = lock_bindings::bindings_of(common.as_path(), lock.as_path());
    let clearance = crate::lock_birth_times::with_birth_times(crate::lock_birth_times::BirthTimes::Coarse(std::time::Duration::from_secs(3600)), || {
        return crate::abandoned_lock_proving::clear_one(lock.as_path(), common.as_path());
    });
    let stands: bool = lock.exists();
    let commit = crate::test_support::git_output(repository.as_path(), &["commit", "--quiet", "--message", "next"]);
    let landed: bool = commit.status.success();
    crate::test_support::remove(directory.as_path());
    assert_eq!(outcome, crate::forwarding::ChildOutcome::Signaled(9), "the control's Git dies by its own kill");
    assert!(!bindings.is_empty(), "the sight of the lock is the evidence, so a record names it though no PID file was written");
    assert!(matches!(clearance, Some(ref found) if found.evidence == crate::abandoned_lock_proving::Evidence::Binding), "the next command clears the lock by that record: {clearance:?}");
    assert!(!stands, "the lock is gone after the next command");
    assert!(landed, "the commit after it lands: {}", String::from_utf8_lossy(commit.stderr.as_slice()));
}

/// What:
///  A tracker returns every lock its child took,
///  and no count drops one silently.
/// Why:
///  The binder wrote records only for the locks the tracker returned,
///  and the tracker stopped at thirty-two:
///  a lock past that count fell back to the birth order,
///  which proves nothing where the two births are equal or unreadable,
///  so the lock stayed and the reference update that needed it failed for good
///  (a review of `0fe1e01e9f`,
/// finding 4).
///  The list is bounded by the names one run follows, and this states the bound does not drop
///  a standing take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// feed(thirtyThreeStandingPairs()); expect(locksTaken(child).length).toBe(33)
/// ```
#[test]
fn a_tracker_returns_every_standing_lock_its_child_took() {
    let directory: PathBuf = crate::test_support::fixture("watch-no-lock-count-drops-one");
    let heads: PathBuf = directory.join("refs/heads");
    let pid: i64 = i64::from(std::process::id());
    let mut tracker: Tracker = Tracker::new();
    let mut written: Vec<PathBuf> = Vec::new();
    for index in 0..33 {
        let namespace: PathBuf = heads.join(format!("namespace-{index:02}"));
        std::fs::create_dir_all(namespace.as_path()).expect("the reference namespace");
        let lock: PathBuf = namespace.join("topic.lock");
        std::fs::write(lock.as_path(), b"an unfinished reference update\n").expect("the lock");
        std::fs::write(pid_file_of(lock.as_path()).as_path(), format!("pid {pid}\n")).expect("its PID file");
        tracker.feed(&[created(namespace.as_path(), "topic.lock"), created(namespace.as_path(), "topic~pid.lock")]);
        written.push(lock);
    }
    let returned: Vec<PathBuf> = tracker.locks_taken(pid);
    crate::test_support::remove(directory.as_path());
    for lock in written.iter() {
        assert!(returned.contains(lock), "the tracker must return the lock at {}: {returned:?}", lock.display());
    }
}

/// What:
///  A Git the bounded runner started records its own child,
///  as every watched start does,
///  so the record the watcher wrote can clear the lock that Git left.
/// Why:
///  The bounded runner watched its Git and wrote a binding for the lock it took,
///  and never wrote the record of the child's start:
///  the proof asks for that record, refused the binding, and the commit after it failed for good
///  (a review of `0fe1e01e9f`,
/// finding 3).
///  One function starts a watched Git and records it, so no runner can watch without recording.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runRealGitUntil(fakeGitLeavingALock()); expect(identitiesRecorded(boundPid)).not.toEqual([]);
/// expect(clearOne(indexLock)).toBeDefined(); expect(git(['commit', '-m', 'next']).status).toBe(0)
/// ```
#[test]
fn a_git_the_bounded_runner_started_records_its_child() {
    let directory: PathBuf = crate::test_support::fixture("watch-bounded-runner-records-its-child");
    let repository: PathBuf = crate::test_support::repository(directory.as_path(), "repo");
    let common: PathBuf = repository.join(".git");
    std::fs::write(repository.join("staged.txt"), "a change the next commit lands\n").expect("a staged change");
    let _added = crate::test_support::git(repository.as_path(), &["add", "staged.txt"]);
    crate::lock_holder_records::remember_repository(common.as_path());
    crate::lock_holder_records::record_this_process(common.as_path());
    crate::lock_watch::remember(Some(Watched { common_dir: common.clone(), roots: vec![common.clone()] }));
    let fake_git: PathBuf = a_fake_git_that_takes_a_lock_and_dies(directory.as_path(), "fake-lock-of-the-bounded-runner", true);
    let outcome = crate::forwarding::run_real_git_until(fake_git.as_path(), &[common.as_os_str().to_owned()], &[], std::time::Duration::from_secs(30), crate::hooks_off::StartOwner::Wrapper)
        .expect("the bounded runner collects its Git");
    crate::lock_watch::remember(None);
    let lock: PathBuf = common.join("index.lock");
    let bindings: Vec<Binding> = lock_bindings::bindings_of(common.as_path(), lock.as_path());
    let bound: Option<i64> = bindings.first().map(|found| { return found.pid; });
    let recorded: Vec<String> = bound.map(|pid| { return crate::lock_holder_record_reads::identities_recorded(common.as_path(), pid); }).unwrap_or_default();
    let clearance = crate::abandoned_lock_proving::clear_one(lock.as_path(), common.as_path());
    let stands: bool = lock.exists();
    let commit = crate::test_support::git_output(repository.as_path(), &["commit", "--quiet", "--message", "next"]);
    let landed: bool = commit.status.success();
    crate::test_support::remove(directory.as_path());
    assert_eq!(outcome, crate::forwarding::ChildOutcome::Signaled(9), "the control's Git dies by its own kill");
    assert!(!bindings.is_empty(), "the bounded runner's watcher records the lock its Git took");
    assert!(!recorded.is_empty(), "the bounded runner records the child it started");
    assert!(matches!(clearance, Some(ref found) if found.evidence == crate::abandoned_lock_proving::Evidence::Binding), "the next command clears the lock by that record: {clearance:?}");
    assert!(!stands, "the lock is gone after the next command");
    assert!(landed, "the commit after it lands: {}", String::from_utf8_lossy(commit.stderr.as_slice()));
}

/// The steps of taking a lock and of clearing one, by the name a control passes to the helper below.
/// Why:
///  A process can be killed at any instruction, so each step of a take and each step of a clearance is
///  one row of the walk control: the state after each must be one the next command clears
///  (a review of `0fe1e01e9f`,
/// and the invariant it states).
const WALK_STEPS: [&str; 5] = [
    "after-the-lock",
    "after-the-pid-file",
    "after-the-pid-file-unlink",
    "after-the-lock-unlink",
    "after-the-record",
];

/// The variable that names the step the helper below takes, and the fixture it takes it in.
const WALK_STEP_VARIABLE: &str = "CLI_GIT_TEST_WALK_STEP";
const WALK_FIXTURE_VARIABLE: &str = "CLI_GIT_TEST_WALK_FIXTURE";

/// What:
///  Take one step of a take or of a clearance,
///  and, for a clearance's step, die there as a killed process does.
/// Why:
///  The walk control needs a process that can be killed at one named instruction,
///  and the helper is a copy of this test binary started for that one step
///  (`test_support::this_test_binary`).
///  Nothing happens when the variable is absent, so the suite's own run of it is a no-op.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const step = process.env.CLI_GIT_TEST_WALK_STEP; if (!step) return;
/// const fixture = process.env.CLI_GIT_TEST_WALK_FIXTURE; takeOrClearUntilTheStep(fixture, step)
/// ```
#[test]
fn a_step_of_a_take_or_a_clearance_is_killed_for_the_walk_control() {
    let Some(step) = std::env::var_os(WALK_STEP_VARIABLE) else {
        return;
    };
    let step: String = step.to_string_lossy().into_owned();
    let fixture: PathBuf = PathBuf::from(std::env::var_os(WALK_FIXTURE_VARIABLE).expect("the walk's fixture"));
    let repository: PathBuf = crate::test_support::repository(fixture.as_path(), "repo");
    let common: PathBuf = repository.join(".git");
    std::fs::write(repository.join("staged.txt"), "a change the next commit lands\n").expect("a staged change");
    let _added = crate::test_support::git(repository.as_path(), &["add", "staged.txt"]);
    match step.as_str() {
        "after-the-lock" | "after-the-pid-file" => {
            crate::lock_holder_records::remember_repository(common.as_path());
            crate::lock_holder_records::record_this_process(common.as_path());
            crate::lock_watch::remember(Some(Watched { common_dir: common.clone(), roots: vec![common.clone()] }));
            let write_pid: bool = step == "after-the-pid-file";
            let fake_git: PathBuf = a_fake_git_that_takes_a_lock_and_dies(fixture.as_path(), "fake-git-of-the-walk", write_pid);
            let mut started = RecordTheChild;
            let outcome = crate::forwarding::run_real_git_naming(fake_git.as_path(), &[common.as_os_str().to_owned()], &[], &mut started, crate::hooks_off::StartOwner::Wrapper)
                .expect("the watched runner collects its Git");
            crate::lock_watch::remember(None);
            assert_eq!(outcome, crate::forwarding::ChildOutcome::Signaled(9), "the step's Git dies by its own kill");
        }
        other => {
            // A clearance's step: the fixture holds a killed Git's lock, its PID file, its record and the pin.
            let lock: PathBuf = common.join("index.lock");
            let (pid, identity) = a_dead_holder_for_the_gap();
            std::fs::write(lock.as_path(), b"unfinished work of the killed Git\n").expect("the lock");
            std::fs::write(pid_file_of(lock.as_path()).as_path(), format!("pid {pid}\n")).expect("its PID file");
            let holders: PathBuf = crate::lock_holder_records::folder_of(common.as_path());
            std::fs::create_dir_all(holders.as_path()).expect("the record folder");
            std::fs::write(holders.join("a-child-record"), crate::lock_holder_records::record_text(&[("child", pid, identity.clone())])).expect("the child record");
            assert!(
                lock_bindings::write(common.as_path(), lock.as_path(), &(pid, identity), folder_named(common.as_path())).is_some(),
                "the plant pins the lock"
            );
            let point: &str = match other {
                "after-the-pid-file-unlink" => crate::abandoned_lock_proving::AFTER_THE_PID_FILE_IS_UNLINKED,
                "after-the-lock-unlink" => crate::abandoned_lock_proving::UNLINKED_THE_LOCK,
                "after-the-record" => crate::abandoned_lock_proving::AFTER_THE_RECORD_IS_REMOVED,
                _ => panic!("unknown walk step {other}"),
            };
            // SAFETY: `_exit` takes one number and touches no memory; the control ends this step's process here, as a
            //         kill at this instruction would.
            crate::service_fault::hooks::set(point, std::rc::Rc::new(|| { unsafe { libc::_exit(7) } }));
            let outcome = crate::abandoned_lock_proving::clear_one(lock.as_path(), common.as_path());
            panic!("the clearance was not killed at step {other}: {outcome:?}");
        }
    }
}

/// What:
///  Kill a process at each step of taking a lock and at each step of clearing one,
///  and require that the next command clears what is left and that a commit lands.
/// Why:
///  This is the property the four fixes are one statement of:
///  a commit is never failed for a reason of the wrapper's own,
///  a process can be killed at any instruction,
///  and what the wrapper saw is the evidence.
///  The steps are the whole of a take (the lock, then its PID file) and the whole of a clearance
///  (the PID file, the lock, the record), and the next command is the pass every command runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const step of walkSteps()) { runTheStepInAChildKilledAt(step); clearAbandonedLocks([common]);
///   expect(indexLock).not.toExist(); expect(git(['commit', '-m', 'next']).status).toBe(0) }
/// ```
#[test]
fn a_kill_at_each_step_of_a_take_and_of_a_clearance_leaves_what_the_next_command_clears() {
    let root: PathBuf = crate::test_support::fixture("walk-a-kill-at-each-step");
    let helper: PathBuf = crate::test_support::this_test_binary().expect("the test binary's own path");
    let mut rows: Vec<String> = Vec::new();
    for step in WALK_STEPS {
        let fixture: PathBuf = root.join(step);
        std::fs::create_dir_all(fixture.as_path()).expect("the step's fixture");
        let ran = std::process::Command::new(helper.as_path())
            .args(["lock::lock_watch_tests::a_step_of_a_take_or_a_clearance_is_killed_for_the_walk_control", "--exact", "--test-threads=1"])
            .env(WALK_STEP_VARIABLE, step)
            .env(WALK_FIXTURE_VARIABLE, fixture.as_path())
            .env("CLI_GIT_TEST_ROOT", root.join("unit-fixtures"))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .expect("the step's own process");
        let common: PathBuf = fixture.join("repo").join(".git");
        // The next command: the pass every command makes, from a fresh process.
        let _outcome = crate::abandoned_locks::clear_abandoned_locks(std::slice::from_ref(&common));
        let lock: PathBuf = common.join("index.lock");
        let stands: bool = lock.exists();
        let commit = crate::test_support::git_output(fixture.join("repo").as_path(), &["commit", "--quiet", "--message", "next"]);
        rows.push(format!(
            "{step}: step status={:?} signal={:?}; lock stands={stands}; commit status={:?}; step said={:?}",
            ran.status.code(),
            std::os::unix::process::ExitStatusExt::signal(&ran.status),
            commit.status.code(),
            String::from_utf8_lossy(ran.stderr.as_slice()).lines().filter(|line| { return line.contains("panicked") || line.contains("walk step"); }).collect::<Vec<&str>>()
        ));
        assert!(!stands, "{step}: the next command clears the lock: {}", rows.last().expect("the row just written"));
        assert!(commit.status.success(), "{step}: the commit after it lands: {}", rows.last().expect("the row just written"));
    }
    crate::test_support::remove(root.as_path());
    let _walk = rows;
}
