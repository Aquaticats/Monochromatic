//! What:
//!  Every read a holder scan could not complete:
//!  the `/proc` root it could not list,
//!  a task that ended before its table was read,
//!  a table it may not open,
//!  a namespace no live task was left to read through,
//!  and the two bounds the scan stops at.
//! Why:
//!  A scan reports unreadable and racing entries rather than treating them as
//!  clear
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!  The causes are kept apart because each is a different thing for a reader to
//!  do:
//!  a task that ended is churn whose namespace another task answered,
//!  while a namespace with no live member is a namespace whose mounts nobody
//!  read,
//!  and the destroy-anyway decision accepts that residual rather than hiding it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal when the namespace exclusion cannot be shown").
//!  A pinned namespace with no live member is not here:
//!  no file the scan reads names it,
//!  so the scan cannot say it exists and never reports it as clear either
//!  (`SPEC.md`,
//!  "The all-namespace holder scan").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const read = { kind: 'no-live-member', namespace: 'mnt:[4026532001]', tasksTried: 2 };
//! ```

use std::path::PathBuf;

/// The word each cause is written as in a result's text.
pub const LISTING_WORD: &str = "listing";
/// The word a task's ending is written as.
pub const TASK_ENDED_WORD: &str = "task-ended";
/// The word a table this process may not open is written as.
pub const TABLE_UNREADABLE_WORD: &str = "table-unreadable";
/// The word a namespace with no live member is written as.
pub const NO_LIVE_MEMBER_WORD: &str = "no-live-member";
/// The word the namespace bound is written as.
pub const NAMESPACES_PAST_THE_BOUND_WORD: &str = "namespaces-past-the-bound";
/// The word the detail bound is written as.
pub const DETAILS_PAST_THE_BOUND_WORD: &str = "details-past-the-bound";

/// What:
///  One read the scan could not complete,
///  and what it was.
/// Why:
///   Every variant names the thing that could not be read,
///  so a report points at a directory,
///  a namespace or a task rather than at a count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IncompleteRead =
///   | { kind: 'listing'; procRoot: string; reason: string }
///   | { kind: 'task-ended' | 'table-unreadable'; namespace: string; task: string; reason: string }
///   | { kind: 'no-live-member'; namespace: string; tasksTried: number; reason: string }
///   | { kind: 'namespaces-past-the-bound' | 'details-past-the-bound'; bound: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncompleteRead {
    /// No task of the `/proc` root could be listed,
    /// so nothing was scanned.
    Listing {
        /// The `/proc` root.
        proc_root: PathBuf,
        /// What the listing answered.
        reason: String,
    },
    /// A task ended between the listing and its table read.
    TaskEnded {
        /// The namespace the task was in.
        namespace: String,
        /// The task.
        task: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// A task's table could not be opened for a reason its ending does not
    /// explain.
    TableUnreadable {
        /// The namespace the task was in.
        namespace: String,
        /// The task.
        task: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// Every task of a namespace ended before its table was read,
    /// so the namespace's mounts are unknown.
    NoLiveMember {
        /// The namespace.
        namespace: String,
        /// How many tasks of it were tried.
        tasks_tried: usize,
        /// What the last read answered.
        reason: String,
    },
    /// The scan stopped reading namespaces at its own bound.
    NamespacesPastTheBound {
        /// The bound.
        bound: usize,
    },
    /// The scan stopped listing and reading at its own detail bound.
    DetailsPastTheBound {
        /// The bound.
        bound: usize,
    },
}

impl IncompleteRead {
    /// The word the result's text carries for this cause.
    pub fn kind(&self) -> &'static str {
        return match self {
            IncompleteRead::Listing { .. } => LISTING_WORD,
            IncompleteRead::TaskEnded { .. } => TASK_ENDED_WORD,
            IncompleteRead::TableUnreadable { .. } => TABLE_UNREADABLE_WORD,
            IncompleteRead::NoLiveMember { .. } => NO_LIVE_MEMBER_WORD,
            IncompleteRead::NamespacesPastTheBound { .. } => NAMESPACES_PAST_THE_BOUND_WORD,
            IncompleteRead::DetailsPastTheBound { .. } => DETAILS_PAST_THE_BOUND_WORD,
        };
    }

    /// The namespace this cause belongs to,
    /// or nothing when it is not about one namespace.
    pub fn namespace(&self) -> Option<&str> {
        return match self {
            IncompleteRead::TaskEnded { namespace, .. } => Some(namespace.as_str()),
            IncompleteRead::TableUnreadable { namespace, .. } => Some(namespace.as_str()),
            IncompleteRead::NoLiveMember { namespace, .. } => Some(namespace.as_str()),
            IncompleteRead::Listing { .. } | IncompleteRead::NamespacesPastTheBound { .. } | IncompleteRead::DetailsPastTheBound { .. } => None,
        };
    }

    /// The task this cause belongs to,
    /// or nothing when it is not about one task.
    pub fn task(&self) -> Option<&std::path::Path> {
        return match self {
            IncompleteRead::TaskEnded { task, .. } => Some(task.as_path()),
            IncompleteRead::TableUnreadable { task, .. } => Some(task.as_path()),
            IncompleteRead::Listing { .. } | IncompleteRead::NoLiveMember { .. } => None,
            IncompleteRead::NamespacesPastTheBound { .. } | IncompleteRead::DetailsPastTheBound { .. } => None,
        };
    }

    /// The cause in words:
    /// what could not be read,
    /// why,
    /// and what a reader may not conclude from it.
    pub fn message(&self) -> String {
        return match self {
            IncompleteRead::Listing { proc_root, reason } => format!(
                "{} could not be listed ({reason}), so no task's mount namespace was read and this scan found no holder without showing there is none. Ways forward: check that the scan may read that directory, then scan again.",
                proc_root.display()
            ),
            IncompleteRead::TaskEnded { namespace, task, reason } => format!(
                "mount namespace {namespace} lost the task {} before its mount table was read ({reason}); the namespace is reported by whichever other task was read, and a namespace whose every task ends this way is reported as having no live member. Ways forward: none is required: a task that exits is the scan's ordinary churn, and it is named so that a report can say the read raced.",
                task.display()
            ),
            IncompleteRead::TableUnreadable { namespace, task, reason } => format!(
                "mount namespace {namespace} answered no table from {} ({reason}), so whether it holds the dataset is unknown and it is never reported as clear. Ways forward: check the scan's rights over that task's mount table, then scan again.",
                task.display()
            ),
            IncompleteRead::NoLiveMember { namespace, tasks_tried, reason } => format!(
                "mount namespace {namespace} has no live task to read through: every one of its {tasks_tried} tasks ended before its mount table was read ({reason}), so this namespace may hold the dataset and the scan cannot say it does not. Ways forward: scan again while a task of it is alive; a namespace held open by a handle and no task is invisible to every scan.",
            ),
            IncompleteRead::NamespacesPastTheBound { bound } => format!(
                "the scan read {bound} mount namespaces and stopped, so every namespace after that one is in none of its lists. Ways forward: this is the scan's own bound; a machine with more namespaces than this needs its own report, and no scan of it can call itself clear.",
            ),
            IncompleteRead::DetailsPastTheBound { bound } => format!(
                "the scan listed {bound} findings and stopped listing and reading, so namespaces after that one are in none of its lists. Ways forward: this is the scan's own bound; a machine with more findings than this needs its own report, and no scan of it can call itself clear.",
            ),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "incomplete_reads_tests.rs"]
mod tests;
