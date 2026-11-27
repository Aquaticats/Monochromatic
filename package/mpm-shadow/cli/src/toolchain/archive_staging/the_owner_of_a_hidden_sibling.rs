//! What:
//! Which process made a hidden sibling,
//! as the pair that no reuse of a number can repeat within one boot:
//! the process id and the moment that process started.
//! Why:
//! A later install asks whether the installer that left a hidden directory still runs,
//! and a process id alone answers wrongly:
//! the system hands those numbers out again,
//! so a dead installer's number can belong to a program that has nothing to do with this tool,
//! and reading that program as the installer leaves a copy of a finished install on disk for ever.
//! The kernel reports a moment at which a process started,
//! and no second process with the same number carries the same moment,
//! so the two fields together are the fact a leftover is judged by.
//!
//! The answer has three values,
//! and only one of them allows anything to be removed:
//! the installer still runs,
//! it has ended,
//! or the system did not say.
//! A read that fails for any reason but "no such process" is the third,
//! because an unreadable record is not a dead process,
//! and treating it as one would let this tool delete a running install's own directory.
//!
//! The pair is read in this process's own process namespace,
//! from the start of this boot:
//! a process in another namespace can carry the same number with another start,
//! and two installs in two namespaces that share one toolchain root are therefore not told apart.
//! That is why an unjudged directory is left alone,
//! and it is the one limit this fact has.
//! Nothing is asked of the process itself:
//! only its `/proc/<pid>/stat` record is read,
//! never its command line,
//! its environment or a descriptor.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-owner-of-a-hidden-sibling.ts
//! type Owner = { pid: number; started: number };
//! type Liveness = 'running' | 'gone' | 'unknown';
//! function here(): Owner;
//! function liveness(owner: Owner): Liveness;
//! ```

/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// What this process can say about the process that made a hidden sibling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Liveness {
    /// The same process is still there:
    /// its number is taken and it started at the recorded moment.
    Running,
    /// The number belongs to no process,
    /// so the installer that made the directory has ended.
    Gone,
    /// The system did not answer,
    /// so nothing may be concluded and nothing may be removed.
    Unknown,
}

/// What:
/// The process that made a hidden sibling.
/// Why:
/// A hidden sibling's name holds these two fields,
/// so the record of who made it travels with the directory itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Owner = { pid: number; started: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Owner {
    /// The process id the system gave it.
    pub(crate) pid: u32,
    /// The moment it started,
    /// in clock ticks since boot,
    /// or `0` where the system reports no such moment.
    pub(crate) started: u64,
}

/// What:
/// The problem file the kernel reports one process's own record at,
/// from the process id alone.
/// Why:
/// The record outlives nothing:
/// it is the kernel's answer for the number as it is now,
/// which is why the moment it holds is the fact this module judges by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const statPath = (pid: number) => `/proc/${pid}/stat`;
/// ```
fn stat_path(pid: u32) -> PathBuf {
    return Path::new("/proc").join(pid.to_string()).join("stat");
}

/// What:
/// What the read of a process's record answered.
/// Why:
/// "No such process" and "the record could not be read" are different answers,
/// and only the first says the installer ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartTime = { at: bigint } | 'gone' | 'unreadable';
/// ```
enum StartTime {
    /// The record says when the process started.
    At {
        /// The moment the record names,
        /// in clock ticks since boot.
        moment: u64,
    },
    /// The number belongs to no process.
    Gone,
    /// The record could not be read or could not be understood.
    Unreadable,
}

/// What:
/// The moment one `/proc/<pid>/stat` record reports as the process's start,
/// in clock ticks since boot,
/// or nothing when the bytes are not that record.
/// Why:
/// The second field is the program's name in brackets,
/// which may hold spaces,
/// brackets and bytes that are not text,
/// so the record is read as bytes and every field is counted from after the last closing bracket.
/// A record cut short is no record at all,
/// and answering a moment from one would judge a running installer by a made-up number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function momentIn(record: Buffer): bigint | undefined;
/// ```
pub(crate) fn moment_in(record: &[u8]) -> Option<u64> {
    let closing: usize = record.iter().rposition(|byte| return *byte == b')')?;
    let tail: &[u8] = record.get(closing + 1..)?;
    // The record's third field is the state, so the twenty-second is the nineteenth after the name.
    let field: &[u8] = tail
        .split(|byte| return byte.is_ascii_whitespace())
        .filter(|part| return !part.is_empty())
        .nth(19)?;
    return std::str::from_utf8(field).ok()?.parse().ok();
}

/// What:
/// The moment `/proc/<pid>/stat` reports as the process's start,
/// in clock ticks since boot.
/// Why:
/// "No such file" is the system saying the process ended,
/// which is an answer;
/// any other failure is the system saying nothing,
/// which is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startTimeOf(pid: number): bigint | 'gone' | 'unreadable';
/// ```
fn start_time_of(pid: u32) -> StartTime {
    let record: Vec<u8> = match std::fs::read(stat_path(pid)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return StartTime::Gone,
        Err(_) => return StartTime::Unreadable,
    };
    return match moment_in(&record) {
        Some(moment) => StartTime::At {
            moment,
        },
        None => StartTime::Unreadable,
    };
}

/// What:
/// The moment a process the system reports started at,
/// or nothing when it reports none,
/// for a control that builds the pair of a process it can see.
/// Why:
/// A control has no use for the difference between a number no process holds and a record it cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startTimeOfAProcess(pid: number): bigint | undefined;
/// ```
#[cfg(test)]
pub(crate) fn start_time_of_a_process(pid: u32) -> Option<u64> {
    return match start_time_of(pid) {
        StartTime::At {
            moment,
        } => Some(moment),
        StartTime::Gone | StartTime::Unreadable => None,
    };
}

/// The methods of `Owner`.
impl Owner {
    /// What:
    /// The owner of what this process is making now.
    /// Why:
    /// Every hidden sibling this process writes carries this pair,
    /// so a later install can tell whether that process is still there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function here(): Owner { return { pid: process.pid, started: startTimeOf(process.pid) ?? 0 }; }
    /// ```
    pub(crate) fn here() -> Owner {
        let pid: u32 = std::process::id();
        let started: u64 = match start_time_of(pid) {
            StartTime::At {
                moment,
            } => moment,
            StartTime::Gone | StartTime::Unreadable => 0,
        };
        return Owner {
            pid,
            started,
        };
    }

    /// What:
    /// What this process can say about the process that made a hidden sibling.
    /// Why:
    /// Only a process the system reports as ended may lose its directory:
    /// a number now held by another program carries a different start and is that program,
    /// while a record that could not be read says nothing at all.
    /// A leftover whose name carries no start time was written where the system reported none,
    /// and nothing can tell it apart from one of a running installer,
    /// so it is never judged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function liveness(owner: Owner): Liveness {
    ///   if (owner.started === 0) return 'unknown';
    ///   const seen = startTimeOf(owner.pid);
    ///   if (seen === 'gone') return 'gone';
    ///   if (seen === 'unreadable') return 'unknown';
    ///   return seen === owner.started ? 'running' : 'gone';
    /// }
    /// ```
    pub(crate) fn liveness(self) -> Liveness {
        if self.started == 0 {
            return Liveness::Unknown;
        }
        return match start_time_of(self.pid) {
            StartTime::At {
                moment,
            } if moment == self.started => Liveness::Running,
            StartTime::At {
                moment: _,
            } => Liveness::Gone,
            StartTime::Gone => Liveness::Gone,
            StartTime::Unreadable => Liveness::Unknown,
        };
    }
}
