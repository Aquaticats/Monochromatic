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
use std::collections::BTreeSet;
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The `listns(2)` number this build may call,
///  or nothing on an architecture whose table this build does not carry.
/// Why:
///  The crate's C library bindings stop at syscall 462,
///  so the number is this build's own,
///  and a build for another architecture must refuse rather than call a
///  number that means something else there.
/// Gotcha:
///  Syscall numbers are per architecture:
///  470 is `listns` on x86-64 and is no promise of anything elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LISTNS_SYSCALL: number | undefined = 470;
/// ```
pub const LISTNS_SYSCALL: Option<libc::c_long> = if cfg!(target_arch = "x86_64") {
    Some(470)
} else {
    None
};

/// What:
///  The bit `_IOC_READ` sets in an ioctl number,
///  which says the call writes its answer into the caller's buffer.
/// Why:
///  `NS_GET_ID` is written as `_IOR(NSIO, 13, __u64)` in
///  `include/uapi/linux/nsfs.h`,
///  and this build carries no binding for that macro's result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IOCTL_READING = 2;
/// ```
pub const IOCTL_READING: libc::c_ulong = 2;

/// What:
///  The ioctl group `NSIO` that `include/uapi/linux/nsfs.h` gives the
///  namespace calls.
/// Why:
///  The group is part of every namespace ioctl's number,
///  and `NS_GET_ID` is built here the way that header's `_IOR` builds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NSIO = 0xb7;
/// ```
pub const NSIO: libc::c_ulong = 0xb7;

/// What:
///  The number `NS_GET_ID` carries inside its group.
/// Why:
///  It is the ioctl that answers a namespace descriptor's own id,
///  which is the only way this build turns a task's `/proc` link into the id
///  `listns` lists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NS_GET_ID_NUMBER = 13;
/// ```
pub const NS_GET_ID_NUMBER: libc::c_ulong = 13;

/// What:
///  The width of the `__u64` `NS_GET_ID` answers.
/// Why:
///  `_IOR` folds the answer's width into the ioctl number,
///  so the number is wrong without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NS_GET_ID_WIDTH = 8;
/// ```
pub const NS_GET_ID_WIDTH: libc::c_ulong = 8;

/// What:
///  `NS_GET_ID`'s ioctl number,
///  built the way `include/uapi/asm-generic/ioctl.h`'s `_IOR` builds it.
/// Why:
///  The walk asks every task's namespace for its id with this call,
///  and a mis-built number would answer `ENOTTY` and look like a kernel
///  without namespace ids.
///  The control beside this file checks the same arithmetic against the C
///  library's own `NS_GET_USERNS`,
///  which is built by the same macro.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NS_GET_ID = (IOCTL_READING << 30) | (NS_GET_ID_WIDTH << 16) | (NSIO << 8) | NS_GET_ID_NUMBER;
/// ```
pub const NS_GET_ID: libc::c_ulong =
    (IOCTL_READING << 30) | (NS_GET_ID_WIDTH << 16) | (NSIO << 8) | NS_GET_ID_NUMBER;

/// What:
///  The `ns_type` `listns(2)` filters on for mount namespaces,
///  which is `MNT_NS` in `include/uapi/linux/nsfs.h`.
/// Why:
///  The walk asks only for mount namespaces,
///  because only a mount namespace holds a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MNT_NAMESPACE_TYPE = 1 << 17;
/// ```
pub const MNT_NAMESPACE_TYPE: u64 = 1 << 17;

/// What:
///  The `user_ns_id` that asks `listns(2)` for the caller's own user
///  namespace's namespaces,
///  which is `LISTNS_CURRENT_USER` in that header.
/// Why:
///  The handler runs in the initial user namespace,
///  so this asks for every mount namespace a task of this machine could hold,
///  and asks for no namespace of another user namespace
///  (a caller without the initial user namespace's `CAP_SYS_ADMIN` is given
///  only its own namespaces whatever it asks for).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LISTNS_CURRENT_USER = (1n << 64n) - 1n;
/// ```
pub const LISTNS_CURRENT_USER: u64 = u64::MAX;

/// What:
///  The width of `struct ns_id_req`, which is `NS_ID_REQ_SIZE_VER0` in
///  `include/uapi/linux/nsfs.h`.
/// Why:
///  The kernel reads the request's own size first,
///  so a request that names a size it does not have would be read past its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NS_ID_REQUEST_BYTES = 32;
/// ```
pub const NS_ID_REQUEST_BYTES: u32 = 32;

/// What:
///  How many namespace ids one `listns(2)` call is given room for.
/// Why:
///  The first version of the call fills this many and the walk asks again from
///  the last id,
///  so one page covers the eleven mount namespaces the design's own guest held
///  (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
///  "T0-10 with the handler's privileges")
///  and the call after it is the one that answers `ENOENT`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IDS_ONE_CALL_ASKS_FOR = 64;
/// ```
pub const IDS_ONE_CALL_ASKS_FOR: usize = 64;

/// What:
///  The width of `struct nsfs_file_handle`,
///  which is `NSFS_FILE_HANDLE_SIZE_VER0` in `include/uapi/linux/nsfs.h`.
/// Why:
///  The kernel reads a handle whose own byte count matches the version it
///  knows,
///  and this build writes that one version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NSFS_FILE_HANDLE_BYTES = 16;
/// ```
pub const NSFS_FILE_HANDLE_BYTES: usize = 16;

/// What:
///  The handle type `nsfs`'s own encode answers,
///  which is `FILEID_NSFS` in `include/linux/exportfs.h`.
/// Why:
///  `open_by_handle_at` hands the handle to the file system's own decoder,
///  and a handle of another type is another file system's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FILEID_NSFS = 0xf1;
/// ```
pub const FILEID_NSFS: libc::c_int = 0xf1;

/// What:
///  Where this machine's mount namespaces' own descriptor type lives,
///  which is the file a namespace descriptor may be opened from.
/// Why:
///  `open_by_handle_at` needs a descriptor on the file system the handle names,
///  and any open namespace file is on `nsfs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OUR_MOUNT_NAMESPACE = '/proc/self/ns/mnt';
/// ```
pub const OUR_MOUNT_NAMESPACE: &str = "/proc/self/ns/mnt";

/// What:
///  The request one `listns(2)` call reads,
///  which is `struct ns_id_req` of `include/uapi/linux/nsfs.h`.
/// Why:
///  The call reads its own size first,
///  then the cursor it lists after,
///  then which kind of namespace it is asked for and whose user namespace's,
///  so one record carries the whole of what the call is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRequest = { size: number; spare: number; cursor: bigint; nsType: number; spare2: number; userNsId: bigint };
/// ```
#[repr(C)]
struct TheRequest {
    /// What:
    ///  The width of this record,
    ///  which is `NS_ID_REQ_SIZE_VER0`.
    /// Why:
    ///  The kernel refuses a request whose stated width is not a version it knows,
    ///  and reads no further into a wider one.
    size: u32,
    /// What:
    ///  The word the header reserves for a later version.
    /// Why:
    ///  The kernel reads it and no version this build knows uses it,
    ///  so it is zero.
    spare: u32,
    /// What:
    ///  The id the listing starts after,
    ///  which is zero for the first page.
    /// Why:
    ///  The call takes the last id of one page as the next call's start,
    ///  which is how the walk reads a list longer than one page.
    cursor: u64,
    /// What:
    ///  Which kind of namespace is asked for,
    ///  as `MNT_NAMESPACE_TYPE`.
    /// Why:
    ///  Only a mount namespace holds a copy,
    ///  and the call lists the kinds a caller names.
    ns_type: u32,
    /// What:
    ///  The word the header reserves inside the filter.
    /// Why:
    ///  Nothing this build knows reads it, so it is zero.
    spare2: u32,
    /// What:
    ///  Whose user namespace's namespaces are asked for,
    ///  as `LISTNS_CURRENT_USER`.
    /// Why:
    ///  The handler asks for the initial user namespace's,
    ///  which is where every mount namespace a task of this machine could hold lives.
    user_ns_id: u64,
}

/// What:
///  The file handle one `listns(2)` id becomes a namespace descriptor through,
///  in the three parts the kernel's own structure holds.
/// Why:
///  The kernel reads the id and the type from these bytes,
///  and the byte count and the handle type come before them in the same
///  record,
///  so one value carries the whole of what the call is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheHandle = { handleBytes: number; handleType: number; id: bigint; nsType: number; nsInum: number };
/// ```
#[repr(C)]
struct TheHandle {
    /// What:
    ///  How many bytes of the handle the kernel may read,
    ///  which is the id and the two fields after it.
    /// Why:
    ///  The kernel refuses a byte count it does not know,
    ///  and this build writes the one version it was measured with.
    handle_bytes: u32,
    /// What:
    ///  Which file system's decoder reads the bytes,
    ///  as `FILEID_NSFS`.
    /// Why:
    ///  A handle carries no file system name of its own,
    ///  so the type is what sends it to `nsfs`.
    handle_type: libc::c_int,
    /// What:
    ///  `struct nsfs_file_handle`: the namespace's id,
    ///  its type and its inode number,
    ///  as the kernel writes them.
    /// Why:
    ///  The measured construction put the id in the first eight bytes and left
    ///  the two leaves zero,
    ///  and it answered the descriptor of the namespace that id names.
    /// Gotcha:
    ///  The two leaves are what the kernel fills when it encodes a handle:
    ///  this build writes zeroes there because the id alone was measured to
    ///  answer,
    ///  and the guest arm re-measures that on the template's own kernel.
    f_handle: [u8; NSFS_FILE_HANDLE_BYTES],
}

/// What:
///  One page of `listns(2)`'s answer:
///  the ids it filled and the errno the call answered.
/// Why:
///  The paging rule is the walk's one piece of real logic,
///  and a control makes pages of its own rather than needing a kernel to
///  answer them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PageAnswer = { ids: bigint[]; errno?: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageAnswer {
    /// The ids this page filled, in the kernel's own order.
    pub ids: Vec<u64>,
    /// The errno the call answered,
    /// or nothing when it filled the page it was given.
    pub errno: Option<i32>,
}

/// What:
///  Why the walk could not answer,
///  and which of the design's refusal words that is.
/// Why:
///  `ENOSYS` is the kernel having no such call at all,
///  which is `holder-unendable`;
///  every other refusal is a read the act could not make,
///  which is `holder-blind` and stops the removal before any act.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TasklessWalkRefusal =
///   | { kind: 'no-listns'; errno }
///   | { kind: 'listing-refused'; errno }
///   | { kind: 'task-unreadable'; task; errno }
///   | { kind: 'namespace-unopened'; id; errno }
///   | { kind: 'table-unread'; identity; said };
/// ```
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
///  Walk the listing one page at a time until the kernel says the list ended,
///  and answer every id it gave.
/// Why:
///  The call fills a page and takes the last id of that page as its next
///  cursor,
///  so the walk is one page after another until `ENOENT`,
///  and a page that adds no new id would loop forever if the walk kept asking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ids = pagesUntilTheEnd((cursor) => theCall(cursor));   // stops at ENOENT, or when a page adds nothing
/// ```
pub fn pages_until_the_end<Page>(mut ask: Page) -> Result<Vec<u64>, TasklessWalkRefusal>
where
    Page: FnMut(Option<u64>) -> PageAnswer,
{
    // What: the ids every page answered, in the order the kernel gave them.
    // Why: the walk's answer is this list, and the cursor is its last id.
    let mut listed: Vec<u64> = Vec::new();
    // What: the id the next call starts after, which the first call has none of.
    // Why: the call takes the last id it listed as its cursor, and zero asks for the first page.
    let mut cursor: Option<u64> = None;
    loop {
        let page: PageAnswer = ask(cursor);
        // What: how many ids the walk held before this page.
        // Why: a page that adds none is a kernel the walk cannot progress on, and asking again would never end.
        let before: usize = listed.len();
        listed.extend(page.ids);
        if let Some(errno) = page.errno {
            // The end of the list is how the call answers a cursor past the last id, and it is not a failure.
            if errno == libc::ENOENT {
                return Ok(listed);
            }
            if errno == libc::ENOSYS {
                return Err(TasklessWalkRefusal::TheKernelHasNoListns {
                    errno,
                });
            }
            return Err(TasklessWalkRefusal::TheListingWasRefused {
                errno,
            });
        }
        if listed.len() == before {
            // A page that filled nothing and answered nothing has ended the list in the only way a kernel that
            // never says `ENOENT` can, and asking again would not move.
            return Ok(listed);
        }
        cursor = listed.last().copied();
    }
}

/// What:
///  Every mount namespace this machine has, as `listns` lists them.
/// Why:
///  The walk's first half is the kernel's own list,
///  and a caller that meets `ENOSYS` here learns this kernel cannot answer for
///  a namespace no task holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ids = theMountNamespacesOnThisMachine();
/// ```
pub fn the_mount_namespaces_on_this_machine() -> Result<Vec<u64>, TasklessWalkRefusal> {
    // What: the syscall number this build may call, or nothing on an architecture whose table it does not carry.
    // Why: a build that called another architecture's number 470 would make a different call entirely.
    let Some(number) = LISTNS_SYSCALL else {
        return Err(TasklessWalkRefusal::TheKernelHasNoListns {
            errno: libc::ENOSYS,
        });
    };
    return pages_until_the_end(|cursor| {
        // What: the request the call reads: its own size, the cursor, the kind of namespace and whose.
        // Why: the kernel reads the size first and refuses a request it does not know.
        let mut request: TheRequest = TheRequest {
            size: NS_ID_REQUEST_BYTES,
            spare: 0,
            // What: the id the call lists after, which is zero for the first page.
            // Why: the call takes the cursor as the listing's start, and zero starts at the first id.
            cursor: cursor.unwrap_or(0),
            ns_type: MNT_NAMESPACE_TYPE as u32,
            spare2: 0,
            user_ns_id: LISTNS_CURRENT_USER,
        };
        // What: the ids the call fills, one page of them.
        // Why: the call writes at most this many and answers how many it wrote.
        let mut ids: Vec<u64> = vec![0; IDS_ONE_CALL_ASKS_FOR];
        // SAFETY: both buffers are this call's own and live across it, the request states its own size, and the call
        // writes no more than the count it is given.
        let answered: libc::c_long = unsafe {
            libc::syscall(
                number,
                (&raw mut request).cast::<libc::c_void>(),
                ids.as_mut_ptr(),
                IDS_ONE_CALL_ASKS_FOR,
                0,
            )
        };
        if answered < 0 {
            let errno: i32 = std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EINVAL);
            return PageAnswer {
                ids: Vec::new(),
                errno: Some(errno),
            };
        }
        // What: how many ids the call says it wrote.
        // Why: the call answers a count, and a count past the room it was given is the call's error rather than ours.
        let filled: usize = usize::try_from(answered)
            .unwrap_or(0)
            .min(IDS_ONE_CALL_ASKS_FOR);
        ids.truncate(filled);
        return PageAnswer {
            ids,
            errno: None,
        };
    });
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
///  The descriptor one listed id names.
/// Why:
///  `setns` takes a descriptor and a taskless namespace has no `/proc` path,
///  so the id becomes one through the handle `nsfs` decodes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namespace = openByTheId(nsfsRoot, id);
/// ```
pub fn open_by_the_id(
    nsfs_root: &std::fs::File,
    id: u64,
) -> Result<std::fs::File, TasklessWalkRefusal> {
    // What: the handle the call decodes: the id in its first eight bytes and zeroes after it.
    // Why: the measured construction answered the descriptor of the namespace the id named.
    let mut handle: TheHandle = TheHandle {
        handle_bytes: NSFS_FILE_HANDLE_BYTES as u32,
        handle_type: FILEID_NSFS,
        f_handle: [0; NSFS_FILE_HANDLE_BYTES],
    };
    handle.f_handle[0..8].copy_from_slice(&id.to_ne_bytes());
    // SAFETY: `handle` is this call's own record and its bytes outlive the call, the kernel reads at most the count
    // the record states, and the descriptor it answers is owned exactly once, by the `File` below.
    let descriptor: libc::c_int = unsafe {
        libc::open_by_handle_at(
            nsfs_root.as_raw_fd(),
            (&raw mut handle).cast::<libc::file_handle>(),
            libc::O_RDONLY,
        )
    };
    if descriptor < 0 {
        return Err(TasklessWalkRefusal::ANamespaceCouldNotBeOpened {
            id,
            errno: std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EINVAL),
        });
    }
    // What: the descriptor as a value that closes it, which is the same bytes as the number the call answered.
    // Why: a bare number would leak the namespace and every mount in it, which is the one thing this act must not do.
    return Ok(std::fs::File::from(unsafe {
        OwnedFd::from_raw_fd(descriptor)
    }));
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
    let listed: Vec<u64> = the_mount_namespaces_on_this_machine()?;
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
        let namespace: std::fs::File = open_by_the_id(&nsfs, id)?;
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
