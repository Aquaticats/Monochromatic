//! What:
//! The members a stop's report names:
//! how one process stood when the stop last looked at it,
//! the group below the scope it stands in,
//! and the read of its threads through a pidfd.
//! Why:
//! The kernel's `populated` word says that something is left,
//! never what,
//! and the state of one process's threads is the whole difference between a member that can still write and one that
//! only waits to be reaped (issue 6528).
//! A process id is read from a list and its descriptor opened after that,
//! and the threads are read only while the descriptor is held:
//! from the open onwards the number cannot name another process,
//! though it was read a moment before.
//! Split out of `control_group_stop_report.rs`,
//! which keeps what the stop found and the words a refusal carries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-stop-report-members.ts
//! export type MemberStanding = 'gone' | 'exited' | 'thread-lives' | 'alive' | { unknown: string }
//! export function standingOfWithin(pid: number, deadline: number): MemberStanding
//! ```

/// What:
/// `use crate::control_group_process_identity::parse_stat;` imports the reader of a `stat` file's state letter
/// and start time.
/// Why:
/// A thread's state is read the same way a launcher's is,
/// and the parser already handles a name field that holds spaces and brackets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseStat } from 'crate/control_group_process_identity';
/// ```
use crate::control_group_process_identity::parse_stat;
/// What:
/// `use crate::control_group_stop::LISTING_BOUND;` imports how long reading one member's threads may take.
/// Why:
/// A member whose threads are not all read within it is unreadable rather than gone,
/// and the words a refusal carries say which bound it hit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LISTING_BOUND } from './control_group_stop.ts';
/// ```
use crate::control_group_stop::LISTING_BOUND;
/// What:
/// `use std::fs;` imports the file system calls.
/// Why:
/// `standing_of_within` reads a process's task folder and each thread's `stat` file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readdirSync, readFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::os::fd::OwnedFd;` imports the owning wrapper of an open file descriptor.
/// Why:
/// The pidfd a member is read through must be closed when the read is over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has no equivalent: a number is a descriptor and nothing closes it for you.
/// ```
use std::os::fd::OwnedFd;
/// What:
/// `use std::time::Instant;` imports a point on the clock (its sibling `Duration` is a span of time).
/// Why:
/// A member's threads are read until the listing's deadline,
/// and a member that is not read by then is not read as gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now()
/// ```
use std::time::Instant;

/// What:
/// `use rustix::event::{PollFd, PollFlags, Timespec, poll};` imports the wait on a descriptor for one event.
/// Why:
/// A pidfd becomes readable when its process ends,
/// which is how a member whose `/proc` entry is gone is told from one this process cannot see at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { poll, PollFd, PollFlags, Timespec } from 'rustix/event';
/// ```
use rustix::event::{
    PollFd,
    PollFlags,
    Timespec,
    poll,
};
/// What:
/// `use rustix::io::Errno;` imports the error code that says no process with this id exists.
/// Why:
/// A pidfd that cannot be opened because the process is gone is the one case that may be read as "gone".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Errno } from 'rustix/io';
/// ```
use rustix::io::Errno;
/// What:
/// `use rustix::process::{Pid, PidfdFlags, pidfd_open};` imports the pid type,
/// the flag set of a pidfd and the call that opens it.
/// Why:
/// A process id read from a list can be reused,
/// so it is turned into a descriptor first and read only while that descriptor is held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { pidfdOpen, Pid, PidfdFlags } from 'rustix/process';
/// ```
use rustix::process::{
    Pid,
    PidfdFlags,
    pidfd_open,
};

/// How one process stood when the stop last looked at it.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MemberStanding {
    /// No process with that id exists any more:
    /// it ended and was reaped,
    /// so it can write nothing.
    Gone,
    /// Every one of its threads has ended and no process has reaped it yet:
    /// it holds no descriptor and can write nothing.
    Exited,
    /// Its leading thread has ended and another thread of the same process still runs,
    /// so it can still write.
    ThreadLives,
    /// At least one of its threads runs,
    /// so it can still write.
    Alive,
    /// Its threads could not be read,
    /// with the reason;
    /// an unreadable member may still write,
    /// so this is never read as gone.
    Unknown(
        /// Why its threads could not be read.
        String,
    ),
}

/// What:
/// `impl MemberStanding` holds the two questions the report asks of one member.
/// Why:
/// The sentence must never call a process that can write "gone",
/// and the refusal's summary must answer whether anything that can write was left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // nothing extra: the union already says which states can write
/// ```
impl MemberStanding {
    /// What:
    /// `fn can_still_write(&self) -> bool` says whether this member may write a lockfile or a manifest.
    /// Why:
    /// A member that ended can write nothing,
    /// whatever a busy host did to the stop's bound,
    /// and only the other states need the journal to be settled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// canStillWrite(): boolean { return this !== 'gone' && this !== 'exited' }
    /// ```
    pub(crate) fn can_still_write(&self) -> bool {
        return match self {
            MemberStanding::Gone | MemberStanding::Exited => false,
            MemberStanding::ThreadLives | MemberStanding::Alive | MemberStanding::Unknown(_) => {
                true
            }
        };
    }

    /// What:
    /// `fn text(&self) -> String` is how this state reads inside the report's sentence.
    /// Why:
    /// A reader of a refusal has the process id and nothing else,
    /// so the state has to be spelled out in words that say whether it can write.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// text(): string { ... }
    /// ```
    pub(crate) fn text(&self) -> String {
        return match self {
            MemberStanding::Gone => {
                "it has ended and been reaped, so it can write nothing".to_string()
            }
            MemberStanding::Exited => {
                "it has exited and only waits to be reaped, so it can write nothing".to_string()
            }
            MemberStanding::ThreadLives => {
                "its leading thread has exited and another of its threads is alive, so it can still write"
                    .to_string()
            }
            MemberStanding::Alive => "it is alive, so it can still write".to_string(),
            MemberStanding::Unknown(reason) => {
                format!("its threads could not be read ({reason}), so it may still write")
            }
        };
    }
}

/// One member of a scope,
/// by process id,
/// with the group below the scope it stands in,
/// when it stands in one,
/// and the state its threads were in.
pub(crate) struct MemberListing {
    /// The process id the scope listed.
    pub(crate) pid: i32,
    /// The group below the scope's own directory the member stands in,
    /// as the path below that directory,
    /// or `None` when it stands in the scope's own group.
    pub(crate) below: Option<String>,
    /// What its threads were doing.
    pub(crate) standing: MemberStanding,
}

/// What:
/// `impl MemberListing` turns one member into the words that name it.
/// Why:
/// "1234 in nested" tells a reader where a process stands,
/// which is what a scope whose own group is empty needs to say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // nothing extra: a function takes the member
/// ```
impl MemberListing {
    /// What:
    /// `fn text(&self) -> String` is the member's process id,
    /// the group it stands in when that is below the scope,
    /// and the state of its threads.
    /// Why:
    /// A member of a group below the scope would otherwise read as a process of the scope itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// text(): string {
    ///   const where = this.below === undefined ? '' : ` in ${this.below}`
    ///   return `${this.pid}${where} (${this.standing.text()})`
    /// }
    /// ```
    pub(crate) fn text(&self) -> String {
        return match &self.below {
            Some(group) => format!("{} in {group} ({})", self.pid, self.standing.text()),
            None => format!("{} ({})", self.pid, self.standing.text()),
        };
    }
}

/// What:
/// `pub(crate) fn standing_of_within(pid: i32, deadline: Instant) -> MemberStanding` reads a process's threads through
/// a pidfd and says what state they are in,
/// or that the bound passed before they were all read.
/// Why:
/// A process id read from a list can be reused,
/// so the descriptor is opened before the threads are read:
/// from that open onwards no other process can take the number,
/// and a `/proc/<pid>` entry that is gone then belongs to the process that ended and not to another one.
/// The window between the read of the list and the open is open still:
/// a number freed and reused inside it would be named here with the threads of whichever process holds it now.
/// `signal_members` lists again after it opens its descriptors,
/// because a signal must name the process it acts on;
/// a listing that said the same would have to hold one descriptor per member across a second read of the group,
/// so this function says only what the descriptor it holds proves.
/// A process this one cannot see (another pid namespace,
/// or a mount that hides `/proc/<pid>`)
/// is not gone,
/// and the descriptor says so:
/// it is not readable when the process has not ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function standingOfWithin(pid: number, deadline: number): MemberStanding
/// ```
pub(crate) fn standing_of_within(
    pid: i32,
    deadline: Instant,
) -> MemberStanding {
    let Some(named) = Pid::from_raw(pid) else {
        // A listed number that names no process this one could ever signal: `cgroup.procs` writes 0 for a process of another
        // pid namespace. It may still write, so it is never read as gone.
        return MemberStanding::Unknown(format!(
            "the scope listed {pid}, which names no process this one can see (a process of another pid namespace reads that way)"
        ));
    };
    let descriptor: OwnedFd = match pidfd_open(named, PidfdFlags::empty()) {
        Ok(found) => found,
        Err(Errno::SRCH) => return MemberStanding::Gone,
        Err(error) => {
            return MemberStanding::Unknown(format!(
                "a descriptor for it could not be opened, so its pid cannot be held while its threads are read ({error})"
            ));
        }
    };
    // What:     `format!("/proc/{pid}/task", ...)` is the folder that lists this process's threads.
    // Why:      Each entry in it is one thread whose `stat` file says whether that thread still runs.
    let folder: String = format!("/proc/{pid}/task");
    // A test makes the member it listed read as one that has ended by now: the kernel takes a process out of `cgroup.procs`
    // and out of `populated` at its last thread's end, so nothing outside a test build can hold a process in the window
    // between the read of the list and this read of its threads.
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::ContainmentMemberEndedAtRead) {
        return MemberStanding::Exited;
    }
    let entries = match fs::read_dir(&folder) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if has_ended(&descriptor) {
                return MemberStanding::Gone;
            }
            return MemberStanding::Unknown(format!(
                "{folder} is not readable although the process has not ended ({error}): a mount option, or another pid namespace, hides it from this process"
            ));
        }
        Err(error) => {
            return MemberStanding::Unknown(format!("{folder} cannot be read ({error})"));
        }
    };
    let mut leading_state: Option<char> = None;
    let mut a_thread_lives: bool = false;
    let mut threads: usize = 0;
    for listed in entries {
        if Instant::now() >= deadline {
            return MemberStanding::Unknown(format!(
                "its threads could not all be read within {LISTING_BOUND:?}"
            ));
        }
        let entry: fs::DirEntry = match listed {
            Ok(found) => found,
            Err(error) => {
                return MemberStanding::Unknown(format!("{folder} cannot be listed ({error})"));
            }
        };
        let thread: i32 = match entry.file_name().to_string_lossy().parse::<i32>() {
            Ok(found) => found,
            Err(error) => {
                return MemberStanding::Unknown(format!(
                    "{folder} holds a name that is not a thread id ({error})"
                ));
            }
        };
        let stat: String = match fs::read_to_string(entry.path().join("stat")) {
            Ok(found) => found,
            // A thread that ended between the listing and the read is not a thread that is left.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return MemberStanding::Unknown(format!(
                    "{} cannot be read ({error})",
                    entry.path().join("stat").display()
                ));
            }
        };
        let state: char = match parse_stat(thread, &stat) {
            Ok((_, found)) => found,
            Err(error) => return MemberStanding::Unknown(error.to_string()),
        };
        threads += 1;
        if thread == pid {
            leading_state = Some(state);
        }
        if state != 'Z' && state != 'X' {
            a_thread_lives = true;
        }
    }
    if threads == 0 {
        return MemberStanding::Unknown(format!("{folder} holds no thread"));
    }
    if a_thread_lives {
        return match leading_state {
            Some('Z') | Some('X') => MemberStanding::ThreadLives,
            _ => MemberStanding::Alive,
        };
    }
    return MemberStanding::Exited;
}

/// What:
/// `fn has_ended(descriptor: &OwnedFd) -> bool` asks a pidfd whether its process has ended,
/// without waiting.
/// Why:
/// A pidfd becomes readable at the process's end,
/// so a `/proc` entry that is gone is either a process that ended (readable) or one this process cannot see (not readable).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasEnded(descriptor: Fd): boolean
/// ```
fn has_ended(descriptor: &OwnedFd) -> bool {
    let mut watched: [PollFd; 1] = [PollFd::new(descriptor, PollFlags::IN)];
    // No wait at all: the question is whether the descriptor is readable now.
    let none: Timespec = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    return match poll(&mut watched, Some(&none)) {
        Ok(count) => count > 0,
        Err(_) => false,
    };
}
