//! What:
//! What the stop of a scope found,
//! in the words a refusal carries:
//! how long the stop waited,
//! whether the kernel still reported the scope populated,
//! and every member still listed,
//! in the scope's own group and in every group below it,
//! each with the group it stands in and the state of its threads.
//! Why:
//! Issue 6528:
//! the install step refused a command because a process cargo started "could not be stopped",
//! and nothing said whether that process had exited (one that waits to be reaped holds no descriptor and can write
//! nothing) or was alive (a writer the lockfiles cannot be trusted under).
//! A person reading the refusal must be able to tell those apart,
//! and a loaded host from a member that cannot be killed,
//! so the refusal says how long the stop waited and what it was waiting for.
//! The kernel's `populated` word counts the groups below the scope too,
//! so a scope whose own `cgroup.procs` is empty can still hold a live process
//! (`doc/troubleshooting/a-cgroup-does-not-count-a-member-that-has-exited.md`),
//! and the listing reads every group of the scope for that reason.
//! A member is read through a pidfd,
//! so a number the kernel listed cannot describe another process by the time its threads are read.
//! Split out of `control_group_stop.rs`,
//! which keeps the stop itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-stop-report.ts
//! export type MemberStanding = 'gone' | 'exited' | 'thread-lives' | 'alive' | { unknown: string }
//! export function standingOf(pid: number): MemberStanding
//! export function standingOfWithin(pid: number, deadline: number): MemberStanding
//! export class StopReport { waited: number; empty: boolean; sentence(scope: string): string; headline(): string }
//! ```

/// What:
/// `use super::LISTING_BOUND;` imports how long reading the members may take.
/// Why:
/// A member's threads, and the listing itself, stop at that bound,
/// and the words a refusal carries say which bound it hit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LISTING_BOUND } from './control_group_stop.ts';
/// ```
use super::LISTING_BOUND;
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
/// `use std::fs;` imports the file system calls.
/// Why:
/// `standing_of` reads a process's task folder and each thread's `stat` file.
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
/// `use std::path::Path;` imports the borrowed view of a path (its sibling `PathBuf` owns one).
/// Why:
/// The sentence names the scope's directory without keeping a copy of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is a plain string in TS
/// ```
use std::path::Path;
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time (its sibling `Instant` is a point on the clock).
/// Why:
/// The time the stop waited is what the refusal must state,
/// and a thread read stops at a point on the clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds, and Date.now() for a point on the clock
/// ```
use std::time::{
    Duration,
    Instant,
};

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
/// `use rustix::process::{Pid, PidfdFlags, pidfd_open};` imports the pid type, the flag set of a pidfd and the call that opens it.
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
/// with the group below the scope it stands in, when it stands in one,
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
    /// `fn text(&self) -> String` is the member's process id, the group it stands in when that is below the scope,
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

/// What the reading of the members found.
pub(crate) enum Listing {
    /// Every group of the scope, and every group below it, was read:
    /// these are the members that were left.
    Read(Vec<MemberListing>),
    /// The listing's own bound passed before every member was read:
    /// these are the members it read, and the rest were not named.
    Cut(Vec<MemberListing>),
    /// The scope's own directory was gone when its members were read,
    /// so nothing was there to name.
    ScopeGone,
    /// The members could not be read,
    /// with the reason.
    Unreadable(String),
}

/// What a refusal may say about what the listing left,
/// which is the one thing the opening clause adds to the sentence.
pub(crate) enum Leftover {
    /// A member that was read can still write.
    Writer,
    /// Every group was read and no member it listed can still write.
    NoWriter,
    /// Every group was read and none of them listed a member.
    Nothing,
    /// The scope's directory was gone when its members were read.
    ScopeGone,
    /// The members could not be read.
    Unreadable,
    /// The listing's own bound passed before every member was read.
    Cut,
}

/// What the stop of one scope found.
pub(crate) struct StopReport {
    /// How long the stop waited for the scope to become empty.
    waited: Duration,
    /// Whether the stop showed the scope empty:
    /// the kernel reported it unpopulated,
    /// or its absence was verified.
    empty: bool,
    /// The kernel's last `populated` word,
    /// or `None` when the group could not be read (it was removed).
    populated: Option<bool>,
    /// Whether the scope's directory was gone when the stop looked and its absence was verified.
    /// Why a flag of its own:
    /// an empty report earned that way has no `populated` word behind it,
    /// and the words must not say the kernel spoke.
    absent: bool,
    /// The members the listing found, or why it found none it could read.
    listing: Listing,
}

/// What:
/// `impl StopReport` builds the report and turns it into the words a refusal carries.
/// Why:
/// The stop's answer and the text a person reads must be built from the same facts,
/// or a message could disagree with the run that produced it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a plain class with a constructor and a few methods
/// ```
impl StopReport {
    /// What:
    /// `pub(crate) fn new(...) -> StopReport` takes what the stop saw of a scope that was not verified absent.
    /// Why:
    /// `control_group_stop.rs` owns the loop and hands it these facts once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(waited: number, empty: boolean, populated: boolean | undefined, listing: Listing)
    /// ```
    pub(crate) fn new(
        waited: Duration,
        empty: bool,
        populated: Option<bool>,
        listing: Listing,
    ) -> StopReport {
        return StopReport {
            waited,
            empty,
            populated,
            absent: false,
            listing,
        };
    }

    /// What:
    /// `pub(crate) fn absent(waited: Duration) -> StopReport` is the report of a scope whose absence was verified.
    /// Why:
    /// The same report type carries both answers,
    /// so no caller has to read two shapes for one question,
    /// and the words say the directory was gone instead of pretending the kernel reported it empty.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static absent(waited: number): StopReport
    /// ```
    pub(crate) fn absent(waited: Duration) -> StopReport {
        return StopReport {
            waited,
            empty: true,
            populated: None,
            absent: true,
            listing: Listing::ScopeGone,
        };
    }

    /// What:
    /// `pub(crate) fn empty(&self) -> bool` says whether the stop showed the scope empty.
    /// Why:
    /// A caller must ask this and nothing else before it treats the run as settled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get isEmpty(): boolean { return this.empty }
    /// ```
    pub(crate) fn empty(&self) -> bool {
        return self.empty;
    }

    /// What:
    /// `pub(crate) fn leftover(&self) -> Leftover` is what the report may say about what was left.
    /// Why:
    /// Every refusal's opening clause is chosen from this one answer,
    /// so a refusal cannot claim a member it never read is harmless
    /// and cannot stay silent about a member it did read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// leftover(): 'writer' | 'no-writer' | 'nothing' | 'scope-gone' | 'unreadable' | 'cut'
    /// ```
    pub(crate) fn leftover(&self) -> Leftover {
        return match &self.listing {
            Listing::Read(members) => {
                if members
                    .iter()
                    .any(|member| return member.standing.can_still_write())
                {
                    Leftover::Writer
                } else if members.is_empty() {
                    Leftover::Nothing
                } else {
                    Leftover::NoWriter
                }
            }
            Listing::Cut(members) => {
                if members
                    .iter()
                    .any(|member| return member.standing.can_still_write())
                {
                    Leftover::Writer
                } else {
                    Leftover::Cut
                }
            }
            Listing::ScopeGone => Leftover::ScopeGone,
            Listing::Unreadable(_) => Leftover::Unreadable,
        };
    }

    /// What:
    /// `pub(crate) fn headline(&self) -> String` is the opening clause of a refusal of a scope that did not read as empty.
    /// Why:
    /// The clause and the sentence after it are one message,
    /// and the clause must name only what the listing found (issue 6528's review, Medium 1).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// headline(): string { ... }
    /// ```
    pub(crate) fn headline(&self) -> String {
        return match self.leftover() {
            Leftover::Writer => "a member of the scope could not be shown stopped".to_string(),
            Leftover::NoWriter => {
                "the scope did not read as empty, and no member it listed can still write".to_string()
            }
            Leftover::Nothing => {
                "the scope did not read as empty, and it listed no member of the scope or of any group below it"
                    .to_string()
            }
            Leftover::ScopeGone => {
                "the scope did not read as empty, and its directory was gone when its members were read"
                    .to_string()
            }
            Leftover::Unreadable => {
                "the scope did not read as empty, and its members could not be read".to_string()
            }
            Leftover::Cut => format!(
                "the scope did not read as empty, and its members could not all be read within {LISTING_BOUND:?}"
            ),
        };
    }

    /// What:
    /// `pub(crate) fn sentence(&self, scope: &Path) -> String` is the report as one clause of a refusal.
    /// Why:
    /// The refusal of a run that cannot be settled must say how long it waited and what it was waiting for,
    /// in words that tell an exited member from a live one,
    /// and must not state a wait that was not the wait.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sentence(scope: string): string { ... }
    /// ```
    pub(crate) fn sentence(
        &self,
        scope: &Path,
    ) -> String {
        if self.empty {
            if self.absent {
                return format!(
                    "the scope {} was gone when the stop looked, and its absence was verified",
                    scope.display()
                );
            }
            return format!(
                "the stop of {} waited {:?} and the kernel reported the scope empty",
                scope.display(),
                self.waited
            );
        }
        let population: &str = match self.populated {
            Some(true) => "the kernel last reported it populated",
            Some(false) => "the kernel last reported it empty",
            None => "its group could not be read",
        };
        return format!(
            "the stop of {} waited {:?} for it to be empty; {population}, and {}",
            scope.display(),
            self.waited,
            self.listing_text()
        );
    }

    /// What:
    /// `fn listing_text(&self) -> String` is what the listing found, in the words that follow the kernel's word.
    /// Why:
    /// Three different findings would otherwise read the same:
    /// no member was left to name, a listing could not be read, and a listing was cut at its bound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// listingText(): string { ... }
    /// ```
    fn listing_text(&self) -> String {
        return match &self.listing {
            Listing::Read(members) if members.is_empty() => {
                "it listed no member of the scope or of any group below it".to_string()
            }
            Listing::Read(members) => named_members(members),
            Listing::Cut(members) if members.is_empty() => format!(
                "the listing of its members was cut at its own bound of {LISTING_BOUND:?} before one was read"
            ),
            Listing::Cut(members) => format!(
                "the listing of its members was cut at its own bound of {LISTING_BOUND:?} after {}",
                named_members(members)
            ),
            Listing::ScopeGone => "its directory was gone when its members were read".to_string(),
            Listing::Unreadable(reason) => format!("its members could not be read ({reason})"),
        };
    }
}

/// What:
/// `fn named_members(members: &[MemberListing]) -> String` is one clause naming every member the listing read.
/// Why:
/// The sentence and the cut case both need the list with the count that agrees with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedMembers(members: MemberListing[]): string
/// ```
fn named_members(members: &[MemberListing]) -> String {
    let listed: Vec<String> = members
        .iter()
        .map(|member| {
            return member.text();
        })
        .collect();
    // What:     `if listed.len() == 1 { "member" } else { "members" }` picks the word that agrees with the count.
    // Why:      A refusal is read by a person, and "1 member(s)" reads like a placeholder nobody finished.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const noun = listed.length === 1 ? 'member' : 'members';
    // ```
    let noun: &str = if listed.len() == 1 {
        "member"
    } else {
        "members"
    };
    return format!("it listed {} {noun}: {}", listed.len(), listed.join(", "));
}

/// What:
/// `pub(crate) fn standing_of_within(pid: i32, deadline: Instant) -> MemberStanding` reads a process's threads through
/// a pidfd and says what state they are in, or that the bound passed before they were all read.
/// Why:
/// A process id read from a list can be reused,
/// so the descriptor is opened first:
/// while it is held no other process can take that number,
/// and a `/proc/<pid>` entry that is gone then belongs to the process that ended and not to another one.
/// A process this one cannot see (another pid namespace, or a mount that hides `/proc/<pid>`)
/// is not gone, and the descriptor says so: it is not readable when the process has not ended.
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
    let folder: String = format!("/proc/{pid}/task");
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
/// `fn has_ended(descriptor: &OwnedFd) -> bool` asks a pidfd whether its process has ended, without waiting.
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
