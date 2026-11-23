//! What:
//!  The kernel calls the unmount route makes:
//!  the bind of the staging folder,
//!  its propagation change,
//!  the move of the clone onto it,
//!  the descriptor unmount,
//!  the file-handle pair a later sweep reopens the folder with,
//!  and the reads that decide whether any of them may run.
//! Why:
//!  The design forbids a path-based mount or unmount on the account's tree,
//!  so every call here takes a descriptor or a `/proc/self/fd/<n>` string
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The unmount call from staging";
//!  the two `/proc` readings were accepted without veto).
//!  The calls are behind one trait because a control on a person's machine
//!  cannot mount:
//!  the unmount turn's identity comparisons are proved against a recording
//!  implementation,
//!  and the real one runs in the guest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bind = calls.bindTheFolder(folder, cache, name);
//! calls.makePrivate(bind); calls.moveClone(clone, bind); calls.unmountTop(bind);
//! ```

use crate::fs_beneath_mount_facts::{mount_id_of, observed_path};
use crate::fs_mount_options::{MountFacts, parse_mount_table, MOUNTINFO_PATH};
use std::ffi::{CString, OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::path::PathBuf;

/// The kernel file that names this boot.
pub const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

/// This process's own mount namespace.
pub const MOUNT_NAMESPACE_PATH: &str = "/proc/self/ns/mnt";

/// What:
///  One call of the staging route,
///  named as a receipt names the failed step.
/// Why:
///  A refusal must say which call refused and its errno,
///  and the words are read by a person and matched by the guest arms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingCall = 'bind' | 'open-bind' | 'make-private' | 'move' | 'unmount' | 'name-handle' | 'open-handle' | 'mount-id' | 'mount-table' | 'namespace' | 'boot';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StagingCall {
    /// `mount(2)` with `MS_BIND` of the folder onto itself.
    Bind,
    /// The by-name open of the bind under the cache descriptor.
    OpenBind,
    /// `mount_setattr(2)` making the bind private.
    MakePrivate,
    /// `move_mount(2)` of the clone onto the bind.
    Move,
    /// `umount2(2)` through the held descriptor.
    Unmount,
    /// `name_to_handle_at(2)` of the folder.
    NameHandle,
    /// `open_by_handle_at(2)` of the folder.
    OpenHandle,
    /// The final slot hop of the unmount's walk.
    OpenSlot,
    /// A second whole walk of the canonical slot.
    LookAgain,
    /// The mount number in `/proc/self/fdinfo`.
    MountId,
    /// This process's mount table.
    MountTable,
    /// This process's mount namespace identity.
    Namespace,
    /// This boot's identifier.
    Boot,
}

impl StagingCall {
    /// The word a refusal and a receipt carry.
    pub fn word(self) -> &'static str {
        return match self {
            StagingCall::Bind => "bind",
            StagingCall::OpenBind => "open-bind",
            StagingCall::MakePrivate => "make-private",
            StagingCall::Move => "move-mount",
            StagingCall::Unmount => "unmount",
            StagingCall::NameHandle => "name-handle",
            StagingCall::OpenHandle => "open-handle",
            StagingCall::OpenSlot => "open-slot",
            StagingCall::LookAgain => "look-again",
            StagingCall::MountId => "mount-id",
            StagingCall::MountTable => "mount-table",
            StagingCall::Namespace => "namespace",
            StagingCall::Boot => "boot",
        };
    }
}

/// What:
///  One refused call:
///  which call,
///  its errno and the kernel's words.
/// Why:
///  The receipt carries the step and errno,
///  and a refusal that lost either would leave a person with nothing to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingCallRefusal = { call: StagingCall; errno: number; detail: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagingCallRefusal {
    /// The call that refused.
    pub call: StagingCall,
    /// The kernel's errno,
    /// or zero when the call answered without one.
    pub errno: i32,
    /// What the call answered.
    pub detail: String,
}

impl StagingCallRefusal {
    /// The refusal in words.
    pub fn message(&self, subject: &str) -> String {
        return format!(
            "{subject}: {} answered {}{}. Nothing later in the unmount ran, so the mount stays where it was and the receipt names this step.",
            self.call.word(),
            self.detail,
            if self.errno == 0 { String::new() } else { format!(" (errno {})", self.errno) },
        );
    }
}

/// The refusal of one call with the thread's last error.
fn refused(call: StagingCall, error: std::io::Error) -> StagingCallRefusal {
    return StagingCallRefusal { call, errno: error.raw_os_error().unwrap_or(0), detail: error.to_string() };
}

/// What:
///  The calls the unmount route and its later sweep make.
/// Why:
///  A control records them,
///  and the release path makes them,
///  so the identity comparisons are provable without a mount and the guest
///  proves the calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface StagingCalls { bindTheFolder(...); makePrivate(...); moveClone(...); unmountTop(...); ... }
/// ```
pub trait StagingCalls {
    /// Bind the folder onto itself and answer the descriptor the bind's name opens.
    fn bind_the_folder(&self, folder: &std::fs::File, cache: &std::fs::File, name: &OsStr) -> Result<std::fs::File, StagingCallRefusal>;
    /// Make the bind private.
    fn make_private(&self, bind: &std::fs::File) -> Result<(), StagingCallRefusal>;
    /// Move the clone onto the bind.
    fn move_clone(&self, clone: &std::fs::File, bind: &std::fs::File) -> Result<(), StagingCallRefusal>;
    /// Unmount the topmost mount at the held descriptor.
    fn unmount_top(&self, held: &std::fs::File) -> Result<(), StagingCallRefusal>;
    /// The folder's handle and its type.
    fn name_handle(&self, folder: &std::fs::File) -> Result<(i32, Vec<u8>), StagingCallRefusal>;
    /// The folder reopened from its handle through the cache's descriptor.
    fn open_from_handle(&self, cache: &std::fs::File, handle_type: i32, handle: &[u8]) -> Result<std::fs::File, StagingCallRefusal>;
    /// The entry at `name` beneath the cache descriptor,
    /// crossing the one mount that may cover it.
    fn open_by_name(&self, cache: &std::fs::File, name: &OsStr) -> Result<std::fs::File, StagingCallRefusal>;
    /// The mount a descriptor is on.
    fn mount_id(&self, file: &std::fs::File) -> Result<u64, StagingCallRefusal>;
    /// The path a descriptor names now.
    fn observed_path(&self, file: &std::fs::File) -> Result<PathBuf, StagingCallRefusal>;
    /// One mount's facts from this process's table.
    fn mount_facts(&self, mount_id: u64) -> Result<MountFacts, StagingCallRefusal>;
    /// This process's whole mount table.
    fn mount_table(&self) -> Result<Vec<MountFacts>, StagingCallRefusal>;
    /// This process's mount namespace device and inode.
    fn namespace_pair(&self) -> Result<(u64, u64), StagingCallRefusal>;
    /// This boot's identifier.
    fn boot(&self) -> Result<String, StagingCallRefusal>;
}

/// What:
///  The kernel's `file_handle` with the bytes the kernel actually writes.
/// Why:
///  The `libc` crate declares `f_handle` as a zero-length array,
///  so the struct it declares is eight bytes and a call writing a 12-byte handle
///  writes past its end (undefined behaviour),
///  while the reopen then copies zero bytes.
///  This declaration carries the kernel's own 128-byte field and is cast to the
///  libc type only at the two calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ours: FileHandle = { handleBytes: 128, handleType: 0, fHandle: new Uint8Array(128) };
/// ```
#[repr(C)]
struct FileHandle {
    /// The bytes the handle holds,
    /// and the buffer's size on entry.
    handle_bytes: libc::c_uint,
    /// The handle's type,
    /// which the kernel reports.
    handle_type: libc::c_int,
    /// The handle's bytes.
    f_handle: [libc::c_uchar; super::staged_folder_record::MAXIMUM_HANDLE_BYTES],
}

/// What:
///  The release calls:
///  the kernel's own.
/// Why:
///  Nothing here reads a path inside the account's tree except the
///  `/proc/self/fd` strings the accepted readings allow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const calls = new LiveStagingCalls();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LiveStagingCalls;

impl StagingCalls for LiveStagingCalls {
    fn bind_the_folder(&self, folder: &std::fs::File, cache: &std::fs::File, name: &OsStr) -> Result<std::fs::File, StagingCallRefusal> {
        let held: CString = CString::new(format!("/proc/self/fd/{}", folder.as_raw_fd())).expect("no NUL in a descriptor path");
        // SAFETY: `held` outlives the call; a bind reads no data pointer and asks for no propagation change here.
        let bound: libc::c_int = unsafe { libc::mount(held.as_ptr(), held.as_ptr(), std::ptr::null(), libc::MS_BIND, std::ptr::null()) };
        if bound != 0 {
            return Err(refused(StagingCall::Bind, std::io::Error::last_os_error()));
        }
        // The open by name follows the topmost mount, so a mount the account stacked after the bind would be taken for
        // the bind. The descriptor's own row says what it is: a bind of the folder (root not `/`) whose parent and source
        // are the cache's.
        let cache_id: u64 = mount_id_of(cache).map_err(|error| return refused(StagingCall::Bind, error))?;
        let opened: std::fs::File = crate::fs_beneath::open_directory_crossing_one_mount(cache, name).map_err(|error| return refused(StagingCall::OpenBind, error))?;
        let opened_id: u64 = mount_id_of(&opened).map_err(|error| return refused(StagingCall::OpenBind, error))?;
        let table: Vec<MountFacts> = parse_mount_table(std::fs::read(MOUNTINFO_PATH).map_err(|error| return refused(StagingCall::MountTable, error))?.as_slice());
        let Some(cache_source) = table.iter().find(|facts| return facts.mount_id == cache_id).map(|facts| return facts.source.clone()) else {
            return Err(StagingCallRefusal { call: StagingCall::MountTable, errno: libc::ENOENT, detail: format!("no row names the cache's mount {cache_id}") });
        };
        let Some(row) = table.iter().find(|facts| return facts.mount_id == opened_id) else {
            return Err(StagingCallRefusal { call: StagingCall::MountTable, errno: libc::ENOENT, detail: format!("no row names the opened mount {opened_id}") });
        };
        // A bind of the folder onto itself has root `/`, like the cache mount; the two differ by their mount id and by
        // the bind's parent being the cache mount.
        if opened_id == cache_id || row.parent_id != cache_id || row.source != cache_source {
            return Err(StagingCallRefusal {
                call: StagingCall::OpenBind,
                errno: libc::EBUSY,
                detail: format!("the topmost mount at the folder is {opened_id}, whose parent is {} and source `{}`; the bind of the cache folder is the cache's own child with a root under it", row.parent_id, row.source),
            });
        }
        return Ok(opened);
    }

    fn make_private(&self, bind: &std::fs::File) -> Result<(), StagingCallRefusal> {
        let attributes: libc::mount_attr = libc::mount_attr { attr_set: 0, attr_clr: 0, propagation: libc::MS_PRIVATE as u64, userns_fd: 0 };
        let empty: CString = CString::new("").expect("no NUL in an empty path");
        // SAFETY: the descriptor is the caller's, the structure outlives the call, and `AT_EMPTY_PATH` reads the empty string.
        let set: libc::c_int = unsafe {
            libc::syscall(libc::SYS_mount_setattr, bind.as_raw_fd(), empty.as_ptr(), libc::AT_EMPTY_PATH, (&attributes as *const libc::mount_attr).cast::<libc::c_void>(), std::mem::size_of::<libc::mount_attr>())
                as libc::c_int
        };
        return if set == 0 { Ok(()) } else { Err(refused(StagingCall::MakePrivate, std::io::Error::last_os_error())) };
    }

    fn move_clone(&self, clone: &std::fs::File, bind: &std::fs::File) -> Result<(), StagingCallRefusal> {
        let empty: CString = CString::new("").expect("no NUL in an empty path");
        // SAFETY: both descriptors are the caller's, and the two empty paths are the NUL-terminated strings the flags read.
        let moved: libc::c_int = unsafe {
            libc::syscall(libc::SYS_move_mount, clone.as_raw_fd(), empty.as_ptr(), bind.as_raw_fd(), empty.as_ptr(), libc::MOVE_MOUNT_F_EMPTY_PATH | libc::MOVE_MOUNT_T_EMPTY_PATH) as libc::c_int
        };
        return if moved == 0 { Ok(()) } else { Err(refused(StagingCall::Move, std::io::Error::last_os_error())) };
    }

    fn unmount_top(&self, held: &std::fs::File) -> Result<(), StagingCallRefusal> {
        let named: CString = CString::new(format!("/proc/self/fd/{}", held.as_raw_fd())).expect("no NUL in a descriptor path");
        // SAFETY: `named` outlives the call, and no flag asks for force or lazy detach.
        let taken: libc::c_int = unsafe { libc::umount2(named.as_ptr(), 0) };
        return if taken == 0 { Ok(()) } else { Err(refused(StagingCall::Unmount, std::io::Error::last_os_error())) };
    }

    fn name_handle(&self, folder: &std::fs::File) -> Result<(i32, Vec<u8>), StagingCallRefusal> {
        let mut handle: FileHandle = FileHandle { handle_bytes: super::staged_folder_record::MAXIMUM_HANDLE_BYTES as u32, handle_type: 0, f_handle: [0; super::staged_folder_record::MAXIMUM_HANDLE_BYTES] };
        let mut mount_id: libc::c_int = 0;
        let empty: CString = CString::new("").expect("no NUL in an empty path");
        // SAFETY: `handle` is a live buffer of the size it declares, `mount_id` is this call's own, and the empty path outlives the call; the cast is to the libc type the call takes, whose zero-length array is not the buffer.
        let named: libc::c_int = unsafe { libc::name_to_handle_at(folder.as_raw_fd(), empty.as_ptr(), (&mut handle as *mut FileHandle).cast::<libc::file_handle>(), &mut mount_id, libc::AT_EMPTY_PATH) };
        if named != 0 {
            return Err(refused(StagingCall::NameHandle, std::io::Error::last_os_error()));
        }
        let bytes: usize = usize::try_from(handle.handle_bytes).unwrap_or(0).min(super::staged_folder_record::MAXIMUM_HANDLE_BYTES);
        return Ok((handle.handle_type, handle.f_handle[..bytes].to_vec()));
    }

    fn open_from_handle(&self, cache: &std::fs::File, handle_type: i32, handle: &[u8]) -> Result<std::fs::File, StagingCallRefusal> {
        if handle.len() > super::staged_folder_record::MAXIMUM_HANDLE_BYTES {
            return Err(StagingCallRefusal { call: StagingCall::OpenHandle, errno: libc::EINVAL, detail: String::from("the handle is longer than the kernel's") });
        }
        let mut owned: FileHandle = FileHandle { handle_bytes: u32::try_from(handle.len()).unwrap_or(0), handle_type, f_handle: [0; super::staged_folder_record::MAXIMUM_HANDLE_BYTES] };
        owned.f_handle[..handle.len()].copy_from_slice(handle);
        // SAFETY: `owned` is a live handle of the size it declares, and the descriptor is the caller's cache mount; the cast is to the libc type the call takes.
        let opened: libc::c_int = unsafe { libc::open_by_handle_at(cache.as_raw_fd(), (&mut owned as *mut FileHandle).cast::<libc::file_handle>(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC) };
        if opened < 0 {
            return Err(refused(StagingCall::OpenHandle, std::io::Error::last_os_error()));
        }
        // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
        return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(opened) }));
    }

    fn open_by_name(&self, cache: &std::fs::File, name: &OsStr) -> Result<std::fs::File, StagingCallRefusal> {
        return crate::fs_beneath::open_directory_crossing_one_mount(cache, name).map_err(|error| return refused(StagingCall::OpenBind, error));
    }

    fn mount_id(&self, file: &std::fs::File) -> Result<u64, StagingCallRefusal> {
        return mount_id_of(file).map_err(|error| return refused(StagingCall::MountId, error));
    }

    fn observed_path(&self, file: &std::fs::File) -> Result<PathBuf, StagingCallRefusal> {
        return observed_path(file).map_err(|error| return refused(StagingCall::MountId, error));
    }

    fn mount_facts(&self, mount_id: u64) -> Result<MountFacts, StagingCallRefusal> {
        let table: Vec<MountFacts> = self.mount_table()?;
        for facts in table {
            if facts.mount_id == mount_id {
                return Ok(facts);
            }
        }
        return Err(StagingCallRefusal { call: StagingCall::MountTable, errno: libc::ENOENT, detail: format!("mount {mount_id} is not on this process's table at {MOUNTINFO_PATH}") });
    }

    fn mount_table(&self) -> Result<Vec<MountFacts>, StagingCallRefusal> {
        let bytes: Vec<u8> = std::fs::read(MOUNTINFO_PATH).map_err(|error| return refused(StagingCall::MountTable, error))?;
        return Ok(parse_mount_table(bytes.as_slice()));
    }

    fn namespace_pair(&self) -> Result<(u64, u64), StagingCallRefusal> {
        use std::os::unix::fs::MetadataExt;
        let facts: std::fs::Metadata = std::fs::metadata(MOUNT_NAMESPACE_PATH).map_err(|error| return refused(StagingCall::Namespace, error))?;
        return Ok((facts.dev(), facts.ino()));
    }

    fn boot(&self) -> Result<String, StagingCallRefusal> {
        let text: String = std::fs::read_to_string(BOOT_ID_PATH).map_err(|error| return refused(StagingCall::Boot, error))?;
        return Ok(String::from(text.trim()));
    }
}

/// The folder's name as the descriptor path a bind takes.
pub fn descriptor_path(file: &std::fs::File) -> OsString {
    return OsString::from(format!("/proc/self/fd/{}", file.as_raw_fd()));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staging_calls_tests.rs"]
mod tests;
