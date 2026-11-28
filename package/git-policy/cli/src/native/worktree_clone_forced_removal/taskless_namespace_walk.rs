//! What:
//!  The walk that finds a mount namespace no task holds:
//!  the kernel's own listing of this machine's mount namespaces,
//!  the ids its tasks' namespaces answer,
//!  the difference between the two,
//!  the descriptor each remaining id becomes,
//!  and the copy that descriptor's namespace holds.
//! Why:
//!  A mount namespace whose tasks are gone still holds the clone's copy and
//!  still keeps the dataset busy,
//!  and no `/proc/<pid>` path names it,
//!  so every read the rest of this arm makes is blind to it
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  F13,
//!  F14 and "Who ends each kind of holder").
//!  `listns(2)` lists the namespaces of the caller's own user namespace,
//!  and `open_by_handle_at` on the nsfs mount turns one listed id back into a
//!  descriptor the act can enter with `setns`,
//!  which is the design's own route for this kind.
//!  The two refusals are the design's:
//!  a kernel without `listns` answers the first call `ENOSYS`,
//!  and this kind is `holder-unendable` there;
//!  a read the walk could not make is `holder-blind`,
//!  and no act follows it.
//!  Nothing here changes a namespace:
//!  the walk lists,
//!  reads and opens,
//!  and the act that follows is the folders' own stages.
//!  The kernel's own numbers and records are the sibling
//!  `the_namespace_listing.rs`,
//!  which this file reads and never repeats.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = findTheTasklessNamespacesHolding({ procRoot: '/proc', dataset, slot });
//! found.map((one) => one.identity);
//! ```

use super::forced_removal_words::RefusalWord;
use super::mount_namespace_child::{
    ActReport,
    run_in,
};
use super::namespace_owner::{
    identity_of,
    open_a_namespace,
};
use super::receiver_closure::MountRow;
use super::the_childs_report::ActOutcome;
use super::the_mount_table_the_act_reads::{
    read_the_table,
    rows_of,
    topmost_at,
};
use super::the_namespace_listing::{
    NS_GET_ID,
    OUR_MOUNT_NAMESPACE,
    TheCallsRefusal,
    open_by_the_id,
    the_mount_namespaces_on_this_machine,
};
use std::collections::BTreeSet;
use std::os::fd::AsRawFd;
use std::path::{
    Path,
    PathBuf,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TasklessWalkRefusal {
    /// The kernel has no `listns`,
    /// which is every guest of `6.8` and older the arms ran on.
    TheKernelHasNoListns {
        /// The errno the call answered,
        /// which is `ENOSYS`.
        errno: i32,
    },
    /// The listing itself was refused.
    TheListingWasRefused {
        /// The errno the call answered.
        errno: i32,
    },
    /// One task's mount namespace could not be read,
    /// so the walk cannot say which ids a task holds.
    ATasksNamespaceWasNotReadable {
        /// The task whose namespace it is.
        task: PathBuf,
        /// The errno the open or the ioctl answered.
        errno: i32,
    },
    /// A listed id did not become a descriptor.
    ANamespaceCouldNotBeOpened {
        /// The id the listing answered.
        id: u64,
        /// The errno the call answered.
        errno: i32,
    },
    /// One taskless namespace's mount table could not be read.
    ANamespacesTableCouldNotBeRead {
        /// The namespace's kernel identity.
        identity: String,
        /// What the read said.
        said: String,
    },
}

impl TasklessWalkRefusal {
    /// What:
    ///  The word the receipt's failed step and the report carry.
    /// Why:
    ///  A kernel without the call is the design's `holder-unendable`,
    ///  and a read that failed is `holder-blind`,
    ///  so a person reads which of the two stopped the act.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusal.word // 'holder-unendable' | 'holder-blind'
    /// ```
    pub fn word(&self) -> RefusalWord {
        return match self {
            TasklessWalkRefusal::TheKernelHasNoListns {
                ..
            } => RefusalWord::HolderUnendable,
            TasklessWalkRefusal::TheListingWasRefused {
                ..
            }
            | TasklessWalkRefusal::ATasksNamespaceWasNotReadable {
                ..
            }
            | TasklessWalkRefusal::ANamespaceCouldNotBeOpened {
                ..
            }
            | TasklessWalkRefusal::ANamespacesTableCouldNotBeRead {
                ..
            } => RefusalWord::HolderBlind,
        };
    }
}

impl std::fmt::Display for TasklessWalkRefusal {
    /// One sentence naming what the walk asked and what it got.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            TasklessWalkRefusal::TheKernelHasNoListns {
                errno,
            } => write!(
                formatter,
                "the kernel answered errno {errno} to listns, so this kernel has no such call and a namespace no task holds cannot be listed here"
            ),
            TasklessWalkRefusal::TheListingWasRefused {
                errno,
            } => write!(formatter, "listns refused the listing with errno {errno}"),
            TasklessWalkRefusal::ATasksNamespaceWasNotReadable {
                task,
                errno,
            } => write!(
                formatter,
                "the mount namespace of the task {} could not be read: errno {errno}",
                task.display()
            ),
            TasklessWalkRefusal::ANamespaceCouldNotBeOpened {
                id,
                errno,
            } => write!(
                formatter,
                "the listed mount namespace id {id} did not become a descriptor: errno {errno}"
            ),
            TasklessWalkRefusal::ANamespacesTableCouldNotBeRead {
                identity,
                said,
            } => write!(
                formatter,
                "the mount table of the namespace {identity} could not be read: {said}"
            ),
        };
    }
}

/// What:
///  The walk's own refusal for a namespace call the kernel refused.
/// Why:
///  `ENOSYS` is the kernel having no such call,
///  which is the design's `holder-unendable`,
///  and every other errno is a read the act could not make,
///  which is `holder-blind`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = refusal.errno === ENOSYS ? { kind: 'no-listns' } : { kind: 'listing-refused' };
/// ```
fn the_walks_refusal_of_the_call(refusal: TheCallsRefusal) -> TasklessWalkRefusal {
    if refusal.errno == libc::ENOSYS {
        return TasklessWalkRefusal::TheKernelHasNoListns {
            errno: refusal.errno,
        };
    }
    return TasklessWalkRefusal::TheListingWasRefused {
        errno: refusal.errno,
    };
}

/// What:
///  One mount namespace no task holds,
///  with the descriptor the act enters it through and the table it read.
/// Why:
///  The act's stages take a descriptor and the namespace's identity,
///  and the preflight needs its rows to plan the receivers,
///  so all four travel together and no caller opens a second descriptor for
///  the same namespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TasklessNamespace = { id: bigint; identity: string; namespace: File; rows: MountRow[] };
/// ```
pub struct TasklessNamespace {
    /// What:
    ///  The id `listns` listed,
    ///  which a report names beside the identity.
    /// Why:
    ///  The identity is the object's inode and the id is the one the listing
    ///  answered,
    ///  and the design records that both may be reused after the object dies,
    ///  so a report names both.
    pub id: u64,
    /// What:
    ///  The namespace's own kernel identity,
    ///  such as `mnt:[4026532409]`.
    /// Why:
    ///  The journal and the refusal name a namespace by this,
    ///  which is the same spelling the scan and a `/proc` link give.
    pub identity: String,
    /// What:
    ///  The descriptor the act enters the namespace through.
    /// Why:
    ///  `setns` takes a descriptor,
    ///  and this one is the namespace the id named when the walk read it.
    pub namespace: std::fs::File,
    /// What:
    ///  The namespace's mount table as its own child read it.
    /// Why:
    ///  The preflight decides which namespaces receive the unmount from these
    ///  rows,
    ///  and a taskless namespace has no `/proc/<pid>/mountinfo` to read.
    pub rows: Vec<MountRow>,
}

/// What:
///  Every mount namespace id one of this machine's tasks is in.
/// Why:
///  The listing holds both kinds,
///  and a namespace some task is in is reached by `/proc` and ended by the
///  signal stage,
///  so only the ids this answer does not hold are the walk's own kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = theIdsThisMachinesTasksHold('/proc');
/// ```
pub fn the_ids_this_machines_tasks_hold(
    proc_root: &Path
) -> Result<BTreeSet<u64>, TasklessWalkRefusal> {
    let mut held: BTreeSet<u64> = BTreeSet::new();
    let entries: std::fs::ReadDir = std::fs::read_dir(proc_root).map_err(|error| {
        return TasklessWalkRefusal::ATasksNamespaceWasNotReadable {
            task: proc_root.to_path_buf(),
            errno: error.raw_os_error().unwrap_or(libc::EINVAL),
        };
    })?;
    for entry in entries.flatten() {
        // What: the name of one entry in `/proc`, which is a process number for a task.
        // Why: only a task's own entry names a namespace, and every other entry is skipped.
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.parse::<i32>().is_err() {
            continue;
        }
        let task: PathBuf = entry.path();
        let link: PathBuf = task.join("ns").join("mnt");
        let namespace: std::fs::File = match open_a_namespace(link.as_path()) {
            Ok(opened) => opened,
            Err(refusal) => {
                // A task that ended between the listing and this read is gone and not blind, which is the rule the
                // scan's own reads keep.
                if refusal.errno == libc::ENOENT || refusal.errno == libc::ESRCH {
                    continue;
                }
                return Err(TasklessWalkRefusal::ATasksNamespaceWasNotReadable {
                    task,
                    errno: refusal.errno,
                });
            }
        };
        // What: the id the namespace descriptor answers.
        // Why: the listing answers ids, and this is the one call that turns a task's namespace into an id to compare.
        let mut id: u64 = 0;
        // SAFETY: the descriptor is this call's own and the answer is written into `id`, which lives across the call.
        let answered: libc::c_int =
            unsafe { libc::ioctl(namespace.as_raw_fd(), NS_GET_ID, &raw mut id) };
        if answered < 0 {
            let errno: i32 = std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EINVAL);
            // `ENOTTY` is the kernel having no namespace ids at all, which is the same answer as no `listns`.
            if errno == libc::ENOTTY || errno == libc::ENOSYS {
                return Err(TasklessWalkRefusal::TheKernelHasNoListns {
                    errno,
                });
            }
            return Err(TasklessWalkRefusal::ATasksNamespaceWasNotReadable {
                task,
                errno,
            });
        }
        held.insert(id);
    }
    return Ok(held);
}

/// What:
///  The ids the listing answered and no task holds.
/// Why:
///  That difference is the kind the design calls a taskless namespace,
///  and it is the walk's whole decision over the two sets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taskless = theIdsNoTaskHolds(listed, held);
/// ```
pub fn the_ids_no_task_holds(
    listed: &[u64],
    held: &BTreeSet<u64>,
) -> Vec<u64> {
    let mut taskless: Vec<u64> = Vec::new();
    for id in listed {
        if !held.contains(id) && !taskless.contains(id) {
            taskless.push(*id);
        }
    }
    return taskless;
}

/// What:
///  One namespace's mount table,
///  read by a child that enters it.
/// Why:
///  `/proc/self/mountinfo` is the namespace the reader is in,
///  and only a child may change its own mount namespace (`setns` in a threaded
///  process is refused `EINVAL`),
///  so the table is read through the folders' own child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = rowsOf(runIn(namespace, () => readTheTable()).table);
/// ```
pub fn the_table_of_namespace(
    identity: &str,
    namespace: &std::fs::File,
) -> Result<Vec<MountRow>, TasklessWalkRefusal> {
    let report: ActReport = run_in(namespace, || {
        return match read_the_table() {
            Ok(table) => ActReport::done(String::from("the taskless namespace's own table"), table),
            Err(error) => ActReport::failed(
                format!("reading the namespace's mount table answered {error}"),
                error.raw_os_error().unwrap_or(libc::EINVAL),
                String::new(),
            ),
        };
    })
    .map_err(|refusal| {
        return TasklessWalkRefusal::ANamespacesTableCouldNotBeRead {
            identity: String::from(identity),
            said: format!(
                "the child that entered the namespace answered {}",
                refusal.said
            ),
        };
    })?;
    if report.outcome != ActOutcome::Done {
        return Err(TasklessWalkRefusal::ANamespacesTableCouldNotBeRead {
            identity: String::from(identity),
            said: report.detail,
        });
    }
    return Ok(rows_of(report.table.as_str()));
}

/// What:
///  Every taskless mount namespace whose copy at the slot is the clone.
/// Why:
///  The walk's last step is the question the arm asks:
///  of the namespaces no task holds,
///  which ones hold this clone's copy,
///  and the answer carries the descriptor and the rows the act needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holders = findTheTasklessNamespacesHolding({ procRoot, dataset, slot });
/// holders.map((one) => one.identity);
/// ```
pub fn find_the_taskless_namespaces_holding(
    proc_root: &Path,
    dataset: &str,
    slot: &Path,
) -> Result<Vec<TasklessNamespace>, TasklessWalkRefusal> {
    // What: the listing, with a kernel that has no such call told apart from one that refused it.
    // Why: the design answers `holder-unendable` to the first and `holder-blind` to the second.
    let listed: Vec<u64> = the_mount_namespaces_on_this_machine()
        .map_err(|refusal| return the_walks_refusal_of_the_call(refusal))?;
    let held: BTreeSet<u64> = the_ids_this_machines_tasks_hold(proc_root)?;
    let taskless: Vec<u64> = the_ids_no_task_holds(listed.as_slice(), &held);
    if taskless.is_empty() {
        return Ok(Vec::new());
    }
    // What: a descriptor on `nsfs`, which is the file system every namespace descriptor is on.
    // Why: `open_by_handle_at` needs one descriptor on the file system its handle names.
    let nsfs: std::fs::File =
        open_a_namespace(Path::new(OUR_MOUNT_NAMESPACE)).map_err(|refusal| {
            return TasklessWalkRefusal::ANamespaceCouldNotBeOpened {
                id: 0,
                errno: refusal.errno,
            };
        })?;
    let mut holders: Vec<TasklessNamespace> = Vec::new();
    for id in taskless {
        let namespace: std::fs::File = open_by_the_id(&nsfs, id).map_err(|refusal| {
            return TasklessWalkRefusal::ANamespaceCouldNotBeOpened {
                id,
                errno: refusal.errno,
            };
        })?;
        let identity: String = identity_of(&namespace).map_err(|refusal| {
            return TasklessWalkRefusal::ANamespaceCouldNotBeOpened {
                id,
                errno: refusal.errno,
            };
        })?;
        let rows: Vec<MountRow> = the_table_of_namespace(identity.as_str(), &namespace)?;
        let Some(copy) = topmost_at(rows.as_slice(), slot) else {
            continue;
        };
        if copy.filesystem != "zfs" || copy.source != dataset {
            continue;
        }
        holders.push(TasklessNamespace {
            id,
            identity,
            namespace,
            rows,
        });
    }
    return Ok(holders);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "taskless_namespace_walk_tests.rs"]
mod tests;
