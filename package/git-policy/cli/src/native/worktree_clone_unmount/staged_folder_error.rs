//! What:
//!  Why a record or a staging folder's identity could not be read,
//!  written or compared.
//! Why:
//!  The record and the identity both use this refusal,
//!  and keeping it in either file made the two files use each other,
//!  which the layout rule forbids.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type StagedFolderRefusal = { kind: 'unreadable' | 'too-large' | 'malformed' | 'wrong-guid' | 'field' | 'handle'; ... };
//! ```

use crate::worktree_clone_mount_protocol::DatasetGuid;

/// What:
///  Why a record could not be read,
///  written or compared.
/// Why:
///  A malformed or oversized record is treated as unrecorded rather than as
///  authority,
///  and each refusal names the field a person would look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedFolderRefusal = { kind: 'unreadable' | 'too-large' | 'malformed' | 'wrong-guid' | 'field' | 'handle'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StagedFolderRefusal {
    /// The record or the folder could not be read.
    Unreadable {
        /// What was read.
        path: String,
        /// What the read answered.
        reason: String,
    },
    /// The record is larger than its bound.
    TooLarge {
        /// How many bytes the file holds.
        bytes: u64,
        /// The bound.
        bound: u64,
    },
    /// The text is not one record of this shape.
    Malformed {
        /// What about it is wrong.
        said: String,
    },
    /// The record names another clone's folder.
    WrongGuid {
        /// The GUID the name derives.
        wanted: DatasetGuid,
        /// The GUID the record carries.
        recorded: String,
    },
    /// One field is not the value this shape holds.
    Field {
        /// The field's name.
        field: &'static str,
        /// What about the value is wrong.
        said: String,
    },
    /// The handle is absent or too long.
    Handle {
        /// What about the handle is wrong.
        said: String,
    },
}

impl StagedFolderRefusal {
    /// The refusal in words.
    pub fn message(&self) -> String {
        return match self {
            StagedFolderRefusal::Unreadable { path, reason } => format!(
                "the staging folder at {path} could not be read: {reason}. Nothing was bound, moved or removed on a folder and record that could not be read."
            ),
            StagedFolderRefusal::TooLarge { bytes, bound } => format!(
                "the staged-folder record is {bytes} bytes long, and the longest record this route reads is {bound}. It is treated as unrecorded: the handler writes a fresh one before it binds, and a sweep unmounts nothing and reports the folder as unrecorded."
            ),
            StagedFolderRefusal::Malformed { said } => format!(
                "the staged-folder record is not one this route reads: {said}. It is treated as unrecorded; a sweep unmounts nothing and reports the folder as unrecorded."
            ),
            StagedFolderRefusal::WrongGuid { wanted, recorded } => format!(
                "the staged-folder record beside GUID {} carries GUID `{recorded}`, so it describes another clone's folder. It is treated as unrecorded for {}.",
                wanted.file_name(),
                wanted.file_name()
            ),
            StagedFolderRefusal::Field { field, said } => format!(
                "the staged-folder record's `{field}` field is not one this route writes: {said}. It is treated as unrecorded and nothing is acted on it."
            ),
            StagedFolderRefusal::Handle { said } => format!(
                "the staged-folder record's handle is not one this route writes: {said}. A record without a usable handle is read as unrecorded; the handler writes a fresh one before it binds, and a sweep unmounts nothing."
            ),
        };
    }
}
