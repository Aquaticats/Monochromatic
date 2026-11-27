//! What:
//!  The read of one boot's readiness record:
//!  the directory opened and proved before a byte is read,
//!  the record's kind,
//!  owner,
//!  mode and length proved,
//!  and the bytes parsed by the route's own reader.
//! Why:
//!  The record admits or refuses an attach,
//!  and the design names readiness as a root-published fact a caller must be
//!  able to check before a mount
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!  Every read is relative to the proved directory's descriptor,
//!  so nothing below a replaced path is read,
//!  and the whole file is bounded before it is parsed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = readTheRecord(readinessDirectory, owner); // undefined when none was published
//! ```

use super::clone_mount_readiness_file_refusals::ReadinessFileRefusal;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    MAXIMUM_READINESS_BYTES,
    READINESS_DIRECTORY_MODE,
    READINESS_MODE,
    READINESS_NAME,
    ReadinessRecord,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_parse::parse_readiness;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_calls::{
    c_string_of,
    open_receipt_at,
};
use std::ffi::CString;
use std::io::Read;
use std::os::fd::{
    FromRawFd,
    OwnedFd,
};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  Read one boot's record from the directory at `directory`,
///  or nothing when none was published.
/// Why:
///   A directory that is not there is no record yet,
///  which is what a boot that has not run the handler answers.
///
/// # Errors
/// The directory or the record is not the designed root-owned file,
/// or the text is not a record this route wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readTheRecord(directory, owner); // ReadinessRecord | undefined
/// ```
pub(super) fn read_the_record(
    directory: &Path,
    owner: u32,
) -> Result<Option<ReadinessRecord>, ReadinessFileRefusal> {
    let opened: std::fs::File = match open_the_directory(directory, owner) {
        Ok(opened) => opened,
        Err(ReadinessFileRefusal::DirectoryAbsent {
            ..
        }) => return Ok(None),
        Err(refusal) => return Err(refusal),
    };
    let Some(file) = open_receipt_at(&opened, READINESS_NAME).map_err(|error| {
        return ReadinessFileRefusal::File {
            reason: error.to_string(),
        };
    })?
    else {
        return Ok(None);
    };
    let facts: std::fs::Metadata = file.metadata().map_err(|error| {
        return ReadinessFileRefusal::File {
            reason: error.to_string(),
        };
    })?;
    if !facts.is_file() {
        return Err(ReadinessFileRefusal::NotRegular);
    }
    if facts.uid() != owner {
        return Err(ReadinessFileRefusal::FileOwner {
            found: facts.uid(),
            wanted: owner,
        });
    }
    let permission_bits: u32 = facts.mode() & 0o7777;
    if permission_bits != READINESS_MODE {
        return Err(ReadinessFileRefusal::FileMode {
            found: permission_bits,
        });
    }
    if facts.len() > MAXIMUM_READINESS_BYTES as u64 {
        return Err(ReadinessFileRefusal::TooLarge {
            bytes: facts.len(),
        });
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(facts.len() as usize);
    let mut bounded: std::fs::File = file;
    Read::by_ref(&mut bounded)
        .take(MAXIMUM_READINESS_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            return ReadinessFileRefusal::File {
                reason: error.to_string(),
            };
        })?;
    if bytes.len() > MAXIMUM_READINESS_BYTES {
        return Err(ReadinessFileRefusal::TooLarge {
            bytes: bytes.len() as u64,
        });
    }
    let Ok(text) = String::from_utf8(bytes) else {
        return Err(ReadinessFileRefusal::NotText);
    };
    return match parse_readiness(text.as_str()) {
        Ok(record) => Ok(Some(record)),
        Err(refusal) => Err(ReadinessFileRefusal::NotAReadiness {
            refusal,
        }),
    };
}

/// What:
///  Open the directory at `directory` and prove it is the designed one.
/// Why:
///   Every later call is relative to this descriptor,
///  so nothing below a replaced path can be read or written.
///
/// # Errors
/// The directory is absent,
/// could not be opened,
/// or is not the designed root-owned folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const opened = openTheDirectory(directory, owner);
/// ```
fn open_the_directory(
    directory: &Path,
    owner: u32,
) -> Result<std::fs::File, ReadinessFileRefusal> {
    let text: CString = c_string_of(directory).map_err(|reason| {
        return ReadinessFileRefusal::Directory {
            path: directory.to_path_buf(),
            reason,
        };
    })?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call.
    let descriptor: libc::c_int = unsafe {
        libc::open(
            text.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if descriptor < 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::NotFound {
            return Err(ReadinessFileRefusal::DirectoryAbsent {
                path: directory.to_path_buf(),
            });
        }
        return Err(ReadinessFileRefusal::Directory {
            path: directory.to_path_buf(),
            reason: error.to_string(),
        });
    }
    // SAFETY: `open` returned a new descriptor that nothing else owns.
    let opened: std::fs::File = std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
    let facts: std::fs::Metadata = opened.metadata().map_err(|error| {
        return ReadinessFileRefusal::Directory {
            path: directory.to_path_buf(),
            reason: error.to_string(),
        };
    })?;
    if facts.uid() != owner {
        return Err(ReadinessFileRefusal::DirectoryOwner {
            path: directory.to_path_buf(),
            found: facts.uid(),
            wanted: owner,
        });
    }
    let permission_bits: u32 = facts.mode() & 0o7777;
    if permission_bits != READINESS_DIRECTORY_MODE {
        return Err(ReadinessFileRefusal::DirectoryMode {
            path: directory.to_path_buf(),
            found: permission_bits,
        });
    }
    return Ok(opened);
}
