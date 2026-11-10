//! What:
//!  The device number and the inode of the file an open descriptor of a process refers to.
//! Why:
//!  A lock is matched to the descriptor that holds it by the device number and the inode the file has,
//!  and the descriptor's own record is only the cheap test that comes first.
//!  Reading those attributes through the path `/proc/<pid>/fd/<n>` walks that path,
//!  and the last step of the walk asks the file system to revalidate its own directory entry before any attribute is read:
//!  a FUSE or NFS server that stops answering holds the walk in uninterruptible sleep for as long as it takes,
//!  and `SIGTERM` does not end it
//!  (`doc/troubleshooting/fuse-descriptor-attributes.md`).
//!  So the walk is taken only where it cannot reach a server:
//!  the descriptor's own mount,
//! named by the `mnt_id` line of its record,
//! is looked up in that process's own mount table,
//!  and the file is read only when the mount is on a fixed list of local file systems,
//!  whose attributes are in the memory of the kernel.
//!  Every other descriptor is refused,
//!  and the caller treats the file as one that may be held rather than one that is not.
//!  A copy of the descriptor (`pidfd_getfd(2)`) would answer a stalled server from its cached inode,
//!  but closing the copy runs the file system's own flush operation,
//!  which a stopped server holds just as its lookup is held
//!  (cross-vendor review of 2026-11-09,
//!  finding 4),
//!  and the copy itself is refused for a process this one may not attach to,
//!  which would read as no holder at all (finding 2).
//!  A kernel `overlay` mount is the one whose type does not settle the question from inside a container:
//!  its layers are the host's directories,
//!  which no process inside the container can name,
//!  and a mount this process refuses there leaves an abandoned `index.lock` that no scan can prove unheld,
//!  so every commit behind it fails at its ceiling for ever.
//!  That read is made by a child of this process instead
//!  (`open_file_of_a_descriptor_in_a_child.rs`),
//!  which is abandoned at its ceiling,
//!  so a layer that a stopped server backs cannot hold the caller either.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const opened = openedFileOf({ process: '/proc/1234', fd: '3' }); // { device, inode }
//! ```

/// `OsStr` is a borrowed file name.
use std::ffi::OsStr;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The device number and the inode number of a file a descriptor holds open.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenedFile {
    /// The device number of the file system that holds the file (`st_dev`).
    pub device: u64,
    /// The inode number of the file on that device.
    pub inode: u64,
}

/// The file systems whose attributes a `statx` answers from the memory of the kernel:
/// disks and memory of this machine,
/// and no file system served by another process or another machine.
/// An overlay file system is not one of them:
/// its attributes are those of the file below it,
/// which such a server may be serving.
const READ_IN_MEMORY_FILE_SYSTEMS: [&str; 7] = ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "zfs", "bcachefs"];

/// What:
///  The mount a descriptor was opened through,
/// as the kernel writes it in the descriptor's record.
/// Why:
///   `/proc/<pid>/fdinfo/<n>` holds a `mnt_id:` line,
///  and the mount table of the same process says what that mount is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseFdinfo(await readFile(`${process}/fdinfo/${fd}`, 'utf8')).mnt_id ?? undefined
/// ```
#[cfg(target_os = "linux")]
fn mount_id_of(process: &Path, fd: &OsStr) -> std::io::Result<Option<u64>> {
    let record: PathBuf = process.join("fdinfo").join(fd);
    let text: String = std::fs::read_to_string(record)?;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("mnt_id:") {
            return Ok(value.trim().parse::<u64>().ok());
        }
    }
    return Ok(None);
}

/// What:
///  The type of the mount with this number in the process's own mount table,
///  when the table names it.
/// Why:
///   `mountinfo` lines end `- <type> <source> <options>`,
///  and a mount that is not in the table is one this process cannot be told the type of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readFile(`${process}/mountinfo`, 'utf8')).split('\n').find(line => Number(line) === mntId)?.split(' - ')[1].split(' ')[0]
/// ```
#[cfg(target_os = "linux")]
fn file_system_of_mount(process: &Path, mount: u64) -> std::io::Result<Option<String>> {
    let table: String = std::fs::read_to_string(process.join("mountinfo"))?;
    for line in table.lines() {
        let fields: Vec<&str> = line.split(' ').collect();
        let first: Option<u64> = match fields.first() {
            Some(text) => text.parse::<u64>().ok(),
            None => None,
        };
        if first != Some(mount) {
            continue;
        }
        // The line is `<id> <parent> <major:minor> <root> <point> <options>... - <type> <source> <options>`, so the type is
        // the field after the separator.
        let mut kind: Option<String> = None;
        let mut after_separator: bool = false;
        for field in fields {
            if after_separator {
                kind = Some(String::from(field));
                break;
            }
            after_separator = field == "-";
        }
        return Ok(kind);
    }
    return Ok(None);
}

/// The file system type of a kernel `overlay` mount:
/// the one whose layers, not itself, decide whether a read can reach a server,
/// and whose layers a process inside a container cannot name.
const OVERLAY_FILE_SYSTEM: &str = "overlay";

/// What:
///  Whether attributes of a file on this mount come from the memory of the kernel.
/// Why:
///   The list is the gate that keeps a stopped server out of the read:
///  a mount that the table does not name is not known to be local,
///  so it is not read either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// READ_IN_MEMORY_FILE_SYSTEMS.includes(kind ?? '')
/// ```
#[cfg(target_os = "linux")]
fn read_in_memory(kind: Option<&str>) -> bool {
    let Some(name) = kind else {
        return false;
    };
    return READ_IN_MEMORY_FILE_SYSTEMS.contains(&name);
}

/// What:
///  The device and inode of the file the descriptor `fd` of the process whose table directory is `process` holds open.
/// Why:
///   The one way a holder of a file is identified here,
///  and the reason no file system is asked to answer anything it must fetch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openedFileOf({ process: '/proc/1234', fd: '3' });
/// ```
#[cfg(target_os = "linux")]
pub fn opened_file_of(process: &Path, fd: &OsStr) -> std::io::Result<OpenedFile> {
    let mount: Option<u64> = mount_id_of(process, fd)?;
    let kind: Option<String> = match mount {
        Some(number) => file_system_of_mount(process, number)?,
        None => None,
    };
    // An overlay's own attributes are in the memory of this kernel, and a read of them asks a layer only when that layer
    //  sets a revalidation of its own (`fs/overlayfs/super.c`, `ovl_revalidate_real`). Whether a layer does is decided by
    //  the layer's own file system, which a container's overlay names with the host's paths and this process cannot read:
    //  so the walk is made where a stopped server cannot hold the caller, and its answer is waited for only until the
    //  child's ceiling.
    if kind.as_deref() == Some(OVERLAY_FILE_SYSTEM) {
        return crate::open_file_of_a_descriptor_in_a_child::opened_file_of_in_a_child(process, fd);
    }
    if !read_in_memory(kind.as_deref()) {
        let named: String = match kind {
            Some(name) => format!("a `{name}` file system"),
            None => String::from("not in the process's mount table"),
        };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("its mount is {named}, whose attributes are not in the memory of this kernel, so the file is not read"),
        ));
    }
    let link: PathBuf = process.join("fd").join(fd);
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    let name: std::ffi::CString = path_name(link.as_path())?;
    // SAFETY: the name outlives the call, `found` is a live `statx`, and the flags only forbid a round trip to a server.
    let answer: libc::c_int = unsafe {
        libc::statx(
            libc::AT_FDCWD,
            name.as_ptr(),
            libc::AT_STATX_DONT_SYNC,
            libc::STATX_BASIC_STATS,
            &mut found,
        )
    };
    if answer < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(OpenedFile { device: libc::makedev(found.stx_dev_major, found.stx_dev_minor), inode: found.stx_ino });
}

/// What:
///  The device and inode of the file one of this process's own open descriptors refers to,
///  named by the number of that descriptor.
/// Why:
///   This process's own descriptors need no mount's leave to be read:
///  `statx(2)` with `AT_EMPTY_PATH` asks for what the kernel already holds of the open file,
///  without walking any path,
///  so no file system revalidates a directory entry and no server is asked anything
///  (`doc/troubleshooting/fuse-descriptor-attributes.md`).
///  A count of this process's own descriptors of a lock is therefore the same on every mount type,
///  and a mount this process cannot classify does not turn that count into an unknown one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fstat({ descriptor: 3 }); // { device, inode }, from what the kernel holds
/// ```
#[cfg(target_os = "linux")]
pub fn opened_file_of_our_own(descriptor: i32) -> std::io::Result<OpenedFile> {
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: the descriptor is this process's own, the empty name is the one `AT_EMPTY_PATH` requires, and `found` is a
    //  live `statx`.
    let answer: libc::c_int = unsafe {
        libc::statx(
            descriptor,
            c_empty_name(),
            libc::AT_EMPTY_PATH | libc::AT_STATX_DONT_SYNC,
            libc::STATX_BASIC_STATS,
            &mut found,
        )
    };
    if answer < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(OpenedFile { device: libc::makedev(found.stx_dev_major, found.stx_dev_minor), inode: found.stx_ino });
}

/// What:
///  The same on a platform that has no `statx`, and no `/proc` to walk for a count of descriptors.
/// Why:
///   Nothing is guessed at where the kernel's own answer is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error('no statx on this platform');
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn opened_file_of_our_own(_descriptor: i32) -> std::io::Result<OpenedFile> {
    return Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "this platform answers no descriptor's own attributes, so the file is not read",
    ));
}

/// What:
///  The empty name `AT_EMPTY_PATH` reads a descriptor's own file by.
/// Why:
///   `statx(2)` takes a terminated name,
///  and the flag says the name is empty and the descriptor itself is the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from('\0')
/// ```
#[cfg(target_os = "linux")]
fn c_empty_name() -> *const libc::c_char {
    return b"\0".as_ptr().cast();
}

/// What:
///  A path as a terminated string for a system call.
/// Why:
///   `statx` takes a C string,
///  and a path with a zero byte in it is refused rather than truncated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(path + '\0')
/// ```
#[cfg(target_os = "linux")]
fn path_name(path: &Path) -> std::io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt as _;
    let bytes: &[u8] = path.as_os_str().as_bytes();
    if bytes.contains(&0) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "the descriptor's path holds a zero byte"));
    }
    return match std::ffi::CString::new(bytes) {
        Ok(name) => Ok(name),
        Err(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "the descriptor's path holds a zero byte")),
    };
}

/// What:
///  The same on a platform that has no `/proc` to read a descriptor's record and mount table from.
/// Why:
///   Nothing is guessed at where the kernel's own tables are not there,
///  and a caller that needs the answer treats this as a process it could not look into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error('no /proc on this platform');
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn opened_file_of(process: &Path, fd: &OsStr) -> std::io::Result<OpenedFile> {
    let _ = (process, fd);
    return Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "this platform has no `/proc`, so a descriptor's file is not read",
    ));
}

/// Controls on the descriptor reads stay out of the release executable.
#[cfg(test)]
#[path = "open_file_of_a_descriptor_tests.rs"]
mod tests;
