//! What:
//!  The kernel's own namespace calls this build makes for the walk:
//!  the syscall and ioctl numbers,
//!  the two records the calls read and answer,
//!  one page of the listing and the rule that walks the pages,
//!  the listing itself,
//!  and one listed id as a namespace descriptor.
//! Why:
//!  `listns(2)` is newer than the C library bindings this crate links,
//!  so its number and its records are written here from the kernel's own
//!  headers
//!  (`include/uapi/linux/nsfs.h`,
//!  `include/linux/exportfs.h` and
//!  `include/uapi/asm-generic/ioctl.h`),
//!  and a call whose numbers were guessed would answer `ENOTTY` or `ESTALE` and
//!  look like a kernel that cannot list namespaces.
//!  Nothing here decides what the walk does with the answer:
//!  this file makes the calls and says which errno each answered,
//!  and `taskless_namespace_walk.rs` reads the design's refusal out of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ids = theMountNamespacesOnThisMachine();   // a refusal carries the errno the call answered
//! const namespace = openByTheId(nsfs, ids[0]);
//! ```

use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
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
///  The width of `struct ns_id_req`,
///  which is `NS_ID_REQ_SIZE_VER0` in
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
    ///  Nothing this build knows reads it,
    ///  so it is zero.
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
    ///  `struct nsfs_file_handle`:
    ///  the namespace's id,
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
    /// The ids this page filled,
    /// in the kernel's own order.
    pub ids: Vec<u64>,
    /// The errno the call answered,
    /// or nothing when it filled the page it was given.
    pub errno: Option<i32>,
}

/// What:
///  A namespace call the kernel refused,
///  named by the call and the errno it answered.
/// Why:
///  The three calls this build makes fail for reasons the design reads as two
///  different refusals,
///  and the errno is what decides which,
///  so the call and its errno travel together and no caller guesses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheCallsRefusal = { calling: string; errno: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheCallsRefusal {
    /// Which call refused,
    /// as `listns`,
    /// `open_by_handle_at` or `ioctl NS_GET_ID`.
    pub calling: &'static str,
    /// The errno the call answered.
    pub errno: i32,
}

impl std::fmt::Display for TheCallsRefusal {
    /// One sentence naming the call and the errno it answered.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return write!(formatter, "{} answered errno {}", self.calling, self.errno);
    }
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
pub fn pages_until_the_end<Page>(mut ask: Page) -> Result<Vec<u64>, TheCallsRefusal>
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
            return Err(TheCallsRefusal {
                calling: "listns",
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
///  Every mount namespace this machine has,
///  as `listns` lists them.
/// Why:
///  The walk's first half is the kernel's own list,
///  and a caller that meets `ENOSYS` here learns this kernel cannot answer for
///  a namespace no task holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ids = theMountNamespacesOnThisMachine();
/// ```
pub fn the_mount_namespaces_on_this_machine() -> Result<Vec<u64>, TheCallsRefusal> {
    // What: the syscall number this build may call, or nothing on an architecture whose table it does not carry.
    // Why: a build that called another architecture's number 470 would make a different call entirely.
    let Some(number) = LISTNS_SYSCALL else {
        return Err(TheCallsRefusal {
            calling: "listns",
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
) -> Result<std::fs::File, TheCallsRefusal> {
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
        return Err(TheCallsRefusal {
            calling: "open_by_handle_at",
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_namespace_listing_tests.rs"]
mod tests;
