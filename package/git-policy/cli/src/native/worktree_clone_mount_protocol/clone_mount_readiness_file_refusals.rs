//! What:
//!  Why the readiness directory or its record could not be used,
//!  and the sentence a person reads for each refusal.
//! Why:
//!  The caller admits an attach on this file,
//!  so each deviation names the path and says nothing was attached on it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!  The refusals travel back through the publish,
//!  the read and the directory's own proof,
//!  so they live in one file a reader can hold,
//!  apart from the steps that answer them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ReadinessFileRefusal = { kind: 'directory' | 'owner' | 'mode' | 'file' | 'shape' | 'staging'; ... };
//! refusal.message();
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    MAXIMUM_READINESS_BYTES,
    READINESS_DIRECTORY_MODE,
    READINESS_MODE,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_parse::ReadinessParseRefusal;
use std::path::PathBuf;
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
    /// The directory is not there at all,
    /// which on a fresh boot is the answer before the handler's first publish.
    DirectoryAbsent {
        /// The path.
        path: PathBuf,
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
    /// The writer built a text its own reader does not read back as the same
    /// record,
    /// so nothing could ever read it.
    NotWhatWasWritten,
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
            ReadinessFileRefusal::Directory {
                path,
                reason,
            } => format!(
                "the readiness directory {} could not be opened ({reason})",
                path.display()
            ),
            ReadinessFileRefusal::DirectoryAbsent {
                path,
            } => format!(
                "the readiness directory {} does not exist yet",
                path.display()
            ),
            ReadinessFileRefusal::NotWhatWasWritten => String::from(
                "the readiness record this route built is not one its own reader reads back, so it was not written; a field of it holds a character or a length the record's grammar refuses",
            ),
            ReadinessFileRefusal::DirectoryOwner {
                path,
                found,
                wanted,
            } => format!(
                "the readiness directory {} is owned by user {found}, and this route requires user {wanted}",
                path.display()
            ),
            ReadinessFileRefusal::DirectoryMode {
                path,
                found,
            } => format!(
                "the readiness directory {} has mode {found:04o}, and this route requires {READINESS_DIRECTORY_MODE:04o}",
                path.display()
            ),
            ReadinessFileRefusal::File {
                reason,
            } => format!("the readiness record could not be opened or written ({reason})"),
            ReadinessFileRefusal::FileOwner {
                found,
                wanted,
            } => {
                format!(
                    "the readiness record is owned by user {found}, and this route requires user {wanted}"
                )
            }
            ReadinessFileRefusal::FileMode {
                found,
            } => format!(
                "the readiness record has mode {found:04o}, and this route requires {READINESS_MODE:04o}"
            ),
            ReadinessFileRefusal::NotRegular => {
                String::from("the readiness name does not hold a regular file")
            }
            ReadinessFileRefusal::TooLarge {
                bytes,
            } => {
                format!(
                    "the readiness record is {bytes} bytes and a record is at most {MAXIMUM_READINESS_BYTES}"
                )
            }
            ReadinessFileRefusal::NotText => String::from("the readiness record is not UTF-8 text"),
            ReadinessFileRefusal::NotAReadiness {
                refusal,
            } => return refusal.message(),
            ReadinessFileRefusal::Staging {
                reason,
            } => {
                format!(
                    "the sibling a readiness record is staged under could not be made or renamed ({reason}); the earlier record is unchanged"
                )
            }
        };
        return format!(
            "{reason}. Nothing was attached, mounted or unmounted on this record, because a caller admits an attach only on a record it can prove root published. Ways forward: run the handler's `boot-reconcile` in this boot and read `journalctl -u reconcile-git-worktree-clone-mounts.service`; a wrong owner or mode needs an administrator, since this route never changes either."
        );
    }
}
