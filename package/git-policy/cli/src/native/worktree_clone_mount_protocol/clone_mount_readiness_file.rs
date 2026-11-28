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
//!  This file holds the directory and the record's own name,
//!  and the refusals,
//!  the write,
//!  the read and the withdrawal each sit in the file named for the job they
//!  do,
//!  so a reader can hold one job at a time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const file = ReadinessFile.live(); file.publish(record); file.read();
//! ```

/// The staging name one publish writes under,
/// and the write of one whole record.
#[path = "clone_mount_readiness_file_publish.rs"]
mod clone_mount_readiness_file_publish;
/// The directory opened and proved,
/// and the read of one whole record.
#[path = "clone_mount_readiness_file_read.rs"]
mod clone_mount_readiness_file_read;
/// Why the readiness directory or its record could not be used.
#[path = "clone_mount_readiness_file_refusals.rs"]
mod clone_mount_readiness_file_refusals;

use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    READINESS_DIRECTORY,
    READINESS_DIRECTORY_MODE,
    READINESS_NAME,
    ReadinessRecord,
};
use crate::worktree_clone_mount_protocol::root_owned_directory::{
    RootDirectoryRefusal,
    make_and_prove,
};
use std::path::{
    Path,
    PathBuf,
};

/// The write of one whole record,
/// behind the directory already proved.
use self::clone_mount_readiness_file_publish::write_the_record;
/// The staging name one publish writes under.
pub use self::clone_mount_readiness_file_publish::{
    READINESS_STAGING_NAME,
    readiness_staging_name,
};
/// The read of one whole record,
/// behind the directory already proved.
use self::clone_mount_readiness_file_read::read_the_record;
/// The refusal the directory,
/// the write and the read answer with.
pub use self::clone_mount_readiness_file_refusals::ReadinessFileRefusal;
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
        return ReadinessFile {
            directory: PathBuf::from(READINESS_DIRECTORY),
            owner: 0,
        };
    }

    /// A place at a fixture directory,
    /// owned by the account the control expects.
    pub fn at(
        directory: PathBuf,
        owner: u32,
    ) -> Self {
        return ReadinessFile {
            directory,
            owner,
        };
    }

    /// The directory.
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// What:
    ///  The account this record's directory and file must belong to.
    /// Why:
    ///   The withdrawal proves a directory and removes a name inside it,
    ///  and only this file object knows which account the route requires there,
    ///  so it is asked rather than a second constant that could drift from it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// file.owner(); // 0 at the installed place
    /// ```
    pub(crate) fn owner(&self) -> u32 {
        return self.owner;
    }

    /// The record's own path.
    pub fn record(&self) -> PathBuf {
        return self.directory.join(READINESS_NAME);
    }

    /// The sibling a record is staged under.
    pub fn staging(&self) -> PathBuf {
        return self.directory.join(readiness_staging_name());
    }

    /// Make the directory when it is absent,
    /// give it its designed mode,
    /// and prove it is the designed root-owned folder.
    ///
    /// # Errors
    /// The directory could not be made or given its mode,
    /// or it is owned by another account.
    pub fn prepare(&self) -> Result<(), ReadinessFileRefusal> {
        let directory: std::fs::File = self.prepare_directory()?;
        drop(directory);
        return Ok(());
    }

    /// Publish one whole record,
    /// replacing the earlier one atomically.
    /// Why:
    ///   The directory is made when a fresh boot has none,
    ///  the text is proved readable by the route's own reader before a byte is
    ///  written,
    ///  and every way out after the staging sibling was made removes it,
    ///  so this process's next publish is never blocked by its own failure.
    ///
    /// # Errors
    /// The directory is not the designed root-owned `0755` folder,
    /// the text is not one this route's reader accepts,
    /// or the staging write,
    /// its sync or the rename answered an error.
    pub fn publish(
        &self,
        record: &ReadinessRecord,
    ) -> Result<(), ReadinessFileRefusal> {
        let directory: std::fs::File = self.prepare_directory()?;
        return write_the_record(&directory, readiness_staging_name().as_str(), record);
    }

    /// Make the directory and prove it,
    /// handing back the descriptor every later call is relative to.
    fn prepare_directory(&self) -> Result<std::fs::File, ReadinessFileRefusal> {
        let path: &Path = self.directory.as_path();
        return make_and_prove(path, self.owner, READINESS_DIRECTORY_MODE).map_err(|refusal| {
            return match refusal {
                RootDirectoryRefusal::Open {
                    reason,
                } => ReadinessFileRefusal::Directory {
                    path: path.to_path_buf(),
                    reason,
                },
                RootDirectoryRefusal::Owner {
                    found,
                    wanted,
                } => ReadinessFileRefusal::DirectoryOwner {
                    path: path.to_path_buf(),
                    found,
                    wanted,
                },
                RootDirectoryRefusal::Mode {
                    found,
                    wanted: _wanted,
                } => ReadinessFileRefusal::DirectoryMode {
                    path: path.to_path_buf(),
                    found,
                },
            };
        });
    }

    /// Read this boot's record,
    /// or nothing when none was published.
    /// A directory that is not there is no record yet,
    /// which is what a boot that has not run the handler answers.
    ///
    /// # Errors
    /// The directory or the record is not the designed root-owned file,
    /// or the text is not a record this route wrote.
    pub fn read(&self) -> Result<Option<ReadinessRecord>, ReadinessFileRefusal> {
        return read_the_record(self.directory.as_path(), self.owner);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_readiness_file_tests.rs"]
mod tests;
