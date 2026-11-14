//! What:
//!  The watch a command keeps on its Git's own directories while that Git runs,
//!  and the events that show which locks its Git took.
//! Why:
//!  A command that keeps its process around its Git can watch the directories Git writes in,
//!  and no reader that arrives later can:
//!  a lock's PID file and the lock itself are written microseconds apart,
//!  and a file system of coarse birth times gives them one stamp
//!  (`lock_birth_times.rs`),
//!  so nothing on disk says which of the two was written for which.
//!  The event stream says it:
//!  the kernel reports the creation of a lock,
//!  then the creation of the PID file beside it,
//!  and every removal or replacement in between.
//!  So a PID file that appears after a lock,
//!  names the command's own child,
//!  and stands beside a lock nothing removed since,
//!  is that Git's PID file,
//!  and the lock beside it is the one that Git took.
//!  The command writes that down at once (`lock_bindings.rs`),
//!  because a kill that takes the command and its Git together leaves no second chance:
//!  a record written after the child ends would never be written for a group kill.
//!  A queue the kernel dropped (`IN_Q_OVERFLOW`),
//!  a watch that ended,
//!  more names than one run follows,
//!  and a platform without a watch leave the lock to the bounded wait and report it has today.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = Watch.over(roots); watch.follow(child.pid); await waitAndBind(child, watch, commonDir, identity);
//! ```

/// The record written beside a lock a command's Git took,
/// and the identity of the file it pinned.
use crate::lock_bindings::{self, Written};
/// The name a lock's PID file has.
use crate::stale_ref_lock_walk::pid_file_beside;
/// The process number a Git PID file names.
use crate::stopped_git_locks::pid_named_by;
/// The repository shape a run works in.
use crate::worktree_identity::WorktreeIdentity;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The lock name's end,
/// which a PID file's name also ends with.
const LOCK_SUFFIX: &str = ".lock";

/// What a lock's PID file is named with,
/// before the lock's own end.
const PID_INFIX: &str = "~pid.lock";

/// How long one wait for an event lasts before the child is asked about,
/// so a watch that reports nothing can never hold a command.
const WAIT_STEP: std::time::Duration = std::time::Duration::from_millis(200);

/// How long one wait lasts where the child's own end cannot wake it:
/// the next look at the child ends the wait then.
const WAIT_STEP_WITHOUT_PIDFD: std::time::Duration = std::time::Duration::from_millis(5);

/// The most distinct names one run follows,
/// so a Git that writes many files cannot grow a command.
const MOST_NAMES: usize = 1024;

/// The most locks one run writes a record for.
const MOST_LOCKS: usize = 32;

/// The instant a command's Git has ended and the command is about to take away the record of a lock that no longer
/// stands,
/// after it decided that this lock is gone.
/// Why:
///   A control states the interleaving the removal must survive:
///  a command that finished its own lock takes its record away while another command's Git takes the same path and
///  writes its own,
///  and the record of that other lock must stay.
///  Named here so both halves of the promise reach one point,
/// and inert unless the test variable or a control names it.
pub const A_FINISHED_CHILD_FORGETS: &str = "a-finished-child-forgets";

/// What:
///  The directories one repository's Git runs are watched in,
///  and the common Git directory the records are written under.
/// Why:
///   A command works in one repository,
///  and the run that starts Git is several calls below the plan that knows the repository,
///  so the plan leaves both here.
///  No value means the command's Git takes no lock this repository's records explain,
///  and every run starts exactly as it did before this watch existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let watched: { commonDir: string; roots: string[] } | undefined;
/// ```
static WATCHED: std::sync::Mutex<Option<Watched>> = std::sync::Mutex::new(None);

/// What one command's runs are watched in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Watched {
    /// The common Git directory the records are written under.
    pub common_dir: PathBuf,
    /// The directories the Git directories and worktree state live in.
    pub roots: Vec<PathBuf>,
}

/// What:
///  The directories a run watches:
///  the Git directory a worktree's index and `HEAD` live in,
///  and the common Git directory its references and worktree records live in.
/// Why:
///   A lock of the worktree's own state sits in its Git directory,
///  and a linked worktree's is separate from the common one,
///  so both are watched and each event is read against the directory it came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rootsOf(identity) // [gitDir, commonDir] without repeats
/// ```
pub fn roots_of(identity: &WorktreeIdentity) -> Vec<PathBuf> {
    let (git_dir, common_dir): (&Path, &Path) = match identity {
        WorktreeIdentity::MainWorktree { common_dir, git_dir, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, git_dir, .. }
        | WorktreeIdentity::BareRepository { common_dir, git_dir } => (git_dir.as_path(), common_dir.as_path()),
        WorktreeIdentity::OutsideWorktree => return Vec::new(),
    };
    let mut roots: Vec<PathBuf> = vec![git_dir.to_path_buf()];
    if common_dir != git_dir {
        roots.push(common_dir.to_path_buf());
    }
    return roots;
}

/// What:
///  Make every Git this process starts be watched in `watched`,
///  or stop watching.
/// Why:
///   Called once by the coordinated forward,
/// before the first Git of the command starts,
///  so no lock its Git takes is created outside a watch that would have reported it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rememberWatched({ commonDir, roots }) // or rememberWatched(undefined)
/// ```
pub fn remember(watched: Option<Watched>) {
    if let Ok(mut held) = WATCHED.lock() {
        *held = watched;
    }
}

/// What:
///  What this process's Git runs are watched in,
///  or nothing when they are watched in nothing.
/// Why:
///   The run reads it once,
///  and a command outside a repository watches nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watched() // { commonDir, roots } | undefined
/// ```
pub fn remembered() -> Option<Watched> {
    return match WATCHED.lock() {
        Ok(held) => held.clone(),
        Err(_poisoned) => None,
    };
}

/// What:
///  What a run in `identity` should watch,
///  or nothing when it works outside a repository.
/// Why:
///   The plan knows the repository shape and the run does not,
///  so the value is built where both are known and handed over once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchedOf(identity) // { commonDir, roots } | undefined
/// ```
pub fn watched_of(identity: &WorktreeIdentity) -> Option<Watched> {
    let roots: Vec<PathBuf> = roots_of(identity);
    if roots.is_empty() {
        return None;
    }
    let common_dir: PathBuf = match identity {
        WorktreeIdentity::MainWorktree { common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, .. }
        | WorktreeIdentity::BareRepository { common_dir, .. } => common_dir.clone(),
        WorktreeIdentity::OutsideWorktree => return None,
    };
    return Some(Watched { common_dir, roots });
}

/// What:
///  One event the kernel reported about a watched directory:
///  the directory it is about,
///  the mask the kernel set,
///  and the name in that directory.
/// Why:
///  The name is the whole of what the tracker needs,
///  and the directory is read from the watch descriptor,
///  so two roots with one name never mix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Event = { directory: string; mask: number; name: string };
/// ```
pub struct Event {
    /// The watched directory the name is in.
    pub directory: PathBuf,
    /// What the kernel said happened.
    pub mask: u32,
    /// The name in that directory.
    pub name: std::ffi::OsString,
}

/// What one name's last events say about the file it refers to.
#[derive(Clone, Copy, Debug)]
struct Standing {
    /// The sequence number of the name's last creation,
    ///  or nothing while the name refers to no file the run saw created.
    created: Option<u64>,
    /// Whether the name was taken away after that.
    removed: bool,
}

/// What:
///  The names one run saw created,
///  and whether the watch reported something that makes the whole run unprovable.
/// Why:
///  Only the last state of each name matters,
///  and an event queue that overflowed,
///  a watch that ended,
///  or more names than one run follows makes every candidate unprovable,
///  which keeps the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Tracker { feed(events); locksTaken(childPid) }
/// ```
pub struct Tracker {
    /// The sequence number of the next event.
    sequence: u64,
    /// The last state of each name,
    ///  by the directory it is in.
    names: std::collections::HashMap<(PathBuf, std::ffi::OsString), Standing>,
    /// Whether something happened that no candidate survives.
    unprovable: bool,
}

impl Default for Tracker {
    fn default() -> Self {
        return Self::new();
    }
}

impl Tracker {
    /// A tracker that has seen nothing.
    pub fn new() -> Self {
        return Self { sequence: 0, names: std::collections::HashMap::new(), unprovable: false };
    }

    /// What:
    ///  Take the events of one wait.
    /// Why:
    ///  An overflow,
    /// a watch that ended,
    /// and too many names each make every candidate of this run unprovable.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tracker.feed(watch.readEvents())
    /// ```
    pub fn feed(&mut self, events: &[Event]) {
        for event in events {
            if event.mask & (libc::IN_Q_OVERFLOW | libc::IN_IGNORED | libc::IN_UNMOUNT | libc::IN_MOVE_SELF) != 0 {
                self.unprovable = true;
                continue;
            }
            self.sequence += 1;
            let key: (PathBuf, std::ffi::OsString) = (event.directory.clone(), event.name.clone());
            if !self.names.contains_key(&key) && self.names.len() >= MOST_NAMES {
                self.unprovable = true;
                continue;
            }
            let standing: &mut Standing = self.names.entry(key).or_insert(Standing { created: None, removed: true });
            if event.mask & (libc::IN_CREATE | libc::IN_MOVED_TO) != 0 {
                standing.created = Some(self.sequence);
                standing.removed = false;
            }
            if event.mask & (libc::IN_DELETE | libc::IN_MOVED_FROM) != 0 {
                standing.removed = true;
                standing.created = None;
            }
        }
    }

    /// What:
    ///  The locks a child of this process took,
    ///  each with the file its creation left at the path.
    /// Why:
    ///  A lock counts only when the name still refers to the file that was created,
    ///  when its PID file was created after it and also still stands,
    ///  and when that PID file names the child:
    ///  Git writes the PID file after it created the lock (`lockfile.c:182-184`,
    /// Git 2.56.0) and writes none at all when it did not create the lock,
    ///  so a PID file naming the child is the child's own lock's.
    ///  A name whose PID file was created before the lock,
    ///  or whose lock was removed and made again,
    ///  is not the child's lock and is not returned.
    ///  A name that stood before this watch began has no creation here and is never returned either:
    ///  a PID file that was already there,
    ///  which a Git finds and leaves alone,
    ///  is not this child's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// locksTaken(childPid) // [lockPath, …] where the PID file’s number is the child’s
    /// ```
    pub fn locks_taken(&self, child_pid: i64) -> Vec<PathBuf> {
        if self.unprovable {
            return Vec::new();
        }
        let mut locks: Vec<PathBuf> = Vec::new();
        for ((directory, name), standing) in &self.names {
            if locks.len() >= MOST_LOCKS {
                break;
            }
            let Some(created) = standing.created else { continue };
            if standing.removed || !is_a_lock_name(name) {
                continue;
            }
            let pid_name: std::ffi::OsString = pid_name_of(name);
            let Some(pid_standing) = self.names.get(&(directory.clone(), pid_name)) else { continue };
            let Some(pid_created) = pid_standing.created else { continue };
            if pid_standing.removed || pid_created <= created {
                continue;
            }
            let lock: PathBuf = directory.join(name);
            if pid_named_by(pid_file_beside(lock.as_path()).as_path()) != Some(i32::try_from(child_pid).unwrap_or(i32::MIN)) {
                continue;
            }
            locks.push(lock);
        }
        locks.sort();
        return locks;
    }
}

/// What:
///  Whether a name is a lock's rather than the PID file Git writes beside one.
/// Why:
///  A PID file is the evidence about a lock and never a lock itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.endsWith('.lock') && !name.endsWith('~pid.lock')
/// ```
fn is_a_lock_name(name: &std::ffi::OsStr) -> bool {
    let text: std::borrow::Cow<'_, str> = name.to_string_lossy();
    return text.ends_with(LOCK_SUFFIX) && !text.ends_with(PID_INFIX);
}

/// What:
///  The name of the PID file Git writes beside the lock called `name`.
/// Why:
///  Git names it after the lock with `.lock` replaced by `~pid.lock`,
///  so the pair is found from the lock's own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.replace(/\.lock$/, '~pid.lock')
/// ```
fn pid_name_of(name: &std::ffi::OsStr) -> std::ffi::OsString {
    let text: std::borrow::Cow<'_, str> = name.to_string_lossy();
    let base: &str = text.strip_suffix(LOCK_SUFFIX).unwrap_or(text.as_ref());
    return std::ffi::OsString::from(format!("{base}{PID_INFIX}"));
}

/// What:
///  The watch on a run's directories,
///  and the child whose end wakes it.
/// Why:
///  One inotify instance serves every root,
///  and the child's own end wakes the wait at once,
///  so a command that keeps its process around Git waits no longer than it did before the watch existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = Watch.over(roots); watch.follow(child.pid);
/// ```
pub struct Watch {
    /// The inotify instance.
    descriptor: i32,
    /// The child's own descriptor,
    ///  which no platform without `pidfd_open` has.
    child: Option<i32>,
    /// The directory each watch descriptor is about.
    directories: Vec<(i32, PathBuf)>,
}

impl Watch {
    /// What:
    ///  Watch `roots`,
    ///  or nothing when the platform offers no watch or refuses one.
    /// Why:
    ///  A run that cannot watch starts its Git exactly as it did before this watch existed;
    ///  the command is never delayed or refused for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Watch.over([gitDir, commonDir]) // undefined when inotify is unavailable
    /// ```
    pub fn over(roots: &[PathBuf]) -> Option<Watch> {
        use std::os::unix::ffi::OsStrExt;
        // SAFETY: a plain system call whose only argument could not be misread; the descriptor is closed by `Drop`.
        let descriptor: i32 = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
        if descriptor < 0 {
            return None;
        }
        let mask: u32 = libc::IN_CREATE | libc::IN_MOVED_TO | libc::IN_DELETE | libc::IN_MOVED_FROM | libc::IN_ONLYDIR;
        let mut directories: Vec<(i32, PathBuf)> = Vec::new();
        for root in roots {
            let Ok(path) = std::ffi::CString::new(root.as_os_str().as_bytes()) else {
                continue;
            };
            // SAFETY: `path` is a valid NUL-terminated name for the call.
            let watched: i32 = unsafe { libc::inotify_add_watch(descriptor, path.as_ptr(), mask) };
            if watched >= 0 {
                directories.push((watched, root.clone()));
            }
        }
        if directories.is_empty() {
            // SAFETY: the descriptor belongs to this value, which no other holder has.
            unsafe { libc::close(descriptor) };
            return None;
        }
        return Some(Watch { descriptor, child: None, directories });
    }

    /// What:
    ///  Let the child's own end wake this watch's waits.
    /// Why:
    ///  Without it the wait would sleep to its step before the run noticed that Git ended,
    ///  which is a delay of every index writer and of nothing else.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watch.follow(child.id()) // pidfd
    /// ```
    pub fn follow(&mut self, child: u32) {
        // SAFETY: `pidfd_open` takes a number this process may ask about and a flag word; it returns a descriptor or an error.
        let opened: i64 = unsafe { libc::syscall(libc::SYS_pidfd_open, libc::pid_t::try_from(child).unwrap_or(0), 0) };
        let Ok(descriptor) = i32::try_from(opened) else {
            return;
        };
        if descriptor >= 0 {
            self.child = Some(descriptor);
        }
    }

    /// What:
    ///  The events the kernel reported,
    ///  waiting no longer than `timeout` and no longer than the child's own life.
    /// Why:
    ///  The caller looks at its child between two waits,
    ///  and a child that ends wakes this call at once where the platform offers `pidfd_open`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watch.wait(WAIT_STEP) // Event[]
    /// ```
    pub fn wait(&self, timeout: std::time::Duration) -> Vec<Event> {
        let mut entries: Vec<libc::pollfd> = vec![libc::pollfd { fd: self.descriptor, events: libc::POLLIN, revents: 0 }];
        if let Some(child) = self.child {
            entries.push(libc::pollfd { fd: child, events: libc::POLLIN, revents: 0 });
        }
        let milliseconds: i32 = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX).max(1);
        // SAFETY: the entries name descriptors this value owns and the count matches the vector's length.
        let ready: i32 = unsafe { libc::poll(entries.as_mut_ptr(), libc::nfds_t::try_from(entries.len()).unwrap_or(1), milliseconds) };
        if ready <= 0 {
            return Vec::new();
        }
        return self.read_events();
    }

    /// The step one wait sleeps for at most:
    /// the child's end wakes it where it can, and the step is short elsewhere.
    fn step(&self) -> std::time::Duration {
        return match self.child {
            Some(_) => WAIT_STEP,
            None => WAIT_STEP_WITHOUT_PIDFD,
        };
    }

    /// What:
    ///  Everything the kernel holds for this watch,
    ///  read now.
    /// Why:
    ///  The queue outlives the run's child,
    ///  so the end of a run reads it once more and finds what arrived while it was collecting the child.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watch.readEvents()
    /// ```
    pub fn read_events(&self) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        let mut buffer: [u8; 8192] = [0; 8192];
        loop {
            // SAFETY: the buffer is valid for its whole length and the descriptor is this watch's.
            let read: isize = unsafe { libc::read(self.descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
            let Ok(count) = usize::try_from(read) else {
                return events;
            };
            if count == 0 {
                return events;
            }
            for (watched, mask, name) in events_in(&buffer[..count]) {
                let found: Option<PathBuf> = self
                    .directories
                    .iter()
                    .find(|(descriptor, _)| { return *descriptor == watched; })
                    .map(|(_, root)| { return root.clone(); });
                let Some(directory) = found else {
                    continue;
                };
                use std::os::unix::ffi::OsStringExt as _;
                events.push(Event { directory, mask, name: std::ffi::OsString::from_vec(name.to_vec()) });
            }
        }
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        // SAFETY: the descriptors belong to this value and are closed once each.
        unsafe { libc::close(self.descriptor) };
        if let Some(child) = self.child {
            // SAFETY: the descriptor was opened by `follow` and belongs to this value.
            unsafe { libc::close(child) };
        }
    }
}

/// What:
///  Open the watch a child's start needs,
///  from what the plan remembered for this command,
///  before that child starts.
/// Why:
///  A lock is created microseconds after its Git starts,
///  so a watch opened after the start would miss the events that prove whose lock it is;
///  a command that remembered nothing watches nothing and starts its Git exactly as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = watchBeforeStarting(); // undefined when the plan remembered no roots
/// ```
pub fn watch_before_starting() -> Option<(Watch, Watched)> {
    let watched: Watched = remembered()?;
    let watch: Watch = Watch::over(watched.roots.as_slice())?;
    return Some((watch, watched));
}

/// What:
///  Wait for `child` while the directories its Git was started in are watched,
///  or wait for it exactly as before when nothing was watched.
/// Why:
///   One call is the whole difference between a run that records what its Git took and one that does not,
///  so every start of a Git that may take a lock of this repository makes it.
///  The child's identity is read while it runs,
///  because a record of a number without a birth names nobody once the child has ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await waitForAWatchedChild(child, watch);
/// ```
pub fn wait_for_a_watched_child(child: &mut std::process::Child, watch: Option<(Watch, Watched)>) -> std::io::Result<std::process::ExitStatus> {
    let Some((opened, about)) = watch else {
        return child.wait();
    };
    let Some(identity) = crate::lock_holder_records::identity_of_process(i64::from(child.id())) else {
        drop(opened);
        return child.wait();
    };
    return wait_and_bind(child, opened, about.common_dir.as_path(), identity.as_str());
}

/// What:
///  Wait for `child` while the watched directories are read,
///  writing a record for each lock the child is seen to take.
/// Why:
///  The record must exist before a kill that takes the child and this command together,
///  so it is written as the event that proves it arrives and never after the child ends alone.
///  The child's identity is read before it can end:
///  a record of a number without a birth names nobody.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await waitAndBind(child, watch, { commonDir, identity });
/// ```
pub fn wait_and_bind(
    child: &mut std::process::Child,
    mut watch: Watch,
    common_dir: &Path,
    identity: &str,
) -> std::io::Result<std::process::ExitStatus> {
    watch.follow(child.id());
    let mut tracker: Tracker = Tracker::new();
    let mut bound: Vec<Written> = Vec::new();
    let child_pid: i64 = i64::from(child.id());
    loop {
        if let Some(status) = child.try_wait()? {
            tracker.feed(watch.read_events().as_slice());
            bind_what_the_child_took(&tracker, common_dir, child_pid, identity, &mut bound);
            forget_what_the_child_finished(common_dir, &bound);
            return Ok(status);
        }
        tracker.feed(watch.wait(watch.step()).as_slice());
        bind_what_the_child_took(&tracker, common_dir, child_pid, identity, &mut bound);
    }
}

/// What:
///  Write a record for every lock the tracker now reports and this run has not recorded.
/// Why:
///  A record that cannot be written leaves the lock to the wait it has today,
///  and the record of a lock that is written once is not written again on every look.
///  What is remembered is the file the record pinned, not the path it was taken at:
///  one Git can take the same lock path twice,
///  renaming the first over its target and taking the path again,
///  and the second lock is a different file whose record the pass needs
///  (`a_child_that_takes_the_same_lock_again_is_recorded_again`).
///  A path alone would tell the second take from the first and skip it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const lock of tracker.locksTaken(pid)) if (!written.some(w => w.name === nameOf(lock))) writeBinding(lock)
/// ```
fn bind_what_the_child_took(tracker: &Tracker, common_dir: &Path, child_pid: i64, identity: &str, written: &mut Vec<Written>) {
    for lock in tracker.locks_taken(child_pid) {
        let Ok(Some(file)) = crate::index_lock_evidence::read_lock_metadata(lock.as_path()) else {
            continue;
        };
        let name: String = lock_bindings::name_of(lock.as_path(), (file.device, file.inode));
        if written.iter().any(|held| { return held.name() == name; }) {
            continue;
        }
        if let Some(recorded) = lock_bindings::write(common_dir, lock.as_path(), (file.device, file.inode), &(child_pid, identity.to_owned())) {
            written.push(recorded);
        }
    }
}

/// What:
///  Take away the record of every lock this child finished,
///  which is a child that renamed its lock over its target or rolled it back.
/// Why:
///  The record of a lock that is gone explains nothing,
///  and this command is the one that knows the child finished:
///  leaving it to the next command's sweep writes a file every index writer would then read and remove.
///  A lock that still stands as the pin does is the killed child's and keeps its record.
///  What goes is the two files of the very record this child's run wrote,
///  named by this child's lock path and the file it took,
///  and never the files a look at the path would find now:
///  another command's Git can take that path the instant this child's lock is renamed away,
///  and a removal that decided on the path would take away the record of that other lock and leave a lock whose
///  proof is gone
///  (a review of `f8f5cba090`,
/// finding 1).
///  The name this child's run worked out cannot belong to that other lock:
///  the pin still holds this child's inode,
///  so the file that took the path afterwards is a different file with a different name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const written of bound) if (!pinHoldsTheLock(written)) removeBinding(written);
/// ```
fn forget_what_the_child_finished(common_dir: &Path, bound: &[Written]) {
    for written in bound {
        if lock_bindings::pin_holds_the_lock(common_dir, written) {
            continue;
        }
        // A control reaches here to act as a command whose Git takes the freed name at this instant.
        crate::service_fault::reach(A_FINISHED_CHILD_FORGETS);
        lock_bindings::remove(common_dir, written);
    }
}

/// What:
///  One 32-bit field of an inotify event,
///  `offset` bytes into the event that starts at `at`,
///  read in the host's own byte order.
/// Why:
///  `events_in` reads the watch descriptor and the mask of each event of a buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldAt = (buffer: Buffer, at: number, offset: number) => buffer.readUInt32LE(at + offset);
/// ```
fn field_at(buffer: &[u8], at: usize, offset: usize) -> u32 {
    let mut bytes: [u8; 4] = [0; 4];
    bytes.copy_from_slice(&buffer[at + offset..at + offset + 4]);
    return u32::from_ne_bytes(bytes);
}

/// What:
///  The events one buffer holds:
///  the watch each is about,
///  its mask,
///  and the name in the watched directory,
///  empty for an event about the watch itself.
/// Why:
///   Bytes are parsed,
///  not cast,
///  so an unaligned buffer is read as safely as an aligned one;
///  a record cut off by the end of the buffer ends the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// eventsIn(buffer) // [{ watched, mask, name }]
/// ```
fn events_in(buffer: &[u8]) -> Vec<(i32, u32, &[u8])> {
    let header: usize = std::mem::size_of::<libc::inotify_event>();
    let mut events: Vec<(i32, u32, &[u8])> = Vec::new();
    let mut at: usize = 0;
    while at + header <= buffer.len() {
        let length: usize = usize::try_from(field_at(buffer, at, 12)).unwrap_or(usize::MAX);
        let start: usize = at + header;
        let end: usize = start.saturating_add(length).min(buffer.len());
        let cut: Option<usize> = buffer[start..end].iter().position(|byte: &u8| { return *byte == 0; });
        let ended: usize = cut.unwrap_or(end - start);
        let name: &[u8] = &buffer[start..start + ended];
        let watched: i32 = i32::try_from(field_at(buffer, at, 0)).unwrap_or(-1);
        events.push((watched, field_at(buffer, at, 4), name));
        at = start.saturating_add(length);
    }
    return events;
}
