//! What:
//!  Why the propagation record's directory or its file could not be used,
//!  and the sentence a person reads for each refusal.
//! Why:
//!  The step reports a refusal it met while filing its answer,
//!  and `doctor` must say which proof failed rather than read a file it could
//!  not prove,
//!  so the causes and their words are one subject
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "What it writes").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type PropagationFileRefusal = { kind: 'directory' | 'record-owner' | 'record-mode' | 'staging' | 'file' | 'shape'; ... };
//! refusal.message();
//! ```

use crate::checkout_propagation::propagation_record_parse::PropagationParseRefusal;
use crate::checkout_propagation::propagation_words::{
    MAXIMUM_PROPAGATION_BYTES,
    PROPAGATION_DIRECTORY,
    PROPAGATION_MODE,
};
use crate::worktree_clone_mount_protocol::root_owned_directory::RootDirectoryRefusal;

/// What:
///  Why the record's directory or its file could not be used.
/// Why:
///  The step reports a refusal it met while filing its answer,
///  and the reader of `doctor` must say which proof failed rather than read a
///  file it could not prove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationFileRefusal = { kind: 'directory' | 'record-owner' | 'record-mode' | 'staging' | 'file' | 'shape'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropagationFileRefusal {
    /// The directory could not be made,
    /// opened or examined,
    /// or is not the designed root-owned folder.
    Directory(RootDirectoryRefusal),
    /// The record is not owned by the account the route requires.
    RecordOwner {
        /// The owner found.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
    /// The record does not carry its designed mode.
    RecordMode {
        /// The mode found.
        found: u32,
    },
    /// The record could not be written,
    /// synced or renamed.
    Staging {
        /// What the call answered.
        reason: String,
    },
    /// The record could not be opened or read.
    File {
        /// What the call answered.
        reason: String,
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
    /// The text is not a record this route wrote.
    NotARecord {
        /// What the parse refused.
        refusal: PropagationParseRefusal,
    },
}

impl PropagationFileRefusal {
    /// The sentence a person reads,
    /// naming the path and the way forward.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const message = (refusal: PropagationFileRefusal): string => ...;
    /// ```
    pub fn message(&self) -> String {
        let reason: String = match self {
            PropagationFileRefusal::Directory(RootDirectoryRefusal::Open {
                reason,
            }) => format!(
                "the propagation record's directory {PROPAGATION_DIRECTORY} could not be made or opened ({reason})"
            ),
            PropagationFileRefusal::Directory(RootDirectoryRefusal::Owner {
                found,
                wanted,
            }) => format!(
                "the propagation record's directory {PROPAGATION_DIRECTORY} is owned by user {found} and this route requires user {wanted}"
            ),
            PropagationFileRefusal::Directory(RootDirectoryRefusal::Mode {
                found,
                wanted,
            }) => format!(
                "the propagation record's directory {PROPAGATION_DIRECTORY} has mode {found:04o} and this route requires {wanted:04o}"
            ),
            PropagationFileRefusal::RecordOwner {
                found,
                wanted,
            } => format!(
                "the propagation record is owned by user {found} and this route requires user {wanted}"
            ),
            PropagationFileRefusal::RecordMode {
                found,
            } => format!(
                "the propagation record has mode {found:04o} and this route requires {PROPAGATION_MODE:04o}"
            ),
            PropagationFileRefusal::Staging {
                reason,
            } => format!(
                "the sibling the propagation record is staged under could not be written or renamed ({reason}); the earlier record is unchanged"
            ),
            PropagationFileRefusal::File {
                reason,
            } => format!("the propagation record could not be opened or read ({reason})"),
            PropagationFileRefusal::NotRegular => {
                String::from("the propagation record's name does not hold a regular file")
            }
            PropagationFileRefusal::TooLarge {
                bytes,
            } => format!(
                "the propagation record is {bytes} bytes and a record is at most {MAXIMUM_PROPAGATION_BYTES}"
            ),
            PropagationFileRefusal::NotText => {
                String::from("the propagation record is not UTF-8 text")
            }
            PropagationFileRefusal::NotWhatWasWritten => String::from(
                "the propagation record this route built is not one its own reader reads back, so it was not written; a field of it holds a character or a length the record's grammar refuses",
            ),
            PropagationFileRefusal::NotARecord {
                refusal,
            } => return refusal.message(),
        };
        return format!(
            "{reason}. Nothing under the checkout was changed by this refusal, and no readiness record was published for it. Ways forward: read `journalctl -u reconcile-git-worktree-clone-mounts.service` and the propagation record again after the next handler run; a wrong owner or mode needs an administrator, since this route never changes either."
        );
    }
}
