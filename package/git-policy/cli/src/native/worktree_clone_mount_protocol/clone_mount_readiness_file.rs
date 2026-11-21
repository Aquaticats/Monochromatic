//! What:
//!  Where one boot's readiness lives and how it is written:
//!  a root-owned `0755` directory under `/run`,
//!  its `result` file at `0644`,
//!  the atomic publish of a whole record,
//!  and the read that proves the directory and the file before either is
//!  believed.
//! Why:
//!  The record admits or refuses an attach,
//!  and the design names readiness as a root-published fact a caller must be
//!  able to check before a mount
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety",
//!  "Requests,
//! results and ordering").
//!  It sits in a directory of its own rather than beside the receipts,
//!  whose file names are only dataset GUIDs,
//!  and the record is a cache that a caller compares with live facts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const file = ReadinessFile.live(); file.publish(record); file.read();
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    MAXIMUM_READINESS_BYTES, READINESS_DIRECTORY, READINESS_DIRECTORY_MODE, READINESS_MODE, READINESS_NAME, ReadinessRecord,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_parse::{ReadinessParseRefusal, parse_readiness};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_calls::{c_string_of, create_at, open_receipt_at, rename_at, sync_directory};
use std::ffi::CString;
use std::io::{Read, Write};
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// The sibling a record is staged under before its rename.
pub const READINESS_STAGING_NAME: &str = "result.staging";

/// What:
///  Why the readiness directory or its record could not be used.
/// Why:
///   The caller admits an attach on this file,
///  so each deviation names the path and says nothing was attached on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadinessFileRefusal = { kind: 'directory' | 'owner' | 'mode' | 'file' | 'shape' | 'staging'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadinessFileRefusal {
    /// The directory could not be opened,
    /// or is not a directory.
    Directory {
        /// The path.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
    /// The directory is not owned by the expected account.
    DirectoryOwner {
        /// The path.
        path: PathBuf,
        /// The owner found.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
    /// The directory does not carry its designed mode.
    DirectoryMode {
        /// The path.
        path: PathBuf,
        /// The mode found.
        found: u32,
    },
    /// The record could not be opened or written.
    File {
        /// What the call answered.
        reason: String,
    },
    /// The record is not owned by the expected account.
    FileOwner {
        /// The owner found.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
    /// The record does not carry its designed mode.
    FileMode {
        /// The mode found.
        found: u32,
    },
    /// The name does not hold a regular file.
    NotRegular,
    /// The record is longer than a record may be.
    TooLarge {
        /// How many bytes the file holds.
        bytes: u64,
    },
    /// The record's bytes are not text.
    NotText,
    /// The text is not a readiness record this route wrote.
    NotAReadiness {
        /// What the parse refused.
        refusal: ReadinessParseRefusal,
    },
    /// The staging sibling could not be made or renamed.
    Staging {
        /// What the call answered.
        reason: String,
    },
}

impl ReadinessFileRefusal {
    /// The refusal in words,
    /// naming the path and the way forward.
    pub fn message(&self) -> String {
        let reason: String = match self {
            ReadinessFileRefusal::Directory { path, reason } => format!("the readiness directory {} could not be opened ({reason})", path.display()),
            ReadinessFileRefusal::DirectoryOwner { path, found, wanted } => format!(
                "the readiness directory {} is owned by user {found}, and this route requires user {wanted}",
                path.display()
            ),
            ReadinessFileRefusal::DirectoryMode { path, found } => format!(
                "the readiness directory {} has mode {found:04o}, and this route requires {READINESS_DIRECTORY_MODE:04o}",
                path.display()
            ),
            ReadinessFileRefusal::File { reason } => format!("the readiness record could not be opened or written ({reason})"),
            ReadinessFileRefusal::FileOwner { found, wanted } => {
                format!("the readiness record is owned by user {found}, and this route requires user {wanted}")
            }
            ReadinessFileRefusal::FileMode { found } => format!("the readiness record has mode {found:04o}, and this route requires {READINESS_MODE:04o}"),
            ReadinessFileRefusal::NotRegular => String::from("the readiness name does not hold a regular file"),
            ReadinessFileRefusal::TooLarge { bytes } => {
                format!("the readiness record is {bytes} bytes and a record is at most {MAXIMUM_READINESS_BYTES}")
            }
            ReadinessFileRefusal::NotText => String::from("the readiness record is not UTF-8 text"),
            ReadinessFileRefusal::NotAReadiness { refusal } => return refusal.message(),
            ReadinessFileRefusal::Staging { reason } => {
                format!("the sibling a readiness record is staged under could not be made or renamed ({reason}); the earlier record is unchanged")
            }
        };
        return format!(
            "{reason}. Nothing was attached, mounted or unmounted on this record, because a caller admits an attach only on a record it can prove root published. Ways forward: run the handler's `boot-reconcile` in this boot and read `journalctl -u reconcile-git-worktree-clone-mounts.service`; a wrong owner or mode needs an administrator, since this route never changes either."
        );
    }
}

/// What:
///  Where one boot's readiness lives and which account must own it.
/// Why:
///   The live place requires root,
///  and a control hands in a fixture directory it owns,
///  so the publish and the read are exercised without `/run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReadinessFile { constructor(readonly directory: string, readonly owner: number) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessFile {
    /// The directory the record lives in.
    directory: PathBuf,
    /// The account the directory and the record must belong to.
    owner: u32,
}

impl ReadinessFile {
    /// The place the release handler publishes in.
    pub fn live() -> Self {
        return ReadinessFile { directory: PathBuf::from(READINESS_DIRECTORY), owner: 0 };
    }

    /// A place at a fixture directory,
    /// owned by the account the control expects.
    pub fn at(directory: PathBuf, owner: u32) -> Self {
        return ReadinessFile { directory, owner };
    }

    /// The directory.
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// The record's own path.
    pub fn record(&self) -> PathBuf {
        return self.directory.join(READINESS_NAME);
    }

    /// The sibling a record is staged under.
    pub fn staging(&self) -> PathBuf {
        return self.directory.join(READINESS_STAGING_NAME);
    }

    /// Publish one whole record,
    /// replacing the earlier one atomically.
    ///
    /// # Errors
    /// The directory is not the designed root-owned `0755` folder,
    /// or the staging write,
    /// its sync or the rename answered an error.
    pub fn publish(&self, record: &ReadinessRecord) -> Result<(), ReadinessFileRefusal> {
        let directory: std::fs::File = self.open_directory()?;
        let staged: Result<(), ReadinessFileRefusal> = (|| {
            let mut file: std::fs::File =
                create_at(&directory, READINESS_STAGING_NAME, READINESS_MODE).map_err(|error| return ReadinessFileRefusal::Staging { reason: error.to_string() })?;
            file.set_permissions(std::fs::Permissions::from_mode(READINESS_MODE))
                .map_err(|error| return ReadinessFileRefusal::Staging { reason: error.to_string() })?;
            file.write_all(record.text().as_bytes())
                .map_err(|error| return ReadinessFileRefusal::Staging { reason: error.to_string() })?;
            file.sync_all().map_err(|error| return ReadinessFileRefusal::Staging { reason: error.to_string() })?;
            rename_at(&directory, READINESS_STAGING_NAME, READINESS_NAME)
                .map_err(|error| return ReadinessFileRefusal::Staging { reason: error.to_string() })?;
            return sync_directory(&directory).map_err(|error| return ReadinessFileRefusal::Staging { reason: error.to_string() });
        })();
        return staged;
    }

    /// Read this boot's record,
    /// or nothing when none was published.
    ///
    /// # Errors
    /// The directory or the record is not the designed root-owned file,
    /// or the text is not a record this route wrote.
    pub fn read(&self) -> Result<Option<ReadinessRecord>, ReadinessFileRefusal> {
        let directory: std::fs::File = self.open_directory()?;
        let Some(file) = open_receipt_at(&directory, READINESS_NAME).map_err(|error| return ReadinessFileRefusal::File { reason: error.to_string() })?
        else {
            return Ok(None);
        };
        let facts: std::fs::Metadata = file.metadata().map_err(|error| return ReadinessFileRefusal::File { reason: error.to_string() })?;
        if !facts.is_file() {
            return Err(ReadinessFileRefusal::NotRegular);
        }
        if facts.uid() != self.owner {
            return Err(ReadinessFileRefusal::FileOwner { found: facts.uid(), wanted: self.owner });
        }
        let permission_bits: u32 = facts.mode() & 0o777;
        if permission_bits != READINESS_MODE {
            return Err(ReadinessFileRefusal::FileMode { found: permission_bits });
        }
        if facts.len() > MAXIMUM_READINESS_BYTES as u64 {
            return Err(ReadinessFileRefusal::TooLarge { bytes: facts.len() });
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(facts.len() as usize);
        let mut bounded: std::fs::File = file;
        Read::by_ref(&mut bounded).take(MAXIMUM_READINESS_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|error| {
            return ReadinessFileRefusal::File { reason: error.to_string() };
        })?;
        if bytes.len() > MAXIMUM_READINESS_BYTES {
            return Err(ReadinessFileRefusal::TooLarge { bytes: bytes.len() as u64 });
        }
        let Ok(text) = String::from_utf8(bytes) else {
            return Err(ReadinessFileRefusal::NotText);
        };
        return match parse_readiness(text.as_str()) {
            Ok(record) => Ok(Some(record)),
            Err(refusal) => Err(ReadinessFileRefusal::NotAReadiness { refusal }),
        };
    }

    /// Open the directory and prove it is the designed one.
    fn open_directory(&self) -> Result<std::fs::File, ReadinessFileRefusal> {
        let path: &Path = self.directory.as_path();
        let text: CString =
            c_string_of(&self.directory).map_err(|reason| return ReadinessFileRefusal::Directory { path: path.to_path_buf(), reason })?;
        // SAFETY: `text` is a NUL-terminated string that outlives the call.
        let descriptor: libc::c_int = unsafe { libc::open(text.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
        if descriptor < 0 {
            return Err(ReadinessFileRefusal::Directory { path: path.to_path_buf(), reason: std::io::Error::last_os_error().to_string() });
        }
        // SAFETY: `open` returned a new descriptor that nothing else owns.
        let directory: std::fs::File = std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
        let facts: std::fs::Metadata =
            directory.metadata().map_err(|error| return ReadinessFileRefusal::Directory { path: path.to_path_buf(), reason: error.to_string() })?;
        if facts.uid() != self.owner {
            return Err(ReadinessFileRefusal::DirectoryOwner { path: path.to_path_buf(), found: facts.uid(), wanted: self.owner });
        }
        let permission_bits: u32 = facts.mode() & 0o777;
        if permission_bits != READINESS_DIRECTORY_MODE {
            return Err(ReadinessFileRefusal::DirectoryMode { path: path.to_path_buf(), found: permission_bits });
        }
        return Ok(directory);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_readiness_file_tests.rs"]
mod tests;
