//! What:
//! Which process made a hidden sibling,
//! as the pair that no reuse of a number can repeat:
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
//! Nothing is asked of a process whose number is reused:
//! only `/proc/<pid>/stat` is read,
//! never its command line,
//! its environment or a descriptor.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-owner-of-a-hidden-sibling.ts
//! type Owner = { pid: number; started: number };
//! function here(): Owner;
//! function stillRuns(owner: Owner): boolean;
//! ```

/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

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
/// The moment `/proc/<pid>/stat` reports as the process's start,
/// in clock ticks since boot.
/// Why:
/// The second field of that record is the program's name in brackets,
/// which may itself hold spaces and brackets,
/// so every field is counted from after the last closing bracket.
/// A record that cannot be read answers nothing:
/// the process may have ended,
/// or the system may keep no such record at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startTimeOf(pid: number): bigint | undefined;
/// ```
pub(crate) fn start_time_of(pid: u32) -> Option<u64> {
    let record: String = std::fs::read_to_string(stat_path(pid)).ok()?;
    let tail: &str = record.get(record.rfind(')')? + 1..)?;
    let fields: Vec<&str> = tail.split_whitespace().collect();
    // The record's third field is the state, so the twenty-second is the nineteenth after the name.
    return fields.get(19)?.parse().ok();
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
        return Owner {
            pid,
            started: start_time_of(pid).unwrap_or(0),
        };
    }

    /// What:
    /// Whether the process that made a hidden sibling still runs.
    /// Why:
    /// The pair is the fact:
    /// a number that now belongs to another program carries a different start,
    /// so that program is not this installer,
    /// and the hidden directory may be read as what it is,
    /// the remains of a process that ended.
    /// A leftover whose name carries no start time was written where the system reports none,
    /// and nothing can tell it apart from one of a running installer,
    /// so it counts as running and is left where it is.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function stillRuns(owner: Owner): boolean { return owner.started !== 0 && startTimeOf(owner.pid) === owner.started; }
    /// ```
    pub(crate) fn still_runs(self) -> bool {
        if self.started == 0 {
            return true;
        }
        return start_time_of(self.pid) == Some(self.started);
    }
}
