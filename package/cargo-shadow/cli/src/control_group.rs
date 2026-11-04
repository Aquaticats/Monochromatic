//! What:
//! A delegated control group (cgroup v2) as the boundary of everything a bounded run starts.
//! Why:
//! Round 7 (sixth review 6a R1 to R3, 6b2 2, 6c R2):
//! a process group, and a child subreaper with a `/proc` scan, failed the one proof that matters,
//! that every writer a command started has stopped.
//! A control group is inherited by every child, also by those that call `setsid` or fork twice,
//! it keeps its members when this program dies,
//! `cgroup.events` says `populated 0` only when no thread is left,
//! and a recovery can ask the same question about a group the journal recorded.
//! The measurements are in `doc/planning/cargo-shadow-update.md` ("Round 7: control group probes").
//! The limits:
//! a member of the same user can move itself out by writing to another group's `cgroup.procs`,
//! so the group contains ordinary escapes, not a deliberate one;
//! `cgroup.kill` alone does not end a thread group whose leader already exited (measured), so the stop also signals each listed
//! process through a pidfd.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group.ts (Linux only)
//! export class ControlGroup {
//!   static createIn(parent: string): ControlGroup  // throws GroupUnavailable
//!   spawn(command: Command): ChildProcess
//!   stop(boundMs: number): boolean
//! }
//! export function query(record: GroupRecord): 'gone' | 'alive' | { unknown: string }
//! ```

/// Imports `std::fmt` for this file.
use std::fmt;

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::io::Write` for this file.
use std::io::Write;

/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `std::os::unix::fs::{FileExt, MetadataExt}` for this file.
use std::os::unix::fs::{FileExt, MetadataExt};

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports `std::process::{Child, Command}` for this file.
use std::process::{Child, Command};

/// Imports `std::sync::atomic::{AtomicU64, Ordering}` for this file.
use std::sync::atomic::{AtomicU64, Ordering};

/// Imports `std::time::{Duration, Instant, SystemTime, UNIX_EPOCH}` for this file.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Imports `rustix::event::{PollFd, PollFlags, Timespec, poll}` for this file.
use rustix::event::{PollFd, PollFlags, Timespec, poll};

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// Imports `rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal}` for this file.
use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

/// Where the unified control group hierarchy is mounted.
pub(crate) const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// The file system type number of a cgroup v2 mount (`CGROUP2_SUPER_MAGIC`, ASCII "cgrp"), from `statfs(2)`.
const CGROUP2_MAGIC: u64 = 0x6367_7270;

/// How long the stop waits for a change of `cgroup.events` before it looks for members to signal again.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto"
/// (handover "Picks made under the owner's delegation"), veto open;
/// measurement beside it:
/// on 2026-11-04 `populated 0` followed `cgroup.kill` within 5 ms for ordinary members and 1 ms after a signal to a zombie leader,
/// so a wake-up every 50 ms only matters for a member that appears or survives the first signals.
const STRAGGLER_INTERVAL: Duration = Duration::from_millis(50);

/// Numbers the groups of this process so two runs never share a name.
static NEXT_GROUP: AtomicU64 = AtomicU64::new(0);

/// Why no control group could be had,
/// in words that name the missing capability.
#[derive(Debug)]
pub(crate) struct GroupUnavailable {
    /// The missing capability and what was tried.
    pub(crate) reason: String,
}

/// How the reason reads to a person.
impl fmt::Display for GroupUnavailable {
    /// What:
    /// `fn fmt(&self, formatter)` writes the reason.
    /// Why:
    /// The one-line notice of the weaker path is this text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.reason);
    }
}

/// A control group this process made and owns.
#[derive(Debug)]
pub(crate) struct ControlGroup {
    /// The directory of the group in the cgroup file system.
    directory: PathBuf,
    /// The group this process was in when it made this one;
    /// the spawning process leaves back into it.
    parent: PathBuf,
    /// The inode number of the directory, which is the id the kernel gives the group.
    id: u64,
    /// The inode number of this process's cgroup namespace, because a path means nothing in another one.
    namespace: u64,
}

/// What a journal remembers of a run's containment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Record {
    /// A control group: its id, the namespace it was made in, and its path.
    Group {
        /// The group's inode number.
        id: u64,
        /// The inode number of the cgroup namespace of the process that made it.
        namespace: u64,
        /// The directory.
        path: PathBuf,
    },
    /// The weaker path: only a process group (round 5), recorded by the process id of its leader.
    ProcessGroup(
        /// The leader's process id, which is the group's number.
        i32,
    ),
    /// The weaker path is about to start a child whose process group is not known yet;
    /// the line that follows it names the group.
    /// A killed wrapper that leaves this line alone may have a running child that no record names.
    ProcessGroupStarting,
}

/// What a recovery learns about a recorded group.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Membership {
    /// No member can exist: the group is absent, or a different group lives at its path now.
    Gone,
    /// At least one thread of the group lives.
    Alive,
    /// The question cannot be answered, with the reason.
    Unknown(
        /// Why.
        String,
    ),
}

/// What:
/// `fn is_cgroup2(path: &Path) -> io::Result<bool>` says whether the directory is on a cgroup v2 file system.
/// Why:
/// `mkdir` succeeds on any directory, so only the file system type shows that a made directory is a control group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCgroup2(path: string): boolean
/// ```
fn is_cgroup2(path: &Path) -> io::Result<bool> {
    let found = rustix::fs::statfs(path).map_err(io::Error::from)?;
    return Ok(u64::try_from(found.f_type).unwrap_or(0) == CGROUP2_MAGIC);
}

/// What:
/// `pub(crate) fn own_group_directory() -> io::Result<PathBuf>` is the directory of the control group this process is in.
/// Why:
/// A group made inside it is writable when the user owns this one,
/// which is true for everything the user manager starts and false for a login session.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ownGroupDirectory(): string
/// ```
pub(crate) fn own_group_directory() -> io::Result<PathBuf> {
    let text: String = fs::read_to_string("/proc/self/cgroup")?;
    for line in text.lines() {
        // The unified hierarchy is the line `0::<path>`.
        if let Some(path) = line.strip_prefix("0::") {
            return Ok(Path::new(CGROUP_ROOT).join(path.trim_start_matches('/')));
        }
    }
    return Err(io::Error::other("this process is not in a unified (cgroup v2) hierarchy"));
}

/// What:
/// `fn namespace_id() -> io::Result<u64>` is the inode number of this process's cgroup namespace.
/// Why:
/// A recorded path means the same thing only inside the same namespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namespaceId(): number
/// ```
fn namespace_id() -> io::Result<u64> {
    return Ok(fs::metadata("/proc/self/ns/cgroup")?.ino());
}

/// What:
/// `fn write_text(path: &Path, text: &str) -> io::Result<()>` writes `text` to a control file in one `write`.
/// Why:
/// The kernel reads a control file's command from a single write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeText(path: string, text: string): void
/// ```
fn write_text(path: &Path, text: &str) -> io::Result<()> {
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    return file.write_all(text.as_bytes());
}

/// The methods of the type above.
impl ControlGroup {
    /// What:
    /// `pub(crate) fn create_in(parent: &Path) -> Result<ControlGroup, GroupUnavailable>` makes an empty group inside `parent`.
    /// Why:
    /// It is the whole of "can this process get a group it may write": the parent must be on cgroup v2,
    /// the `mkdir` must be allowed,
    /// and the new directory must have the control files (`cgroup.kill` exists since Linux 5.14).
    /// A failure names the missing capability and nothing is left behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static createIn(parent: string): ControlGroup  // throws GroupUnavailable
    /// ```
    pub(crate) fn create_in(parent: &Path) -> Result<ControlGroup, GroupUnavailable> {
        let unavailable = |reason: String| -> GroupUnavailable { return GroupUnavailable { reason }; };
        match is_cgroup2(parent) {
            Ok(true) => {}
            Ok(false) => return Err(unavailable(format!("{} is not on a cgroup v2 file system", parent.display()))),
            Err(error) => return Err(unavailable(format!("{} cannot be examined ({error})", parent.display()))),
        }
        let Some(parent_text) = parent.to_str() else {
            return Err(unavailable(format!("{} is not UTF-8, which the journal record cannot hold", parent.display())));
        };
        if parent_text.contains('\n') {
            return Err(unavailable(format!("{parent_text:?} holds a line feed, which the journal record cannot hold")));
        }
        let nanos: u128 = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| { return elapsed.as_nanos(); }).unwrap_or(0);
        let name: String = format!("cargo-shadow-{}-{nanos}-{}", std::process::id(), NEXT_GROUP.fetch_add(1, Ordering::Relaxed));
        let directory: PathBuf = parent.join(name);
        if let Err(error) = fs::create_dir(&directory) {
            return Err(unavailable(format!("making a control group in {} failed ({error})", parent.display())));
        }
        let namespace: u64 = match namespace_id() {
            Ok(found) => found,
            Err(error) => {
                let _ = fs::remove_dir(&directory);
                return Err(unavailable(format!("the control group namespace cannot be identified ({error})")));
            }
        };
        let id: u64 = match fs::metadata(&directory) {
            Ok(found) => found.ino(),
            Err(error) => {
                let _ = fs::remove_dir(&directory);
                return Err(unavailable(format!("the new control group cannot be examined ({error})")));
            }
        };
        for control in ["cgroup.procs", "cgroup.kill", "cgroup.events"] {
            if let Err(error) = fs::metadata(directory.join(control)) {
                let _ = fs::remove_dir(&directory);
                return Err(unavailable(format!("the new control group has no {control} ({error}); Linux 5.14 or later is needed")));
            }
        }
        return Ok(ControlGroup { directory, parent: parent.to_path_buf(), id, namespace });
    }

    /// What:
    /// `pub(crate) fn directory(&self) -> &Path` is the group's directory.
    /// Why:
    /// Tests and the removal name it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// directory(): string
    /// ```
    pub(crate) fn directory(&self) -> &Path {
        return &self.directory;
    }

    /// What:
    /// `pub(crate) fn record(&self) -> Record` is what a journal keeps of this group.
    /// Why:
    /// The journal line is written and forced to disk before anything starts in the group.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record(): Record
    /// ```
    pub(crate) fn record(&self) -> Record {
        return Record::Group { id: self.id, namespace: self.namespace, path: self.directory.clone() };
    }

    /// What:
    /// `pub(crate) fn spawn(&self, command: &mut Command) -> io::Result<Child>` starts the command with this group as its group.
    /// Why:
    /// A child inherits the group of the process that forks it.
    /// The spawning process writes `0` (itself) to this group's `cgroup.procs`,
    /// spawns,
    /// and writes `0` to its own group's `cgroup.procs` again,
    /// so no hook runs between `fork` and `exec` and the caller's `Command` carries no per-run state.
    /// The caller must make sure no other thread starts a process meanwhile.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spawn(command: Command): ChildProcess
    /// ```
    pub(crate) fn spawn(&self, command: &mut Command) -> io::Result<Child> {
        let enter = fs::OpenOptions::new().write(true).open(self.directory.join("cgroup.procs"))?;
        let leave = fs::OpenOptions::new().write(true).open(self.parent.join("cgroup.procs"))?;
        (&enter).write_all(b"0")?;
        let spawned: io::Result<Child> = command.spawn();
        let left: io::Result<()> = (&leave).write_all(b"0");
        return match (spawned, left) {
            (Ok(child), Ok(())) => Ok(child),
            (Ok(mut child), Err(error)) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(io::Error::other(format!("the child was stopped because this process could not leave the new control group ({error})")))
            }
            (Err(error), Ok(())) => Err(error),
            (Err(error), Err(left_error)) => Err(io::Error::other(format!("{error}; this process could not leave the new control group either ({left_error})"))),
        };
    }

    /// What:
    /// `pub(crate) fn members(&self) -> io::Result<Vec<i32>>` lists the processes in the group (the leader of each thread group).
    /// Why:
    /// A thread group whose leader exited is still listed while another of its threads lives.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// members(): number[]
    /// ```
    pub(crate) fn members(&self) -> io::Result<Vec<i32>> {
        let text: String = match fs::read_to_string(self.directory.join("cgroup.procs")) {
            Ok(found) => found,
            // A group that was removed has no members; `ENODEV` is what a read of a removed group's file says.
            Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut found: Vec<i32> = Vec::new();
        for line in text.lines() {
            found.push(line.trim().parse::<i32>().map_err(|error| { return io::Error::new(io::ErrorKind::InvalidData, format!("cgroup.procs holds {line:?} ({error})")); })?);
        }
        return Ok(found);
    }

    /// What:
    /// `pub(crate) fn populated(&self) -> io::Result<bool>` says whether any thread of the group or of a group below it lives.
    /// Why:
    /// This is the kernel's own statement of quiescence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// populated(): boolean
    /// ```
    #[cfg(test)]
    pub(crate) fn populated(&self) -> io::Result<bool> {
        return populated_at(&self.directory);
    }

    /// What:
    /// `fn signal_members(&self) -> io::Result<()>` sends `SIGKILL` to every listed process through a pidfd that was checked
    /// against the listing a second time.
    /// Why:
    /// `cgroup.kill` leaves a thread group alive when its leader already exited (measured on Linux 7.2.15),
    /// and a signal to the process ends it.
    /// A process id read from a list can be reused,
    /// so the pidfd is opened first and the process must still be listed after that.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// signalMembers(): void
    /// ```
    fn signal_members(&self) -> io::Result<()> {
        let mut held: Vec<(i32, OwnedFd)> = Vec::new();
        for pid in self.members()? {
            let Some(found) = Pid::from_raw(pid) else {
                continue;
            };
            // A process that ended between the listing and now is simply not signalled.
            if let Ok(descriptor) = pidfd_open(found, PidfdFlags::empty()) {
                held.push((pid, descriptor));
            }
        }
        let still_listed: Vec<i32> = self.members()?;
        for (pid, descriptor) in &held {
            if still_listed.contains(pid) {
                let _ = pidfd_send_signal(descriptor, Signal::KILL);
            }
        }
        return Ok(());
    }

    /// What:
    /// `pub(crate) fn stop(&self, bound: Duration) -> io::Result<bool>` kills every member and returns `true` once the kernel
    /// reports the group unpopulated,
    /// or `false` when `bound` passes first.
    /// Why:
    /// `cgroup.kill` is recursive and asynchronous,
    /// so the answer is the wait for `populated 0`, which can only be earned.
    /// A member in an uninterruptible wait cannot be killed and that is `false`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stop(boundMs: number): boolean
    /// ```
    pub(crate) fn stop(&self, bound: Duration) -> io::Result<bool> {
        let deadline: Instant = Instant::now() + bound;
        match write_text(&self.directory.join("cgroup.kill"), "1") {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
            // The signal pass below does the stopping when the kill file cannot be written.
            Err(error) => eprintln!("cargo-shadow: warning: cgroup.kill of {} could not be written ({error}); signalling the members one by one", self.directory.display()),
        }
        let events = match fs::File::open(self.directory.join("cgroup.events")) {
            Ok(found) => found,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
            Err(error) => return Err(error),
        };
        loop {
            if !populated_through(&events)? {
                return Ok(true);
            }
            self.signal_members()?;
            let now: Instant = Instant::now();
            if now >= deadline {
                return Ok(false);
            }
            let wait: Duration = STRAGGLER_INTERVAL.min(deadline - now);
            let timeout: Timespec = Timespec { tv_sec: i64::try_from(wait.as_secs()).unwrap_or(0), tv_nsec: i64::from(wait.subsec_nanos()) };
            let mut watched = [PollFd::new(&events, PollFlags::PRI)];
            match poll(&mut watched, Some(&timeout)) {
                Ok(_) | Err(Errno::INTR) => {}
                Err(error) => return Err(io::Error::from(error)),
            }
        }
    }

    /// What:
    /// `pub(crate) fn remove(&self) -> io::Result<()>` removes the empty group.
    /// Why:
    /// The kernel refuses (`EBUSY`) while a member lives,
    /// so a removal never hides a survivor.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// remove(): void
    /// ```
    pub(crate) fn remove(&self) -> io::Result<()> {
        return match fs::remove_dir(&self.directory) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        };
    }
}

/// What:
/// `fn parse_populated(text: &str) -> io::Result<bool>` reads the `populated` line of `cgroup.events`.
/// Why:
/// Anything but `populated 0` or `populated 1` is an error, never "empty".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePopulated(text: string): boolean
/// ```
fn parse_populated(text: &str) -> io::Result<bool> {
    for line in text.lines() {
        match line.trim() {
            "populated 1" => return Ok(true),
            "populated 0" => return Ok(false),
            _ => {}
        }
    }
    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("cgroup.events has no populated line: {text:?}")));
}

/// What:
/// `fn populated_through(events: &fs::File) -> io::Result<bool>` re-reads an open `cgroup.events` from its start.
/// Why:
/// The read also arms the file so that a later `poll` wakes on the next change.
/// A removed group reads as unpopulated (`ENODEV`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function populatedThrough(events: FileHandle): boolean
/// ```
fn populated_through(events: &fs::File) -> io::Result<bool> {
    let mut buffer: [u8; 128] = [0; 128];
    let count: usize = match events.read_at(&mut buffer, 0) {
        Ok(found) => found,
        Err(error) if error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) => return Ok(false),
        Err(error) => return Err(error),
    };
    return parse_populated(&String::from_utf8_lossy(&buffer[..count]));
}

/// What:
/// `fn populated_at(directory: &Path) -> io::Result<bool>` reads `cgroup.events` of a group directory once.
/// Why:
/// A recovery has no open file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function populatedAt(directory: string): boolean
/// ```
fn populated_at(directory: &Path) -> io::Result<bool> {
    return parse_populated(&fs::read_to_string(directory.join("cgroup.events"))?);
}

/// How a record reads as a journal line.
impl fmt::Display for Record {
    /// What:
    /// `fn fmt(&self, formatter)` writes the one-line form of the record, without the line feed.
    /// Why:
    /// The journal's `groups` file holds one line per record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Record::Group { id, namespace, path } => write!(formatter, "cgroup {id} {namespace} {}", path.display()),
            Record::ProcessGroup(leader) => write!(formatter, "pgroup {leader}"),
            Record::ProcessGroupStarting => write!(formatter, "pgroup starting"),
        };
    }
}

/// What:
/// `pub(crate) fn parse_record(line: &str) -> Result<Record, String>` reads one line of a `groups` file.
/// Why:
/// A line that is not understood is uncertainty about a possible writer, and the caller says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseRecord(line: string): Record  // throws the reason
/// ```
pub(crate) fn parse_record(line: &str) -> Result<Record, String> {
    if line == "pgroup starting" {
        return Ok(Record::ProcessGroupStarting);
    }
    if let Some(rest) = line.strip_prefix("pgroup ") {
        let leader: i32 = rest.trim().parse::<i32>().map_err(|error| { return format!("{line:?} is not a process group ({error})"); })?;
        if leader <= 0 {
            return Err(format!("{line:?} is not a process group"));
        }
        return Ok(Record::ProcessGroup(leader));
    }
    if let Some(rest) = line.strip_prefix("cgroup ") {
        let mut parts = rest.splitn(3, ' ');
        let id: u64 = parts.next().unwrap_or("").parse::<u64>().map_err(|error| { return format!("{line:?} has no group id ({error})"); })?;
        let namespace: u64 = parts.next().unwrap_or("").parse::<u64>().map_err(|error| { return format!("{line:?} has no namespace id ({error})"); })?;
        let path: &str = parts.next().unwrap_or("");
        if path.is_empty() {
            return Err(format!("{line:?} has no path"));
        }
        return Ok(Record::Group { id, namespace, path: PathBuf::from(path) });
    }
    return Err(format!("{line:?} is not a record of a control group or a process group"));
}

/// What:
/// `pub(crate) fn query(id: u64, namespace: u64, path: &Path) -> Membership` says whether the recorded group has a member.
/// Why:
/// A group cannot be removed while it has a member,
/// so a path that is gone, or that is a different group now (another inode number),
/// had no member when it went.
/// Only the same cgroup namespace gives the path its meaning,
/// and only a mounted cgroup v2 can answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function query(id: number, namespace: number, path: string): 'gone' | 'alive' | { unknown: string }
/// ```
pub(crate) fn query(id: u64, namespace: u64, path: &Path) -> Membership {
    match namespace_id() {
        Ok(found) if found == namespace => {}
        Ok(_) => return Membership::Unknown(format!("{} was made in another cgroup namespace than this process is in", path.display())),
        Err(error) => return Membership::Unknown(format!("the cgroup namespace of this process cannot be identified ({error})")),
    }
    match is_cgroup2(Path::new(CGROUP_ROOT)) {
        Ok(true) => {}
        Ok(false) => return Membership::Unknown(format!("{CGROUP_ROOT} is not a cgroup v2 mount, so {} cannot be examined", path.display())),
        Err(error) => return Membership::Unknown(format!("{CGROUP_ROOT} cannot be examined ({error})")),
    }
    let found = match fs::metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Membership::Gone,
        Err(error) => return Membership::Unknown(format!("{} cannot be examined ({error})", path.display())),
    };
    if found.ino() != id {
        return Membership::Gone;
    }
    return match populated_at(path) {
        Ok(true) => Membership::Alive,
        Ok(false) => Membership::Gone,
        Err(error) => Membership::Unknown(format!("{} cannot be read ({error})", path.join("cgroup.events").display())),
    };
}

/// What:
/// `pub(crate) fn reclaim(id: u64, path: &Path)` removes a recorded group that is empty and still the recorded one.
/// Why:
/// A killed run leaves its empty group behind,
/// and the recovery that settles the journal tidies it;
/// an occupied group refuses the removal and is left for the next look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function reclaim(id: number, path: string): void
/// ```
pub(crate) fn reclaim(id: u64, path: &Path) {
    if let Ok(found) = fs::metadata(path)
        && found.ino() == id
    {
        let _ = fs::remove_dir(path);
    }
}
