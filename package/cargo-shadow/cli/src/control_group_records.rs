//! What:
//! The records a run writes about its scope and the verdict on whether a scope still has members.
//! Why:
//! Split out of `control_group.rs`,
//! which holds the containment of a run in a control group,
//! to keep that file under the code-line limit.
//! The items are re-exported from `control_group`,
//! so their paths there are unchanged.

/// The view,
/// the namespace and mount reads,
/// and the membership read the verdict builds on.
use crate::control_group::{CGROUP_ROOT, View, is_cgroup2, namespace_id, populated_at, view_of};
/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::{ExplainedError, OrText};
/// Imports `std::fmt` for this file.
use std::fmt;

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;




/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};








/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;


/// What a journal remembers of a run's containment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Record {
    /// A transient scope:
    /// the cgroup namespace and the view of the directory that holds it,
    /// recorded before the program starts,
    /// and the scope's path.
    Scope {
        /// The inode number of the cgroup namespace of the process that planned it.
        namespace: u64,
        /// How the directory that holds the scope was seen.
        parent_view: View,
        /// The directory of the scope.
        path: PathBuf,
    },
    /// The run that recorded this scope ended and every process of its launcher was reaped,
    /// so no launcher can still create the scope:
    /// from here an absent scope means a scope that ended,
    /// not one that was never made (round 9,
    /// 8a F1).
    Settled {
        /// The directory of the scope that is settled.
        path: PathBuf,
    },
    /// The launcher of the scope with this path,
    /// noted right after it was spawned (round 11):
    /// its process id,
    /// its start time (clock
    /// ticks since boot,
    /// field 22 of `/proc/<pid>/stat`) and the inode of its process id namespace.
    /// A launcher that is gone can never join the scope later,
    /// so a recovery that finds it gone may read the scope as final.
    Launcher {
        /// The process id of the launcher,
        /// which becomes the program.
        pid: i32,
        /// The start time of that process.
        start: u64,
        /// The inode number of the process id namespace of the wrapper,
        /// which is the one the pid belongs to.
        pid_namespace: u64,
        /// The directory of the scope the launcher was to enter.
        path: PathBuf,
    },
    /// The weaker path:
    /// only a process group (round 5),
    /// recorded by the process id of its leader.
    ProcessGroup(
        /// The leader's process id,
        /// which is the group's number.
        i32,
    ),
    /// The weaker path is about to start a child whose process group is not known yet;
    /// the line that follows it names the group.
    /// A killed wrapper that leaves this line alone may have a running child that no record names.
    ProcessGroupStarting,
}

/// What a recovery learns about a recorded scope.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Membership {
    /// No member can exist:
    /// the scope is verifiably absent,
    /// or present and empty.
    Gone,
    /// At least one thread of the scope lives.
    Alive,
    /// The question cannot be answered,
    /// with the reason.
    Unknown(
        /// Why.
        String,
    ),
}

/// How a record reads as a journal line.
impl fmt::Display for Record {
    /// What:
    /// `fn fmt(&self, formatter)` writes the one-line form of the record,
    /// without the line feed.
    /// Why:
    /// The journal's `groups` file holds one line per record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Record::Scope { namespace, parent_view, path } => write!(formatter, "scope {namespace} {parent_view} {}", path.display()),
            Record::Settled { path } => write!(formatter, "settled {}", path.display()),
            Record::Launcher { pid, start, pid_namespace, path } => write!(formatter, "launcher {pid} {start} {pid_namespace} {}", path.display()),
            Record::ProcessGroup(leader) => write!(formatter, "pgroup {leader}"),
            Record::ProcessGroupStarting => write!(formatter, "pgroup starting"),
        };
    }
}

/// What:
/// `fn parse_number(text: &str, what: &str, line: &str) -> Result<u64, String>` reads one number of a record.
/// Why:
/// The reason names the line and the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseNumber(text: string, what: string, line: string): number
/// ```
pub(crate) fn parse_number(text: &str, what: &str, line: &str) -> Result<u64, String> {
    return text.parse::<u64>().explained(format_args!("{line:?} has no {what}"));
}

/// What:
/// `pub(crate) fn parse_record(line: &str) -> Result<Record, String>` reads one line of a `groups` file.
/// Why:
/// A line that is not understood is uncertainty about a possible writer,
/// and the caller says so.
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
        let leader: i32 = rest.trim().parse::<i32>().explained(format_args!("{line:?} is not a process group"))?;
        if leader <= 0 {
            return Err(format!("{line:?} is not a process group"));
        }
        return Ok(Record::ProcessGroup(leader));
    }
    if let Some(rest) = line.strip_prefix("launcher ") {
        let mut parts = rest.splitn(4, ' ');
        let pid_text: &str = parts.next().unwrap_or("");
        let pid: i32 = pid_text.parse::<i32>().explained(format_args!("{line:?} has no launcher process id"))?;
        if pid <= 0 {
            return Err(format!("{line:?} has no launcher process id"));
        }
        let start: u64 = parse_number(parts.next().unwrap_or(""), "start time", line)?;
        let pid_namespace: u64 = parse_number(parts.next().unwrap_or(""), "process id namespace", line)?;
        let path: &str = parts.next().unwrap_or("");
        // The path is absolute: a line with anything else there (a field of another version) is not understood.
        if !path.starts_with('/') {
            return Err(format!("{line:?} has no absolute path"));
        }
        return Ok(Record::Launcher { pid, start, pid_namespace, path: PathBuf::from(path) });
    }
    if let Some(rest) = line.strip_prefix("settled ") {
        if rest.is_empty() {
            return Err(format!("{line:?} has no path"));
        }
        return Ok(Record::Settled { path: PathBuf::from(rest) });
    }
    if let Some(rest) = line.strip_prefix("scope ") {
        let mut parts = rest.splitn(3, ' ');
        let namespace: u64 = parse_number(parts.next().unwrap_or(""), "namespace id", line)?;
        let view_text: &str = parts.next().unwrap_or("");
        let (mount_text, inode_text) = view_text.split_once(':').or_text(format_args!("{line:?} has no mount view"))?;
        let parent_view: View = View { mount: parse_number(mount_text, "mount id", line)?, inode: parse_number(inode_text, "inode", line)? };
        let path: &str = parts.next().unwrap_or("");
        if path.is_empty() {
            return Err(format!("{line:?} has no path"));
        }
        return Ok(Record::Scope { namespace, parent_view, path: PathBuf::from(path) });
    }
    return Err(format!("{line:?} is not a record of a scope or a process group"));
}

/// What:
/// `pub(crate) fn query(namespace: u64, parent_view: View, path: &Path) -> Membership` says whether the recorded scope has a member.
/// Why:
/// An absent scope is read as gone only when this process sees the cgroup file system as the run did:
/// the same cgroup namespace,
/// a mounted cgroup v2,
/// and the directory that held the scope with the recorded mount id and inode
/// (contract assumption c,
/// the second check).
/// A scope that is present is asked for its `populated` flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function query(namespace: number, parentView: View, path: string): 'gone' | 'alive' | { unknown: string }
/// ```
pub(crate) fn query(namespace: u64, parent_view: View, path: &Path) -> Membership {
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
    let Some(parent) = path.parent() else {
        return Membership::Unknown(format!("{} has no parent directory", path.display()));
    };
    match view_of(parent) {
        Ok(seen) if seen == parent_view => {}
        Ok(seen) => {
            return Membership::Unknown(format!(
                "{} is seen as mount and inode {seen} here and was {parent_view} when the scope was recorded, so the recorded path may lead somewhere else",
                parent.display()
            ));
        }
        Err(error) => return Membership::Unknown(format!("{} cannot be examined ({error})", parent.display())),
    }
    match fs::metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Membership::Gone,
        Err(error) => return Membership::Unknown(format!("{} cannot be examined ({error})", path.display())),
    }
    return match populated_at(path) {
        Ok(true) => Membership::Alive,
        Ok(false) => Membership::Gone,
        // A scope removed between the two reads is absent.
        Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) => Membership::Gone,
        Err(error) => Membership::Unknown(format!("{} cannot be read ({error})", path.join("cgroup.events").display())),
    };
}
