//! What:
//!  One mount namespace's table read:
//!  one task tried at a time until one answers,
//!  every attempt kept with the reason it answered,
//!  and the two kinds of failure it separates,
//!  a task that ended and a table this process may not read.
//! Why:
//!  A namespace's table is the namespace's,
//!  so a task that exits between the listing and the read costs a retry and
//!  not the namespace:
//!  the next live task of it answers the same mounts
//!  (`proc_pid_mountinfo(5)`).
//!  The two failures are kept apart because they mean different things to a
//!  caller:
//!  a task that ended leaves no doubt about the namespace once a sibling
//!  answers,
//!  while a table this process may not open is a namespace whose mounts are
//!  unknown,
//!  and the holder scan reports it rather than treating it as empty
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!  The start probe's arm keeps refusing what it may not read,
//!  and the holder scan keeps scanning and reports instead,
//!  so this reader refuses nothing and hands its caller every attempt.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const read = readOneNamespace(group); // { namespace, attempts: [{ task, mounts? , ended?, reason? }] }
//! ```

use crate::fs_mount_options::{MountFacts, parse_mount_table};
use crate::mount_namespace_scan::mount_namespace_tasks::MountNamespaceTasks;
use std::path::{Path, PathBuf};

/// The bound one representative mount table is read within,
/// the wrapper's own bound for a table another namespace holds.
pub const MOUNT_TABLE_LIMIT: u64 = crate::plain_file::MOUNT_TABLE_LIMIT;

/// The file a task's mount table is read from.
pub const MOUNT_TABLE_FILE: &str = "mountinfo";

/// What:
///  What one task's table read answered.
/// Why:
///  The caller decides what each answer means:
///  a table to scan,
///  a task that ended,
///  or a read this process may not make,
///  and each names the task it belongs to so a report can point at it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TaskTableAttempt =
///   | { kind: 'read'; task: string; mounts: MountFacts[] }
///   | { kind: 'ended'; task: string; reason: string }
///   | { kind: 'unreadable'; task: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskTableAttempt {
    /// The task answered its namespace's table.
    Read {
        /// The task the table came from.
        task: PathBuf,
        /// Its mounts.
        mounts: Vec<MountFacts>,
    },
    /// The task ended before its table could be read,
    /// which is what `ENOENT`, `ESRCH` and `EINVAL` say about a task that is
    /// gone.
    Ended {
        /// The task.
        task: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The table could not be read for a reason no task's ending explains.
    Unreadable {
        /// The task.
        task: PathBuf,
        /// What the read answered.
        reason: String,
    },
}

impl TaskTableAttempt {
    /// The task this attempt belongs to.
    pub fn task(&self) -> &Path {
        return match self {
            TaskTableAttempt::Read { task, .. } => task.as_path(),
            TaskTableAttempt::Ended { task, .. } => task.as_path(),
            TaskTableAttempt::Unreadable { task, .. } => task.as_path(),
        };
    }

    /// What the read answered,
    /// or nothing when it answered a table.
    pub fn reason(&self) -> Option<&str> {
        return match self {
            TaskTableAttempt::Read { .. } => None,
            TaskTableAttempt::Ended { reason, .. } => Some(reason.as_str()),
            TaskTableAttempt::Unreadable { reason, .. } => Some(reason.as_str()),
        };
    }
}

/// What:
///  One namespace and what reading one of its tables answered.
/// Why:
///   The attempts stay in the order they were made,
///  so a caller sees the namespace's first live task and every task that ended
///  before it,
///  and the table is the one the read stopped at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceTableRead = { namespace: string; attempts: TaskTableAttempt[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceTableRead {
    /// The namespace the attempts belong to.
    pub namespace: String,
    /// One attempt per task tried,
    /// in pid order,
    /// ending with the first that answered.
    pub attempts: Vec<TaskTableAttempt>,
}

impl NamespaceTableRead {
    /// The task and mounts of the first attempt that answered,
    /// or nothing when no task of the namespace answered.
    pub fn table(&self) -> Option<(&Path, &[MountFacts])> {
        for attempt in &self.attempts {
            if let TaskTableAttempt::Read { task, mounts } = attempt {
                return Some((task.as_path(), mounts.as_slice()));
            }
        }
        return None;
    }

    /// Whether no task of the namespace answered its table,
    /// which is a namespace with no live member to read through.
    pub fn has_no_live_member(&self) -> bool {
        return self.table().is_none();
    }
}

/// What:
///  Read one table for one namespace,
///  trying its tasks in order and stopping at the first that answers.
/// Why:
///  One table per namespace is what the scan costs,
///  and the first live task answers it:
///  a namespace whose tasks all ended before the read has no table here,
///  and the caller reports that rather than reading nothing and calling the
///  namespace clear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = readOneNamespace(group);
/// ```
pub fn read_one_namespace(group: &MountNamespaceTasks) -> NamespaceTableRead {
    let mut attempts: Vec<TaskTableAttempt> = Vec::with_capacity(1);
    for task in &group.tasks {
        let table_path: PathBuf = task.join(MOUNT_TABLE_FILE);
        match crate::plain_file::read_plain_file(table_path.as_path(), MOUNT_TABLE_LIMIT) {
            Ok(bytes) => {
                attempts.push(TaskTableAttempt::Read { task: task.clone(), mounts: parse_mount_table(bytes.as_slice()) });
                break;
            }
            Err(error) => {
                let ended: bool = matches!(error.raw_os_error(), Some(libc::ENOENT) | Some(libc::ESRCH) | Some(libc::EINVAL));
                let reason: String = error.to_string();
                attempts.push(if ended {
                    TaskTableAttempt::Ended { task: task.clone(), reason }
                } else {
                    TaskTableAttempt::Unreadable { task: task.clone(), reason }
                });
            }
        }
    }
    return NamespaceTableRead { namespace: group.namespace.clone(), attempts };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "namespace_table_reads_tests.rs"]
mod tests;
