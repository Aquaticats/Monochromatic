//! What:
//!  The directory the root handler files clone-mount results in:
//!  its path and modes,
//!  the file each dataset GUID names,
//!  its atomic publish,
//!  and its read through the directory's own descriptor.
//! Why:
//!  The design publishes receipts in `/run/git-worktree-clone-mount-results`,
//!  named only by the canonical numeric dataset GUID,
//!  with a root-owned `0755` directory and root-owned `0644` receipts,
//!  opened relative to that directory
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  "Decided defaults").
//!  A reader therefore trusts the directory before any file in it:
//!  the directory and the receipt must both be owned by root and carry their
//!  modes,
//!  the receipt is opened with no link followed,
//!  and every byte is bounded.
//!  A staging sibling is filled and renamed over the receipt,
//!  so a caller reads the whole earlier answer or the whole new one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = ReceiptsFolder.live(); folder.publish(guid, receipt); folder.read(guid);
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_receipt::{CloneMountReceipt, MAXIMUM_RECEIPT_BYTES};
use crate::worktree_clone_mount_protocol::clone_mount_receipt_parse::parse_receipt;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_calls::{c_string_of, create_at, open_receipt_at, rename_at, sync_directory};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::{DIRECTORY_MODE, RECEIPT_MODE, RESULTS_DIRECTORY, ReceiptsRefusal};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use std::ffi::CString;
use std::io::{Read, Write};
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// The suffix of the sibling a receipt is staged under before its rename.
/// It is never a receipt and never read as one.
pub const STAGING_SUFFIX: &str = ".staging";

/// What:
///  Where receipts live and which account must own them.
/// Why:
///   The live folder requires root,
///  and a control hands in a fixture directory it owns,
///  so the same reads and writes are exercised without `/run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReceiptsFolder { constructor(readonly directory: string, readonly owner: number) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptsFolder {
    /// The directory the receipts live in.
    directory: PathBuf,
    /// The account every directory and receipt must belong to.
    owner: u32,
}

impl ReceiptsFolder {
    /// The folder the root handler files in.
    pub fn live() -> Self {
        return ReceiptsFolder { directory: PathBuf::from(RESULTS_DIRECTORY), owner: 0 };
    }

    /// A folder at a fixture directory,
    /// owned by the account the control expects.
    pub fn at(directory: PathBuf, owner: u32) -> Self {
        return ReceiptsFolder { directory, owner };
    }

    /// The directory.
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// The file one dataset's receipt is filed under:
    /// its canonical numeric GUID, and nothing else.
    pub fn entry_name(&self, dataset: DatasetGuid) -> String {
        return dataset.file_name();
    }

    /// The sibling a receipt is staged under before its rename.
    pub fn staging_name(&self, dataset: DatasetGuid) -> String {
        return format!("{}{STAGING_SUFFIX}", dataset.file_name());
    }

    /// What:
    ///  File one receipt,
    ///  replacing the dataset's own earlier one whole.
    /// Why:
    ///   The staging sibling is written and synced first,
    ///  so the GUID's name always holds one whole receipt,
    ///  and a live clone's next result overwrites its own entry as the design
    ///  says.
    ///
    /// # Errors
    /// The directory is not the designed root-owned `0755` folder,
    /// or the staging write,
    /// its sync or the rename answered an error.
    pub fn publish(&self, receipt: &CloneMountReceipt) -> Result<(), ReceiptsRefusal> {
        let directory: std::fs::File = self.open_directory()?;
        let name: String = self.entry_name(receipt.dataset);
        let staging: String = self.staging_name(receipt.dataset);
        let staged: Result<(), ReceiptsRefusal> = (|| {
            let mut file: std::fs::File = create_at(&directory, staging.as_str(), RECEIPT_MODE).map_err(|error| {
                return ReceiptsRefusal::Staging { name: staging.clone(), reason: error.to_string() };
            })?;
            file.set_permissions(std::fs::Permissions::from_mode(RECEIPT_MODE)).map_err(|error| {
                return ReceiptsRefusal::Staging { name: staging.clone(), reason: error.to_string() };
            })?;
            file.write_all(receipt.text().as_bytes()).map_err(|error| {
                return ReceiptsRefusal::Staging { name: staging.clone(), reason: error.to_string() };
            })?;
            file.sync_all().map_err(|error| {
                return ReceiptsRefusal::Staging { name: staging.clone(), reason: error.to_string() };
            })?;
            rename_at(&directory, staging.as_str(), name.as_str()).map_err(|error| {
                return ReceiptsRefusal::Staging { name: staging.clone(), reason: error.to_string() };
            })?;
            return sync_directory(&directory).map_err(|error| return ReceiptsRefusal::Staging { name: name.clone(), reason: error.to_string() });
        })();
        return staged;
    }

    /// What:
    ///  Read one dataset's receipt,
    ///  or nothing when it has none.
    /// Why:
    ///   The directory and the file are checked before their bytes are read,
    ///  and the answer is the parsed receipt or a refusal,
    ///  never a half-read file.
    ///
    /// # Errors
    /// The directory is not the designed root-owned `0755` folder,
    /// or the file is not a root-owned `0644` regular receipt.
    pub fn read(&self, dataset: DatasetGuid) -> Result<Option<CloneMountReceipt>, ReceiptsRefusal> {
        let directory: std::fs::File = self.open_directory()?;
        let name: String = self.entry_name(dataset);
        let Some(file) = open_receipt_at(&directory, name.as_str())
            .map_err(|error| return ReceiptsRefusal::File { name: name.clone(), reason: error.to_string() })?
        else {
            return Ok(None);
        };
        let facts: std::fs::Metadata = file.metadata().map_err(|error| return ReceiptsRefusal::File { name: name.clone(), reason: error.to_string() })?;
        if !facts.is_file() {
            return Err(ReceiptsRefusal::NotRegular { name });
        }
        if facts.uid() != self.owner {
            return Err(ReceiptsRefusal::FileOwner { name, found: facts.uid(), wanted: self.owner });
        }
        let permission_bits: u32 = facts.mode() & 0o777;
        if permission_bits != RECEIPT_MODE {
            return Err(ReceiptsRefusal::FileMode { name, found: permission_bits });
        }
        if facts.len() > MAXIMUM_RECEIPT_BYTES as u64 {
            return Err(ReceiptsRefusal::TooLarge { name, bytes: facts.len() });
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(facts.len() as usize);
        let mut bounded: std::fs::File = file;
        Read::by_ref(&mut bounded).take(MAXIMUM_RECEIPT_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|error| {
            return ReceiptsRefusal::File { name: name.clone(), reason: error.to_string() };
        })?;
        if bytes.len() > MAXIMUM_RECEIPT_BYTES {
            return Err(ReceiptsRefusal::TooLarge { name, bytes: bytes.len() as u64 });
        }
        let Ok(text) = String::from_utf8(bytes) else {
            return Err(ReceiptsRefusal::NotText { name });
        };
        return match parse_receipt(text.as_str()) {
            Ok(receipt) => Ok(Some(receipt)),
            Err(refusal) => Err(ReceiptsRefusal::NotAReceipt { name, refusal }),
        };
    }

    /// What:
    ///  Open the directory and prove it is the designed one.
    /// Why:
    ///   Every later call is relative to this descriptor,
    ///  so nothing below a replaced path can be read or written.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const directory = openTrusted(resultsDirectory, owner);
    /// ```
    fn open_directory(&self) -> Result<std::fs::File, ReceiptsRefusal> {
        let path: &Path = self.directory.as_path();
        let text: CString = c_string_of(&self.directory).map_err(|reason| return ReceiptsRefusal::Directory { path: path.to_path_buf(), reason })?;
        // SAFETY: `text` is a NUL-terminated string that outlives the call.
        let descriptor: libc::c_int = unsafe { libc::open(text.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
        if descriptor < 0 {
            return Err(ReceiptsRefusal::Directory { path: path.to_path_buf(), reason: std::io::Error::last_os_error().to_string() });
        }
        // SAFETY: `open` returned a new descriptor that nothing else owns.
        let directory: std::fs::File = std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
        let facts: std::fs::Metadata = directory
            .metadata()
            .map_err(|error| return ReceiptsRefusal::Directory { path: path.to_path_buf(), reason: error.to_string() })?;
        if facts.uid() != self.owner {
            return Err(ReceiptsRefusal::DirectoryOwner { path: path.to_path_buf(), found: facts.uid(), wanted: self.owner });
        }
        let permission_bits: u32 = facts.mode() & 0o777;
        if permission_bits != DIRECTORY_MODE {
            return Err(ReceiptsRefusal::DirectoryMode { path: path.to_path_buf(), found: permission_bits });
        }
        return Ok(directory);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_receipts_folder_tests.rs"]
mod tests;
