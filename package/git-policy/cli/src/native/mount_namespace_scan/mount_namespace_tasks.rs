//! What:
//!  Every live task of one `/proc` root,
//!  grouped by the mount namespace the kernel says the task is in,
//!  and the refusal that names a `/proc` root it could not list.
//! Why:
//!  Two readers ask which mount namespaces a machine has:
//!  the start probe's arm 7,
//!  which compares the set it expected with the set that mentions the probe's
//!  source,
//!  and the root handler's holder scan,
//!  which reports who holds a clone
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!  Both group the same way:
//!  the kernel's own `mnt:[...]` text of `/proc/<pid>/ns/mnt`,
//!  read once per task and never guessed from a pid,
//!  so two tasks of one namespace are one namespace and one table is read for
//!  both.
//!  A task whose link answers nothing is skipped rather than named:
//!  it ended between the listing and the link,
//!  and a namespace it alone held has no live member,
//!  which is the caller's finding to report and not this reader's to invent.
//!  Reading tasks is done from a `/proc` root the caller names,
//!  so a control reads a fixture tree and never this machine's processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const groups = tasksByMountNamespace('/proc'); // [{ namespace: 'mnt:[4026531840]', tasks: ['/proc/11', '/proc/12'] }]
//! ```

use std::path::{Path, PathBuf};

/// The link whose target is a task's mount-namespace identity.
pub const MOUNT_NAMESPACE_LINK: [&str; 2] = ["ns", "mnt"];

/// What:
///  One mount namespace and one path to every live task in it.
/// Why:
///   The tasks are kept in the kernel's own order by pid,
///  so a call that reads one table per namespace reads the same task each
///  time and a control's fixture answers the same way twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountNamespaceTasks = { namespace: string; tasks: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountNamespaceTasks {
    /// The namespace's own identity,
    /// such as `mnt:[4026531840]`.
    pub namespace: String,
    /// One path below the `/proc` root per live task,
    /// ordered by the kernel's pid.
    pub tasks: Vec<PathBuf>,
}

/// What:
///  Why the `/proc` root could not be listed at all.
/// Why:
///  A listing that failed says nothing about who holds anything,
///  so a caller reports this rather than an empty set:
///  the mount table of no task was read,
///  and that is the whole finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ListingRefusal = { procRoot: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingRefusal {
    /// The `/proc` root.
    pub path: PathBuf,
    /// What the listing answered.
    pub reason: String,
}

impl ListingRefusal {
    /// The refusal in words,
    /// naming the directory and what to change.
    pub fn message(&self) -> String {
        return format!(
            "{} could not be listed ({reason}), so no task's mount namespace was read. Ways forward: check that the process may read that directory; a scan that listed no task has found no holder and has not shown there is none.",
            self.path.display(),
            reason = self.reason
        );
    }
}

/// What:
///  One task's mount-namespace identity,
///  or nothing when the task ended before its link could be read.
/// Why:
///   The identity is the kernel's own text and never a count this code keeps,
///  so a namespace that two tasks share is one group;
///  a task that ended between the listing and the read has no identity to
///  group under,
///  and `None` says that rather than inventing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = mountNamespaceOf('/proc/11'); // 'mnt:[4026531840]' | undefined
/// ```
pub fn mount_namespace_of(task: &Path) -> Option<String> {
    let mut link: PathBuf = task.to_path_buf();
    for component in MOUNT_NAMESPACE_LINK {
        link.push(component);
    }
    let target: PathBuf = std::fs::read_link(link.as_path()).ok()?;
    return Some(target.to_string_lossy().into_owned());
}

/// What:
///  Read every live task's namespace under one `/proc` root and group them.
/// Why:
///   Grouping before any table is read keeps the read to one table per
///  namespace however many tasks share it,
///  which is what makes the scan's cost follow the namespaces rather than the
///  task count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const groups = tasksByMountNamespace('/proc');
/// ```
///
/// # Errors
/// The `/proc` root could not be listed.
pub fn tasks_by_mount_namespace(proc_root: &Path) -> Result<Vec<MountNamespaceTasks>, ListingRefusal> {
    let listing: std::fs::ReadDir = match std::fs::read_dir(proc_root) {
        Ok(iter) => iter,
        Err(error) => return Err(ListingRefusal { path: proc_root.to_path_buf(), reason: error.to_string() }),
    };
    let mut grouped: std::collections::BTreeMap<String, Vec<(u32, PathBuf)>> = std::collections::BTreeMap::new();
    for entry in listing.filter_map(|entry| return entry.ok()) {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.is_empty() || !name.bytes().all(|byte| return byte.is_ascii_digit()) {
            continue;
        }
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        let task: PathBuf = entry.path();
        let Some(namespace) = mount_namespace_of(task.as_path()) else {
            continue;
        };
        grouped.entry(namespace).or_default().push((pid, task));
    }
    let mut groups: Vec<MountNamespaceTasks> = Vec::with_capacity(grouped.len());
    for (namespace, mut tasks) in grouped {
        tasks.sort_by_key(|(pid, _task)| return *pid);
        groups.push(MountNamespaceTasks { namespace, tasks: tasks.into_iter().map(|(_pid, task)| return task).collect() });
    }
    return Ok(groups);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_namespace_tasks_tests.rs"]
mod tests;
