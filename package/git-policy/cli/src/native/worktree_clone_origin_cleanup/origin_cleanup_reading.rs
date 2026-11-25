//! What:
//!  Where one repository's origin cleanup records live,
//!  the name grammar that tells a record from a foreign entry and from a staging name,
//!  the listing a command makes once,
//!  and the read that checks the folder's and the file's owner and mode first.
//! Why:
//!  The record outlives the worktree,
//!  its registration and its folder,
//!  so it sits in the common Git directory's wrapper folder beside the binding's home
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The cleanup folder" and "The name grammar and the key").
//!  The look destroys a snapshot on a record's word,
//!  so it refuses a record,
//!  a folder or an entry it cannot prove is this account's,
//!  which catches a mistake rather than an attack:
//!  the account is trusted under this route's threat model.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = originCleanupsFolder(commonDir);
//! const names = listRecordNames(folder); // { records, foreign, staging }
//! const record = readTheCleanupRecord(commonDir, name);
//! ```

use crate::worktree_clone_binding_record::registration_home::wrapper_folder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_bounds::ORIGIN_CLEANUP_MAXIMUM_BYTES;
use crate::worktree_clone_origin_cleanup::origin_cleanup_parse::parse_the_record;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::{CleanupRefusal, CleanupRefusalCause};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::{
    ORIGIN_CLEANUPS_FOLDER, ORIGIN_CLEANUP_NAME_BYTES, ORIGIN_SNAPSHOT_PREFIX,
};
use std::path::{Path, PathBuf};

/// The folder one repository's origin cleanup records live in.
pub fn origin_cleanups_folder(common_dir: &Path) -> PathBuf {
    return wrapper_folder(common_dir).join(ORIGIN_CLEANUPS_FOLDER);
}

/// Whether one entry's name is a record this reader reads:
/// the prefix and the add's 32 lowercase hexadecimal digits,
/// and nothing else.
pub fn is_a_record_name(name: &str) -> bool {
    if name.len() != ORIGIN_CLEANUP_NAME_BYTES {
        return false;
    }
    let Some(digits) = name.strip_prefix(ORIGIN_SNAPSHOT_PREFIX) else {
        return false;
    };
    return digits.chars().all(|digit| return digit.is_ascii_digit() || ('a'..='f').contains(&digit));
}

/// Whether one entry's name is a staging name of this package's publication:
/// a leading dot and the record's suffix.
pub fn is_a_staging_name(name: &str) -> bool {
    return name.starts_with('.') && name.ends_with(".partial");
}

/// What:
///  What one listing found:
///  the records this reader reads,
///  the entries that are no record of this family,
///  and this package's staging names.
/// Why:
///  A command pays one open and one listing while nothing is owed,
///  and a foreign entry is reported and never read,
///  which is the rule the three records share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Listing = { records: string[]; foreign: string[]; staging: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanupListing {
    /// The record names,
    /// sorted.
    pub records: Vec<String>,
    /// The entries that are no record and no staging name.
    pub foreign: Vec<String>,
    /// The staging names a killed writer left.
    pub staging: Vec<String>,
}

impl CleanupListing {
    /// Whether the folder holds nothing this family wrote.
    pub fn is_empty(&self) -> bool {
        return self.records.is_empty() && self.foreign.is_empty() && self.staging.is_empty();
    }
}

/// What:
///  One folder's listing,
///  or nothing at all when the folder is not there.
/// Why:
///  A repository that never removed a ZFS clone has no folder,
///  and the look ends there rather than reporting an error.
///
/// # Throws
/// A folder that cannot be listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listing = listTheOriginCleanups(commonDir); // undefined when the folder is absent
/// ```
pub fn list_the_origin_cleanups(common_dir: &Path) -> Result<Option<CleanupListing>, CleanupRefusal> {
    let folder: PathBuf = origin_cleanups_folder(common_dir);
    let entries: std::fs::ReadDir = match std::fs::read_dir(folder.as_path()) {
        Ok(listed) => listed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CleanupRefusal::at(CleanupRefusalCause::Unreadable, "folder", error.to_string())),
    };
    let mut listing: CleanupListing = CleanupListing { records: Vec::new(), foreign: Vec::new(), staging: Vec::new() };
    for listed in entries {
        let held: std::fs::DirEntry = listed.map_err(|error| return CleanupRefusal::at(CleanupRefusalCause::Unreadable, "folder", error.to_string()))?;
        let name: String = held.file_name().to_string_lossy().into_owned();
        if is_a_record_name(name.as_str()) {
            listing.records.push(name);
        } else if is_a_staging_name(name.as_str()) {
            listing.staging.push(name);
        } else {
            listing.foreign.push(name);
        }
    }
    listing.records.sort();
    listing.foreign.sort();
    listing.staging.sort();
    return Ok(Some(listing));
}

/// What:
///  One record's bytes as its value,
///  after the folder's owner and mode and the file's own are checked.
/// Why:
///  The file decides what a destroy may target,
///  and a mistake there is not recoverable,
///  so a folder or a record another account owns is refused before a byte is read.
///
/// # Throws
/// The folder or file that is not this account's,
/// the read that failed,
/// or the grammar's refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = readTheCleanupRecord(commonDir, name);
/// ```
pub fn read_the_cleanup_record(common_dir: &Path, name: &str) -> Result<OriginCleanupRecord, CleanupRefusal> {
    if !is_a_record_name(name) {
        return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "name", String::from(name)));
    }
    let owner: u32 = common_directory_owner(common_dir)?;
    let folder: PathBuf = origin_cleanups_folder(common_dir);
    check_the_private_folder(folder.as_path(), owner)?;
    let path: PathBuf = folder.join(name);
    check_the_private_file(path.as_path(), owner)?;
    let bytes: Vec<u8> = std::fs::read(path.as_path()).map_err(|error| return CleanupRefusal::at(CleanupRefusalCause::Unreadable, "file", error.to_string()))?;
    let record: OriginCleanupRecord = parse_the_record(bytes.as_slice())?;
    if record.origin_snapshot != name {
        return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "originSnapshot", format!("`{}` in a file named `{name}`", record.origin_snapshot)));
    }
    return Ok(record);
}

/// What:
///  One record when one stands,
///  and nothing when the folder or the file is not there.
/// Why:
///  A plan whose head is gone and a folder with no record is the healthy state after a finished
///  cleanup,
///  and a caller that had to branch on a name it cannot read would turn a permission error into
///  an absence sooner or later.
///
/// # Throws
/// The record that is there and does not read,
/// and never an absent one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = anExistingCleanupRecord(commonDir, name); // undefined when none stands
/// ```
pub fn an_existing_cleanup_record(common_dir: &Path, name: &str) -> Result<Option<OriginCleanupRecord>, CleanupRefusal> {
    let path: PathBuf = origin_cleanups_folder(common_dir).join(name);
    match std::fs::symlink_metadata(path.as_path()) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CleanupRefusal::at(CleanupRefusalCause::Unreadable, "file", error.to_string())),
        Ok(_) => {}
    }
    return read_the_cleanup_record(common_dir, name).map(Some);
}

/// The account that owns the common Git directory.
pub fn common_directory_owner(common_dir: &Path) -> Result<u32, CleanupRefusal> {
    let metadata: std::fs::Metadata = std::fs::metadata(common_dir).map_err(|error| return CleanupRefusal::at(CleanupRefusalCause::Unreadable, "common directory", error.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return Ok(metadata.uid());
    }
    #[cfg(not(unix))]
    {
        drop(metadata);
        return Ok(0);
    }
}

/// Refuse a folder that is not the common directory's owner's `0700` folder.
///
/// # Throws
/// The mode or the owner that differs.
pub fn check_the_private_folder(folder: &Path, owner: u32) -> Result<(), CleanupRefusal> {
    let metadata: std::fs::Metadata = std::fs::metadata(folder).map_err(|error| return CleanupRefusal::at(CleanupRefusalCause::Unreadable, "folder", error.to_string()))?;
    if !metadata.is_dir() {
        return Err(CleanupRefusal::at(CleanupRefusalCause::NotARegularFile, "folder", String::from("not a directory")));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let mode: u32 = metadata.mode() & 0o777;
        if mode != 0o700 {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotThisAccounts, "folder", format!("mode {mode:04o}")));
        }
        if metadata.uid() != owner {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotThisAccounts, "folder", format!("owner {}", metadata.uid())));
        }
    }
    #[cfg(not(unix))]
    drop(owner);
    return Ok(());
}

/// Refuse a file that is no regular file this account owns and only it may write.
///
/// # Throws
/// The kind,
/// the owner or the mode that differs.
pub fn check_the_private_file(path: &Path, owner: u32) -> Result<(), CleanupRefusal> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).map_err(|error| return CleanupRefusal::at(CleanupRefusalCause::Unreadable, "file", error.to_string()))?;
    if !metadata.is_file() {
        return Err(CleanupRefusal::at(CleanupRefusalCause::NotARegularFile, "file", path.display().to_string()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != owner {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotThisAccounts, "file", format!("owner {}", metadata.uid())));
        }
        let mode: u32 = metadata.mode() & 0o777;
        if mode & 0o022 != 0 {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotThisAccounts, "file", format!("mode {mode:04o}")));
        }
    }
    #[cfg(not(unix))]
    drop(owner);
    return Ok(());
}

/// The most bytes one record's file may hold,
/// which is the parser's own bound.
pub const RECORD_MAXIMUM_BYTES: u64 = ORIGIN_CLEANUP_MAXIMUM_BYTES;

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_reading_tests.rs"]
mod tests;
