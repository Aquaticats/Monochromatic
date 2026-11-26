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

use crate::worktree_clone_mount_protocol::clone_mount_receipt::{
    CloneMountReceipt,
    MAXIMUM_RECEIPT_BYTES,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipt_parse::parse_receipt;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_calls::{
    c_string_of,
    create_at,
    open_receipt_at,
    rename_at,
    sync_directory,
    unlink_at,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::{
    DIRECTORY_MODE,
    RECEIPT_MODE,
    RESULTS_DIRECTORY,
    ReceiptsRefusal,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::root_owned_directory::{
    RootDirectoryRefusal,
    make_and_prove,
};
use std::ffi::CString;
use std::io::{
    Read,
    Write,
};
use std::os::fd::{
    FromRawFd,
    OwnedFd,
};
use std::os::unix::fs::{
    MetadataExt,
    PermissionsExt,
};
use std::path::{
    Path,
    PathBuf,
};
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// The suffix of the sibling a receipt is staged under before its rename.
/// It is never a receipt and never read as one.
pub const STAGING_SUFFIX: &str = ".staging";

/// Distinguishes the staging names of two publishes one process makes,
/// so no publish ever shares another's sibling.
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

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
        return ReceiptsFolder {
            directory: PathBuf::from(RESULTS_DIRECTORY),
            owner: 0,
        };
    }

    /// A folder at a fixture directory,
    /// owned by the account the control expects.
    pub fn at(
        directory: PathBuf,
        owner: u32,
    ) -> Self {
        return ReceiptsFolder {
            directory,
            owner,
        };
    }

    /// The directory.
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// The file one dataset's receipt is filed under:
    /// its canonical numeric GUID,
    /// and nothing else.
    pub fn entry_name(
        &self,
        dataset: DatasetGuid,
    ) -> String {
        return dataset.file_name();
    }

    /// What:
    ///  The sibling one publish of a receipt is staged under before its rename.
    /// Why:
    ///   The name carries this process's id and a counter of this process's own
    ///  attempts,
    ///  so two publishes in one process never share a staging name (a shared name
    ///  lets the second publish remove the first one's live sibling and rename a
    ///  half-written file into the receipt's name),
    ///  and a writer killed before its rename leaves a name no later attempt
    ///  reuses and no receipt is ever read from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const staging = `${guid}.staging.${process.pid}.${nextAttempt()}`;
    /// ```
    pub fn staging_name(
        &self,
        dataset: DatasetGuid,
    ) -> String {
        let attempt: u64 = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
        return format!(
            "{}{STAGING_SUFFIX}.{}.{attempt}",
            dataset.file_name(),
            std::process::id()
        );
    }

    /// What:
    /// Make the directory when it is absent,
    /// give it its designed mode,
    /// and prove it is the designed root-owned folder.
    ///
    /// # Errors
    /// The directory could not be made or given its mode,
    /// or it is owned by another account.
    pub fn prepare(&self) -> Result<(), ReceiptsRefusal> {
        let directory: std::fs::File = self.prepare_directory()?;
        drop(directory);
        return Ok(());
    }

    ///  File one receipt,
    ///  replacing the dataset's own earlier one whole.
    /// Why:
    ///   The directory is made when a fresh boot has none,
    ///  the text is proved readable by the route's own reader before a byte is
    ///  written,
    ///  and the staging sibling is written and synced first,
    ///  so the GUID's name always holds one whole receipt,
    ///  and a live clone's next result overwrites its own entry as the design
    ///  says.
    ///  Every way out after the staging sibling was made removes it,
    ///  so this process's next publish is never blocked by its own failure,
    ///  and a sibling an earlier run of this process left is replaced rather
    ///  than waited on.
    ///
    /// # Errors
    /// The directory is not the designed root-owned `0755` folder,
    /// the text is not one this route's reader accepts,
    /// or the staging write,
    /// its sync or the rename answered an error.
    pub fn publish(
        &self,
        receipt: &CloneMountReceipt,
    ) -> Result<(), ReceiptsRefusal> {
        let directory: std::fs::File = self.prepare_directory()?;
        let name: String = self.entry_name(receipt.dataset);
        let text: String = receipt.text();
        match parse_receipt(text.as_str()) {
            Ok(parsed) if parsed == *receipt => {}
            Ok(_) => {
                return Err(ReceiptsRefusal::NotWhatWasWritten {
                    name,
                });
            }
            Err(refusal) => {
                return Err(ReceiptsRefusal::NotAReceipt {
                    name,
                    refusal,
                });
            }
        }
        let staging: String = self.staging_name(receipt.dataset);
        let staged: Result<(), ReceiptsRefusal> = (|| {
            let mut file: std::fs::File = create_at(&directory, staging.as_str(), RECEIPT_MODE)
                .map_err(|error| {
                    return ReceiptsRefusal::Staging {
                        name: staging.clone(),
                        reason: error.to_string(),
                    };
                })?;
            file.set_permissions(std::fs::Permissions::from_mode(RECEIPT_MODE))
                .map_err(|error| {
                    return ReceiptsRefusal::Staging {
                        name: staging.clone(),
                        reason: error.to_string(),
                    };
                })?;
            file.write_all(text.as_bytes()).map_err(|error| {
                return ReceiptsRefusal::Staging {
                    name: staging.clone(),
                    reason: error.to_string(),
                };
            })?;
            file.sync_all().map_err(|error| {
                return ReceiptsRefusal::Staging {
                    name: staging.clone(),
                    reason: error.to_string(),
                };
            })?;
            rename_at(&directory, staging.as_str(), name.as_str()).map_err(|error| {
                return ReceiptsRefusal::Staging {
                    name: staging.clone(),
                    reason: error.to_string(),
                };
            })?;
            return sync_directory(&directory).map_err(|error| {
                return ReceiptsRefusal::Staging {
                    name: name.clone(),
                    reason: error.to_string(),
                };
            });
        })();
        if staged.is_err() {
            // A rename may have completed before a failed directory sync, and then there is no sibling to remove; a
            // removal's own failure changes nothing about the refusal being reported.
            let _removed: std::io::Result<()> = unlink_at(&directory, staging.as_str());
        }
        return staged;
    }

    /// Make the directory and prove it,
    /// handing back the descriptor every later call is relative to.
    fn prepare_directory(&self) -> Result<std::fs::File, ReceiptsRefusal> {
        let path: &Path = self.directory.as_path();
        return make_and_prove(path, self.owner, DIRECTORY_MODE).map_err(|refusal| {
            return match refusal {
                RootDirectoryRefusal::Open {
                    reason,
                } => ReceiptsRefusal::Directory {
                    path: path.to_path_buf(),
                    reason,
                },
                RootDirectoryRefusal::Owner {
                    found,
                    wanted,
                } => ReceiptsRefusal::DirectoryOwner {
                    path: path.to_path_buf(),
                    found,
                    wanted,
                },
                RootDirectoryRefusal::Mode {
                    found,
                    wanted: _wanted,
                } => ReceiptsRefusal::DirectoryMode {
                    path: path.to_path_buf(),
                    found,
                },
            };
        });
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
    pub fn read(
        &self,
        dataset: DatasetGuid,
    ) -> Result<Option<CloneMountReceipt>, ReceiptsRefusal> {
        let directory: std::fs::File = self.open_directory()?;
        let name: String = self.entry_name(dataset);
        let Some(file) = open_receipt_at(&directory, name.as_str()).map_err(|error| {
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
        if facts.uid() != self.owner {
            return Err(ReceiptsRefusal::FileOwner {
                name,
                found: facts.uid(),
                wanted: self.owner,
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
        let text: CString = c_string_of(&self.directory).map_err(|reason| {
            return ReceiptsRefusal::Directory {
                path: path.to_path_buf(),
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
                    path: path.to_path_buf(),
                });
            }
            return Err(ReceiptsRefusal::Directory {
                path: path.to_path_buf(),
                reason: error.to_string(),
            });
        }
        // SAFETY: `open` returned a new descriptor that nothing else owns.
        let directory: std::fs::File =
            std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
        let facts: std::fs::Metadata = directory.metadata().map_err(|error| {
            return ReceiptsRefusal::Directory {
                path: path.to_path_buf(),
                reason: error.to_string(),
            };
        })?;
        if facts.uid() != self.owner {
            return Err(ReceiptsRefusal::DirectoryOwner {
                path: path.to_path_buf(),
                found: facts.uid(),
                wanted: self.owner,
            });
        }
        let permission_bits: u32 = facts.mode() & 0o7777;
        if permission_bits != DIRECTORY_MODE {
            return Err(ReceiptsRefusal::DirectoryMode {
                path: path.to_path_buf(),
                found: permission_bits,
            });
        }
        return Ok(directory);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_receipts_folder_tests.rs"]
mod tests;
