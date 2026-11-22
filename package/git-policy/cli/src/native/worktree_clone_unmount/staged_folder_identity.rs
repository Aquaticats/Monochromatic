//! What:
//!  The five fields the route records for a staging folder:
//!  device,
//!  inode,
//!  birth time,
//!  owner and mode,
//!  read from the folder's own descriptor,
//!  and their field-by-field compare.
//! Why:
//!  The design says to check the target folder's recorded identity rather than
//!  its kind,
//!  and lists the fields every check compares
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "What a writer can settle without asking").
//!  A folder the account replaced under the same name has another inode and
//!  often another birth time,
//!  so the compare names the field that moved rather than refusing "the folder
//!  changed".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = stagedFolderIdentityOf(fd); // { device, inode, birth, owner, mode }
//! const fields = identity.differences(recorded); // ['inode', 'birth']
//! ```

use super::staged_folder_record::StagedFolderRefusal;
use std::os::unix::fs::MetadataExt;

/// What:
///  One staging folder's identity as its descriptor reports it.
/// Why:
///  The record keeps this,
///  the wrapper compares the folder with it before any removal,
///  and the handler compares the folder it opened with the one the record
///  describes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedFolderIdentity = { device: bigint; inode: bigint; birth?: [number, number]; owner: number; mode: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StagedFolderIdentity {
    /// The device the folder is on.
    pub device: u64,
    /// The folder's inode.
    pub inode: u64,
    /// The folder's birth time in seconds and nanoseconds,
    /// when the filesystem reports one.
    pub birth: Option<(i64, u32)>,
    /// The folder's owner.
    pub owner: u32,
    /// The folder's permission bits,
    /// with the set-user-ID,
    /// set-group-ID and sticky bits.
    pub mode: u32,
}

/// What:
///  One identity field,
///  named as a refusal names it.
/// Why:
///  A mismatch report must say which field moved,
///  and a closed set keeps the words from drifting between the record,
///  the handler and the wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedFolderField = 'device' | 'inode' | 'birth' | 'owner' | 'mode';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StagedFolderField {
    /// The device.
    Device,
    /// The inode.
    Inode,
    /// The birth time.
    Birth,
    /// The owner.
    Owner,
    /// The permission bits.
    Mode,
}

impl StagedFolderField {
    /// The field's word.
    pub fn word(self) -> &'static str {
        return match self {
            StagedFolderField::Device => "device",
            StagedFolderField::Inode => "inode",
            StagedFolderField::Birth => "birth time",
            StagedFolderField::Owner => "owner",
            StagedFolderField::Mode => "mode",
        };
    }
}

impl StagedFolderIdentity {
    /// The identity the folder's own descriptor reports.
    ///
    /// # Errors
    /// The descriptor's metadata could not be read.
    pub fn of(folder: &std::fs::File) -> Result<Self, StagedFolderRefusal> {
        let facts: std::fs::Metadata = folder.metadata().map_err(|error| {
            return StagedFolderRefusal::Unreadable { path: String::from("the staging folder's descriptor"), reason: error.to_string() };
        })?;
        let birth: Option<(i64, u32)> = facts.created().ok().and_then(|created| {
            let since: std::time::Duration = created.duration_since(std::time::UNIX_EPOCH).ok()?;
            return Some((i64::try_from(since.as_secs()).unwrap_or(i64::MAX), since.subsec_nanos()));
        });
        return Ok(StagedFolderIdentity {
            device: facts.dev(),
            inode: facts.ino(),
            birth,
            owner: facts.uid(),
            mode: facts.mode() & 0o7777,
        });
    }

    /// Every field in which this identity differs from `recorded`.
    pub fn differences(&self, recorded: &StagedFolderIdentity) -> Vec<StagedFolderField> {
        let mut fields: Vec<StagedFolderField> = Vec::new();
        if self.device != recorded.device {
            fields.push(StagedFolderField::Device);
        }
        if self.inode != recorded.inode {
            fields.push(StagedFolderField::Inode);
        }
        if self.birth != recorded.birth {
            fields.push(StagedFolderField::Birth);
        }
        if self.owner != recorded.owner {
            fields.push(StagedFolderField::Owner);
        }
        if self.mode != recorded.mode {
            fields.push(StagedFolderField::Mode);
        }
        return fields;
    }

    /// The fields as one phrase,
    /// for a refusal that must name more than one.
    pub fn field_names(fields: &[StagedFolderField]) -> String {
        let mut words: Vec<&str> = Vec::with_capacity(fields.len());
        for field in fields {
            words.push(field.word());
        }
        return words.join(", ");
    }

    /// The identity in words,
    /// for a report that must say what it read.
    pub fn describe(&self) -> String {
        let birth: String = match self.birth {
            Some((seconds, nanos)) => format!("{seconds}.{nanos:09}"),
            None => String::from("not reported"),
        };
        return format!("device {}, inode {}, birth {}, owner {}, mode {:04o}", self.device, self.inode, birth, self.owner, self.mode);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staged_folder_identity_tests.rs"]
mod tests;
