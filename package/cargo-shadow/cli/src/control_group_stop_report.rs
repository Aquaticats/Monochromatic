//! What:
//! What the stop of a scope found,
//! in the words a refusal carries:
//! how long the stop waited,
//! whether the kernel still reported the scope populated,
//! and which members were still listed,
//! each with the state of its threads.
//! Why:
//! Issue 6528:
//! the install step refused a command because a process cargo started "could not be stopped",
//! and nothing said whether that process had exited (one that waits to be reaped holds no descriptor and can write
//! nothing) or was alive (a writer the lockfiles cannot be trusted under).
//! A person reading the refusal must be able to tell those apart,
//! and a loaded host from a member that cannot be killed,
//! so the refusal says how long the stop waited and what it was waiting for.
//! Split out of `control_group_stop.rs`,
//! which keeps the stop itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-stop-report.ts
//! export type MemberStanding = 'gone' | 'exited' | 'thread-lives' | 'alive' | { unknown: string }
//! export function standingOf(pid: number): MemberStanding
//! export class StopReport { waited: number; empty: boolean; members: Member[]; sentence(scope: string): string }
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
/// `use std::time::Duration;` imports a span of time (its sibling `Instant` is a point on the clock).
/// Why:
/// The time the stop waited is what the refusal must state,
/// because a bound that passed on a loaded host looks the same as one that passed on a member that cannot be killed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds
/// ```
use std::time::Duration;

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

/// One member of a scope, by process id,
/// with the state its threads were in.
pub(crate) struct MemberListing {
    /// The process id the scope listed.
    pub(crate) pid: i32,
    /// What its threads were doing.
    pub(crate) standing: MemberStanding,
}

/// What the stop of one scope found.
pub(crate) struct StopReport {
    /// How long the stop waited for the scope to become empty.
    waited: Duration,
    /// Whether the kernel reported the scope empty (or its absence was verified).
    empty: bool,
    /// The kernel's last `populated` word,
    /// or `None` when the group could not be read (it was removed).
    populated: Option<bool>,
    /// The members the scope listed when the stop gave up.
    members: Vec<MemberListing>,
    /// Why the members could not be listed,
    /// when that read failed.
    listing_failure: Option<String>,
}

/// What:
/// `impl StopReport` builds the report and turns it into the words a refusal carries.
/// Why:
/// The stop's answer and the text a person reads must be built from the same facts,
/// or a message could disagree with the run that produced it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a plain class with a constructor and two methods
/// ```
impl StopReport {
    /// What:
    /// `pub(crate) fn new(...) -> StopReport` takes what the stop saw.
    /// Why:
    /// `control_group_stop.rs` owns the loop and hands it these facts once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(waited: number, empty: boolean, populated: boolean | undefined, members: Member[], listingFailure?: string)
    /// ```
    pub(crate) fn new(
        waited: Duration,
        empty: bool,
        populated: Option<bool>,
        members: Vec<MemberListing>,
        listing_failure: Option<String>,
    ) -> StopReport {
        return StopReport {
            waited,
            empty,
            populated,
            members,
            listing_failure,
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
    /// `pub(crate) fn may_still_write(&self) -> bool` says whether a listed member can still write.
    /// Why:
    /// This is the question the lockfiles hang on,
    /// so it is answered from the states and not from the count.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// mayStillWrite(): boolean { return this.members.some((member) => member.standing.canStillWrite()) }
    /// ```
    pub(crate) fn may_still_write(&self) -> bool {
        return self
            .members
            .iter()
            .any(|member| return member.standing.can_still_write());
    }

    /// What:
    /// `pub(crate) fn sentence(&self, scope: &Path) -> String` is the report as one clause of a refusal.
    /// Why:
    /// The refusal of a run that cannot be settled must say how long it waited and what it was waiting for,
    /// in words that tell an exited member from a live one.
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
            return format!(
                "the stop of {} waited {:?} and the kernel reported the scope empty",
                scope.display(),
                self.waited
            );
        }
        let listing: String = match (&self.listing_failure, self.members.is_empty()) {
            (Some(reason), _) => format!("it listed no member it could read ({reason})"),
            (None, true) => match self.populated {
                Some(true) => "it listed no member, so what is left is not a process of this scope's own group".to_string(),
                _ => "it listed no member".to_string(),
            },
            (None, false) => {
                let listed: Vec<String> = self
                    .members
                    .iter()
                    .map(|member| {
                        return format!("{} ({})", member.pid, member.standing.text());
                    })
                    .collect();
                // What:     `if listed.len() == 1 { "member" } else { "members" }` picks the word that agrees with the count.
                // Why:      A refusal is read by a person, and "1 member(s)" reads like a placeholder nobody finished.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const noun = listed.length === 1 ? 'member' : 'members';
                // ```
                let noun: &str = if listed.len() == 1 { "member" } else { "members" };
                format!("it listed {} {noun}: {}", listed.len(), listed.join(", "))
            }
        };
        let population: &str = match self.populated {
            Some(true) => "the kernel still reported it populated",
            Some(false) => "the kernel reported it empty",
            None => "the group could not be read",
        };
        return format!(
            "the stop of {} waited {:?} for it to be empty; {population}, and {listing}",
            scope.display(),
            self.waited
        );
    }
}

/// What:
/// `pub(crate) fn standing_of(pid: i32) -> MemberStanding` reads a process's threads and says what state they are in.
/// Why:
/// The kernel's `populated` word says that something is left,
/// never what,
/// and the state of one process's threads is the whole difference between a member that can still write and one that
/// only waits to be reaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function standingOf(pid: number): MemberStanding { ... }
/// ```
pub(crate) fn standing_of(pid: i32) -> MemberStanding {
    let folder: String = format!("/proc/{pid}/task");
    let entries = match fs::read_dir(&folder) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return MemberStanding::Gone,
        Err(error) => {
            return MemberStanding::Unknown(format!("{folder} cannot be read ({error})"));
        }
    };
    let mut leading_state: Option<char> = None;
    let mut a_thread_lives: bool = false;
    let mut threads: usize = 0;
    for listed in entries {
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
