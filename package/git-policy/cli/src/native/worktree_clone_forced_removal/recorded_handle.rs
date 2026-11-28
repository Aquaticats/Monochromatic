//! What:
//!  The handle the add records of the slot's underlay before a clone is mounted,
//!  the two calls that take it and reopen it,
//!  and the mount id of the descriptor either answers.
//! Why:
//!  Way 8 unmounts the copy in place by a descriptor that names the covered
//!  directory without crossing the mount,
//!  so it changes no sharing and writes nothing
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  F31,
//!  F32,
//!  F53,
//!  and "Way 8,
//!  in place by the handle").
//!  `open_by_handle_at` needs a readable directory descriptor of the mount it
//!  resolves on and refuses a path-only one with `EBADF`,
//!  which the `6.8` guest answered,
//!  and the descriptor it answers is what `umount2` then names through
//!  `/proc/self/fd/<n>`.
//!  The record itself belongs to the add's step (design D7) and this build
//!  only fixes the three values the act reads from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const handle = nameTheHandle(folder);            // { handleType, handle }
//! const back = openFromTheHandle(checkoutMount, handle);  // a descriptor on the covered dentry
//! const id = mountIdOf(back);
//! ```

use std::ffi::CString;
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
use std::os::unix::fs::MetadataExt;

/// What:
///  The kernel's `struct file_handle` with room for its bytes.
/// Why:
///  The C binding declares the byte array with length zero,
///  because the kernel's structure ends in a flexible array,
///  so a caller that wants the bytes declares them itself and hands the
///  structure's address to the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileHandle = { handleBytes: number; handleType: number; bytes: Uint8Array };
/// ```
#[repr(C)]
struct FileHandle {
    /// The byte length the buffer holds.
    handle_bytes: u32,
    /// The handle's type.
    handle_type: i32,
    /// The handle's bytes,
    /// one byte more than the kernel's bound so a longer one is refused by
    /// the call rather than cut.
    f_handle: [u8; MAXIMUM_HANDLE_BYTES],
}

/// The largest file handle `name_to_handle_at` may report,
/// which is the kernel's own `MAX_HANDLE_SZ`.
pub const MAXIMUM_HANDLE_BYTES: usize = 128;

/// The record's format version,
/// which the reader checks before anything else.
pub const HANDLE_RECORD_VERSION: &str = "v1";

/// How many hex digits one byte takes in the record.
const HEX_DIGITS_ONE_BYTE: usize = 2;

/// What:
///  The kernel's handle of one folder,
///  as `name_to_handle_at` answered it.
/// Why:
///  A handle's type and bytes travel together,
///  and the add's record carries both so a later removal can reopen the folder
///  after its name moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedHandle = { handleType: number; handle: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedHandle {
    /// The handle's type,
    /// which names the file system's own encoding.
    pub handle_type: i32,
    /// The handle's bytes.
    pub handle: Vec<u8>,
}

/// What:
///  Why a handle could not be taken,
///  reopened or described.
/// Why:
///  Each call refuses for its own reason,
///  and the report names which one the act stopped at rather than "the handle
///  failed".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HandleRefusal = { calling: 'name_to_handle_at' | 'open_by_handle_at' | 'fdinfo' | 'record'; errno: number; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandleRefusal {
    /// Which call refused,
    /// or `record` for a text this build will not read.
    pub calling: &'static str,
    /// The errno the call answered,
    /// or an errno the record's own grammar uses.
    pub errno: i32,
    /// What it answered,
    /// in plain words.
    pub said: String,
}

impl std::fmt::Display for HandleRefusal {
    /// One sentence naming the call,
    /// the errno and what it said.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return write!(
            formatter,
            "{} answered errno {}: {}",
            self.calling, self.errno, self.said
        );
    }
}

impl RecordedHandle {
    /// What:
    ///  The one line the add's record carries,
    ///  `v1:<type>:<hex>`.
    /// Why:
    ///  The record is read back by a process that did not take the handle,
    ///  so its text must say the type and the bytes with nothing hidden.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = `v1:${handleType}:${toHex(handle)}`;
    /// ```
    pub fn encode(&self) -> String {
        let mut digits: String = String::with_capacity(self.handle.len() * HEX_DIGITS_ONE_BYTE);
        for byte in &self.handle {
            digits.push_str(format!("{byte:02x}").as_str());
        }
        return format!("{HANDLE_RECORD_VERSION}:{}:{digits}", self.handle_type);
    }

    /// What:
    ///  Read one line of the record.
    /// Why:
    ///  A handle longer than the kernel's,
    ///  a digit that is not hex,
    ///  and a version this build does not read are three refusals,
    ///  each named before any call is made with what it holds.
    ///
    /// # Errors
    /// The text is not one this grammar reads.
    pub fn decode(text: &str) -> Result<RecordedHandle, HandleRefusal> {
        let refusal = |said: &str| -> HandleRefusal {
            return HandleRefusal {
                calling: "record",
                errno: libc::EINVAL,
                said: String::from(said),
            };
        };
        if text.is_empty() {
            return Err(refusal("the line is empty"));
        }
        let mut parts = text.split(':');
        let Some(version) = parts.next() else {
            return Err(refusal("the line is empty"));
        };
        let Some(type_text) = parts.next() else {
            return Err(refusal("the line names no handle type"));
        };
        let Some(digits) = parts.next() else {
            return Err(refusal("the line names no handle bytes"));
        };
        if parts.next().is_some() {
            return Err(refusal("the line holds more than three fields"));
        }
        if version != HANDLE_RECORD_VERSION {
            return Err(refusal("the line's version is not one this build reads"));
        }
        let Ok(handle_type) = type_text.parse::<i32>() else {
            return Err(refusal("the handle type is not a decimal number"));
        };
        if digits.len() % HEX_DIGITS_ONE_BYTE != 0 {
            return Err(refusal("the handle's hex digits do not pair into bytes"));
        }
        let bytes: usize = digits.len() / HEX_DIGITS_ONE_BYTE;
        if bytes > MAXIMUM_HANDLE_BYTES {
            return Err(refusal("the handle is longer than the kernel's own"));
        }
        let mut handle: Vec<u8> = Vec::with_capacity(bytes);
        let mut at: usize = 0;
        while at < digits.len() {
            let Ok(byte) = u8::from_str_radix(&digits[at..at + HEX_DIGITS_ONE_BYTE], 16) else {
                return Err(refusal("a pair of the handle's digits is not hexadecimal"));
            };
            handle.push(byte);
            at += HEX_DIGITS_ONE_BYTE;
        }
        return Ok(RecordedHandle {
            handle_type,
            handle,
        });
    }
}

/// What:
///  The kernel's handle of the folder one descriptor names.
/// Why:
///  The add takes this before the clone is mounted,
///  so the handle names the underlay and never the copy.
///
/// # Errors
/// `name_to_handle_at` refused,
/// which a file system without handles answers
/// with `EOPNOTSUPP`.
pub fn name_the_handle(folder: &std::fs::File) -> Result<RecordedHandle, HandleRefusal> {
    let mut handle: FileHandle = FileHandle {
        handle_bytes: u32::try_from(MAXIMUM_HANDLE_BYTES).expect("the kernel's handle bound fits"),
        handle_type: 0,
        f_handle: [0; MAXIMUM_HANDLE_BYTES],
    };
    let mut mount_id: libc::c_int = 0;
    let empty: CString = CString::new("").expect("no NUL in an empty path");
    // SAFETY: `handle` is a live buffer of the size it declares, `mount_id` is this call's own, and the empty path
    // outlives the call; `AT_EMPTY_PATH` reads the empty string and the descriptor names the folder; the cast is to the
    // C binding's own structure, whose zero-length array is not the buffer.
    let named: libc::c_int = unsafe {
        libc::name_to_handle_at(
            folder.as_raw_fd(),
            empty.as_ptr(),
            (&mut handle as *mut FileHandle).cast::<libc::file_handle>(),
            &mut mount_id,
            libc::AT_EMPTY_PATH,
        )
    };
    if named != 0 {
        return Err(HandleRefusal {
            calling: "name_to_handle_at",
            errno: std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EINVAL),
            said: String::from("the folder's handle"),
        });
    }
    let bytes: usize = usize::try_from(handle.handle_bytes)
        .unwrap_or(0)
        .min(MAXIMUM_HANDLE_BYTES);
    return Ok(RecordedHandle {
        handle_type: handle.handle_type,
        handle: handle.f_handle[..bytes].to_vec(),
    });
}

/// What:
///  Reopen the folder a recorded handle names,
///  on one mount's descriptor,
///  without crossing a mount.
/// Why:
///  This is the descriptor way 8 unmounts by:
///  it names the dentry the copy covers,
///  and it must not cross into the copy,
///  which is why a path-only descriptor is refused `EBADF` by the kernel.
///
/// # Errors
/// The handle is longer than the kernel's,
/// or `open_by_handle_at` refused,
/// which inside a namespace whose user namespace is another answers `EINVAL`.
pub fn open_from_the_handle(
    mount: &std::fs::File,
    recorded: &RecordedHandle,
) -> Result<std::fs::File, HandleRefusal> {
    if recorded.handle.len() > MAXIMUM_HANDLE_BYTES {
        return Err(HandleRefusal {
            calling: "record",
            errno: libc::EINVAL,
            said: String::from("the handle is longer than the kernel's own"),
        });
    }
    let mut owned: FileHandle = FileHandle {
        handle_bytes: u32::try_from(recorded.handle.len()).unwrap_or(0),
        handle_type: recorded.handle_type,
        f_handle: [0; MAXIMUM_HANDLE_BYTES],
    };
    owned.f_handle[..recorded.handle.len()].copy_from_slice(recorded.handle.as_slice());
    // SAFETY: `owned` is a live handle of the size it declares, and the descriptor is the caller's own mount; the
    // descriptor it answers is owned exactly once, by the `File` below; the cast is to the C binding's own structure.
    let opened: libc::c_int = unsafe {
        libc::open_by_handle_at(
            mount.as_raw_fd(),
            (&mut owned as *mut FileHandle).cast::<libc::file_handle>(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    if opened < 0 {
        return Err(HandleRefusal {
            calling: "open_by_handle_at",
            errno: std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EINVAL),
            said: String::from("the recorded handle on this mount"),
        });
    }
    // SAFETY: the call answered a descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(opened) }));
}

/// What:
///  The mount id of the mount one descriptor names its file on.
/// Why:
///  The kernel's own mount number is how the act knows a descriptor is on the
///  checkout's mount and not on the copy,
///  and how it re-reads that a mount is gone.
///
/// # Errors
/// The descriptor's `fdinfo` could not be read or carries no mount id.
pub fn mount_id_of(descriptor: &std::fs::File) -> Result<u64, HandleRefusal> {
    let path: String = format!("/proc/self/fdinfo/{}", descriptor.as_raw_fd());
    let text: String = crate::plain_file::read_plain_text(std::path::Path::new(path.as_str()))
        .map_err(|error| {
            return HandleRefusal {
                calling: "fdinfo",
                errno: error.raw_os_error().unwrap_or(libc::ENOENT),
                said: String::from("a descriptor's own record"),
            };
        })?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("mnt_id:") {
            let Ok(number) = rest.trim().parse::<u64>() else {
                return Err(HandleRefusal {
                    calling: "fdinfo",
                    errno: libc::EINVAL,
                    said: String::from("the mount id is not a number"),
                });
            };
            return Ok(number);
        }
    }
    return Err(HandleRefusal {
        calling: "fdinfo",
        errno: libc::ENOENT,
        said: String::from("no line names a mount id"),
    });
}

/// What:
///  The folder's device and inode,
///  as a descriptor describes it.
/// Why:
///  A report names the folder the handle reopened,
///  and a control compares it with the underlay it named before the mount.
pub fn folder_facts(folder: &std::fs::File) -> Result<(u64, u64), HandleRefusal> {
    let facts: std::fs::Metadata = folder.metadata().map_err(|error| {
        return HandleRefusal {
            calling: "fdinfo",
            errno: error.raw_os_error().unwrap_or(libc::EINVAL),
            said: String::from("a folder descriptor's own facts"),
        };
    })?;
    return Ok((facts.dev(), facts.ino()));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "recorded_handle_tests.rs"]
mod tests;
