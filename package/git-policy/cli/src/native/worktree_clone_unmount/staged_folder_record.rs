//! What:
//!  The staged-folder record's file:
//!  the names it is written under,
//!  its atomic write through the `~/.cache` descriptor,
//!  and its bounded,
//!  no-follow read.
//! Why:
//!  The handler writes the record before the first bind,
//!  as one temporary file renamed over the final name,
//!  so a reader sees one whole record or the one before it
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staged-folder record").
//!  A name swapped for a pipe or a link cannot redirect the read,
//!  because the bytes come from an open handle that followed no link and the
//!  file has a size bound.
//!  The file's calls sit apart from the record's fields and its text,
//!  so a reader of the write sees the order of the calls and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeStagedFolderRecord(cache, record);           // one atomic rename
//! const text = readRecordText(cache, guid);
//! ```

use super::staging_folder_name::{
    staging_record_name,
    staging_temporary_name,
};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::ffi::OsString;
use std::io::Write;

pub use super::staged_folder_error::StagedFolderRefusal;

/// The record's fields and the one JSON object it is written as.
pub use super::staged_folder_record_text::{
    MAXIMUM_HANDLE_BYTES,
    MAXIMUM_RECORD_BYTES,
    RECORD_KIND,
    RECORD_VERSION,
    StagedFolderRecord,
    record_from_text,
};

/// The names a record is written under.
fn names(guid: DatasetGuid) -> (OsString, OsString) {
    return (
        OsString::from(staging_temporary_name(guid)),
        OsString::from(staging_record_name(guid)),
    );
}

/// What:
///  Write one record atomically through the `~/.cache` descriptor:
///  a temporary name created exclusively,
///  written and synced,
///  renamed over the final name,
///  and the folder synced.
/// Why:
///  A reader sees the whole record of one write or the one before it,
///  and a killed writer leaves only the temporary name the next write replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeStagedFolderRecord(cache, record);
/// ```
///
/// # Errors
/// The temporary name could not be created or written,
/// the rename refused,
/// or the folder could not be synced.
pub fn write_record(
    cache: &std::fs::File,
    record: &StagedFolderRecord,
) -> Result<(), StagedFolderRefusal> {
    let (temporary, final_name): (OsString, OsString) = names(record.guid);
    let _stale = crate::fs_beneath::remove_at(cache, temporary.as_os_str(), false);
    let mut file: std::fs::File =
        crate::fs_beneath::create_file_at(cache, temporary.as_os_str(), 0o644).map_err(
            |error| {
                return StagedFolderRefusal::Unreadable {
                    path: temporary.to_string_lossy().to_string(),
                    reason: error.to_string(),
                };
            },
        )?;
    file.write_all(record.encode().as_bytes())
        .map_err(|error| {
            return StagedFolderRefusal::Unreadable {
                path: temporary.to_string_lossy().to_string(),
                reason: error.to_string(),
            };
        })?;
    file.sync_all().map_err(|error| {
        return StagedFolderRefusal::Unreadable {
            path: temporary.to_string_lossy().to_string(),
            reason: error.to_string(),
        };
    })?;
    drop(file);
    crate::fs_beneath::rename_at(
        cache,
        temporary.as_os_str(),
        cache,
        final_name.as_os_str(),
        true,
    )
    .map_err(|error| {
        return StagedFolderRefusal::Unreadable {
            path: final_name.to_string_lossy().to_string(),
            reason: error.to_string(),
        };
    })?;
    return cache.sync_all().map_err(|error| {
        return StagedFolderRefusal::Unreadable {
            path: String::from("~/.cache"),
            reason: error.to_string(),
        };
    });
}

/// What:
///  Read the record beside one GUID's folder,
///  or nothing when the name is absent or is not a plain file.
/// Why:
///  The reader takes the bytes from an open handle that followed no link and
///  refuses a file past the record's bound,
///  so a name swapped for a pipe or a link cannot redirect the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readRecord(cache, guid); // { kind: 'absent' | 'text' | 'unreadable' }
/// ```
pub fn read_record_text(
    cache: &std::fs::File,
    guid: DatasetGuid,
) -> Result<Option<String>, StagedFolderRefusal> {
    let name: OsString = OsString::from(staging_record_name(guid));
    let file: std::fs::File = match crate::fs_beneath::open_file_at(cache, name.as_os_str()) {
        Ok(file) => file,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) => return Ok(None),
        Err(error) => {
            return Err(StagedFolderRefusal::Unreadable {
                path: name.to_string_lossy().to_string(),
                reason: error.to_string(),
            });
        }
    };
    let facts: std::fs::Metadata = file.metadata().map_err(|error| {
        return StagedFolderRefusal::Unreadable {
            path: name.to_string_lossy().to_string(),
            reason: error.to_string(),
        };
    })?;
    if facts.len() > MAXIMUM_RECORD_BYTES {
        return Err(StagedFolderRefusal::TooLarge {
            bytes: facts.len(),
            bound: MAXIMUM_RECORD_BYTES,
        });
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(usize::try_from(facts.len()).unwrap_or(0));
    use std::io::Read;
    std::io::Read::take(&file, MAXIMUM_RECORD_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            return StagedFolderRefusal::Unreadable {
                path: name.to_string_lossy().to_string(),
                reason: error.to_string(),
            };
        })?;
    if bytes.len() as u64 > MAXIMUM_RECORD_BYTES {
        return Err(StagedFolderRefusal::TooLarge {
            bytes: bytes.len() as u64,
            bound: MAXIMUM_RECORD_BYTES,
        });
    }
    return match std::str::from_utf8(bytes.as_slice()) {
        Ok(text) => Ok(Some(String::from(text))),
        Err(_) => Err(StagedFolderRefusal::Malformed {
            said: String::from("the file is not UTF-8 text"),
        }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staged_folder_record_tests.rs"]
mod tests;
