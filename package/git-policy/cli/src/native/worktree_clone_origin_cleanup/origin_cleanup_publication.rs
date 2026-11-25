//! What:
//!  The origin cleanup folder's creation,
//!  the one write a removal or a failed add makes into it,
//!  and the one removal that ends a record.
//! Why:
//!  The record is published before the removal's first destroy request,
//!  because it is what carries the cleanup past the clone
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals",
//!  step 6),
//!  and a record is removed only after an absence proof,
//!  which is never an intention
//!  (its "What ends it").
//!  A rewrite re-emits the whole record and publishes it by rename,
//!  so a reader sees the old record or the new one and never a torn one,
//!  and the writer reads its own bytes back first,
//!  because a writer that published bytes its reader refuses would leave the cleanup with no
//!  record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = CleanupFolder.openOrCreate(commonDir);
//! folder.publish(record); folder.remove(name);
//! ```

use crate::private_storage::{create_private_directory_all, replace_private_file_unsynced};
use crate::worktree_clone_origin_cleanup::origin_cleanup_parse::parse_the_record;
use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::{
    check_the_private_file, check_the_private_folder, common_directory_owner, is_a_record_name, origin_cleanups_folder,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::{CleanupRefusal, CleanupRefusalCause};
use std::path::{Path, PathBuf};

/// What:
///  The folder one repository's origin cleanup records live in,
///  open for the writes a command makes.
/// Why:
///  The record outlives the clone,
///  so the folder cannot live in the slot or in the clone's Git directory,
///  and a folder another account owns is refused rather than written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CleanupFolder { readonly path: string; static openOrCreate(commonDir): CleanupFolder }
/// ```
pub struct CleanupFolder {
    path: PathBuf,
}

impl std::fmt::Debug for CleanupFolder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.debug_struct("CleanupFolder").field("path", &self.path).finish();
    }
}

impl CleanupFolder {
    /// What:
    ///  The folder,
    /// made when it is not there yet and checked when it is.
    /// Why:
    ///  A repository with nothing owed has no folder,
    ///  and the first removal that owes a cleanup makes it once.
    ///
    /// # Throws
    /// The folder cannot be made,
    /// or it is not the common directory's owner's `0700` folder.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const folder = CleanupFolder.openOrCreate(commonDir);
    /// ```
    pub fn open_or_create(common_dir: &Path) -> Result<CleanupFolder, CleanupRefusal> {
        let path: PathBuf = origin_cleanups_folder(common_dir);
        create_private_directory_all(path.as_path()).map_err(|error| return CleanupRefusal::at(CleanupRefusalCause::Unreadable, "folder", error.to_string()))?;
        let owner: u32 = common_directory_owner(common_dir)?;
        check_the_private_folder(path.as_path(), owner)?;
        return Ok(CleanupFolder { path });
    }

    /// The folder's path.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// What:
    ///  Publish one record under its snapshot's name,
    ///  after this build's reader has read the bytes back.
    /// Why:
    ///  The name is the key,
    ///  so a record whose own snapshot line disagreed with its name is one no reader would read,
    ///  and the removal would have destroyed snapshots with no cleanup record to say so.
    ///
    /// # Throws
    /// A field past its bound,
    /// bytes this build's own reader refuses,
    /// or a write the filesystem refused.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const path = folder.publish(record);
    /// ```
    pub fn publish(&self, record: &OriginCleanupRecord) -> Result<PathBuf, CleanupRefusal> {
        let name: String = record.origin_snapshot.clone();
        if !is_a_record_name(name.as_str()) {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "originSnapshot", name));
        }
        let bytes: Vec<u8> = record.encode()?;
        match parse_the_record(bytes.as_slice()) {
            Ok(read_back) if read_back == *record => {}
            Ok(_) => return Err(CleanupRefusal::of(CleanupRefusalCause::NotCanonical, String::from("the writer would not read its own bytes back"))),
            Err(refusal) => return Err(refusal),
        }
        let path: PathBuf = self.path.join(name.as_str());
        replace_private_file_unsynced(path.as_path(), bytes.as_slice()).map_err(|error| return CleanupRefusal::at(CleanupRefusalCause::Unreadable, "file", error.to_string()))?;
        return Ok(path);
    }

    /// What:
    ///  Remove one record after its absence proof.
    /// Why:
    ///  Completion is not a state;
    ///  it is the record's absence,
    ///  and only a proof may write it.
    ///  A second removal that finds the name gone is the same completion.
    ///
    /// # Throws
    /// The file is not this account's,
    /// or the unlink failed for a reason other than a name already gone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// folder.remove(name);
    /// ```
    pub fn remove(&self, name: &str) -> Result<(), CleanupRefusal> {
        if !is_a_record_name(name) {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "name", String::from(name)));
        }
        let path: PathBuf = self.path.join(name);
        match std::fs::symlink_metadata(path.as_path()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(CleanupRefusal::at(CleanupRefusalCause::Unreadable, "file", error.to_string())),
            Ok(_) => {}
        }
        let owner: u32 = {
            use std::os::unix::fs::MetadataExt as _;
            let metadata: std::fs::Metadata = std::fs::symlink_metadata(path.as_path()).map_err(|error| return CleanupRefusal::at(CleanupRefusalCause::Unreadable, "file", error.to_string()))?;
            metadata.uid()
        };
        check_the_private_file(path.as_path(), owner)?;
        return match std::fs::remove_file(path.as_path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CleanupRefusal::at(CleanupRefusalCause::Unreadable, "file", error.to_string())),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_publication_tests.rs"]
mod tests;
