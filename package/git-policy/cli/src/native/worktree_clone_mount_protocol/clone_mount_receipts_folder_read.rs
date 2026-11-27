//! What:
//!  The read of one dataset's receipt:
//!  the directory opened and proved before a byte is read,
//!  the GUID's file opened with no link followed,
//!  its kind,
//!  owner,
//!  mode and length proved,
//!  and the bytes parsed by the route's own reader.
//! Why:
//!  A reader trusts the directory before any file in it,
//!  so the directory and the receipt are both proved root-owned and of their
//!  modes before the receipt's text is believed
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  "Decided defaults").
//!  A receipt named by another dataset is refused rather than returned,
//!  so a caller never reads one clone's answer as another's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const receipt = readAReceipt(resultsDirectory, owner, guid); // undefined when none was filed
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_receipt::{
    CloneMountReceipt,
    MAXIMUM_RECEIPT_BYTES,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipt_parse::parse_receipt;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_calls::{
    c_string_of,
    open_receipt_at,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::{
    DIRECTORY_MODE,
    RECEIPT_MODE,
    ReceiptsRefusal,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use std::ffi::CString;
use std::io::Read;
use std::os::fd::{
    FromRawFd,
    OwnedFd,
};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  Read one dataset's receipt from the directory at `directory`,
///  or nothing when it has none.
/// Why:
///   The directory and the file are checked before their bytes are read,
///  and the answer is the parsed receipt or a refusal,
///  never a half-read file.
///
/// # Errors
/// The directory is not the designed root-owned `0755` folder,
/// or the file is not a root-owned `0644` regular receipt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readAReceipt(directory, owner, dataset); // CloneMountReceipt | undefined
/// ```
pub(super) fn read_a_receipt(
    directory: &Path,
    owner: u32,
    dataset: DatasetGuid,
) -> Result<Option<CloneMountReceipt>, ReceiptsRefusal> {
    let opened: std::fs::File = open_the_directory(directory, owner)?;
    let name: String = dataset.file_name();
    let Some(file) = open_receipt_at(&opened, name.as_str()).map_err(|error| {
        return ReceiptsRefusal::File {
            name: name.clone(),
            reason: error.to_string(),
        };
    })?
    else {
        return Ok(None);
    };
    let facts: std::fs::Metadata = file.metadata().map_err(|error| {
        return ReceiptsRefusal::File {
            name: name.clone(),
            reason: error.to_string(),
        };
    })?;
    if !facts.is_file() {
        return Err(ReceiptsRefusal::NotRegular {
            name,
        });
    }
    if facts.uid() != owner {
        return Err(ReceiptsRefusal::FileOwner {
            name,
            found: facts.uid(),
            wanted: owner,
        });
    }
    let permission_bits: u32 = facts.mode() & 0o7777;
    if permission_bits != RECEIPT_MODE {
        return Err(ReceiptsRefusal::FileMode {
            name,
            found: permission_bits,
        });
    }
    if facts.len() > MAXIMUM_RECEIPT_BYTES as u64 {
        return Err(ReceiptsRefusal::TooLarge {
            name,
            bytes: facts.len(),
        });
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(facts.len() as usize);
    let mut bounded: std::fs::File = file;
    Read::by_ref(&mut bounded)
        .take(MAXIMUM_RECEIPT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            return ReceiptsRefusal::File {
                name: name.clone(),
                reason: error.to_string(),
            };
        })?;
    if bytes.len() > MAXIMUM_RECEIPT_BYTES {
        return Err(ReceiptsRefusal::TooLarge {
            name,
            bytes: bytes.len() as u64,
        });
    }
    let Ok(text) = String::from_utf8(bytes) else {
        return Err(ReceiptsRefusal::NotText {
            name,
        });
    };
    return match parse_receipt(text.as_str()) {
        Ok(receipt) if receipt.dataset == dataset => Ok(Some(receipt)),
        Ok(receipt) => Err(ReceiptsRefusal::WrongDataset {
            name,
            found: receipt.dataset.file_name(),
        }),
        Err(refusal) => Err(ReceiptsRefusal::NotAReceipt {
            name,
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
/// The directory could not be opened,
/// or is not the designed root-owned folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = openTrusted(resultsDirectory, owner);
/// ```
fn open_the_directory(
    directory: &Path,
    owner: u32,
) -> Result<std::fs::File, ReceiptsRefusal> {
    let text: CString = c_string_of(directory).map_err(|reason| {
        return ReceiptsRefusal::Directory {
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
            return Err(ReceiptsRefusal::DirectoryAbsent {
                path: directory.to_path_buf(),
            });
        }
        return Err(ReceiptsRefusal::Directory {
            path: directory.to_path_buf(),
            reason: error.to_string(),
        });
    }
    // SAFETY: `open` returned a new descriptor that nothing else owns.
    let opened: std::fs::File = std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
    let facts: std::fs::Metadata = opened.metadata().map_err(|error| {
        return ReceiptsRefusal::Directory {
            path: directory.to_path_buf(),
            reason: error.to_string(),
        };
    })?;
    if facts.uid() != owner {
        return Err(ReceiptsRefusal::DirectoryOwner {
            path: directory.to_path_buf(),
            found: facts.uid(),
            wanted: owner,
        });
    }
    let permission_bits: u32 = facts.mode() & 0o7777;
    if permission_bits != DIRECTORY_MODE {
        return Err(ReceiptsRefusal::DirectoryMode {
            path: directory.to_path_buf(),
            found: permission_bits,
        });
    }
    return Ok(opened);
}
