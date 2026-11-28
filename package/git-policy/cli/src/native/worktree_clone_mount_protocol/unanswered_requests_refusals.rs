//! What:
//!  Why the folder of unanswered requests refused one call:
//!  the folder itself,
//!  a file that is not one of this route's entries,
//!  or a file whose text is not one.
//! Why:
//!  A command's read path answers silence and an add reports what it left,
//!  so the cause has to name the path and the way forward rather than only
//!  failing
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The record:
//! one file per clone").
//!  The folder and the read of one entry both answer with this one family,
//!  and it stands in a file of its own so neither uses the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = 'folder' | 'file' | 'text';
//! ```

use crate::worktree_clone_mount_protocol::unanswered_request_entry::EntryRefusal;
use std::path::PathBuf;

/// What:
///  Why the folder,
/// one entry or one publish was refused.
/// Why:
///   A command's read path answers silence and an add reports what it left,
///  so the cause has to name the path and the way forward rather than only
///  failing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = 'folder' | 'name' | 'file' | 'text';
/// ```
#[derive(Debug)]
pub(crate) enum RequestsRefusal {
    /// The folder or a file in it could not be read,
    /// made or written.
    Folder {
        /// The path the call named.
        path: PathBuf,
        /// What the operating system answered.
        reason: String,
    },
    /// A file in the folder is not a regular file of the account's own,
    /// or it is larger than one page.
    File {
        /// The path the read named.
        path: PathBuf,
        /// Why the file is not readable as an entry.
        reason: String,
    },
    /// The file's text is not one entry this route wrote.
    Text {
        /// The path the read named.
        path: PathBuf,
        /// Which field is wrong.
        refusal: EntryRefusal,
    },
}

impl RequestsRefusal {
    /// The refusal in words,
    /// naming the path and the way forward.
    pub(crate) fn message(&self) -> String {
        let what: String = match self {
            RequestsRefusal::Folder {
                path,
                reason,
            } => format!(
                "the folder of unanswered requests at {} could not be used ({reason})",
                path.display()
            ),
            RequestsRefusal::File {
                path,
                reason,
            } => format!(
                "the entry at {} is not a record this route wrote ({reason})",
                path.display()
            ),
            RequestsRefusal::Text {
                path,
                refusal,
            } => format!(
                "the entry at {} cannot be read: {}",
                path.display(),
                refusal.message()
            ),
        };
        return format!(
            "{what}. Nothing was read as an unanswered request, and nothing of the clone's request was changed."
        );
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unanswered_requests_refusals_tests.rs"]
mod tests;
