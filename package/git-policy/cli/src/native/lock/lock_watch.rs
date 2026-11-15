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
//!  a watch that ended (`IN_IGNORED`),
//!  a folder's file system that was unmounted (`IN_UNMOUNT`),
//!  a watched folder that was moved or deleted (`IN_MOVE_SELF`),
//!  more names than one run follows,
//!  and a platform without a watch leave the lock to the bounded wait and report it has today,
//!  and each of them makes that run's record say nothing:
//!  the reader delivers every event that says events were lost or a watch ended,
//!  whatever watch descriptor it carries,
//!  and the tracker holds the whole run unprovable from it
//!  (`Tracker::feed`).
//!  A folder made after the watch began is watched in its turn,
//!  the folders are watched one depth at a time so the bound reaches the ones locks are taken in
//!  first,
//!  and a folder the watch's bound leaves unwatched reports nothing:
//!  a lock taken there gets no record, and a lock with no record is kept.
//!  A repository whose Git directory holds more folders than the bound
//!  (this repository's does) therefore keeps a lock in a deep folder,
//!  and the sentence about that lock says what is known.
//!  Every Git the wrapper starts for its own ladder is watched too,
//!  from a watcher thread of its own:
//!  those steps wait for their Git through pipes, a descriptor and a progress watch,
//!  and a branch write is one of them,
//!  so a merge killed while it moved the branch leaves a lock whose record exists.
//!  What a kill at each step of a run leaves,
//!  and who removes it:
//!
//!  - after the path's text and before any record:
//!    a file naming the path and no record,
//!    which the next write for that path reuses and nothing removes,
//!    because a record is a pin and there is none;
//!  - after the record, which is one file:
//!    a record whose pin names the lock,
//!    which this command takes away as soon as a look finds the child finished that lock
//!    (`forget_what_the_child_finished`, called on every look and not only at the child's end,
//!    so one run of many takes holds at most the take that stands and the one just finished)
//!    and which the next commands keep while the pin names the file standing at the path,
//!    because it is the proof that clears a lock a killed child left;
//!  - inside a removal:
//!    one `unlink`, which the kernel does whole,
//!    so no half-removed record exists.
//!
//!  No step of a sweep, a forget or a write can take away the record of a lock another command's Git
//!  holds or was killed holding:
//!  a record is one file,
//!  written by one `link`,
//!  named for the take it belongs to
//!  (the path's digest with that Git's process number and birth identity,
//! `lock_bindings.rs`),
//!  so nothing ever meets a half-published record
//!  and no removal acts on a name another command's write produces
//!  (a review of `2ac27d2a29`,
//! finding 1).
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

/// The most folders one watch opens,
/// so a repository with a vast tree cannot exhaust the machine's inotify budget.
/// A folder this bound leaves unwatched costs only what an unwatched folder cost before the watch
/// learned to descend:
/// a lock taken there gets no record, and is then cleared only by a proof that does not need one.
/// The walk spends the bound one depth at a time,
/// so the folders locks are taken in are watched first;
/// this repository's common Git directory alone holds more folders than this
/// (more than 3500 on 2026-11-14, most of them the rerere cache's and the linked worktrees' own),
/// so a lock in one of its deeper folders is met by a command that keeps it and says why.
const MOST_WATCHED_FOLDERS: usize = 1024;

/// How deep below a root the watch descends,
/// the depth the pass that judges locks reads to.
const MOST_WATCH_DEPTH: usize = 8;

/// What one watch asks the kernel for:
/// the creations and removals of names in a folder,
/// and a folder that is made into one.
/// Why:
///   A lock is a name created in a folder and renamed away from it,
///  and a folder made after the watch began has to be watched in its turn,
///  which is why the kernel's own "this is a directory" bit matters here.
const WATCHED_EVENTS: u32 = libc::IN_CREATE | libc::IN_MOVED_TO | libc::IN_DELETE | libc::IN_MOVED_FROM | libc::IN_ONLYDIR;

/// The folders below a Git directory that hold no lock of the worktree's state,
/// which the pass skips too:
/// the object store, whose `*.lock` files are a pack writer's own,
/// and the logs.
const UNWATCHED_FOLDERS: [&str; 2] = ["objects", "logs"];

/// The most locks one run writes a record for.
const MOST_LOCKS: usize = 32;

/// The instant the tracker has reported a take of a lock path,
/// and before the record of that take is pinned:
/// the two looks the record's name and its holder come from are this one and the pinning itself.
/// Why:
///   A control states the interleaving between them:
///  the Git whose take was reported finishes that take and another Git takes the path in the gap,
///  and the record must not name a Git that did not hold the file it pins
///  (`a_take_that_finished_before_it_was_pinned_is_not_recorded_for_that_git`).
///  Named here so both halves of that promise reach one point,
/// and inert unless the test variable or a control names it.
pub const BEFORE_A_TAKE_IS_PINNED: &str = "before-a-take-is-pinned";

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

    /// Whether this run has already seen something no record of it may be kept through,
    /// so a watcher may stop reading and holding its watch.
    /// Why:
    ///  An overflow, a watch that ended or too many names make every candidate unprovable,
    ///  so no later event of this run can produce a record,
    ///  and a run that waits for a long Git then costs no descriptor and no thread.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tracker.nothingMoreCanBeRecorded() // true after an overflow
    /// ```
    pub fn nothing_more_can_be_recorded(&self) -> bool {
        return self.unprovable;
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

// The events a control states as made up, delivered by `read_events` as if the kernel had reported them,
// so a shape the kernel rarely or never gives this machine can be tested.  Never part of the release executable.
#[cfg(test)]
thread_local! {
    /// The made-up events of the running thread, in the order they are delivered.
    static MADE_UP: std::cell::RefCell<Vec<(i32, u32, std::ffi::OsString)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// What:
///  Run `body` with `events` delivered by every read as if the kernel had reported them,
///  and put the thread's own list back however the body ends.
/// Why:
///  A queue overflow and a watch that ended are shapes this machine's limits make impossible to reach by
///  producing them,
///  and what the wrapper believes after one of them is what a control has to state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withMadeUpEvents([[(-1, IN_Q_OVERFLOW, '')]], () => expect(readEvents()).toContainOverflowEvent())
/// ```
#[cfg(test)]
pub fn with_made_up_events<T>(events: Vec<(i32, u32, std::ffi::OsString)>, body: impl FnOnce() -> T) -> T {
    let put_back: Vec<(i32, u32, std::ffi::OsString)> = MADE_UP.with(|held| { return held.replace(events); });
    let ended: T = body();
    MADE_UP.with(|held| { let _replaced = held.replace(put_back); });
    return ended;
}

/// What:
///  The made-up events of this thread,
///  taken out so that one read delivers them once.
/// Why:
///   An event is reported once,
///  as the kernel reports it once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takeMadeUpEvents() // []
/// ```
#[cfg(test)]
fn take_made_up_events() -> Vec<(i32, u32, std::ffi::OsString)> {
    return MADE_UP.with(|held| { return std::mem::take(&mut *held.borrow_mut()); });
}

/// Does nothing in the release executable.
#[cfg(not(test))]
fn take_made_up_events() -> Vec<(i32, u32, std::ffi::OsString)> {
    return Vec::new();
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
    /// The roots the watch was opened for,
    ///  which an event that names no folder is reported against.
    roots: Vec<PathBuf>,
}

impl Watch {
    /// What:
    ///  Watch `roots` and every folder below them,
    ///  or nothing when the platform offers no watch or refuses one.
    /// Why:
    ///  A watch on a folder does not cover its subfolders,
    ///  and Git takes reference locks in `refs/heads`, `refs/tags`, the reftable's folder and others:
    ///  a merge or a rebase updates a branch through such a lock,
    ///  so a watch of the roots alone would leave a killed Git's reference lock with no record
    ///  (a review of `fee5557595`,
    /// finding 1).
    ///  So the folders of the roots are watched at the depth the lock pass reads,
    ///  the same two skipped folders that hold no lock of the worktree's state are skipped,
    ///  and the count is bounded.
    ///  A folder the bound leaves unwatched costs no safety:
    ///  a lock taken there gets no record, which is the state every lock was in before this watch
    ///  existed.
    ///  A run that cannot watch starts its Git exactly as it did before this watch existed;
    ///  the command is never delayed or refused for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Watch.over([gitDir, commonDir]) // the two roots and every folder below them
    /// ```
    pub fn over(roots: &[PathBuf]) -> Option<Watch> {
        // SAFETY: a plain system call whose only argument could not be misread; the descriptor is closed by `Drop`.
        let descriptor: i32 = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
        if descriptor < 0 {
            return None;
        }
        let mut watch: Watch = Watch { descriptor, child: None, directories: Vec::new(), roots: roots.to_vec() };
        for root in roots {
            watch.watch_folder_and_below(root.as_path());
        }
        if watch.directories.is_empty() {
            return None;
        }
        return Some(watch);
    }

    /// What:
    ///  Open a watch on `directory`,
    ///  and on every folder below it that the bounds leave room for,
    ///  the shallowest folders first.
    /// Why:
    ///  One walk covers the folders that are there when the watch opens,
    ///  and the events the walk's own watches report cover the folders made later.
    ///  The walk goes one depth at a time,
    ///  so the bound of `MOST_WATCHED_FOLDERS` is spent on the folders locks are taken in first:
    ///  `refs/heads`, `refs/tags`, `reftable` and a linked worktree's own directory all sit a few steps below a root,
    ///  and a deep tree of a program's own files
    ///  (this repository's `.git` holds more than three thousand folders,
    ///  most of them the rerere cache's) would otherwise use the whole bound before the walk reached them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watchFolderAndBelow(gitDir) // gitDir, then refs and the other shallow folders, one depth at a time
    /// ```
    fn watch_folder_and_below(&mut self, directory: &Path) {
        let mut pending: std::collections::VecDeque<(PathBuf, usize)> = std::collections::VecDeque::new();
        pending.push_back((directory.to_path_buf(), 0));
        while let Some((folder, depth)) = pending.pop_front() {
            if self.watch_one(folder.as_path()) && depth < MOST_WATCH_DEPTH {
                let Ok(entries) = std::fs::read_dir(folder.as_path()) else {
                    continue;
                };
                let mut below: Vec<PathBuf> = Vec::new();
                for entry in entries.flatten() {
                    let Ok(kind) = entry.file_type() else {
                        continue;
                    };
                    let name: std::borrow::Cow<'_, str> = entry.file_name().to_string_lossy().into_owned().into();
                    if kind.is_dir() && !UNWATCHED_FOLDERS.contains(&name.as_ref()) {
                        below.push(entry.path());
                    }
                }
                // One folder's names are read in the order the file system lists them, and sorted so two runs of the
                //  same repository watch the same folders within the bound.
                below.sort();
                for path in below {
                    if self.directories.len() >= MOST_WATCHED_FOLDERS {
                        return;
                    }
                    pending.push_back((path, depth + 1));
                }
            }
        }
    }

    /// What:
    ///  Open a watch on one folder,
    ///  and say whether it is watched now.
    /// Why:
    ///  A folder the bound or the platform refuses is not watched,
    ///  and the caller then reads nothing below it;
    ///  a folder this watch already has is not opened twice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watchOne(folder) // true when it is watched
    /// ```
    fn watch_one(&mut self, folder: &Path) -> bool {
        use std::os::unix::ffi::OsStrExt as _;
        if self.directories.len() >= MOST_WATCHED_FOLDERS {
            return false;
        }
        let Ok(path) = std::ffi::CString::new(folder.as_os_str().as_bytes()) else {
            return false;
        };
        // SAFETY: `path` is a valid NUL-terminated name for the call.
        let watched: i32 = unsafe { libc::inotify_add_watch(self.descriptor, path.as_ptr(), WATCHED_EVENTS) };
        if watched < 0 {
            return false;
        }
        if !self.directories.iter().any(|(descriptor, _)| { return *descriptor == watched; }) {
            self.directories.push((watched, folder.to_path_buf()));
        }
        return true;
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
    pub fn wait(&mut self, timeout: std::time::Duration) -> Vec<Event> {
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

    /// Whether the child's own descriptor is open,
    /// so its end wakes this watch and is readable from it.
    /// Why:
    ///  A watcher that reads instead of waiting for the child needs to know whether it can ask this watch
    ///  about the child's end at all, or has to ask the process table.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watch.hasChildDescriptor()
    /// ```
    fn has_child_descriptor(&self) -> bool {
        return self.child.is_some();
    }

    /// Whether the child this watch follows has ended,
    /// asked without waiting.
    /// Why:
    ///  A watcher that is not the child's waiter still has to stop when the child ends,
    ///  and the descriptor reports the end whether or not the waiter has collected the child.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watch.theChildHasEnded() // poll(pidfd, 0) > 0
    /// ```
    fn the_child_has_ended(&self) -> bool {
        let Some(child) = self.child else {
            return false;
        };
        let mut entry: libc::pollfd = libc::pollfd { fd: child, events: libc::POLLIN, revents: 0 };
        // SAFETY: the entry is one this call owns and the timeout is nothing, so the call cannot block; it reports
        //         whether the child's descriptor is ready and touches no memory beyond the entry.
        let ready: i32 = unsafe { libc::poll(&mut entry, 1, 0) };
        return ready > 0;
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
    pub fn read_events(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        // What: A control's made-up events are delivered first, as if the kernel had reported them before the queue.
        // Why:  A shape this machine cannot produce still has to reach the tracker in a control.
        for (watched, mask, name) in take_made_up_events() {
            self.deliver(watched, mask, name.as_encoded_bytes(), &mut events);
        }
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
                self.deliver(watched, mask, name, &mut events);
            }
        }
    }

    /// What:
    ///  Deliver one event of the kernel's,
    ///  or the one a control stated,
    ///  to `events`.
    /// Why:
    ///  One mapping serves the kernel's events and a control's made-up ones,
    ///  so a shape a control states is read exactly as the kernel's would be read:
    ///  an event that names a watched folder is about that folder,
    ///  an event that says events were lost or a watch ended is about the whole run whatever
    ///  descriptor it carries,
    ///  and a folder made into a watched one is watched in its turn.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// deliver(wd, mask, name) // push an Event, or watch a new folder
    /// ```
    fn deliver(&mut self, watched: i32, mask: u32, name: &[u8], events: &mut Vec<Event>) {
        use std::os::unix::ffi::OsStringExt as _;
        let found: Option<PathBuf> = self.directories.iter().find(|(descriptor, _)| { return *descriptor == watched; }).map(|(_, root)| { return root.clone(); });
        let Some(directory) = found else {
                    // What: An event that names no watched folder is delivered as the whole run's event when it says the
                    //       kernel dropped events or ended a watch: the kernel reports an overflow with a watch
                    //       descriptor of -1 (`inotify(7)`), and the tracker must hear it.
                    // Why:  With events lost, a replacement lock another live command took can be taken for the first
                    //       child's and recorded under that child, and later cleared while the other command works with
                    //       no descriptor open (a review of `fee5557595`, finding 2).
            if mask & (libc::IN_Q_OVERFLOW | libc::IN_IGNORED | libc::IN_UNMOUNT | libc::IN_MOVE_SELF) != 0 {
                let somewhere: PathBuf = self.roots.first().cloned().unwrap_or_default();
                events.push(Event { directory: somewhere, mask, name: std::ffi::OsString::new() });
            }
            return;
        };
        if mask & libc::IN_ISDIR != 0 {
                    // What: A folder was made or moved into a watched one: it is watched in its turn, and the folder
                    //       itself is no name a lock is recorded from.
                    // Why:  Git makes `refs/heads/feature/` for a branch whose name has a slash in it, and a reftable
                    //       repository has no `reftable` folder until the first write; a watch that knew only the
                    //       folders of its first look would miss the locks taken in those.
                    //       A lock made in such a folder before this watch was opened on it is missed, and a lock with no
                    //       record is one the pass keeps: the safe side.
            if mask & (libc::IN_CREATE | libc::IN_MOVED_TO) != 0 {
                let made: PathBuf = directory.join(std::ffi::OsString::from_vec(name.to_vec()));
                self.watch_folder_and_below(made.as_path());
            }
            return;
        }
        events.push(Event { directory, mask, name: std::ffi::OsString::from_vec(name.to_vec()) });
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
    let mut binding: Binding = Binding { tracker: Tracker::new(), bound: Vec::new(), common_dir: common_dir.to_path_buf(), child_pid: i64::from(child.id()), identity: identity.to_owned() };
    loop {
        if let Some(status) = child.try_wait()? {
            binding.drain(&mut watch);
            return Ok(status);
        }
        // Each lock's creation and each lock's rename away wake this wait, so a take that is gone is taken away
        //  before the next one is written: what this run holds for one path is one take, and two in the instant
        //  between them.
        binding.look(&mut watch);
    }
}

/// What:
///  What one run's watch has been turned into so far:
///  the events read,
///  the records written,
///  and the Git those records may name.
/// Why:
///  Turning events into records is one job,
///  and two callers need it:
///  a wait that blocks on the child's own end (`wait_and_bind`),
///  and a thread that reads the same watch while its caller waits for the child some other way
///  (`bind_while_the_caller_waits`,
///  which is how a Git of the wrapper's own ladder is watched).
///  Written once, the two cannot disagree about when a record is pinned or taken away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Binding { look(watch); drain(watch); } // feed, pin, forget
/// ```
struct Binding {
    /// What the events read so far say about every name.
    tracker: Tracker,
    /// The records this run wrote and still holds.
    bound: Vec<Written>,
    /// The common Git directory the records are written under.
    common_dir: PathBuf,
    /// The Git this run may write records about.
    child_pid: i64,
    /// That Git's birth identity, read while it ran.
    identity: String,
}

impl Binding {
    /// What:
    ///  Read what one wait reported, and turn it into records.
    /// Why:
    ///  The wait reports every creation and every removal between two looks,
    ///  and a take that is gone is taken away before the next one is written.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// binding.look(watch)
    /// ```
    fn look(&mut self, watch: &mut Watch) {
        let events: Vec<Event> = watch.wait(watch.step());
        self.take(events);
    }

    /// What:
    ///  Read everything the kernel holds for this watch now, and turn it into records.
    /// Why:
    ///  The queue outlives the child,
    ///  so the last look finds what arrived while the run was collecting it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// binding.drain(watch)
    /// ```
    fn drain(&mut self, watch: &mut Watch) {
        let events: Vec<Event> = watch.read_events();
        self.take(events);
    }

    /// What:
    ///  Turn `events` into the records they prove, and take away the records of the takes they end.
    /// Why:
    ///  One place, so a wait and a watcher thread record the same thing for the same events.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// binding.take(watch.readEvents())
    /// ```
    fn take(&mut self, events: Vec<Event>) {
        self.tracker.feed(events.as_slice());
        bind_what_the_child_took(&self.tracker, self.common_dir.as_path(), self.child_pid, self.identity.as_str(), &mut self.bound);
        forget_what_the_child_finished(&mut self.bound);
    }
}

/// How long a binder's caller waits for the watcher thread once the child has ended.
/// Why:
///  The thread wakes at the child's end at once where the platform offers a descriptor for it,
///  and this bound only covers the microseconds of the drain that follows,
///  so a caller that waits for its child is never held by the watcher.
const BINDER_FINISH_WAIT: std::time::Duration = std::time::Duration::from_millis(250);

/// What:
///  A thread that turns one child's lock takes into records while somebody else waits for that child.
/// Why:
///  The record must exist before a kill that takes the command and its Git together,
///  so it cannot wait for the child's end,
///  and the wrapper's own ladder steps cannot be watched by their waiter:
///  they wait for their Git through pipes, a pidfd and a progress watch,
///  on the runtime,
///  and the caller of such a step is no place to read a watch.
///  So the watch is read on a thread of its own,
///  which ends when the child ends,
///  and the caller finishes it after its own wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binder = bindWhileTheCallerWaits(watched, child.id()); await step(); binder.finish();
/// ```
pub struct Binder {
    /// The end of the watcher thread, which says its last look was made.
    ended: Option<std::sync::mpsc::Receiver<()>>,
    /// The watcher thread itself, so it is not waited for when the caller left its child running.
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Binder {
    /// What:
    ///  Wait, briefly, for the watcher's last look after the child has ended.
    /// Why:
    ///  A caller that collected its child is about to go on,
    ///  and a record of the child's last take has to be written before the command can be killed.
    ///  The wait is short because the thread ends with the child,
    ///  and a command never waits longer for its own bookkeeping than for its Git.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// binder.finish()
    /// ```
    pub fn finish(mut self) {
        if let Some(ended) = self.ended.take() {
            let _outlived = ended.recv_timeout(BINDER_FINISH_WAIT);
        }
    }
}

impl Drop for Binder {
    /// Leave the thread to end with its child,
    /// which is what a step the command stopped waiting for needs.
    fn drop(&mut self) {
        let _thread = self.thread.take();
        let _end = self.ended.take();
    }
}

/// What:
///  Watch the child `child_pid` in the directories `watched` names,
///  from a thread of its own,
///  and write a record for each lock it is seen to take.
/// Why:
///  A step of the wrapper's own ladder starts its Git through the transaction runner,
///  which waits for it through pipes and a progress watch;
///  the Git that moves a branch (`merge --ff-only`, `update-ref`) takes `refs/heads/<name>.lock` there,
///  and a watch opened only for the command's own forwarded child never sees it:
///  a merge killed while it moved the branch left a reference lock no later command could clear
///  (the fifth review's finding 1,
///  and `a_reference_lock_of_a_killed_merge_is_cleared_by_its_record`).
///  A command outside a repository,
///  and one whose plan remembered no directories,
///  watches nothing.
///  A thread that cannot start leaves the lock to the bounded wait it had before this watch existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bindWhileTheCallerWaits(watched, child.id()) // Binder | undefined
/// ```
pub fn bind_while_the_caller_waits(watched: Option<(Watch, Watched)>, child_pid: u32) -> Option<Binder> {
    let (mut watch, about) = watched?;
    let child_pid_named: i64 = i64::from(child_pid);
    let identity: String = crate::lock_holder_records::identity_of_process(child_pid_named)?;
    watch.follow(child_pid);
    let (ended, listened) = std::sync::mpsc::channel::<()>();
    let thread: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .name(String::from("cli-git-lock-binder"))
        .spawn(move || {
            bind_until_the_child_ends(watch, about, child_pid_named, identity, ended);
        })
        .ok()?;
    return Some(Binder { ended: Some(listened), thread: Some(thread) });
}

/// What:
///  Read the watch until the child it follows has ended,
///  turning every take into a record,
///  then read what the kernel holds once more and say the watch is done.
/// Why:
///  The thread is the waiter of nothing but the watch:
///  its child's end is read from the child's own descriptor,
///  or from the process table where the platform has none,
///  so it never waits for the process the command itself collects.
///  A run whose events were lost stops at once: no later event of it can produce a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (!childEnded()) { binding.look(watch) } binding.drain(watch);
/// ```
fn bind_until_the_child_ends(mut watch: Watch, about: Watched, child_pid: i64, identity: String, ended: std::sync::mpsc::Sender<()>) {
    let mut binding: Binding = Binding { tracker: Tracker::new(), bound: Vec::new(), common_dir: about.common_dir, child_pid, identity };
    loop {
        binding.look(&mut watch);
        if binding.tracker.nothing_more_can_be_recorded() || the_child_has_ended(&watch, child_pid, binding.identity.as_str()) {
            break;
        }
    }
    binding.drain(&mut watch);
    drop(ended);
}

/// What:
///  Whether the child `child_pid` has ended,
///  by its own descriptor where the watch has one,
///  and by the process table otherwise.
/// Why:
///  A watcher that is not the child's waiter still has to end with the child,
///  and a process number alone is no proof:
///  a number the table gives to another process is another birth,
///  which is what the record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watch.hasChildDescriptor() ? watch.theChildHasEnded() : identityOfProcess(pid) !== identity
/// ```
fn the_child_has_ended(watch: &Watch, child_pid: i64, identity: &str) -> bool {
    if watch.has_child_descriptor() {
        return watch.the_child_has_ended();
    }
    return !matches!(crate::lock_holder_records::identity_of_process(child_pid), Some(now) if now == identity);
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
        // A take of this path this run has already recorded, and whose record still names the file standing there, is
        //  recorded once: a Git that renames its lock away and takes the path again is a different file and is written
        //  again (`a_child_that_takes_the_same_lock_again_is_recorded_again`).
        if written.iter().any(|held| { return held.lock == lock && lock_bindings::pin_holds_the_lock(held); }) {
            continue;
        }
        // A control reaches here, with the take reported and its record not yet pinned, to act as the Git finishing
        //  the take and another Git taking the path in the gap.
        crate::service_fault::reach(BEFORE_A_TAKE_IS_PINNED);
        if let Some(recorded) = lock_bindings::write(common_dir, lock.as_path(), &(child_pid, identity.to_owned())) {
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
///  What goes is the one file that is the record this child's run wrote,
///  named by this child's lock path and the file it took,
///  and never the files a look at the path would find now:
///  another command's Git can take that path the instant this child's lock is renamed away,
///  and a removal that decided on the path would take away the record of that other lock and leave a lock whose
///  proof is gone
///  (a review of `f8f5cba090`,
/// finding 1).
///  The record this child's run wrote is named for the take it belongs to,
///  the path's digest with the file the take made and that Git's process number and birth
///  identity,
///  which no other command's write produces,
///  so a later lock at that path has a record of its own and this one is never it.
///  This runs on every look the wait takes, not only when the child ends:
///  a Git of many takes, such as a rebase of many commits, takes the same path once per pick,
///  and every record still held keeps one whole copy of the index alive on disk
///  (a review of `1ee73d2728`, finding F1).
///  What this run holds for one path is therefore at most the take standing at it and the take whose
///  rename the wait has not yet looked at, whatever the length of the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const written of bound) if (!pinHoldsTheLock(written)) { removeBinding(written); bound.delete(written) }
/// ```
fn forget_what_the_child_finished(bound: &mut Vec<Written>) {
    let mut finished: Vec<Written> = Vec::new();
    bound.retain(|written| {
        if lock_bindings::pin_holds_the_lock(written) {
            return true;
        }
        finished.push(written.clone());
        return false;
    });
    for written in finished {
        // A control reaches here to act as a command whose Git takes the freed name at this instant.
        crate::service_fault::reach(A_FINISHED_CHILD_FORGETS);
        lock_bindings::remove(&written);
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
