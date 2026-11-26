//! What:
//!  The mount-namespace read two of the wrapper's scans share:
//!  every live task grouped by the kernel's own namespace identity,
//!  one `mountinfo` read per distinct namespace,
//!  and the match that says which mounts are mounts of one exact ZFS source.
//! Why:
//!  The start probe's arm 7 and the root handler's holder scan ask the same
//!  question of the same kernel files,
//!  and a second scanner of the same mechanism would answer it twice and
//!  disagree once
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms",
//!  which proposes that plan step 9 reuse the probe's grouping and source match;
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 9).
//!  Neither caller's own policy lives here:
//!  the probe refuses a table it may not read and compares the namespaces it
//!  found with the ones it made,
//!  the holder scan reports every incomplete read and every holder,
//!  and both read the answer this folder gives them.
//!  Reading a `/proc` root the caller names is what lets a control read a
//!  fixture tree,
//!  so no control on a person's machine reads another account's processes.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `mount_namespace_tasks.rs`:
//!   one `/proc` root's live threads grouped by `ns/mnt`,
//!   each thread's root,
//!   the links it may not read and the listing's refusal.
//! - `namespace_table_reads.rs`:
//!   one namespace's table read,
//!   one task at a time,
//!   with each attempt and the two kinds of failure kept.
//! - `zfs_source_mounts.rs`:
//!   the exact ZFS source match both callers use.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listing = threadsByMountNamespace({ procRoot: '/proc' });
//! const read = readOneNamespace(listing.namespaces[0], undefined);
//! const held = mountsOfSource(read.table()?.mounts ?? [], source);
//! ```

/// The live threads of one `/proc` root,
/// grouped by mount namespace,
/// with their roots and the links this reader may not read.
pub mod mount_namespace_tasks;

/// One namespace's table read and the attempts it took.
pub mod namespace_table_reads;

/// The exact ZFS source match.
pub mod zfs_source_mounts;

/// The thread grouping,
/// its identity and root reads and its refusal.
pub use mount_namespace_tasks::{
    ListingRefusal,
    ListingRequest,
    MountNamespaceTasks,
    NamespaceListing,
    ROOT_LINK,
    RootAt,
    RootIdentity,
    THREADS_FOLDER,
    TaskLink,
    ThreadAt,
    UnreadableLink,
    is_a_thread_that_ended,
    namespace_target,
    task_link,
    threads_by_mount_namespace,
};
/// One namespace's table read and what each attempt answered.
pub use namespace_table_reads::{
    MOUNT_TABLE_FILE,
    MOUNT_TABLE_LIMIT,
    NamespaceTableRead,
    TaskTableAttempt,
    read_one_namespace,
};
/// The exact source match.
pub use zfs_source_mounts::{
    is_of_source,
    mounts_of_source,
};
