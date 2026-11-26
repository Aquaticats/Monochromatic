//! What:
//!  One mount namespace's table read:
//!  one thread tried at a time until one answers,
//!  in the order that finds a mount above a chrooted thread's root,
//!  every attempt kept with the reason it answered,
//!  and the two kinds of failure it separates,
//!  a thread that ended and a table this process may not read.
//! Why:
//!  A namespace's table is the namespace's,
//!  so a thread that exits between the listing and the read costs a retry and
//!  not the namespace:
//!  the next live thread of it answers the same mounts
//!  (`proc_pid_mountinfo(5)`).
//!  The order matters because a table is rendered from the root of the thread
//!  it is read through:
//!  a thread chrooted away from the clone renders a table without it and
//!  answers no error,
//!  so the threads at the reader's own root are tried first
//!  (`crate::mount_namespace_scan::mount_namespace_tasks`).
//!  The two failures are kept apart because they mean different things to a
//!  caller:
//!  a thread that ended leaves no doubt about the namespace once a sibling
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
//! const read = readOneNamespace(group, readersRoot); // { namespace, attempts: [{ thread, mounts? , ended?, reason? }] }
//! ```

use crate::fs_mount_options::{
    MountFacts,
    parse_mount_table,
};
use crate::mount_namespace_scan::mount_namespace_tasks::{
    MountNamespaceTasks,
    RootIdentity,
    is_a_thread_that_ended,
};
use std::path::{
    Path,
    PathBuf,
};

/// The bound one representative mount table is read within,
/// the wrapper's own bound for a table another namespace holds.
pub const MOUNT_TABLE_LIMIT: u64 = crate::plain_file::MOUNT_TABLE_LIMIT;

/// The file a thread's mount table is read from.
pub const MOUNT_TABLE_FILE: &str = "mountinfo";

/// What:
///  What one thread's table read answered.
/// Why:
///  The caller decides what each answer means:
///  a table to scan,
///  a thread that ended,
///  or a read this process may not make,
///  and each names the thread it belongs to so a report can point at it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TaskTableAttempt =
///   | { kind: 'read'; thread: string; mounts: MountFacts[] }
///   | { kind: 'ended'; thread: string; reason: string }
///   | { kind: 'unreadable'; thread: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskTableAttempt {
    /// The thread answered its namespace's table.
    Read {
        /// The thread the table came from.
        thread: PathBuf,
        /// Its mounts.
        mounts: Vec<MountFacts>,
    },
    /// The thread ended before its table could be read,
    /// which is what `ENOENT`,
    /// `ESRCH` and `EINVAL` say about a thread that is
    /// gone.
    Ended {
        /// The thread.
        thread: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The table could not be read for a reason no thread's ending explains.
    Unreadable {
        /// The thread.
        thread: PathBuf,
        /// What the read answered.
        reason: String,
    },
}

impl TaskTableAttempt {
    /// The thread this attempt belongs to.
    pub fn thread(&self) -> &Path {
        return match self {
            TaskTableAttempt::Read {
                thread,
                ..
            } => thread.as_path(),
            TaskTableAttempt::Ended {
                thread,
                ..
            } => thread.as_path(),
            TaskTableAttempt::Unreadable {
                thread,
                ..
            } => thread.as_path(),
        };
    }

    /// What the read answered,
    /// or nothing when it answered a table.
    pub fn reason(&self) -> Option<&str> {
        return match self {
            TaskTableAttempt::Read {
                ..
            } => None,
            TaskTableAttempt::Ended {
                reason,
                ..
            } => Some(reason.as_str()),
            TaskTableAttempt::Unreadable {
                reason,
                ..
            } => Some(reason.as_str()),
        };
    }
}

/// What:
///  One namespace and what reading one of its tables answered.
/// Why:
///   The attempts stay in the order they were made,
///  so a caller sees the threads that ended before the one that answered,
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
    /// One attempt per thread tried,
    /// in the order they were tried,
    /// ending with the first that answered.
    pub attempts: Vec<TaskTableAttempt>,
}

impl NamespaceTableRead {
    /// The thread and mounts of the first attempt that answered,
    /// or nothing when no thread of the namespace answered.
    pub fn table(&self) -> Option<(&Path, &[MountFacts])> {
        for attempt in &self.attempts {
            if let TaskTableAttempt::Read {
                thread,
                mounts,
            } = attempt
            {
                return Some((thread.as_path(), mounts.as_slice()));
            }
        }
        return None;
    }

    /// Whether no thread of the namespace answered its table,
    /// which is a namespace with no live thread to read through.
    pub fn has_no_live_thread(&self) -> bool {
        return self.table().is_none();
    }
}

/// What:
///  Read one table for one namespace,
///  trying its threads in the order the group gives and stopping at the first
///  that answers.
/// Why:
///  One table per namespace is what the scan costs,
///  and the first thread that answers it is the one whose root sees the most:
///  a namespace whose threads all ended before the read has no table here,
///  and the caller reports that rather than reading nothing and calling the
///  namespace clear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = readOneNamespace(group, readersRoot);
/// ```
pub fn read_one_namespace(
    group: &MountNamespaceTasks,
    readers_root: Option<RootIdentity>,
) -> NamespaceTableRead {
    let mut attempts: Vec<TaskTableAttempt> = Vec::with_capacity(1);
    for thread in group.threads_in_reading_order(readers_root) {
        let table_path: PathBuf = thread.thread.join(MOUNT_TABLE_FILE);
        match crate::plain_file::read_plain_file(table_path.as_path(), MOUNT_TABLE_LIMIT) {
            Ok(bytes) => {
                attempts.push(TaskTableAttempt::Read {
                    thread: thread.thread.clone(),
                    mounts: parse_mount_table(bytes.as_slice()),
                });
                break;
            }
            Err(error) => {
                let reason: String = error.to_string();
                attempts.push(if is_a_thread_that_ended(&error) {
                    TaskTableAttempt::Ended {
                        thread: thread.thread.clone(),
                        reason,
                    }
                } else {
                    TaskTableAttempt::Unreadable {
                        thread: thread.thread.clone(),
                        reason,
                    }
                });
            }
        }
    }
    return NamespaceTableRead {
        namespace: group.namespace.clone(),
        attempts,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "namespace_table_reads_tests.rs"]
mod tests;
