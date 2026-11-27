//! What:
//!  Whether one descriptor of one process is the file a scan looks for,
//!  asked of the descriptor's own record first,
//!  and of a copy of the descriptor itself after.
//! Why:
//!  A lock has more than one name,
//!  so a scan finds its holders by the file's device number and inode,
//!  and the answer never comes from walking the path of a descriptor's link:
//!  that asks the file's own file system,
//!  and a stalled one holds that for ever.
//!  `rust/max-lines` splits a file by role,
//!  and these are the checks a walk of the table makes for each descriptor.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = await descriptorIsFile(process, fd, device, inode);
//! const anonymous = await mountedDescriptorIsFile(process, fd, mount, inode);
//! ```

use super::process_use_targets_and_answers::Unread;
use std::path::Path;
/// What:
///  Whether the descriptor `fd` of the process whose table directory is `process` is the file with this device number and inode.
/// Why:
///   A device number and an inode name one file on a machine,
/// which is what `stat(2)` says of `st_dev` and `st_ino`.
/// The pair the kernel writes for a descriptor,
/// its mount number and its inode,
/// is neither unique nor complete (audit of 2026-11-09,
/// finding 4):
/// two files of two btrfs subvolumes reached through one mount have the same inode number and the same mount number,
/// so a process holding one of them read as the holder of the other,
/// and a bind mount gives the same file another mount number,
/// so a holder that reached it that way was not found at all.
/// The record's inode number is the cheap test that comes first:
/// only a descriptor whose record already names the file's inode number is resolved,
/// and the answer comes from a copy of the descriptor itself (`opened_file_of`),
/// never from the path of its link,
/// because walking that path asks the file's own file system to revalidate its directory entry and a stalled one holds it
/// for ever (cross-vendor review of 2026-11-09,
/// finding 3;
/// `doc/troubleshooting/fuse-descriptor-attributes.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { ino } = descriptorRecord({ process, fd, targeted: 'real-file' });
/// const opened = ino === inode ? openedFileOf({ process, fd }) : undefined;
/// opened?.device === device && opened?.inode === inode;
/// ```
#[cfg(unix)]
pub fn descriptor_is_file(
    process: &Path,
    fd: &std::ffi::OsStr,
    device: u64,
    inode: u64,
) -> Result<bool, Unread> {
    return descriptor_is_file_with(
        process,
        fd,
        device,
        inode,
        crate::open_file_of_a_descriptor::opened_file_of,
    );
}

/// What:
///  `descriptor_is_file` with the reader of a descriptor's open file given:
///  the kernel's copy of it in a judgement,
///  and a control's own reader for a table the kernel does not know.
/// Why:
///   A control's `/proc` is a directory of links,
///  so the file a descriptor has open there can only be read from its link,
///  while the production read copies the descriptor and never walks the path
///  (`src/native/process/open_file_of_a_descriptor.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// descriptorIsFileWith({ process, fd, device, inode, opened });
/// ```
#[cfg(unix)]
pub fn descriptor_is_file_with(
    process: &Path,
    fd: &std::ffi::OsStr,
    device: u64,
    inode: u64,
    opened: fn(
        &Path,
        &std::ffi::OsStr,
    ) -> std::io::Result<crate::open_file_of_a_descriptor::OpenedFile>,
) -> Result<bool, Unread> {
    let text: String = match crate::descriptor_record::descriptor_record(
        process,
        fd,
        crate::descriptor_record::Targeted::RealFile,
    ) {
        // A pipe, a socket or a kernel anonymous inode cannot be a file of a file system, its record is never opened, and
        //  a descriptor that closed while the scan walked the directory is open in nothing.
        crate::descriptor_record::Record::NotTheTarget | crate::descriptor_record::Record::Gone => {
            return Ok(false);
        }
        crate::descriptor_record::Record::Text(text) => text,
        // A record past the bound is one this scan did not read: the descriptor may hold the file, so nothing is concluded.
        crate::descriptor_record::Record::BeyondBound => {
            return Err(Unread::File(format!(
                "its record is longer than the {} bytes this scan takes, so the descriptor may hold the file",
                crate::descriptor_record::RECORD_BYTES_AT_MOST
            )));
        }
        crate::descriptor_record::Record::Unreadable {
            reason,
            ..
        } => return Err(Unread::Process(reason)),
        // The kind of a descriptor whose link text cannot be read is unknown, so it may be the file.
        crate::descriptor_record::Record::KindUnknown {
            reason,
            ..
        } => {
            return Err(Unread::File(format!(
                "the text of its link cannot be read ({reason}), so the descriptor may hold the file"
            )));
        }
    };
    let held: Option<u64> = crate::descriptor_record::inode_in(text.as_str());
    if held != Some(inode) {
        return Ok(false);
    }
    let file: crate::open_file_of_a_descriptor::OpenedFile = match opened(process, fd) {
        Ok(found) => found,
        Err(error) => return Err(Unread::File(format!("{error}"))),
    };
    return Ok(file.device == device && file.inode == inode);
}

/// What:
///  The mount number the kernel gives the mount a path is reached through.
/// Why:
///   `/proc/<pid>/fdinfo` names a descriptor's mount by that number and its file by the inode,
/// so a descriptor of another process is recognised without resolving it on its own file system,
/// which waits there as long as that file system does
/// (`open_file_of_a_descriptor.rs`).
/// Zero is "not known",
/// and matches no descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = statx(path, { mask: STATX_MNT_ID }).stx_mnt_id ?? 0;
/// ```
pub fn mount_of(path: &Path) -> u64 {
    let Ok(found) = rustix::fs::statx(
        rustix::fs::CWD,
        path,
        rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
        rustix::fs::StatxFlags::MNT_ID,
    ) else {
        return 0;
    };
    return found.stx_mnt_id;
}

/// What:
///  Whether the descriptor `fd` of the process whose table directory is `process` is the file with this inode on this
///  mount,
///  as the descriptor's own record names it.
/// Why:
///   The mount number and the inode are both in the text the kernel writes for the descriptor,
///  so this asks no file system anything,
///  and it answers for a pipe or another anonymous file that the gate refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { mnt_id, ino } = descriptorRecord({ process, fd, targeted: 'anonymous-file' }); ino === inode && mnt_id === mount;
/// ```
pub fn mounted_descriptor_is_file(
    process: &Path,
    fd: &std::ffi::OsStr,
    mount: u64,
    inode: u64,
) -> Result<bool, Unread> {
    let text: String = match crate::descriptor_record::descriptor_record(
        process,
        fd,
        crate::descriptor_record::Targeted::AnonymousFile,
    ) {
        // A kernel anonymous inode is not a pipe or another named anonymous file, its record is never opened, and a
        //  descriptor that closed while the scan walked the directory is open in nothing.
        crate::descriptor_record::Record::NotTheTarget | crate::descriptor_record::Record::Gone => {
            return Ok(false);
        }
        crate::descriptor_record::Record::Text(text) => text,
        crate::descriptor_record::Record::BeyondBound => {
            return Err(Unread::File(format!(
                "its record is longer than the {} bytes this scan takes, so the descriptor may hold the file",
                crate::descriptor_record::RECORD_BYTES_AT_MOST
            )));
        }
        crate::descriptor_record::Record::Unreadable {
            reason,
            ..
        } => return Err(Unread::Process(reason)),
        crate::descriptor_record::Record::KindUnknown {
            reason,
            ..
        } => {
            return Err(Unread::File(format!(
                "the text of its link cannot be read ({reason}), so the descriptor may hold the file"
            )));
        }
    };
    let reached: Option<u64> = crate::descriptor_record::mount_id_in(text.as_str());
    let held: Option<u64> = crate::descriptor_record::inode_in(text.as_str());
    return Ok(held == Some(inode) && reached == Some(mount) && mount != 0);
}

/// What:
///  The same on a platform that has no `/proc` to read a descriptor's record from.
/// Why:
///   The scan of another process's descriptors is Linux's,
///  and a platform without it answers "not this file" rather than promising something it cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.platform !== 'linux') return false;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn descriptor_is_file(
    process: &Path,
    fd: &std::ffi::OsStr,
    device: u64,
    inode: u64,
) -> Result<bool, Unread> {
    let _ = (process, fd, device, inode);
    return Ok(false);
}
