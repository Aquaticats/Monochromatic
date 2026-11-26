//! What:
//!  Every live thread of one `/proc` root,
//!  grouped by the mount namespace the kernel says the thread is in,
//!  with each thread's own root where it could be read,
//!  the links this reader may not read,
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
//!  the kernel's own `mnt:[...]` text,
//!  read once per **thread** and never
//!  guessed from a pid,
//!  so two threads of one namespace are one namespace and one table is read for
//!  both,
//!  and a thread that unshared a namespace of its own is its own group
//!  ([`THREADS_FOLDER`],
//!  and `unshare(2)` on the mount namespace).
//!  A process whose leader has ended while a thread lives is read through that
//!  thread,
//!  because every thread's link is read and the leader's ending is not the
//!  process's.
//!  A link that answers an error other than a thread's ending is kept and
//!  named,
//!  never skipped:
//!  a handler without `CAP_SYS_PTRACE` may read no other account's link,
//!  and a scan that dropped those threads would miss a holder and say nothing.
//!  Each thread's root is read too,
//!  because a table is rendered from the root of the task it is read through:
//!  a task chrooted away from a mount renders a table without it,
//!  so each namespace's own root is worked out from its threads and a table is
//!  read through a thread at that root
//!  (`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`,
//!  "Whether the scan reads every thread and every root").
//!  Reading tasks is done from a `/proc` root the caller names,
//!  so a control reads a fixture tree and never this machine's processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listing = threadsByMountNamespace({ procRoot: '/proc', readersRoot });
//! listing.namespaces[0].threads[0].atTheReadersRoot; // false for a container's task
//! ```

use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

/// The folder below a task that holds one folder per thread.
pub const THREADS_FOLDER: &str = "task";

/// The link whose target is a thread's mount-namespace identity.
pub const MOUNT_NAMESPACE_LINK: [&str; 2] = ["ns", "mnt"];

/// The link to a thread's own root directory.
pub const ROOT_LINK: &str = "root";

/// What:
///  One directory as the kernel identifies it:
///  the device and inode two links are compared by.
/// Why:
///   A task's root is a directory and not a path,
///  and two paths that name one directory can be spelled differently,
///  so the comparison is the kernel's own pair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootIdentity = { device: bigint; inode: bigint };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RootIdentity {
    /// The device the directory is on.
    pub device: u64,
    /// The inode the directory has.
    pub inode: u64,
}

impl RootIdentity {
    /// The identity of one directory a link names,
    /// or nothing when it cannot be read.
    pub fn of(path: &Path) -> Option<Self> {
        let fields = std::fs::metadata(path).ok()?;
        return Some(RootIdentity {
            device: fields.dev(),
            inode: fields.ino(),
        });
    }
}

/// What:
///  How one thread's root was read.
/// Why:
///   A root this reader may not read is not the same as a root that differs:
///  the first says nothing and the second says the thread renders its mount
///  table from another root,
///  so the two are separate answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootAt = { kind: 'known'; root: RootIdentity } | { kind: 'unknown'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RootAt {
    /// The root directory,
    /// as the kernel identifies it.
    Known(RootIdentity),
    /// A root this reader may not read,
    /// and why.
    Unknown {
        /// What the read answered.
        reason: String,
    },
}

/// What:
///  One thread of one task,
///  and the root its mount table would be rendered
///  from.
/// Why:
///   The thread is the path a table is read through,
///  so it travels with the group it was put in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ThreadAt = { thread: string; root: RootAt };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadAt {
    /// The thread's own folder,
    /// such as `/proc/11/task/12`.
    pub thread: PathBuf,
    /// Its root,
    /// and whether this reader could read it.
    pub root: RootAt,
}

impl ThreadAt {
    /// Whether this thread's table is rendered from the root the reader
    /// compares against.
    pub fn is_at_the_readers_root(
        &self,
        readers_root: Option<RootIdentity>,
    ) -> bool {
        let (Some(wanted), RootAt::Known(found)) = (readers_root, &self.root) else {
            return false;
        };
        return *found == wanted;
    }
}

/// What:
///  One mount namespace and every live thread of it,
///  in the order the kernel lists the threads that answered.
/// Why:
///   The threads are kept so a table read can try them in the order that finds
///  a mount above another task's root,
///  and so a report can name the thread whose table it read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountNamespaceTasks = { namespace: string; threads: ThreadAt[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountNamespaceTasks {
    /// The namespace's own identity,
    /// such as `mnt:[4026531840]`.
    pub namespace: String,
    /// One entry per live thread of it,
    /// in the kernel's own order.
    pub threads: Vec<ThreadAt>,
}

/// What:
///  Which root one mount namespace renders its tables from.
/// Why:
///  A table is rendered from the root of the thread it is read through,
///  so the root to read at is the namespace's own and not any thread's:
///  a namespace made by `unshare(2)` inherits the reader's root,
///  a container's is the one all of its tasks share,
///  and a namespace whose tasks disagree while none is at the reader's root
///  has no root this reader can name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceRoot = { kind: 'readers' } | { kind: 'shared'; root: RootIdentity } | { kind: 'unknown' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamespaceRoot {
    /// The reader's own root,
    /// which a namespace made after the reader's
    /// inherits.
    Readers,
    /// One root every thread of the namespace shares.
    Shared(RootIdentity),
    /// Its threads disagree about their root and none is at the reader's,
    /// so no table can be shown to be the namespace's.
    Unknown,
}

impl MountNamespaceTasks {
    /// The root this namespace's tables are rendered from.
    /// Why:
    ///  The reader's own root wins when a thread is at it,
    ///  because a mount namespace inherits the root it was made from;
    ///  otherwise a root every thread shares is the namespace's own,
    ///  which is what a container's namespace looks like;
    ///  and a namespace whose threads disagree,
    ///  none of them at the reader's root,
    ///  has none this reader can name.
    pub fn namespace_root(
        &self,
        readers_root: Option<RootIdentity>,
    ) -> NamespaceRoot {
        if self.has_a_thread_at(readers_root) {
            return NamespaceRoot::Readers;
        }
        let mut shared: Option<RootIdentity> = None;
        for thread in &self.threads {
            let RootAt::Known(found) = thread.root else {
                return NamespaceRoot::Unknown;
            };
            match shared {
                None => shared = Some(found),
                Some(first) if first == found => continue,
                Some(_other) => return NamespaceRoot::Unknown,
            }
        }
        return shared.map_or(NamespaceRoot::Unknown, NamespaceRoot::Shared);
    }

    /// Whether one thread is at the root this namespace renders from.
    fn is_at_the_namespace_root(
        thread: &ThreadAt,
        root: NamespaceRoot,
        readers_root: Option<RootIdentity>,
    ) -> bool {
        return match (&thread.root, root) {
            (RootAt::Known(found), NamespaceRoot::Shared(wanted)) => *found == wanted,
            (RootAt::Known(_found), NamespaceRoot::Readers) => {
                thread.is_at_the_readers_root(readers_root)
            }
            (RootAt::Known(_found), NamespaceRoot::Unknown) => false,
            (
                RootAt::Unknown {
                    ..
                },
                NamespaceRoot::Readers,
            ) => false,
            (
                RootAt::Unknown {
                    ..
                },
                NamespaceRoot::Shared(_wanted),
            ) => false,
            (
                RootAt::Unknown {
                    ..
                },
                NamespaceRoot::Unknown,
            ) => false,
        };
    }

    /// The threads to try,
    /// in the order that is most likely to find a mount:
    /// first the ones at the namespace's own root,
    /// then the ones whose root could be read,
    /// then the ones whose root could not.
    /// Why:
    ///  A thread at the namespace's root sees every mount a chrooted one
    ///  cannot,
    ///  and a thread whose root could not be read may be anywhere,
    ///  so it is tried last.
    pub fn threads_in_reading_order(
        &self,
        readers_root: Option<RootIdentity>,
    ) -> Vec<&ThreadAt> {
        let root: NamespaceRoot = self.namespace_root(readers_root);
        let mut at_the_root: Vec<&ThreadAt> = Vec::with_capacity(self.threads.len());
        let mut at_another_root: Vec<&ThreadAt> = Vec::new();
        let mut at_no_readable_root: Vec<&ThreadAt> = Vec::new();
        for thread in &self.threads {
            if Self::is_at_the_namespace_root(thread, root, readers_root) {
                at_the_root.push(thread);
            } else if matches!(thread.root, RootAt::Known(_)) {
                at_another_root.push(thread);
            } else {
                at_no_readable_root.push(thread);
            }
        }
        at_the_root.extend(at_another_root);
        at_the_root.extend(at_no_readable_root);
        return at_the_root;
    }

    /// Whether any thread of this namespace renders its table from the reader's
    /// root.
    /// Why:
    ///  A namespace whose every thread is chrooted away cannot be shown to hold
    ///  nothing:
    ///  the table read through one of them omits everything above its root.
    pub fn has_a_thread_at(
        &self,
        readers_root: Option<RootIdentity>,
    ) -> bool {
        return self
            .threads
            .iter()
            .any(|thread| return thread.is_at_the_readers_root(readers_root));
    }
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
///  One link this reader could not read,
///  and why.
/// Why:
///  A link the kernel refuses to show says the thread is there and its
///  namespace is unknown,
///  which is a holder the scan cannot rule out;
///  naming it keeps a reduced capability set from turning into a silent clear
///  answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnreadableLink = { thread: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnreadableLink {
    /// The thread whose link it was.
    pub thread: PathBuf,
    /// What the read answered.
    pub reason: String,
}

/// What:
///  What reading one thread's namespace link answered.
/// Why:
///  The three answers mean different things to a caller:
///  a namespace to group under,
///  a thread that ended between the listing and the read,
///  and a refusal the caller reports rather than dropping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TaskLink = { kind: 'namespace'; namespace: string } | { kind: 'ended' } | { kind: 'unreadable'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskLink {
    /// The thread's namespace identity.
    Namespace(String),
    /// The thread ended between the listing and this read.
    Ended,
    /// The link could not be read for a reason the thread's ending does not
    /// explain.
    Unreadable {
        /// What the read answered.
        reason: String,
    },
}

/// What:
///  Every namespace one `/proc` root holds,
///  and every link this reader could not read.
/// Why:
///  The two travel together so a caller cannot report the namespaces it read
///  and forget the threads it did not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceListing = { namespaces: MountNamespaceTasks[]; unreadable: UnreadableLink[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceListing {
    /// One entry per namespace,
    /// in the kernel's order of the identities.
    pub namespaces: Vec<MountNamespaceTasks>,
    /// One entry per link this reader could not read.
    pub unreadable: Vec<UnreadableLink>,
}

/// Whether one error is the thread having ended between the listing and the
/// read:
/// `ENOENT` for a thread that is gone,
/// `ESRCH` and `EINVAL` for one the kernel
/// is taking down.
/// Why:
///  Every other error is a refusal this reader must report,
///  so the three that mean "it ended" are written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = (errno) => errno === 'ENOENT' || errno === 'ESRCH' || errno === 'EINVAL';
/// ```
pub fn is_a_thread_that_ended(error: &std::io::Error) -> bool {
    return matches!(
        error.raw_os_error(),
        Some(libc::ENOENT) | Some(libc::ESRCH) | Some(libc::EINVAL)
    );
}

/// What:
///  What one `ns/mnt` link says about the thread it belongs to.
/// Why:
///   The identity is the kernel's own text and never a count this code keeps,
///  and the two failures are kept apart because only one of them may be
///  dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const link = taskLink('/proc/11/task/12'); // { kind: 'namespace', namespace: 'mnt:[…]' }
/// ```
pub fn task_link(thread: &Path) -> TaskLink {
    let link: PathBuf = thread
        .join(MOUNT_NAMESPACE_LINK[0])
        .join(MOUNT_NAMESPACE_LINK[1]);
    return match std::fs::read_link(link.as_path()) {
        Ok(target) => TaskLink::Namespace(target.to_string_lossy().into_owned()),
        Err(error) if is_a_thread_that_ended(&error) => TaskLink::Ended,
        Err(error) => TaskLink::Unreadable {
            reason: error.to_string(),
        },
    };
}

/// What:
///  The namespace identity one link names,
///  or nothing when it names none.
/// Why:
///  A caller that wants the reader's own namespace names one link and wants
///  only its target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mine = namespace_target('/proc/self/ns/mnt');
/// ```
pub fn namespace_target(link: &Path) -> Option<String> {
    return std::fs::read_link(link)
        .ok()
        .map(|target| return target.to_string_lossy().into_owned());
}

/// What:
///  One `/proc` root and the root to compare each thread's against.
/// Why:
///   The reader's own root is a directory and not a path,
///  and a fixture tree has none,
///  so the caller hands in the identity it read and the listing compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ListingRequest = { procRoot: string; readersRoot?: RootIdentity };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ListingRequest<'a> {
    /// The `/proc` root to read.
    pub proc_root: &'a Path,
    /// The reader's own root,
    /// or nothing when it could not be read.
    pub readers_root: Option<RootIdentity>,
}

/// What:
///  Read every live thread's namespace under one `/proc` root and group them,
///  keeping every link it could not read.
/// Why:
///   Grouping before any table is read keeps the read to one table per
///  namespace however many threads share it,
///  which is what makes the scan's cost follow the namespaces rather than the
///  thread count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listing = threadsByMountNamespace({ procRoot: '/proc' });
/// ```
///
/// # Errors
/// The `/proc` root could not be listed.
pub fn threads_by_mount_namespace(
    request: &ListingRequest<'_>
) -> Result<NamespaceListing, ListingRefusal> {
    let listing: std::fs::ReadDir = match std::fs::read_dir(request.proc_root) {
        Ok(iter) => iter,
        Err(error) => {
            return Err(ListingRefusal {
                path: request.proc_root.to_path_buf(),
                reason: error.to_string(),
            });
        }
    };
    let mut grouped: std::collections::BTreeMap<String, Vec<(u32, ThreadAt)>> =
        std::collections::BTreeMap::new();
    let mut unreadable: Vec<UnreadableLink> = Vec::new();
    for entry in listing.filter_map(|entry| return entry.ok()) {
        // One guard reads the name: a `/proc` entry that is no pid is no task, and a number too large for a pid
        // is none either.
        if entry.file_name().to_string_lossy().parse::<u32>().is_err() {
            continue;
        }
        let mut threads: std::fs::ReadDir =
            match std::fs::read_dir(entry.path().join(THREADS_FOLDER)) {
                Ok(iter) => iter,
                // The process ended between the listing and its thread folder, which is a task that ended.
                Err(_ended) => continue,
            };
        loop {
            let Some(thread_entry) = threads.next() else {
                break;
            };
            let Ok(listed) = thread_entry else {
                break;
            };
            let Ok(tid) = listed.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            let thread: PathBuf = listed.path();
            match task_link(thread.as_path()) {
                TaskLink::Namespace(namespace) => {
                    let root: RootAt = match RootIdentity::of(thread.join(ROOT_LINK).as_path()) {
                        Some(identity) => RootAt::Known(identity),
                        None => RootAt::Unknown {
                            reason: String::from("its root could not be read"),
                        },
                    };
                    grouped.entry(namespace).or_default().push((
                        tid,
                        ThreadAt {
                            thread,
                            root,
                        },
                    ));
                }
                TaskLink::Ended => continue,
                TaskLink::Unreadable {
                    reason,
                } => unreadable.push(UnreadableLink {
                    thread,
                    reason,
                }),
            }
        }
    }
    let mut namespaces: Vec<MountNamespaceTasks> = Vec::with_capacity(grouped.len());
    for (namespace, mut threads) in grouped {
        threads.sort_by_key(|(tid, _thread)| return *tid);
        namespaces.push(MountNamespaceTasks {
            namespace,
            threads: threads
                .into_iter()
                .map(|(_tid, thread)| return thread)
                .collect(),
        });
    }
    return Ok(NamespaceListing {
        namespaces,
        unreadable,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_namespace_tasks_tests.rs"]
mod tests;
